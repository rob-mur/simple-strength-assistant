use crate::models::{CompletedSet, ExerciseMetadata};
use thiserror::Error;
use wasm_bindgen::prelude::*;
use web_sys::js_sys;

#[derive(Error, Debug, Clone)]
pub enum DatabaseError {
    #[error("Failed to initialize database: {0}")]
    InitializationError(String),

    #[error("Failed to execute query: {0}")]
    QueryError(String),

    #[error("Database not initialized")]
    NotInitialized,

    #[error("JavaScript error: {0}")]
    JsError(String),
}

impl From<JsValue> for DatabaseError {
    fn from(err: JsValue) -> Self {
        DatabaseError::JsError(format!("{:?}", err))
    }
}

#[wasm_bindgen(module = "/public/db-module.js")]
extern "C" {
    #[wasm_bindgen(js_name = initDatabase)]
    async fn init_database(file_data: Option<Vec<u8>>) -> JsValue;

    #[wasm_bindgen(js_name = executeQuery)]
    async fn execute_query(sql: &str, params: JsValue) -> JsValue;

    #[wasm_bindgen(js_name = exportDatabase)]
    async fn export_database() -> JsValue;
}

#[derive(Clone, PartialEq)]
pub struct Database {
    initialized: bool,
}

impl Database {
    pub fn new() -> Self {
        Self { initialized: false }
    }

    pub async fn init(&mut self, file_data: Option<Vec<u8>>) -> Result<(), DatabaseError> {
        log::debug!("[DB] Calling JS initDatabase...");
        let result = init_database(file_data).await;

        if result.is_truthy() {
            log::debug!("[DB] initDatabase succeeded, creating tables...");
            self.create_tables().await?;
            self.initialized = true;
            log::debug!("[DB] Tables created successfully and database initialized");
            Ok(())
        } else {
            let error_msg = "Failed to initialize SQLite database - JS returned false".to_string();
            log::error!("{}", error_msg);
            Err(DatabaseError::InitializationError(error_msg))
        }
    }

    async fn create_tables(&self) -> Result<(), DatabaseError> {
        let create_sessions = r#"
            CREATE TABLE IF NOT EXISTS sessions (
                id INTEGER PRIMARY KEY AUTOINCREMENT,
                exercise_name TEXT NOT NULL,
                started_at INTEGER NOT NULL,
                completed_at INTEGER
            )
        "#;

        let create_sets = r#"
            CREATE TABLE IF NOT EXISTS completed_sets (
                id INTEGER PRIMARY KEY AUTOINCREMENT,
                session_id INTEGER NOT NULL,
                set_number INTEGER NOT NULL,
                reps INTEGER NOT NULL,
                rpe REAL NOT NULL,
                weight REAL,
                is_bodyweight INTEGER NOT NULL,
                FOREIGN KEY (session_id) REFERENCES sessions(id) ON DELETE CASCADE
            )
        "#;

        let create_exercises = r#"
            CREATE TABLE IF NOT EXISTS exercises (
                id INTEGER PRIMARY KEY AUTOINCREMENT,
                name TEXT NOT NULL UNIQUE,
                is_weighted INTEGER NOT NULL,
                min_weight REAL,
                increment REAL
            )
        "#;

        let create_index = r#"
            CREATE INDEX IF NOT EXISTS idx_sets_session_id
            ON completed_sets(session_id)
        "#;

        log::debug!("[DB] Creating sessions table...");
        self.execute_internal(create_sessions, &[]).await?;
        log::debug!("[DB] Creating completed_sets table...");
        self.execute_internal(create_sets, &[]).await?;
        log::debug!("[DB] Creating exercises table...");
        self.execute_internal(create_exercises, &[]).await?;
        log::debug!("[DB] Creating index on session_id...");
        self.execute_internal(create_index, &[]).await?;

        Ok(())
    }

    pub async fn execute(&self, sql: &str, params: &[JsValue]) -> Result<JsValue, DatabaseError> {
        if !self.initialized {
            return Err(DatabaseError::NotInitialized);
        }

        self.execute_internal(sql, params).await
    }

    async fn execute_internal(
        &self,
        sql: &str,
        params: &[JsValue],
    ) -> Result<JsValue, DatabaseError> {
        let params_array = js_sys::Array::new();
        for param in params {
            params_array.push(param);
        }

        let result = execute_query(sql, params_array.into()).await;

        if let Some(error) = result.dyn_ref::<js_sys::Error>() {
            return Err(DatabaseError::QueryError(
                error.message().as_string().unwrap_or_default(),
            ));
        }

        Ok(result)
    }

