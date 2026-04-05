use crate::models::{CompletedSet, ExerciseMetadata, HistorySet, SetType};
use thiserror::Error;
use uuid::Uuid;
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

/// Current schema version. Bump this when the schema changes.
const SCHEMA_VERSION: i64 = 3;

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
            self.migrate_and_create_tables().await?;
            self.initialized = true;
            log::debug!("[DB] Tables created successfully and database initialized");
            Ok(())
        } else {
            let error_msg = "Failed to initialize SQLite database - JS returned false".to_string();
            log::error!("{}", error_msg);
            Err(DatabaseError::InitializationError(error_msg))
        }
    }

    async fn migrate_and_create_tables(&self) -> Result<(), DatabaseError> {
        // Detect old schema by checking user_version pragma
        let current_version = self.get_schema_version().await.unwrap_or(0);
        log::debug!("[DB] Current schema version: {}", current_version);

        if current_version < SCHEMA_VERSION {
            log::debug!(
                "[DB] Migrating schema from v{} to v{}",
                current_version,
                SCHEMA_VERSION
            );

            if current_version < 2 {
                // v0→v2: drop incompatible tables (sessions table was removed in v2).
                self.execute_internal("DROP TABLE IF EXISTS completed_sets", &[])
                    .await?;
                self.execute_internal("DROP TABLE IF EXISTS sessions", &[])
                    .await?;
            }
        }

        let create_exercises = r#"
            CREATE TABLE IF NOT EXISTS exercises (
                id INTEGER PRIMARY KEY AUTOINCREMENT,
                uuid TEXT NOT NULL UNIQUE DEFAULT '',
                name TEXT NOT NULL UNIQUE,
                is_weighted INTEGER NOT NULL,
                min_weight REAL,
                increment REAL,
                updated_at INTEGER NOT NULL DEFAULT 0,
                deleted_at INTEGER
            )
        "#;

        let create_sets = r#"
            CREATE TABLE IF NOT EXISTS completed_sets (
                id INTEGER PRIMARY KEY AUTOINCREMENT,
                uuid TEXT NOT NULL UNIQUE DEFAULT '',
                exercise_id INTEGER NOT NULL,
                set_number INTEGER NOT NULL,
                reps INTEGER NOT NULL,
                rpe REAL NOT NULL,
                weight REAL,
                is_bodyweight INTEGER NOT NULL,
                recorded_at INTEGER NOT NULL,
                updated_at INTEGER NOT NULL DEFAULT 0,
                deleted_at INTEGER,
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

        if current_version < SCHEMA_VERSION {
            // v2→v3: add sync columns to pre-existing tables.
            // ALTER TABLE ADD COLUMN is idempotent when the column already
            // exists in a fresh database created by this function (SQLite
            // returns an error, which we silently ignore here).
            log::debug!("[DB] Applying v3 sync-column migrations...");
            self.apply_v3_migrations().await?;
        }

        // Stamp the new version
        self.execute_internal(&format!("PRAGMA user_version = {}", SCHEMA_VERSION), &[])
            .await?;

        Ok(())
    }

    /// Applies the v3 schema additions: uuid, updated_at, deleted_at columns
    /// on exercises and completed_sets, then backfills existing rows.
    async fn apply_v3_migrations(&self) -> Result<(), DatabaseError> {
        let now_ms = js_sys::Date::now() as i64;

        // Add columns to exercises (ignore errors if columns already exist).
        let _ = self
            .execute_internal(
                "ALTER TABLE exercises ADD COLUMN uuid TEXT NOT NULL DEFAULT ''",
                &[],
            )
            .await;
        let _ = self
            .execute_internal(
                "ALTER TABLE exercises ADD COLUMN updated_at INTEGER NOT NULL DEFAULT 0",
                &[],
            )
            .await;
        let _ = self
            .execute_internal("ALTER TABLE exercises ADD COLUMN deleted_at INTEGER", &[])
            .await;

        // Add columns to completed_sets (ignore errors if columns already exist).
        let _ = self
            .execute_internal(
                "ALTER TABLE completed_sets ADD COLUMN uuid TEXT NOT NULL DEFAULT ''",
                &[],
            )
            .await;
        let _ = self
            .execute_internal(
                "ALTER TABLE completed_sets ADD COLUMN updated_at INTEGER NOT NULL DEFAULT 0",
                &[],
            )
            .await;
        let _ = self
            .execute_internal(
                "ALTER TABLE completed_sets ADD COLUMN deleted_at INTEGER",
                &[],
            )
            .await;

        // Backfill exercises: set updated_at for any rows that have the default 0 value.
        self.execute_internal(
            "UPDATE exercises SET updated_at = ? WHERE updated_at = 0",
            &[JsValue::from_f64(now_ms as f64)],
        )
        .await?;

        // Backfill completed_sets: set updated_at for any rows that have the default 0 value.
        self.execute_internal(
            "UPDATE completed_sets SET updated_at = ? WHERE updated_at = 0",
            &[JsValue::from_f64(now_ms as f64)],
        )
        .await?;

        // Backfill UUIDs for exercises that still have the empty-string default.
        let exercises_needing_uuid = self
            .execute_internal("SELECT id FROM exercises WHERE uuid = ''", &[])
            .await?;
        if let Some(arr) = exercises_needing_uuid.dyn_ref::<js_sys::Array>() {
            for i in 0..arr.length() {
                let row = arr.get(i);
                let id = js_sys::Reflect::get(&row, &JsValue::from_str("id"))
                    .unwrap_or(JsValue::NULL)
                    .as_f64()
                    .unwrap_or(0.0);
                let new_uuid = Uuid::new_v4().to_string();
                let _ = self
                    .execute_internal(
                        "UPDATE exercises SET uuid = ? WHERE id = ?",
                        &[JsValue::from_str(&new_uuid), JsValue::from_f64(id)],
                    )
                    .await;
            }
        }

        // Backfill UUIDs for completed_sets that still have the empty-string default.
        let sets_needing_uuid = self
            .execute_internal("SELECT id FROM completed_sets WHERE uuid = ''", &[])
            .await?;
        if let Some(arr) = sets_needing_uuid.dyn_ref::<js_sys::Array>() {
            for i in 0..arr.length() {
                let row = arr.get(i);
                let id = js_sys::Reflect::get(&row, &JsValue::from_str("id"))
                    .unwrap_or(JsValue::NULL)
                    .as_f64()
                    .unwrap_or(0.0);
                let new_uuid = Uuid::new_v4().to_string();
                let _ = self
                    .execute_internal(
                        "UPDATE completed_sets SET uuid = ? WHERE id = ?",
                        &[JsValue::from_str(&new_uuid), JsValue::from_f64(id)],
                    )
                    .await;
            }
        }

        Ok(())
    }

    async fn get_schema_version(&self) -> Result<i64, DatabaseError> {
        let result = self.execute_internal("PRAGMA user_version", &[]).await?;
        let array = match result.dyn_ref::<js_sys::Array>() {
            Some(a) => a,
            None => return Ok(0),
        };
        if array.length() == 0 {
            return Ok(0);
        }
        let row = array.get(0);
        let version = js_sys::Reflect::get(&row, &JsValue::from_str("user_version"))
            .unwrap_or(JsValue::from_f64(0.0))
            .as_f64()
            .unwrap_or(0.0) as i64;
        Ok(version)
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

    /// Log a single set for the given exercise. Records the current timestamp.
    pub async fn log_set(
        &self,
        exercise_id: i64,
        set: &CompletedSet,
    ) -> Result<i64, DatabaseError> {
        let (weight, is_bodyweight) = match set.set_type {
            SetType::Weighted { weight } => (Some(weight), false),
            SetType::Bodyweight => (None, true),
        };

        let now = js_sys::Date::now();
        let new_uuid = Uuid::new_v4().to_string();

        let sql = r#"
            INSERT INTO completed_sets (uuid, exercise_id, set_number, reps, rpe, weight, is_bodyweight, recorded_at, updated_at)
            VALUES (?, ?, ?, ?, ?, ?, ?, ?, ?)
            RETURNING id
        "#;

        let params = vec![
            JsValue::from_str(&new_uuid),
            JsValue::from_f64(exercise_id as f64),
            JsValue::from_f64(set.set_number as f64),
            JsValue::from_f64(set.reps as f64),
            JsValue::from_f64(set.rpe as f64),
            weight
                .map(|w| JsValue::from_f64(w as f64))
                .unwrap_or(JsValue::NULL),
            JsValue::from_bool(is_bodyweight),
            JsValue::from_f64(now),
            JsValue::from_f64(now),
        ];

        let result = self.execute(sql, &params).await?;
        self.extract_id(&result, "set")
    }

    /// Log a single set with an explicit timestamp (Unix ms). Used in tests and
    /// data-import scenarios where the recording time is known.
    pub async fn log_set_at(
        &self,
        exercise_id: i64,
        set: &CompletedSet,
        recorded_at: f64,
    ) -> Result<i64, DatabaseError> {
        let (weight, is_bodyweight) = match set.set_type {
            SetType::Weighted { weight } => (Some(weight), false),
            SetType::Bodyweight => (None, true),
        };

        let now = js_sys::Date::now();
        let new_uuid = Uuid::new_v4().to_string();

        let sql = r#"
            INSERT INTO completed_sets (uuid, exercise_id, set_number, reps, rpe, weight, is_bodyweight, recorded_at, updated_at)
            VALUES (?, ?, ?, ?, ?, ?, ?, ?, ?)
            RETURNING id
        "#;

        let params = vec![
            JsValue::from_str(&new_uuid),
            JsValue::from_f64(exercise_id as f64),
            JsValue::from_f64(set.set_number as f64),
            JsValue::from_f64(set.reps as f64),
            JsValue::from_f64(set.rpe as f64),
            weight
                .map(|w| JsValue::from_f64(w as f64))
                .unwrap_or(JsValue::NULL),
            JsValue::from_bool(is_bodyweight),
            JsValue::from_f64(recorded_at),
            JsValue::from_f64(now),
        ];

        let result = self.execute(sql, &params).await?;
        self.extract_id(&result, "set")
    }

    /// Returns sets for one exercise in reverse-chronological order with pagination.
    pub async fn get_sets_for_exercise(
        &self,
        exercise_id: i64,
        limit: i64,
        offset: i64,
    ) -> Result<Vec<HistorySet>, DatabaseError> {
        let sql = r#"
            SELECT cs.id, cs.exercise_id, e.name AS exercise_name,
                   cs.set_number, cs.reps, cs.rpe, cs.weight, cs.is_bodyweight, cs.recorded_at
            FROM completed_sets cs
            JOIN exercises e ON cs.exercise_id = e.id
            WHERE cs.exercise_id = ? AND cs.deleted_at IS NULL
            ORDER BY cs.recorded_at DESC, cs.id DESC
            LIMIT ? OFFSET ?
        "#;

        let params = vec![
            JsValue::from_f64(exercise_id as f64),
            JsValue::from_f64(limit as f64),
            JsValue::from_f64(offset as f64),
        ];

        let result = self.execute(sql, &params).await?;
        self.parse_history_sets(&result)
    }

    /// Returns sets for one exercise recorded **before** `before_ms` (Unix ms),
    /// in reverse-chronological order with pagination.
    ///
    /// Used by the "Previous Sessions" panel so that sets logged during the
    /// current (today's) session are not shown alongside historical data.
    pub async fn get_sets_for_exercise_before(
        &self,
        exercise_id: i64,
        before_ms: f64,
        limit: i64,
        offset: i64,
    ) -> Result<Vec<HistorySet>, DatabaseError> {
        let sql = r#"
            SELECT cs.id, cs.exercise_id, e.name AS exercise_name,
                   cs.set_number, cs.reps, cs.rpe, cs.weight, cs.is_bodyweight, cs.recorded_at
            FROM completed_sets cs
            JOIN exercises e ON cs.exercise_id = e.id
            WHERE cs.exercise_id = ? AND cs.recorded_at < ? AND cs.deleted_at IS NULL
            ORDER BY cs.recorded_at DESC, cs.id DESC
            LIMIT ? OFFSET ?
        "#;

        let params = vec![
            JsValue::from_f64(exercise_id as f64),
            JsValue::from_f64(before_ms),
            JsValue::from_f64(limit as f64),
            JsValue::from_f64(offset as f64),
        ];

        let result = self.execute(sql, &params).await?;
        self.parse_history_sets(&result)
    }

    /// Returns sets across all exercises in reverse-chronological order with pagination.
    pub async fn get_all_sets_paginated(
        &self,
        limit: i64,
        offset: i64,
    ) -> Result<Vec<HistorySet>, DatabaseError> {
        let sql = r#"
            SELECT cs.id, cs.exercise_id, e.name AS exercise_name,
                   cs.set_number, cs.reps, cs.rpe, cs.weight, cs.is_bodyweight, cs.recorded_at
            FROM completed_sets cs
            JOIN exercises e ON cs.exercise_id = e.id
            WHERE cs.deleted_at IS NULL
            ORDER BY cs.recorded_at DESC, cs.id DESC
            LIMIT ? OFFSET ?
        "#;

        let params = vec![
            JsValue::from_f64(limit as f64),
            JsValue::from_f64(offset as f64),
        ];

        let result = self.execute(sql, &params).await?;
        self.parse_history_sets(&result)
    }

    /// Updates reps, rpe, weight, and recorded_at for an existing set.
    /// Also updates `updated_at` to the current timestamp.
    pub async fn update_set(
        &self,
        set_id: i64,
        reps: u32,
        rpe: f32,
        weight: Option<f32>,
        recorded_at: f64,
    ) -> Result<(), DatabaseError> {
        let (weight_val, is_bodyweight) = match weight {
            Some(w) => (JsValue::from_f64(w as f64), false),
            None => (JsValue::NULL, true),
        };

        let now = js_sys::Date::now();

        let sql = r#"
            UPDATE completed_sets
            SET reps = ?, rpe = ?, weight = ?, is_bodyweight = ?, recorded_at = ?, updated_at = ?
            WHERE id = ?
        "#;

        let params = vec![
            JsValue::from_f64(reps as f64),
            JsValue::from_f64(rpe as f64),
            weight_val,
            JsValue::from_bool(is_bodyweight),
            JsValue::from_f64(recorded_at),
            JsValue::from_f64(now),
            JsValue::from_f64(set_id as f64),
        ];

        self.execute(sql, &params).await?;
        Ok(())
    }

    /// Soft-deletes a set by setting its `deleted_at` timestamp.
    /// The row is retained in the database for sync purposes.
    pub async fn delete_set(&self, set_id: i64) -> Result<(), DatabaseError> {
        let now = js_sys::Date::now();
        let sql = "UPDATE completed_sets SET deleted_at = ?, updated_at = ? WHERE id = ?";
        let params = vec![
            JsValue::from_f64(now),
            JsValue::from_f64(now),
            JsValue::from_f64(set_id as f64),
        ];
        self.execute(sql, &params).await?;
        Ok(())
    }

    pub async fn save_exercise(&self, exercise: &ExerciseMetadata) -> Result<i64, DatabaseError> {
        let (is_weighted, min_weight, increment) = match exercise.set_type_config {
            crate::models::SetTypeConfig::Weighted {
                min_weight,
                increment,
            } => (true, Some(min_weight), Some(increment)),
            crate::models::SetTypeConfig::Bodyweight => (false, None, None),
        };

        let now = js_sys::Date::now();

        let result = if let Some(id) = exercise.id {
            let sql = r#"
                UPDATE exercises SET name = ?, is_weighted = ?, min_weight = ?, increment = ?, updated_at = ?
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
                JsValue::from_f64(now),
                JsValue::from_f64(id as f64),
            ];
            self.execute(sql, &params).await?
        } else {
            let new_uuid = Uuid::new_v4().to_string();
            let sql = r#"
                INSERT INTO exercises (uuid, name, is_weighted, min_weight, increment, updated_at)
                VALUES (?, ?, ?, ?, ?, ?)
                ON CONFLICT(name) DO UPDATE SET
                    is_weighted = excluded.is_weighted,
                    min_weight = excluded.min_weight,
                    increment = excluded.increment,
                    updated_at = excluded.updated_at
                RETURNING id
            "#;
            let params = vec![
                JsValue::from_str(&new_uuid),
                JsValue::from_str(&exercise.name),
                JsValue::from_bool(is_weighted),
                min_weight
                    .map(|w| JsValue::from_f64(w as f64))
                    .unwrap_or(JsValue::NULL),
                increment
                    .map(|i| JsValue::from_f64(i as f64))
                    .unwrap_or(JsValue::NULL),
                JsValue::from_f64(now),
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
        let sql = "SELECT id, name, is_weighted, min_weight, increment FROM exercises WHERE deleted_at IS NULL ORDER BY name";
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

    /// Returns the most recent set for the given exercise (used for predictions).
    pub async fn get_last_set_for_exercise(
        &self,
        exercise_id: i64,
    ) -> Result<Option<crate::models::CompletedSet>, DatabaseError> {
        let sql = r#"
            SELECT set_number, reps, rpe, weight, is_bodyweight
            FROM completed_sets
            WHERE exercise_id = ? AND deleted_at IS NULL
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

        let is_bodyweight = self.parse_bool_field(&row, "is_bodyweight")?;

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

    // ── Private helpers ───────────────────────────────────────────────────────

    fn extract_id(&self, result: &JsValue, label: &str) -> Result<i64, DatabaseError> {
        let array = result
            .dyn_ref::<js_sys::Array>()
            .ok_or_else(|| DatabaseError::QueryError("Expected array result".to_string()))?;

        if array.length() == 0 {
            return Err(DatabaseError::QueryError("No rows returned".to_string()));
        }

        let first_row = array.get(0);
        let id = js_sys::Reflect::get(&first_row, &JsValue::from_str("id"))?
            .as_f64()
            .ok_or_else(|| DatabaseError::QueryError(format!("Failed to get {} id", label)))?
            as i64;

        Ok(id)
    }

    fn parse_bool_field(&self, row: &JsValue, field: &str) -> Result<bool, DatabaseError> {
        let val = js_sys::Reflect::get(row, &JsValue::from_str(field))?;
        if let Some(b) = val.as_bool() {
            Ok(b)
        } else if let Some(f) = val.as_f64() {
            Ok(f == 1.0)
        } else {
            Err(DatabaseError::QueryError(format!(
                "Failed to get {} as bool or number",
                field
            )))
        }
    }

    fn parse_history_sets(&self, result: &JsValue) -> Result<Vec<HistorySet>, DatabaseError> {
        let array = result
            .dyn_ref::<js_sys::Array>()
            .ok_or_else(|| DatabaseError::QueryError("Expected array result".to_string()))?;

        let mut sets = Vec::new();
        for i in 0..array.length() {
            let row = array.get(i);

            let id = js_sys::Reflect::get(&row, &JsValue::from_str("id"))?
                .as_f64()
                .ok_or_else(|| DatabaseError::QueryError("Failed to get id".to_string()))?
                as i64;

            let exercise_id = js_sys::Reflect::get(&row, &JsValue::from_str("exercise_id"))?
                .as_f64()
                .ok_or_else(|| DatabaseError::QueryError("Failed to get exercise_id".to_string()))?
                as i64;

            let exercise_name = js_sys::Reflect::get(&row, &JsValue::from_str("exercise_name"))?
                .as_string()
                .ok_or_else(|| {
                    DatabaseError::QueryError("Failed to get exercise_name".to_string())
                })?;

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

            let recorded_at = js_sys::Reflect::get(&row, &JsValue::from_str("recorded_at"))?
                .as_f64()
                .ok_or_else(|| {
                    DatabaseError::QueryError("Failed to get recorded_at".to_string())
                })?;

            let is_bodyweight = self.parse_bool_field(&row, "is_bodyweight")?;

            let set_type = if is_bodyweight {
                SetType::Bodyweight
            } else {
                let weight = js_sys::Reflect::get(&row, &JsValue::from_str("weight"))?
                    .as_f64()
                    .ok_or_else(|| DatabaseError::QueryError("Failed to get weight".to_string()))?
                    as f32;
                SetType::Weighted { weight }
            };

            sets.push(HistorySet {
                id,
                exercise_id,
                exercise_name,
                set_number,
                reps,
                rpe,
                set_type,
                recorded_at,
            });
        }

        Ok(sets)
    }
}

impl Default for Database {
    fn default() -> Self {
        Self::new()
    }
}
