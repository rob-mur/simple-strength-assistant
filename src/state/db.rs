use crate::models::{CompletedSet, ExerciseMetadata};
use thiserror::Error;
use wasm_bindgen::prelude::*;
use web_sys::js_sys;

/// The current database schema version. Bump this when the schema changes.
/// On first launch after an upgrade, the app detects a version mismatch and
/// replaces any existing database with a fresh schema.
const DB_VERSION: i64 = 2;

#[derive(Error, Debug, Clone)]
pub enum DatabaseError {
    #[error("Failed to initialize database: {0}")]
    InitializationError(String),

    #[error("Failed to execute query: {0}")]
    QueryError(String),

    #[error("Database not initialized")]
    NotInitialized,

    #[error("Exercise not found")]
    ExerciseNotFound,

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
            log::debug!("[DB] initDatabase succeeded, checking schema version...");

            // Check user_version to detect stale schema from a previous app version.
            // If it doesn't match DB_VERSION we discard the loaded data and start fresh.
            let version = self.read_user_version().await.unwrap_or(0);
            log::debug!("[DB] Current user_version: {}", version);

            if version != DB_VERSION {
                log::debug!(
                    "[DB] Schema version mismatch (got {}, want {}), resetting database",
                    version,
                    DB_VERSION
                );
                let reset_result = init_database(None).await;
                if !reset_result.is_truthy() {
                    return Err(DatabaseError::InitializationError(
                        "Failed to reset database to new schema".to_string(),
                    ));
                }
            }