    pub async fn create_session(&self, exercise_name: &str) -> Result<i64, DatabaseError> {
        let sql = "INSERT INTO sessions (exercise_name, started_at) VALUES (?, ?) RETURNING id";
        let now = js_sys::Date::now() as i64;

        let params = vec![
            JsValue::from_str(exercise_name),
            JsValue::from_f64(now as f64),
        ];

        let result = self.execute(sql, &params).await?;

        let array = result
            .dyn_ref::<js_sys::Array>()
            .ok_or_else(|| DatabaseError::QueryError("Expected array result".to_string()))?;

        if array.length() == 0 {
            return Err(DatabaseError::QueryError("No rows returned".to_string()));
        }

        let first_row = array.get(0);
        let id = js_sys::Reflect::get(&first_row, &JsValue::from_str("id"))?
            .as_f64()
            .ok_or_else(|| DatabaseError::QueryError("Failed to get session id".to_string()))?
            as i64;

        Ok(id)
    }

    pub async fn complete_session(&self, session_id: i64) -> Result<(), DatabaseError> {
        let sql = "UPDATE sessions SET completed_at = ? WHERE id = ?";
        let now = js_sys::Date::now() as i64;

        let params = vec![
            JsValue::from_f64(now as f64),
            JsValue::from_f64(session_id as f64),
        ];

        self.execute(sql, &params).await?;
        Ok(())
    }

    pub async fn insert_set(
        &self,
        session_id: i64,
        set: &CompletedSet,
    ) -> Result<i64, DatabaseError> {
        let (weight, is_bodyweight) = match set.set_type {
            crate::models::SetType::Weighted { weight } => (Some(weight), false),
            crate::models::SetType::Bodyweight => (None, true),
        };

        let sql = r#"
            INSERT INTO completed_sets (session_id, set_number, reps, rpe, weight, is_bodyweight)
            VALUES (?, ?, ?, ?, ?, ?)
            RETURNING id
        "#;

        let params = vec![
            JsValue::from_f64(session_id as f64),
            JsValue::from_f64(set.set_number as f64),
            JsValue::from_f64(set.reps as f64),
            JsValue::from_f64(set.rpe as f64),
            weight
                .map(|w| JsValue::from_f64(w as f64))
                .unwrap_or(JsValue::NULL),
            JsValue::from_bool(is_bodyweight),
        ];

        let result = self.execute(sql, &params).await?;

        let array = result
            .dyn_ref::<js_sys::Array>()
            .ok_or_else(|| DatabaseError::QueryError("Expected array result".to_string()))?;

        if array.length() == 0 {
            return Err(DatabaseError::QueryError("No rows returned".to_string()));
        }

        let first_row = array.get(0);
        let id = js_sys::Reflect::get(&first_row, &JsValue::from_str("id"))?
            .as_f64()
            .ok_or_else(|| DatabaseError::QueryError("Failed to get set id".to_string()))?
            as i64;

        Ok(id)
    }

    pub async fn save_exercise(&self, exercise: &ExerciseMetadata) -> Result<(), DatabaseError> {
        let (is_weighted, min_weight, increment) = match exercise.set_type_config {
            crate::models::SetTypeConfig::Weighted {
                min_weight,
                increment,
            } => (true, Some(min_weight), Some(increment)),
            crate::models::SetTypeConfig::Bodyweight => (false, None, None),
        };

        let sql = r#"
            INSERT OR REPLACE INTO exercises (name, is_weighted, min_weight, increment)
            VALUES (?, ?, ?, ?)
        "#;

        let params = vec![
            JsValue::from_str(&exercise.name),
            JsValue::from_bool(is_weighted),
            min_weight
                .map(|w| JsValue::from_f64(w as f64))
                .unwrap_or(JsValue::NULL),
            increment
                .map(|i| JsValue::from_f64(i as f64))
                .unwrap_or(JsValue::NULL),
        ];

        self.execute(sql, &params).await?;
        Ok(())
    }

    pub async fn export(&self) -> Result<Vec<u8>, DatabaseError> {
        if !self.initialized {
            return Err(DatabaseError::NotInitialized);
        }

        let result = export_database().await;

        let uint8_array = js_sys::Uint8Array::new(&result);
        let mut buffer = vec![0; uint8_array.length() as usize];
        uint8_array.copy_to(&mut buffer);

        Ok(buffer)
    }
}

impl Default for Database {
    fn default() -> Self {
        Self::new()
    }
}