            log::debug!("[DB] Creating/verifying tables...");
            self.create_tables().await?;
            self.initialized = true;
            log::debug!("[DB] Tables verified and database initialized");
            Ok(())
        } else {
            let error_msg = "Failed to initialize SQLite database - JS returned false".to_string();
            log::error!("{}", error_msg);
            Err(DatabaseError::InitializationError(error_msg))
        }
    }

    async fn read_user_version(&self) -> Result<i64, DatabaseError> {
        let result = execute_query("PRAGMA user_version", JsValue::NULL).await;

        if let Some(error) = result.dyn_ref::<js_sys::Error>() {
            return Err(DatabaseError::QueryError(
                error.message().as_string().unwrap_or_default(),
            ));
        }

        let array = result
            .dyn_ref::<js_sys::Array>()
            .ok_or_else(|| DatabaseError::QueryError("Expected array result".to_string()))?;

        if array.length() == 0 {
            return Ok(0);
        }

        let row = array.get(0);
        let version = js_sys::Reflect::get(&row, &JsValue::from_str("user_version"))?
            .as_f64()
            .unwrap_or(0.0) as i64;

        Ok(version)
    }

    async fn create_tables(&self) -> Result<(), DatabaseError> {
        let create_exercises = r#"
            CREATE TABLE IF NOT EXISTS exercises (
                id INTEGER PRIMARY KEY AUTOINCREMENT,
                name TEXT NOT NULL UNIQUE,
                is_weighted INTEGER NOT NULL,
                min_weight REAL,
                increment REAL
            )
        "#;

        let create_sets = r#"
            CREATE TABLE IF NOT EXISTS completed_sets (
                id INTEGER PRIMARY KEY AUTOINCREMENT,
                exercise_id INTEGER NOT NULL,
                set_number INTEGER NOT NULL,
                reps INTEGER NOT NULL,
                rpe REAL NOT NULL,
                weight REAL,
                is_bodyweight INTEGER NOT NULL,
                recorded_at INTEGER NOT NULL,
                FOREIGN KEY (exercise_id) REFERENCES exercises(id)
            )
        "#;

        let create_index = r#"
            CREATE INDEX IF NOT EXISTS idx_sets_exercise_id
            ON completed_sets(exercise_id)
        "#;

        log::debug!("[DB] Creating exercises table...");
        self.execute_internal(create_exercises, &[]).await?;
        log::debug!("[DB] Creating completed_sets table...");
        self.execute_internal(create_sets, &[]).await?;
        log::debug!("[DB] Creating index on exercise_id...");
        self.execute_internal(create_index, &[]).await?;

        // Stamp the schema version so future launches can detect mismatches.
        let set_version = format!("PRAGMA user_version = {}", DB_VERSION);
        log::debug!("[DB] Setting user_version = {}", DB_VERSION);
        self.execute_internal(&set_version, &[]).await?;

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

    /// Inserts a completed set directly linked to an exercise.
    /// Records the current timestamp as `recorded_at`.
    pub async fn insert_set(
        &self,
        exercise_id: i64,
        set: &CompletedSet,
    ) -> Result<i64, DatabaseError> {
        let (weight, is_bodyweight) = match set.set_type {
            crate::models::SetType::Weighted { weight } => (Some(weight), false),
            crate::models::SetType::Bodyweight => (None, true),
        };

        let now = js_sys::Date::now();

        let sql = r#"
            INSERT INTO completed_sets (exercise_id, set_number, reps, rpe, weight, is_bodyweight, recorded_at)
            VALUES (?, ?, ?, ?, ?, ?, ?)
            RETURNING id
        "#;

        let params = vec![
            JsValue::from_f64(exercise_id as f64),
            JsValue::from_f64(set.set_number as f64),
            JsValue::from_f64(set.reps as f64),
            JsValue::from_f64(set.rpe as f64),
            weight
                .map(|w| JsValue::from_f64(w as f64))
                .unwrap_or(JsValue::NULL),
            JsValue::from_bool(is_bodyweight),
            JsValue::from_f64(now),
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

    pub async fn save_exercise(&self, exercise: &ExerciseMetadata) -> Result<i64, DatabaseError> {
        let (is_weighted, min_weight, increment) = match exercise.set_type_config {
            crate::models::SetTypeConfig::Weighted {
                min_weight,
                increment,
            } => (true, Some(min_weight), Some(increment)),
            crate::models::SetTypeConfig::Bodyweight => (false, None, None),
        };

        let result = if let Some(id) = exercise.id {
            let sql = r#"
                UPDATE exercises SET name = ?, is_weighted = ?, min_weight = ?, increment = ?
                WHERE id = ?
                RETURNING id
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
                JsValue::from_f64(id as f64),
            ];
            self.execute(sql, &params).await?
        } else {
            let sql = r#"
                INSERT INTO exercises (name, is_weighted, min_weight, increment)
                VALUES (?, ?, ?, ?)
                ON CONFLICT(name) DO UPDATE SET
                    is_weighted = excluded.is_weighted,
                    min_weight = excluded.min_weight,
                    increment = excluded.increment
                RETURNING id
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
            self.execute(sql, &params).await?
        };

        let array = result
            .dyn_ref::<js_sys::Array>()
            .ok_or_else(|| DatabaseError::QueryError("Expected array result".to_string()))?;

        if array.length() == 0 {
            if exercise.id.is_some() {
                return Err(DatabaseError::ExerciseNotFound);
            }
            return Err(DatabaseError::QueryError("No rows returned".to_string()));
        }

        let first_row = array.get(0);
        let id = js_sys::Reflect::get(&first_row, &JsValue::from_str("id"))?
            .as_f64()
            .ok_or_else(|| DatabaseError::QueryError("Failed to get exercise id".to_string()))?
            as i64;

        Ok(id)
    }

    pub async fn get_exercises(&self) -> Result<Vec<ExerciseMetadata>, DatabaseError> {
        let sql =
            "SELECT id, name, is_weighted, min_weight, increment FROM exercises ORDER BY name";
        let result = self.execute(sql, &[]).await?;

        let array = result
            .dyn_ref::<js_sys::Array>()
            .ok_or_else(|| DatabaseError::QueryError("Expected array result".to_string()))?;

        let mut exercises = Vec::new();
        for i in 0..array.length() {
            let row = array.get(i);
            let id = js_sys::Reflect::get(&row, &JsValue::from_str("id"))?
                .as_f64()
                .map(|f| f as i64);

            let name = js_sys::Reflect::get(&row, &JsValue::from_str("name"))?
                .as_string()
                .ok_or_else(|| DatabaseError::QueryError("Failed to get name".to_string()))?;

            let is_weighted_val = js_sys::Reflect::get(&row, &JsValue::from_str("is_weighted"))?;
            let is_weighted = if let Some(b) = is_weighted_val.as_bool() {
                b
            } else if let Some(f) = is_weighted_val.as_f64() {
                f == 1.0
            } else {
                return Err(DatabaseError::QueryError(
                    "Failed to get is_weighted as bool or number".to_string(),
                ));
            };

            let set_type_config = if is_weighted {
                let min_weight = js_sys::Reflect::get(&row, &JsValue::from_str("min_weight"))?
                    .as_f64()
                    .ok_or_else(|| {
                        DatabaseError::QueryError("Failed to get min_weight".to_string())
                    })? as f32;
                let increment = js_sys::Reflect::get(&row, &JsValue::from_str("increment"))?
                    .as_f64()
                    .ok_or_else(|| {
                        DatabaseError::QueryError("Failed to get increment".to_string())
                    })? as f32;
                crate::models::SetTypeConfig::Weighted {
                    min_weight,
                    increment,
                }
            } else {
                crate::models::SetTypeConfig::Bodyweight
            };

            exercises.push(ExerciseMetadata {
                id,
                name,
                set_type_config,
            });
        }

        Ok(exercises)
    }

    /// Returns the most recently recorded set for the given exercise, ordered by
    /// `recorded_at DESC` then `id DESC`. Uses the exercise_id FK directly.
    pub async fn get_last_set_for_exercise(
        &self,
        exercise_id: i64,
    ) -> Result<Option<crate::models::CompletedSet>, DatabaseError> {
        let sql = r#"
            SELECT set_number, reps, rpe, weight, is_bodyweight
            FROM completed_sets
            WHERE exercise_id = ?
            ORDER BY recorded_at DESC, id DESC
            LIMIT 1
        "#;

        let params = vec![JsValue::from_f64(exercise_id as f64)];
        let result = self.execute(sql, &params).await?;

        let array = result
            .dyn_ref::<js_sys::Array>()
            .ok_or_else(|| DatabaseError::QueryError("Expected array result".to_string()))?;

        if array.length() == 0 {
            return Ok(None);
        }

        let row = array.get(0);
        let set_number = js_sys::Reflect::get(&row, &JsValue::from_str("set_number"))?
            .as_f64()
            .ok_or_else(|| DatabaseError::QueryError("Failed to get set_number".to_string()))?
            as u32;

        let reps = js_sys::Reflect::get(&row, &JsValue::from_str("reps"))?
            .as_f64()
            .ok_or_else(|| DatabaseError::QueryError("Failed to get reps".to_string()))?
            as u32;

        let rpe = js_sys::Reflect::get(&row, &JsValue::from_str("rpe"))?
            .as_f64()
            .ok_or_else(|| DatabaseError::QueryError("Failed to get rpe".to_string()))?
            as f32;

        let is_bodyweight_val = js_sys::Reflect::get(&row, &JsValue::from_str("is_bodyweight"))?;
        let is_bodyweight = if let Some(b) = is_bodyweight_val.as_bool() {
            b
        } else if let Some(f) = is_bodyweight_val.as_f64() {
            f == 1.0
        } else {
            return Err(DatabaseError::QueryError(
                "Failed to get is_bodyweight as bool or number".to_string(),
            ));
        };

        let set_type = if is_bodyweight {
            crate::models::SetType::Bodyweight
        } else {
            let weight = js_sys::Reflect::get(&row, &JsValue::from_str("weight"))?
                .as_f64()
                .ok_or_else(|| DatabaseError::QueryError("Failed to get weight".to_string()))?
                as f32;
            crate::models::SetType::Weighted { weight }
        };

        Ok(Some(crate::models::CompletedSet {
            set_number,
            reps,
            rpe,
            set_type,
        }))
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
