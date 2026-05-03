use crate::models::{
    CompletedSet, ExerciseMetadata, HistorySet, PlanExercise, SetType, SetTypeConfig, WorkoutPlan,
    WorkoutTemplate,
};
use thiserror::Error;
#[cfg(test)]
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

    #[wasm_bindgen(js_name = downloadBytes)]
    async fn download_bytes(data: &[u8], filename: &str) -> JsValue;

    #[wasm_bindgen(js_name = importDatabase)]
    async fn import_database(file_data: Vec<u8>) -> JsValue;

    #[wasm_bindgen(js_name = ensureCrrTables)]
    async fn ensure_crr_tables() -> JsValue;

    /// Returns the error message from the most recent failed `initDatabase()` call.
    /// Returns an empty string when there is no error.
    #[wasm_bindgen(js_name = getDbInitError)]
    fn get_db_init_error() -> String;

    /// Returns true when the sync module could not be loaded during init.
    /// The app is still functional; sync is simply unavailable.
    #[wasm_bindgen(js_name = isSyncUnavailable)]
    fn is_sync_unavailable() -> bool;
}

/// Current schema version. Bump this when the schema changes.
const SCHEMA_VERSION: i64 = 7;

#[derive(Clone, PartialEq)]
pub struct Database {
    initialized: bool,
    /// Set to true when the sync module failed to load during `init()`.
    /// The database is fully functional; only sync is affected.
    pub sync_unavailable: bool,
}

impl Database {
    pub fn new() -> Self {
        Self {
            initialized: false,
            sync_unavailable: false,
        }
    }

    pub async fn init(&mut self, file_data: Option<Vec<u8>>) -> Result<(), DatabaseError> {
        log::debug!("[DB] Calling JS initDatabase...");
        let result = init_database(file_data).await;

        if result.is_truthy() {
            // Check whether the sync module failed to load (non-fatal).
            if is_sync_unavailable() {
                log::warn!("[DB] Sync module unavailable — sync will not function this session");
                self.sync_unavailable = true;
            }
            log::debug!("[DB] initDatabase succeeded, creating tables...");
            self.migrate_and_create_tables().await?;
            self.initialized = true;
            log::debug!("[DB] Tables created successfully and database initialized");
            Ok(())
        } else {
            // Retrieve the detailed error from JS to surface a useful message.
            let js_reason = get_db_init_error();
            let error_msg = if js_reason.is_empty() {
                "Failed to initialize SQLite database".to_string()
            } else {
                format!("Failed to initialize SQLite database: {}", js_reason)
            };
            log::error!("{}", error_msg);
            Err(DatabaseError::InitializationError(error_msg))
        }
    }

    /// Import a user-supplied SQLite file.  Unlike `init`, this always loads
    /// the provided bytes (it does not check the one-time migration sentinel).
    pub async fn import(&mut self, file_data: Vec<u8>) -> Result<(), DatabaseError> {
        log::debug!("[DB] Calling JS importDatabase...");
        let result = import_database(file_data).await;

        if result.is_truthy() {
            log::debug!("[DB] importDatabase succeeded, running migrations...");
            self.migrate_and_create_tables().await?;
            self.initialized = true;
            log::debug!("[DB] Import complete and database initialized");
            Ok(())
        } else {
            let error_msg = "Failed to import database - JS returned false".to_string();
            log::error!("{}", error_msg);
            Err(DatabaseError::InitializationError(error_msg))
        }
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

    /// Exports the database and triggers a browser download of the `.sqlite` file.
    ///
    /// Uses platform feature-detection to pick the best download strategy:
    /// Web Share API (Android), window.open fallback, or anchor click (desktop).
    /// Returns an error if all strategies fail.
    pub async fn download(&self, filename: &str) -> Result<(), DatabaseError> {
        let data = self.export().await?;
        let result = download_bytes(&data, filename).await;

        // downloadBytes returns { ok: bool, method?: string, error?: string }
        let ok = js_sys::Reflect::get(&result, &JsValue::from_str("ok"))
            .unwrap_or(JsValue::FALSE)
            .as_bool()
            .unwrap_or(false);

        if ok {
            Ok(())
        } else {
            let error_msg = js_sys::Reflect::get(&result, &JsValue::from_str("error"))
                .unwrap_or(JsValue::from_str("Unknown export error"))
                .as_string()
                .unwrap_or_else(|| "Unknown export error".to_string());
            Err(DatabaseError::JsError(error_msg))
        }
    }

    async fn migrate_and_create_tables(&self) -> Result<(), DatabaseError> {
        // Detect old schema by checking user_version pragma
        let current_version = self.get_schema_version().await.unwrap_or(0);
        log::debug!("[DB] Current schema version: {}", current_version);

        if current_version < 2 {
            log::debug!(
                "[DB] Migrating schema from v{} to v2: dropping incompatible tables",
                current_version
            );
            // Drop old tables that are incompatible with the v2 schema.
            // Exercises table is retained (compatible across versions).
            self.execute_internal("DROP TABLE IF EXISTS completed_sets", &[])
                .await?;
            self.execute_internal("DROP TABLE IF EXISTS sessions", &[])
                .await?;
        }

        // Create base tables (v2 schema) if they don't exist yet.
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

        // ── v3 migration: add sync-readiness columns ──────────────────────────
        // Runs for any database not yet at v3, including fresh databases
        // (whose CREATE TABLE statements use the v2 schema without sync columns)
        // and existing v2 databases that need the new columns added.
        if current_version < 3 {
            log::debug!("[DB] Applying v3 migration: adding sync columns");
            self.apply_v3_migration().await?;
        }

        // ── v4 migration: rep ranges + settings table ───────────────────────────
        if current_version < 4 {
            log::debug!("[DB] Applying v4 migration: rep ranges and settings table");
            self.apply_v4_migration().await?;
        }

        // ── v5 migration: CRR-compatible table schemas ──────────────────────────
        // crsqlite requires: explicit NOT NULL on PK (AUTOINCREMENT leaves
        // notnull=0 in pragma table_info), no UNIQUE indices besides PK, and
        // no CHECK constraints.  Rebuild tables to match the server schema.
        if current_version < 5 {
            log::debug!("[DB] Applying v5 migration: CRR-compatible table schemas");
            self.apply_v5_migration().await?;
        }

        // ── v6 migration: UUID primary key for exercises ────────────────────────
        if current_version < 6 {
            log::debug!("[DB] Applying v6 migration: UUID primary key for exercises");
            self.apply_v6_migration().await?;
        }

        // ── v7 migration: workout plans, templates, and default_planned_sets ─
        if current_version < 7 {
            log::debug!("[DB] Applying v7 migration: workout plans and templates tables");
            self.apply_v7_migration().await?;
        }

        // Stamp the new version
        self.execute_internal(&format!("PRAGMA user_version = {}", SCHEMA_VERSION), &[])
            .await?;

        // Mark tables as CRRs now that they exist.  applyCrrMigration() in
        // db-module.js runs during initDatabase() — before Rust creates the
        // tables — so we must re-run it here.
        log::debug!("[DB] Ensuring CRR tables are marked after schema migration");
        let crr_result = ensure_crr_tables().await;
        if !crr_result.is_truthy() {
            log::warn!("[DB] ensureCrrTables returned false — CRR marking may have failed");
        }

        Ok(())
    }

    /// Adds `uuid`, `updated_at`, and `deleted_at` columns to both record tables,
    /// then backfills existing rows.  Uses ADD COLUMN (non-destructive) so that
    /// any pre-existing data is preserved.
    async fn apply_v3_migration(&self) -> Result<(), DatabaseError> {
        let now = js_sys::Date::now() as i64;

        // Add columns to exercises (suppress only "duplicate column" errors).
        self.add_column_if_missing(
            "ALTER TABLE exercises ADD COLUMN uuid TEXT NOT NULL DEFAULT ''",
        )
        .await?;
        self.add_column_if_missing(
            "ALTER TABLE exercises ADD COLUMN updated_at INTEGER NOT NULL DEFAULT 0",
        )
        .await?;
        self.add_column_if_missing("ALTER TABLE exercises ADD COLUMN deleted_at INTEGER")
            .await?;

        // Backfill existing exercises that still have an empty uuid.
        let existing_exercises = self
            .execute_internal(
                "SELECT id FROM exercises WHERE uuid = '' OR uuid IS NULL",
                &[],
            )
            .await?;

        if let Some(arr) = existing_exercises.dyn_ref::<js_sys::Array>() {
            for i in 0..arr.length() {
                let row = arr.get(i);
                let id_val = js_sys::Reflect::get(&row, &JsValue::from_str("id"))
                    .unwrap_or(JsValue::NULL)
                    .as_f64()
                    .unwrap_or(0.0);
                if id_val == 0.0 {
                    continue;
                }
                let uuid = Self::generate_uuid();
                let _ = self
                    .execute_internal(
                        "UPDATE exercises SET uuid = ?, updated_at = ? WHERE id = ?",
                        &[
                            JsValue::from_str(&uuid),
                            JsValue::from_f64(now as f64),
                            JsValue::from_f64(id_val),
                        ],
                    )
                    .await;
            }
        }

        // Add columns to completed_sets (suppress only "duplicate column" errors).
        self.add_column_if_missing(
            "ALTER TABLE completed_sets ADD COLUMN uuid TEXT NOT NULL DEFAULT ''",
        )
        .await?;
        self.add_column_if_missing(
            "ALTER TABLE completed_sets ADD COLUMN updated_at INTEGER NOT NULL DEFAULT 0",
        )
        .await?;
        self.add_column_if_missing("ALTER TABLE completed_sets ADD COLUMN deleted_at INTEGER")
            .await?;

        // Backfill existing sets.
        let existing_sets = self
            .execute_internal(
                "SELECT id FROM completed_sets WHERE uuid = '' OR uuid IS NULL",
                &[],
            )
            .await?;

        if let Some(arr) = existing_sets.dyn_ref::<js_sys::Array>() {
            for i in 0..arr.length() {
                let row = arr.get(i);
                let id_val = js_sys::Reflect::get(&row, &JsValue::from_str("id"))
                    .unwrap_or(JsValue::NULL)
                    .as_f64()
                    .unwrap_or(0.0);
                if id_val == 0.0 {
                    continue;
                }
                let uuid = Self::generate_uuid();
                let _ = self
                    .execute_internal(
                        "UPDATE completed_sets SET uuid = ?, updated_at = ? WHERE id = ?",
                        &[
                            JsValue::from_str(&uuid),
                            JsValue::from_f64(now as f64),
                            JsValue::from_f64(id_val),
                        ],
                    )
                    .await;
            }
        }

        log::debug!("[DB] v3 migration complete");
        Ok(())
    }

    /// Adds `min_reps` and `max_reps` columns to the exercises table,
    /// creates the `settings` table, and auto-seeds a default settings row.
    async fn apply_v4_migration(&self) -> Result<(), DatabaseError> {
        // Add rep-range columns to exercises.
        self.add_column_if_missing(
            "ALTER TABLE exercises ADD COLUMN min_reps INTEGER NOT NULL DEFAULT 1",
        )
        .await?;
        self.add_column_if_missing("ALTER TABLE exercises ADD COLUMN max_reps INTEGER")
            .await?;

        // Create settings table.
        self.execute_internal(
            r#"
            CREATE TABLE IF NOT EXISTS settings (
                id INTEGER PRIMARY KEY CHECK (id = 1),
                target_rpe REAL NOT NULL DEFAULT 8.0,
                history_window_days INTEGER NOT NULL DEFAULT 30,
                today_blend_factor REAL NOT NULL DEFAULT 0.5
            )
            "#,
            &[],
        )
        .await?;

        // Seed default settings row if absent.
        self.seed_settings().await?;

        log::debug!("[DB] v4 migration complete");
        Ok(())
    }

    /// Rebuild tables to be CRR-compatible:
    ///   - `INTEGER PRIMARY KEY NOT NULL` (explicit NOT NULL so pragma table_info
    ///     reports notnull=1, which crsqlite requires)
    ///   - No UNIQUE indices besides the primary key
    ///   - No CHECK constraints
    ///   - No FOREIGN KEY constraints (CRRs manage their own consistency)
    async fn apply_v5_migration(&self) -> Result<(), DatabaseError> {
        // ── exercises ───────────────────────────────────────────────────────
        self.execute_internal(
            r#"CREATE TABLE IF NOT EXISTS exercises_v5 (
                id INTEGER PRIMARY KEY NOT NULL,
                name TEXT NOT NULL DEFAULT '',
                is_weighted INTEGER NOT NULL DEFAULT 0,
                min_weight REAL,
                increment REAL,
                uuid TEXT NOT NULL DEFAULT '',
                updated_at INTEGER NOT NULL DEFAULT 0,
                deleted_at INTEGER,
                min_reps INTEGER NOT NULL DEFAULT 1,
                max_reps INTEGER
            )"#,
            &[],
        )
        .await?;
        self.execute_internal(
            "INSERT OR IGNORE INTO exercises_v5 SELECT id, name, is_weighted, min_weight, increment, uuid, updated_at, deleted_at, min_reps, max_reps FROM exercises",
            &[],
        )
        .await?;
        self.execute_internal("DROP TABLE exercises", &[]).await?;
        self.execute_internal("ALTER TABLE exercises_v5 RENAME TO exercises", &[])
            .await?;

        // ── completed_sets ──────────────────────────────────────────────────
        self.execute_internal(
            r#"CREATE TABLE IF NOT EXISTS completed_sets_v5 (
                id INTEGER PRIMARY KEY NOT NULL,
                exercise_id INTEGER NOT NULL DEFAULT 0,
                set_number INTEGER NOT NULL DEFAULT 0,
                reps INTEGER NOT NULL DEFAULT 0,
                rpe REAL NOT NULL DEFAULT 0.0,
                weight REAL,
                is_bodyweight INTEGER NOT NULL DEFAULT 0,
                recorded_at INTEGER NOT NULL DEFAULT 0,
                uuid TEXT NOT NULL DEFAULT '',
                updated_at INTEGER NOT NULL DEFAULT 0,
                deleted_at INTEGER
            )"#,
            &[],
        )
        .await?;
        self.execute_internal(
            "INSERT OR IGNORE INTO completed_sets_v5 SELECT id, exercise_id, set_number, reps, rpe, weight, is_bodyweight, recorded_at, uuid, updated_at, deleted_at FROM completed_sets",
            &[],
        )
        .await?;
        self.execute_internal("DROP TABLE completed_sets", &[])
            .await?;
        self.execute_internal(
            "ALTER TABLE completed_sets_v5 RENAME TO completed_sets",
            &[],
        )
        .await?;
        // Re-create the index (dropped with the old table).
        self.execute_internal(
            "CREATE INDEX IF NOT EXISTS idx_sets_exercise_id ON completed_sets(exercise_id)",
            &[],
        )
        .await?;

        // ── settings ────────────────────────────────────────────────────────
        self.execute_internal(
            r#"CREATE TABLE IF NOT EXISTS settings_v5 (
                id INTEGER PRIMARY KEY NOT NULL,
                target_rpe REAL NOT NULL DEFAULT 8.0,
                history_window_days INTEGER NOT NULL DEFAULT 30,
                today_blend_factor REAL NOT NULL DEFAULT 0.5
            )"#,
            &[],
        )
        .await?;
        self.execute_internal(
            "INSERT OR IGNORE INTO settings_v5 SELECT id, target_rpe, history_window_days, today_blend_factor FROM settings",
            &[],
        )
        .await?;
        self.execute_internal("DROP TABLE settings", &[]).await?;
        self.execute_internal("ALTER TABLE settings_v5 RENAME TO settings", &[])
            .await?;
        // Re-seed in case settings was empty.
        self.seed_settings().await?;

        log::debug!("[DB] v5 migration complete — tables are now CRR-compatible");
        Ok(())
    }

    /// V6: Migrate exercises to UUID primary key for CRR compatibility.
    ///
    /// INTEGER PRIMARY KEY causes CRR collisions when two devices independently
    /// create exercises (both get id=1). Using UUID as PK ensures globally unique
    /// identifiers.
    ///
    /// Also changes completed_sets.exercise_id from INTEGER to TEXT (UUID reference).
    async fn apply_v6_migration(&self) -> Result<(), DatabaseError> {
        // Ensure all exercises have a uuid before migration
        let exercises_without_uuid = self
            .execute_internal(
                "SELECT id FROM exercises WHERE uuid = '' OR uuid IS NULL",
                &[],
            )
            .await?;
        if let Some(arr) = exercises_without_uuid.dyn_ref::<js_sys::Array>() {
            for i in 0..arr.length() {
                let row = arr.get(i);
                let id = js_sys::Reflect::get(&row, &JsValue::from_str("id"))
                    .ok()
                    .and_then(|v| v.as_f64());
                if let Some(id) = id {
                    let uuid = Self::generate_uuid();
                    self.execute_internal(
                        "UPDATE exercises SET uuid = ? WHERE id = ?",
                        &[JsValue::from_str(&uuid), JsValue::from_f64(id)],
                    )
                    .await?;
                }
            }
        }

        // Rebuild exercises with uuid as PRIMARY KEY
        self.execute_internal(
            "CREATE TABLE IF NOT EXISTS exercises_v6 (
                uuid TEXT PRIMARY KEY NOT NULL,
                name TEXT NOT NULL DEFAULT '',
                is_weighted INTEGER NOT NULL DEFAULT 0,
                min_weight REAL,
                increment REAL,
                updated_at INTEGER NOT NULL DEFAULT 0,
                deleted_at INTEGER,
                min_reps INTEGER NOT NULL DEFAULT 1,
                max_reps INTEGER
            )",
            &[],
        )
        .await?;
        self.execute_internal(
            "INSERT OR IGNORE INTO exercises_v6 SELECT uuid, name, is_weighted, min_weight, increment, updated_at, deleted_at, min_reps, max_reps FROM exercises",
            &[],
        )
        .await?;

        // Rebuild completed_sets with TEXT exercise_id (uuid reference)
        self.execute_internal(
            "CREATE TABLE IF NOT EXISTS completed_sets_v6 (
                id INTEGER PRIMARY KEY NOT NULL,
                exercise_id TEXT NOT NULL DEFAULT '',
                set_number INTEGER NOT NULL DEFAULT 0,
                reps INTEGER NOT NULL DEFAULT 0,
                rpe REAL NOT NULL DEFAULT 0.0,
                weight REAL,
                is_bodyweight INTEGER NOT NULL DEFAULT 0,
                recorded_at INTEGER NOT NULL DEFAULT 0,
                uuid TEXT NOT NULL DEFAULT '',
                updated_at INTEGER NOT NULL DEFAULT 0,
                deleted_at INTEGER
            )",
            &[],
        )
        .await?;
        // Check for orphaned completed_sets (exercise_id pointing to deleted exercises)
        let orphan_result = self
            .execute_internal(
                "SELECT count(*) as cnt FROM completed_sets cs WHERE NOT EXISTS (SELECT 1 FROM exercises e WHERE e.id = cs.exercise_id)",
                &[],
            )
            .await;
        if let Ok(ref v) = orphan_result
            && let Some(arr) = v.dyn_ref::<js_sys::Array>()
            && arr.length() > 0
        {
            let cnt = js_sys::Reflect::get(&arr.get(0), &JsValue::from_str("cnt"))
                .ok()
                .and_then(|v| v.as_f64())
                .unwrap_or(0.0);
            if cnt > 0.0 {
                #[cfg(target_arch = "wasm32")]
                web_sys::console::warn_1(&JsValue::from_str(&format!(
                    "[DB] v6 migration: {} orphaned completed_sets rows will lose exercise linkage",
                    cnt
                )));
            }
        }

        self.execute_internal(
            "INSERT OR IGNORE INTO completed_sets_v6 SELECT cs.id, e.uuid, cs.set_number, cs.reps, cs.rpe, cs.weight, cs.is_bodyweight, cs.recorded_at, cs.uuid, cs.updated_at, cs.deleted_at FROM completed_sets cs INNER JOIN exercises e ON cs.exercise_id = e.id",
            &[],
        )
        .await?;

        // Drop old tables and rename
        self.execute_internal("DROP TABLE IF EXISTS completed_sets", &[])
            .await?;
        self.execute_internal("DROP TABLE IF EXISTS exercises", &[])
            .await?;
        self.execute_internal("ALTER TABLE exercises_v6 RENAME TO exercises", &[])
            .await?;
        self.execute_internal(
            "ALTER TABLE completed_sets_v6 RENAME TO completed_sets",
            &[],
        )
        .await?;

        // Recreate index
        self.execute_internal(
            "CREATE INDEX IF NOT EXISTS idx_sets_exercise_id ON completed_sets(exercise_id)",
            &[],
        )
        .await?;

        log::debug!("[DB] v6 migration complete — exercises now use UUID primary key");
        Ok(())
    }

    /// v7 migration: create workout_plans, workout_plan_exercises,
    /// workout_templates, workout_template_exercises tables and add
    /// default_planned_sets to settings.
    async fn apply_v7_migration(&self) -> Result<(), DatabaseError> {
        self.execute_internal(
            "CREATE TABLE IF NOT EXISTS workout_plans (
                id TEXT PRIMARY KEY NOT NULL,
                started_at INTEGER,
                ended_at INTEGER,
                updated_at INTEGER NOT NULL DEFAULT 0,
                deleted_at INTEGER
            )",
            &[],
        )
        .await?;

        self.execute_internal(
            "CREATE TABLE IF NOT EXISTS workout_plan_exercises (
                id TEXT PRIMARY KEY NOT NULL,
                plan_id TEXT NOT NULL DEFAULT '',
                exercise_id TEXT NOT NULL DEFAULT '',
                planned_sets INTEGER NOT NULL DEFAULT 1,
                position INTEGER NOT NULL DEFAULT 0,
                updated_at INTEGER NOT NULL DEFAULT 0,
                deleted_at INTEGER
            )",
            &[],
        )
        .await?;

        self.execute_internal(
            "CREATE TABLE IF NOT EXISTS workout_templates (
                id TEXT PRIMARY KEY NOT NULL,
                name TEXT NOT NULL DEFAULT '',
                updated_at INTEGER NOT NULL DEFAULT 0,
                deleted_at INTEGER
            )",
            &[],
        )
        .await?;

        self.execute_internal(
            "CREATE TABLE IF NOT EXISTS workout_template_exercises (
                id TEXT PRIMARY KEY NOT NULL,
                template_id TEXT NOT NULL DEFAULT '',
                exercise_id TEXT NOT NULL DEFAULT '',
                planned_sets INTEGER NOT NULL DEFAULT 1,
                position INTEGER NOT NULL DEFAULT 0,
                updated_at INTEGER NOT NULL DEFAULT 0,
                deleted_at INTEGER
            )",
            &[],
        )
        .await?;

        self.add_column_if_missing(
            "ALTER TABLE settings ADD COLUMN default_planned_sets INTEGER NOT NULL DEFAULT 3",
        )
        .await?;

        log::debug!("[DB] v7 migration complete — workout plans and templates tables created");
        Ok(())
    }

    /// Inserts the default settings row if no row exists yet.
    /// Uses a SELECT guard instead of INSERT OR IGNORE because CRR tables
    /// don't support ON CONFLICT clauses.
    async fn seed_settings(&self) -> Result<(), DatabaseError> {
        let existing = self
            .execute_internal("SELECT id FROM settings WHERE id = 1", &[])
            .await?;
        let has_row = existing
            .dyn_ref::<js_sys::Array>()
            .map(|a| a.length() > 0)
            .unwrap_or(false);
        if !has_row {
            self.execute_internal(
                "INSERT INTO settings (id, target_rpe, history_window_days, today_blend_factor) VALUES (1, 8.0, 30, 0.5)",
                &[],
            )
            .await?;
        }
        Ok(())
    }

    /// Executes an ALTER TABLE ADD COLUMN statement, suppressing only
    /// "duplicate column" errors (which mean the column already exists).
    /// Any other error is propagated.
    ///
    /// Note: Detection relies on the English error message "duplicate column"
    /// returned by SQLite for `SQLITE_ERROR` on `ALTER TABLE ADD COLUMN` when
    /// the column already exists.  SQLite error messages are not localised, so
    /// this is stable across platforms, but it is version-sensitive in
    /// principle. The crsqlite-wasm build pins the SQLite version, mitigating
    /// this risk.
    async fn add_column_if_missing(&self, sql: &str) -> Result<(), DatabaseError> {
        match self.execute_internal(sql, &[]).await {
            Ok(_) => Ok(()),
            Err(DatabaseError::QueryError(msg)) if msg.contains("duplicate column") => {
                log::debug!("[DB] Column already exists, skipping: {}", sql);
                Ok(())
            }
            Err(e) => Err(e),
        }
    }

    /// Generate a UUID v4 string using the browser's crypto API.
    fn generate_uuid() -> String {
        // Use crypto.randomUUID() if available (all modern browsers).
        let global = js_sys::global();
        if let Ok(crypto) = js_sys::Reflect::get(&global, &JsValue::from_str("crypto"))
            && let Ok(uuid_fn) = js_sys::Reflect::get(&crypto, &JsValue::from_str("randomUUID"))
            && let Some(f) = uuid_fn.dyn_ref::<js_sys::Function>()
            && let Ok(result) = f.call0(&crypto)
            && let Some(s) = result.as_string()
        {
            return s;
        }

        // Fallback: construct a UUID-shaped string from Math.random().
        let r = || (js_sys::Math::random() * 65535.0_f64).floor() as u32;
        format!(
            "{:04x}{:04x}-{:04x}-4{:03x}-{:04x}-{:04x}{:04x}{:04x}",
            r(),
            r(),
            r(),
            r() & 0x0fff,
            (r() & 0x3fff) | 0x8000,
            r(),
            r(),
            r()
        )
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
        exercise_id: &str,
        set: &CompletedSet,
    ) -> Result<i64, DatabaseError> {
        let (weight, is_bodyweight) = match set.set_type {
            SetType::Weighted { weight } => (Some(weight), false),
            SetType::Bodyweight => (None, true),
        };

        let now = js_sys::Date::now();
        let uuid = Self::generate_uuid();

        let sql = r#"
            INSERT INTO completed_sets (exercise_id, set_number, reps, rpe, weight, is_bodyweight, recorded_at, uuid, updated_at)
            VALUES (?, ?, ?, ?, ?, ?, ?, ?, ?)
            RETURNING id
        "#;

        let params = vec![
            JsValue::from_str(exercise_id),
            JsValue::from_f64(set.set_number as f64),
            JsValue::from_f64(set.reps as f64),
            JsValue::from_f64(set.rpe as f64),
            weight
                .map(|w| JsValue::from_f64(w as f64))
                .unwrap_or(JsValue::NULL),
            JsValue::from_bool(is_bodyweight),
            JsValue::from_f64(now),
            JsValue::from_str(&uuid),
            JsValue::from_f64(now),
        ];

        let result = self.execute(sql, &params).await?;
        self.extract_id(&result, "set")
    }

    /// Log a single set with an explicit timestamp (Unix ms). Used in tests and
    /// data-import scenarios where the recording time is known.
    pub async fn log_set_at(
        &self,
        exercise_id: &str,
        set: &CompletedSet,
        recorded_at: f64,
    ) -> Result<i64, DatabaseError> {
        let (weight, is_bodyweight) = match set.set_type {
            SetType::Weighted { weight } => (Some(weight), false),
            SetType::Bodyweight => (None, true),
        };

        let now = js_sys::Date::now();
        let uuid = Self::generate_uuid();

        let sql = r#"
            INSERT INTO completed_sets (exercise_id, set_number, reps, rpe, weight, is_bodyweight, recorded_at, uuid, updated_at)
            VALUES (?, ?, ?, ?, ?, ?, ?, ?, ?)
            RETURNING id
        "#;

        let params = vec![
            JsValue::from_str(exercise_id),
            JsValue::from_f64(set.set_number as f64),
            JsValue::from_f64(set.reps as f64),
            JsValue::from_f64(set.rpe as f64),
            weight
                .map(|w| JsValue::from_f64(w as f64))
                .unwrap_or(JsValue::NULL),
            JsValue::from_bool(is_bodyweight),
            JsValue::from_f64(recorded_at),
            JsValue::from_str(&uuid),
            JsValue::from_f64(now),
        ];

        let result = self.execute(sql, &params).await?;
        self.extract_id(&result, "set")
    }

    /// Returns sets for one exercise in reverse-chronological order with pagination.
    pub async fn get_sets_for_exercise(
        &self,
        exercise_id: &str,
        limit: i64,
        offset: i64,
    ) -> Result<Vec<HistorySet>, DatabaseError> {
        let sql = r#"
            SELECT cs.id, cs.exercise_id, e.name AS exercise_name,
                   cs.set_number, cs.reps, cs.rpe, cs.weight, cs.is_bodyweight, cs.recorded_at
            FROM completed_sets cs
            JOIN exercises e ON cs.exercise_id = e.uuid
            WHERE cs.exercise_id = ? AND cs.deleted_at IS NULL
            ORDER BY cs.recorded_at DESC, cs.id DESC
            LIMIT ? OFFSET ?
        "#;

        let params = vec![
            JsValue::from_str(exercise_id),
            JsValue::from_f64(limit as f64),
            JsValue::from_f64(offset as f64),
        ];

        let result = self.execute(sql, &params).await?;
        self.parse_history_sets(&result)
    }

    /// Returns sets for one exercise recorded **before** `before_ms` (Unix ms),
    /// in reverse-chronological order with pagination.
    ///
    /// Useful for excluding recent sets so that only historical data is shown.
    pub async fn get_sets_for_exercise_before(
        &self,
        exercise_id: &str,
        before_ms: f64,
        limit: i64,
        offset: i64,
    ) -> Result<Vec<HistorySet>, DatabaseError> {
        let sql = r#"
            SELECT cs.id, cs.exercise_id, e.name AS exercise_name,
                   cs.set_number, cs.reps, cs.rpe, cs.weight, cs.is_bodyweight, cs.recorded_at
            FROM completed_sets cs
            JOIN exercises e ON cs.exercise_id = e.uuid
            WHERE cs.exercise_id = ? AND cs.recorded_at < ? AND cs.deleted_at IS NULL
            ORDER BY cs.recorded_at DESC, cs.id DESC
            LIMIT ? OFFSET ?
        "#;

        let params = vec![
            JsValue::from_str(exercise_id),
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
            JOIN exercises e ON cs.exercise_id = e.uuid
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
    /// The row is retained in the database but excluded from all normal queries.
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

    pub async fn save_exercise(
        &self,
        exercise: &ExerciseMetadata,
    ) -> Result<String, DatabaseError> {
        let (is_weighted, min_weight, increment) = match exercise.set_type_config {
            crate::models::SetTypeConfig::Weighted {
                min_weight,
                increment,
            } => (true, Some(min_weight), Some(increment)),
            crate::models::SetTypeConfig::Bodyweight => (false, None, None),
        };

        let now = js_sys::Date::now();
        let min_reps_val = JsValue::from_f64(exercise.min_reps as f64);
        let max_reps_val = exercise
            .max_reps
            .map(|r| JsValue::from_f64(r as f64))
            .unwrap_or(JsValue::NULL);

        let result = if let Some(ref id) = exercise.id {
            let sql = r#"
                UPDATE exercises SET name = ?, is_weighted = ?, min_weight = ?, increment = ?, min_reps = ?, max_reps = ?, updated_at = ?
                WHERE uuid = ?
                RETURNING uuid
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
                min_reps_val,
                max_reps_val,
                JsValue::from_f64(now),
                JsValue::from_str(id),
            ];
            self.execute(sql, &params).await?
        } else {
            // Check if an exercise with this name already exists.
            // CRR tables don't support UNIQUE constraints or ON CONFLICT
            // clauses, so we check-then-upsert manually.
            let existing = self
                .execute(
                    "SELECT uuid FROM exercises WHERE name = ?",
                    &[JsValue::from_str(&exercise.name)],
                )
                .await?;
            let existing_uuid = existing
                .dyn_ref::<js_sys::Array>()
                .and_then(|a| if a.length() > 0 { Some(a.get(0)) } else { None })
                .and_then(|row| {
                    js_sys::Reflect::get(&row, &JsValue::from_str("uuid"))
                        .ok()
                        .and_then(|v| v.as_string())
                });

            if let Some(euuid) = existing_uuid {
                // Update existing exercise by name match.
                let sql = r#"
                    UPDATE exercises SET is_weighted = ?, min_weight = ?, increment = ?, min_reps = ?, max_reps = ?, updated_at = ?
                    WHERE uuid = ?
                    RETURNING uuid
                "#;
                let params = vec![
                    JsValue::from_bool(is_weighted),
                    min_weight
                        .map(|w| JsValue::from_f64(w as f64))
                        .unwrap_or(JsValue::NULL),
                    increment
                        .map(|i| JsValue::from_f64(i as f64))
                        .unwrap_or(JsValue::NULL),
                    min_reps_val,
                    max_reps_val,
                    JsValue::from_f64(now),
                    JsValue::from_str(&euuid),
                ];
                self.execute(sql, &params).await?
            } else {
                let uuid = Self::generate_uuid();
                let sql = r#"
                    INSERT INTO exercises (uuid, name, is_weighted, min_weight, increment, min_reps, max_reps, updated_at)
                    VALUES (?, ?, ?, ?, ?, ?, ?, ?)
                    RETURNING uuid
                "#;
                let params = vec![
                    JsValue::from_str(&uuid),
                    JsValue::from_str(&exercise.name),
                    JsValue::from_bool(is_weighted),
                    min_weight
                        .map(|w| JsValue::from_f64(w as f64))
                        .unwrap_or(JsValue::NULL),
                    increment
                        .map(|i| JsValue::from_f64(i as f64))
                        .unwrap_or(JsValue::NULL),
                    min_reps_val,
                    max_reps_val,
                    JsValue::from_f64(now),
                ];
                self.execute(sql, &params).await?
            }
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
        let id = js_sys::Reflect::get(&first_row, &JsValue::from_str("uuid"))?
            .as_string()
            .ok_or_else(|| DatabaseError::QueryError("Failed to get exercise uuid".to_string()))?;

        Ok(id)
    }

    pub async fn get_exercises(&self) -> Result<Vec<ExerciseMetadata>, DatabaseError> {
        let sql = "SELECT uuid, name, is_weighted, min_weight, increment, min_reps, max_reps FROM exercises WHERE deleted_at IS NULL ORDER BY name";
        let result = self.execute(sql, &[]).await?;

        let array = result
            .dyn_ref::<js_sys::Array>()
            .ok_or_else(|| DatabaseError::QueryError("Expected array result".to_string()))?;

        let mut exercises = Vec::new();
        for i in 0..array.length() {
            let row = array.get(i);
            let id = js_sys::Reflect::get(&row, &JsValue::from_str("uuid"))?.as_string();

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

            let min_reps = js_sys::Reflect::get(&row, &JsValue::from_str("min_reps"))?
                .as_f64()
                .unwrap_or(1.0) as i32;
            let max_reps_val = js_sys::Reflect::get(&row, &JsValue::from_str("max_reps"))?;
            let max_reps = if max_reps_val.is_null() || max_reps_val.is_undefined() {
                None
            } else {
                max_reps_val.as_f64().map(|v| v as i32)
            };

            exercises.push(ExerciseMetadata {
                id,
                name,
                set_type_config,
                min_reps,
                max_reps,
            });
        }

        Ok(exercises)
    }

    /// Returns the current settings, seeding the default row if absent.
    pub async fn get_settings(&self) -> Result<crate::models::Settings, DatabaseError> {
        // Ensure the settings row exists (idempotent).
        self.seed_settings().await?;

        let sql = "SELECT target_rpe, history_window_days, today_blend_factor, default_planned_sets FROM settings WHERE id = 1";
        let result = self.execute(sql, &[]).await?;

        let array = result
            .dyn_ref::<js_sys::Array>()
            .ok_or_else(|| DatabaseError::QueryError("Expected array result".to_string()))?;

        if array.length() == 0 {
            return Ok(crate::models::Settings::default());
        }

        let row = array.get(0);
        let target_rpe = js_sys::Reflect::get(&row, &JsValue::from_str("target_rpe"))?
            .as_f64()
            .unwrap_or(8.0);
        let history_window_days =
            js_sys::Reflect::get(&row, &JsValue::from_str("history_window_days"))?
                .as_f64()
                .unwrap_or(30.0) as i32;
        let today_blend_factor =
            js_sys::Reflect::get(&row, &JsValue::from_str("today_blend_factor"))?
                .as_f64()
                .unwrap_or(0.5);

        let default_planned_sets =
            js_sys::Reflect::get(&row, &JsValue::from_str("default_planned_sets"))
                .ok()
                .and_then(|v| v.as_f64())
                .unwrap_or(3.0) as u32;

        Ok(crate::models::Settings {
            target_rpe,
            history_window_days,
            today_blend_factor,
            default_planned_sets,
        })
    }

    /// Updates the settings row in the database.
    pub async fn update_settings(
        &self,
        settings: &crate::models::Settings,
    ) -> Result<(), DatabaseError> {
        let sql = "UPDATE settings SET target_rpe = ?, history_window_days = ?, today_blend_factor = ?, default_planned_sets = ? WHERE id = 1";
        self.execute(
            sql,
            &[
                JsValue::from_f64(settings.target_rpe),
                JsValue::from_f64(settings.history_window_days as f64),
                JsValue::from_f64(settings.today_blend_factor),
                JsValue::from_f64(settings.default_planned_sets as f64),
            ],
        )
        .await?;
        Ok(())
    }

    /// Returns the most recent set for the given exercise (used for predictions).
    pub async fn get_last_set_for_exercise(
        &self,
        exercise_id: &str,
    ) -> Result<Option<crate::models::CompletedSet>, DatabaseError> {
        let sql = r#"
            SELECT set_number, reps, rpe, weight, is_bodyweight
            FROM completed_sets
            WHERE exercise_id = ? AND deleted_at IS NULL
            ORDER BY recorded_at DESC, id DESC
            LIMIT 1
        "#;

        let params = vec![JsValue::from_str(exercise_id)];
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

    /// Returns the set with the highest computed e1RM for the given exercise
    /// within the history window `[since_ms, now)`, **excluding** sets whose
    /// `recorded_at` falls in `[exclude_start_ms, exclude_end_ms)`.
    ///
    /// e1RM comparison is performed in Rust using `domain::e1rm::e1rm()` so
    /// that the ranking logic stays in one place.
    ///
    /// Bodyweight sets are skipped because e1RM is undefined without a weight.
    pub async fn get_best_set_for_exercise(
        &self,
        exercise_id: &str,
        since_ms: f64,
        exclude_start_ms: f64,
        exclude_end_ms: f64,
    ) -> Result<Option<CompletedSet>, DatabaseError> {
        let sql = r#"
            SELECT set_number, reps, rpe, weight, is_bodyweight
            FROM completed_sets
            WHERE exercise_id = ?
              AND deleted_at IS NULL
              AND recorded_at >= ?
              AND (recorded_at < ? OR recorded_at >= ?)
              AND is_bodyweight = 0
        "#;

        let params = vec![
            JsValue::from_str(exercise_id),
            JsValue::from_f64(since_ms),
            JsValue::from_f64(exclude_start_ms),
            JsValue::from_f64(exclude_end_ms),
        ];

        let result = self.execute(sql, &params).await?;

        let array = result
            .dyn_ref::<js_sys::Array>()
            .ok_or_else(|| DatabaseError::QueryError("Expected array result".to_string()))?;

        let mut best: Option<(CompletedSet, f64)> = None;

        for i in 0..array.length() {
            let row = array.get(i);
            let completed = self.parse_completed_set_row(&row)?;

            if let SetType::Weighted { weight } = completed.set_type {
                let estimate =
                    crate::domain::e1rm::e1rm(weight as f64, completed.reps, completed.rpe as f64);
                match &best {
                    Some((_, best_e1rm)) if estimate <= *best_e1rm => {}
                    _ => best = Some((completed, estimate)),
                }
            }
        }

        Ok(best.map(|(set, _)| set))
    }

    /// Returns the most recently logged set for the given exercise on "today",
    /// defined as the half-open interval `[today_start_ms, today_end_ms)`.
    ///
    /// Returns `None` when no sets were logged today for this exercise.
    pub async fn get_latest_set_today(
        &self,
        exercise_id: &str,
        today_start_ms: f64,
        today_end_ms: f64,
    ) -> Result<Option<CompletedSet>, DatabaseError> {
        let sql = r#"
            SELECT set_number, reps, rpe, weight, is_bodyweight
            FROM completed_sets
            WHERE exercise_id = ?
              AND deleted_at IS NULL
              AND recorded_at >= ?
              AND recorded_at < ?
            ORDER BY recorded_at DESC, id DESC
            LIMIT 1
        "#;

        let params = vec![
            JsValue::from_str(exercise_id),
            JsValue::from_f64(today_start_ms),
            JsValue::from_f64(today_end_ms),
        ];

        let result = self.execute(sql, &params).await?;

        let array = result
            .dyn_ref::<js_sys::Array>()
            .ok_or_else(|| DatabaseError::QueryError("Expected array result".to_string()))?;

        if array.length() == 0 {
            return Ok(None);
        }

        let row = array.get(0);
        Ok(Some(self.parse_completed_set_row(&row)?))
    }

    /// Parses a single row (with set_number, reps, rpe, weight, is_bodyweight)
    /// into a `CompletedSet`.
    fn parse_completed_set_row(&self, row: &JsValue) -> Result<CompletedSet, DatabaseError> {
        let set_number = js_sys::Reflect::get(row, &JsValue::from_str("set_number"))?
            .as_f64()
            .ok_or_else(|| DatabaseError::QueryError("Failed to get set_number".to_string()))?
            as u32;

        let reps = js_sys::Reflect::get(row, &JsValue::from_str("reps"))?
            .as_f64()
            .ok_or_else(|| DatabaseError::QueryError("Failed to get reps".to_string()))?
            as u32;

        let rpe = js_sys::Reflect::get(row, &JsValue::from_str("rpe"))?
            .as_f64()
            .ok_or_else(|| DatabaseError::QueryError("Failed to get rpe".to_string()))?
            as f32;

        let is_bodyweight = self.parse_bool_field(row, "is_bodyweight")?;

        let set_type = if is_bodyweight {
            SetType::Bodyweight
        } else {
            let weight = js_sys::Reflect::get(row, &JsValue::from_str("weight"))?
                .as_f64()
                .ok_or_else(|| DatabaseError::QueryError("Failed to get weight".to_string()))?
                as f32;
            SetType::Weighted { weight }
        };

        Ok(CompletedSet {
            set_number,
            reps,
            rpe,
            set_type,
        })
    }

    /// Execute a raw SQL statement with string parameters.
    ///
    /// Each parameter value is bound as a JsValue string. For NULL handling,
    /// pass the literal string "NULL" which will be converted to JsValue::NULL.
    pub async fn execute_raw(
        &self,
        sql: &str,
        params: &[String],
    ) -> Result<JsValue, DatabaseError> {
        if !self.initialized {
            return Err(DatabaseError::NotInitialized);
        }

        let js_params: Vec<JsValue> = params
            .iter()
            .map(|p| {
                if p == "NULL" {
                    JsValue::NULL
                } else {
                    JsValue::from_str(p)
                }
            })
            .collect();

        self.execute_internal(sql, &js_params).await
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
                .as_string()
                .unwrap_or_default();

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

    // NOTE: merge_databases() was removed as part of the crsqlite-wasm
    // migration (#179). Merge is now handled by CRR-based CRDT replication.

    // ── Test-only helpers ─────────────────────────────────────────────────────
    //
    // These methods expose surgical INSERT paths for merge tests, letting tests
    // inject rows with explicit UUIDs and timestamps without going through the
    // normal `save_exercise` path (which auto-assigns UUIDs and uses `NOW()`).

    /// Inserts a single exercise row with an explicit UUID, name, updated_at,
    /// and optional deleted_at. Returns the generated UUID.
    ///
    /// For use in unit/integration tests only; not part of the public API.
    #[cfg(test)]
    pub async fn insert_exercise_for_test(
        &self,
        name: &str,
        updated_at: f64,
        deleted_at: Option<f64>,
    ) -> Result<String, DatabaseError> {
        let uuid = Uuid::new_v4().to_string();
        self.insert_exercise_with_uuid_for_test(&uuid, name, updated_at, deleted_at)
            .await?;
        Ok(uuid)
    }

    /// Inserts a single exercise row with a *caller-supplied* UUID.
    ///
    /// This is the lowest-level helper; use it when you need the same UUID to
    /// appear in two different databases (to exercise the merge collision logic).
    ///
    /// For use in unit/integration tests only; not part of the public API.
    #[cfg(test)]
    pub async fn insert_exercise_with_uuid_for_test(
        &self,
        uuid: &str,
        name: &str,
        updated_at: f64,
        deleted_at: Option<f64>,
    ) -> Result<(), DatabaseError> {
        let sql = if deleted_at.is_some() {
            r#"
                INSERT INTO exercises (uuid, name, is_weighted, updated_at, deleted_at)
                VALUES (?, ?, 0, ?, ?)
            "#
        } else {
            r#"
                INSERT INTO exercises (uuid, name, is_weighted, updated_at)
                VALUES (?, ?, 0, ?)
            "#
        };

        let mut params: Vec<JsValue> = vec![
            JsValue::from_str(uuid),
            JsValue::from_str(name),
            JsValue::from_f64(updated_at),
        ];

        if let Some(da) = deleted_at {
            params.push(JsValue::from_f64(da));
        }

        self.execute(sql, &params).await?;
        Ok(())
    }

    // ── Workout Plan CRUD ────────────────────────────────────────────────────

    pub async fn create_plan(&self) -> Result<String, DatabaseError> {
        let id = Self::generate_uuid();
        let now = js_sys::Date::now();
        self.execute(
            "INSERT INTO workout_plans (id, updated_at) VALUES (?, ?)",
            &[JsValue::from_str(&id), JsValue::from_f64(now)],
        )
        .await?;
        Ok(id)
    }

    pub async fn add_exercise_to_plan(
        &self,
        plan_id: &str,
        exercise_id: &str,
        planned_sets: u32,
    ) -> Result<String, DatabaseError> {
        let id = Self::generate_uuid();
        let now = js_sys::Date::now();

        // Position = count of existing non-deleted exercises in this plan
        let result = self
            .execute(
                "SELECT COUNT(*) as cnt FROM workout_plan_exercises WHERE plan_id = ? AND deleted_at IS NULL",
                &[JsValue::from_str(plan_id)],
            )
            .await?;
        let position = result
            .dyn_ref::<js_sys::Array>()
            .and_then(|a| if a.length() > 0 { Some(a.get(0)) } else { None })
            .and_then(|row| {
                js_sys::Reflect::get(&row, &JsValue::from_str("cnt"))
                    .ok()
                    .and_then(|v| v.as_f64())
            })
            .unwrap_or(0.0) as u32;

        self.execute(
            "INSERT INTO workout_plan_exercises (id, plan_id, exercise_id, planned_sets, position, updated_at) VALUES (?, ?, ?, ?, ?, ?)",
            &[
                JsValue::from_str(&id),
                JsValue::from_str(plan_id),
                JsValue::from_str(exercise_id),
                JsValue::from_f64(planned_sets as f64),
                JsValue::from_f64(position as f64),
                JsValue::from_f64(now),
            ],
        )
        .await?;
        Ok(id)
    }

    pub async fn remove_exercise_from_plan(
        &self,
        plan_exercise_id: &str,
    ) -> Result<(), DatabaseError> {
        let now = js_sys::Date::now();
        self.execute(
            "UPDATE workout_plan_exercises SET deleted_at = ?, updated_at = ? WHERE id = ?",
            &[
                JsValue::from_f64(now),
                JsValue::from_f64(now),
                JsValue::from_str(plan_exercise_id),
            ],
        )
        .await?;
        Ok(())
    }

    pub async fn start_plan(&self, plan_id: &str) -> Result<(), DatabaseError> {
        let now = js_sys::Date::now();
        self.execute(
            "UPDATE workout_plans SET started_at = ?, updated_at = ? WHERE id = ?",
            &[
                JsValue::from_f64(now),
                JsValue::from_f64(now),
                JsValue::from_str(plan_id),
            ],
        )
        .await?;
        Ok(())
    }

    pub async fn end_plan(&self, plan_id: &str) -> Result<(), DatabaseError> {
        let now = js_sys::Date::now();
        self.execute(
            "UPDATE workout_plans SET ended_at = ?, updated_at = ? WHERE id = ?",
            &[
                JsValue::from_f64(now),
                JsValue::from_f64(now),
                JsValue::from_str(plan_id),
            ],
        )
        .await?;
        Ok(())
    }

    pub async fn get_active_plan(&self) -> Result<Option<WorkoutPlan>, DatabaseError> {
        let result = self
            .execute(
                "SELECT id, started_at, ended_at FROM workout_plans WHERE started_at IS NOT NULL AND ended_at IS NULL AND deleted_at IS NULL LIMIT 1",
                &[],
            )
            .await?;

        let array = match result.dyn_ref::<js_sys::Array>() {
            Some(a) if a.length() > 0 => a,
            _ => return Ok(None),
        };

        let row = array.get(0);
        let plan_id = js_sys::Reflect::get(&row, &JsValue::from_str("id"))
            .ok()
            .and_then(|v| v.as_string())
            .unwrap_or_default();

        let started_at = js_sys::Reflect::get(&row, &JsValue::from_str("started_at"))
            .ok()
            .and_then(|v| v.as_f64());
        let ended_at = js_sys::Reflect::get(&row, &JsValue::from_str("ended_at"))
            .ok()
            .and_then(|v| v.as_f64());

        let exercises = self.get_plan_exercises(&plan_id).await?;

        Ok(Some(WorkoutPlan {
            id: plan_id,
            started_at,
            ended_at,
            exercises,
        }))
    }

    pub async fn get_plan(&self, plan_id: &str) -> Result<Option<WorkoutPlan>, DatabaseError> {
        let result = self
            .execute(
                "SELECT id, started_at, ended_at FROM workout_plans WHERE id = ? AND deleted_at IS NULL",
                &[JsValue::from_str(plan_id)],
            )
            .await?;

        let array = match result.dyn_ref::<js_sys::Array>() {
            Some(a) if a.length() > 0 => a,
            _ => return Ok(None),
        };

        let row = array.get(0);
        let started_at = js_sys::Reflect::get(&row, &JsValue::from_str("started_at"))
            .ok()
            .and_then(|v| v.as_f64());
        let ended_at = js_sys::Reflect::get(&row, &JsValue::from_str("ended_at"))
            .ok()
            .and_then(|v| v.as_f64());

        let exercises = self.get_plan_exercises(plan_id).await?;

        Ok(Some(WorkoutPlan {
            id: plan_id.to_string(),
            started_at,
            ended_at,
            exercises,
        }))
    }

    async fn get_plan_exercises(&self, plan_id: &str) -> Result<Vec<PlanExercise>, DatabaseError> {
        let result = self
            .execute(
                r#"
                SELECT pe.id, pe.exercise_id, pe.planned_sets, pe.position,
                       e.name, e.is_weighted, e.min_weight, e.increment, e.min_reps, e.max_reps
                FROM workout_plan_exercises pe
                JOIN exercises e ON pe.exercise_id = e.uuid
                WHERE pe.plan_id = ? AND pe.deleted_at IS NULL
                ORDER BY pe.position ASC
                "#,
                &[JsValue::from_str(plan_id)],
            )
            .await?;

        let array = match result.dyn_ref::<js_sys::Array>() {
            Some(a) => a,
            None => return Ok(Vec::new()),
        };

        let mut exercises = Vec::new();
        for i in 0..array.length() {
            let row = array.get(i);
            let get_str = |key: &str| -> String {
                js_sys::Reflect::get(&row, &JsValue::from_str(key))
                    .ok()
                    .and_then(|v| v.as_string())
                    .unwrap_or_default()
            };
            let get_f64 = |key: &str| -> f64 {
                js_sys::Reflect::get(&row, &JsValue::from_str(key))
                    .ok()
                    .and_then(|v| v.as_f64())
                    .unwrap_or(0.0)
            };

            let is_weighted = get_f64("is_weighted") != 0.0;
            let set_type_config = if is_weighted {
                SetTypeConfig::Weighted {
                    min_weight: get_f64("min_weight") as f32,
                    increment: get_f64("increment") as f32,
                }
            } else {
                SetTypeConfig::Bodyweight
            };

            let exercise_id = get_str("exercise_id");
            let max_reps_val = js_sys::Reflect::get(&row, &JsValue::from_str("max_reps"))
                .ok()
                .and_then(|v| v.as_f64());

            exercises.push(PlanExercise {
                id: get_str("id"),
                exercise: ExerciseMetadata {
                    id: Some(exercise_id),
                    name: get_str("name"),
                    set_type_config,
                    min_reps: get_f64("min_reps") as i32,
                    max_reps: max_reps_val.map(|v| v as i32),
                },
                planned_sets: get_f64("planned_sets") as u32,
                position: get_f64("position") as u32,
            });
        }

        Ok(exercises)
    }

    /// Get the most recent unstarted plan (for resuming plan builder).
    pub async fn get_unstarted_plan(&self) -> Result<Option<WorkoutPlan>, DatabaseError> {
        let result = self
            .execute(
                "SELECT id, started_at, ended_at FROM workout_plans WHERE started_at IS NULL AND deleted_at IS NULL ORDER BY rowid DESC LIMIT 1",
                &[],
            )
            .await?;

        let array = match result.dyn_ref::<js_sys::Array>() {
            Some(a) if a.length() > 0 => a,
            _ => return Ok(None),
        };

        let row = array.get(0);
        let plan_id = js_sys::Reflect::get(&row, &JsValue::from_str("id"))
            .ok()
            .and_then(|v| v.as_string())
            .unwrap_or_default();

        let exercises = self.get_plan_exercises(&plan_id).await?;

        Ok(Some(WorkoutPlan {
            id: plan_id,
            started_at: None,
            ended_at: None,
            exercises,
        }))
    }

    /// Count completed sets per exercise since a given timestamp.
    /// Returns a Vec of (exercise_id, count) pairs.
    pub async fn count_sets_since(
        &self,
        exercise_ids: &[String],
        since_ms: f64,
    ) -> Result<Vec<(String, u32)>, DatabaseError> {
        if exercise_ids.is_empty() {
            return Ok(Vec::new());
        }

        let placeholders: Vec<&str> = exercise_ids.iter().map(|_| "?").collect();
        let sql = format!(
            "SELECT exercise_id, COUNT(*) as cnt FROM completed_sets WHERE exercise_id IN ({}) AND recorded_at >= ? AND deleted_at IS NULL GROUP BY exercise_id",
            placeholders.join(",")
        );

        let mut params: Vec<JsValue> = exercise_ids
            .iter()
            .map(|id| JsValue::from_str(id))
            .collect();
        params.push(JsValue::from_f64(since_ms));

        let result = self.execute(&sql, &params).await?;
        let array = match result.dyn_ref::<js_sys::Array>() {
            Some(a) => a,
            None => return Ok(Vec::new()),
        };

        let mut counts = Vec::new();
        for i in 0..array.length() {
            let row = array.get(i);
            let eid = js_sys::Reflect::get(&row, &JsValue::from_str("exercise_id"))
                .ok()
                .and_then(|v| v.as_string())
                .unwrap_or_default();
            let cnt = js_sys::Reflect::get(&row, &JsValue::from_str("cnt"))
                .ok()
                .and_then(|v| v.as_f64())
                .unwrap_or(0.0) as u32;
            counts.push((eid, cnt));
        }
        Ok(counts)
    }

    /// Get the most recent recorded_at timestamp for any set belonging to
    /// the given exercise IDs since a given time. Returns None if no sets found.
    pub async fn get_latest_set_time(
        &self,
        exercise_ids: &[String],
        since_ms: f64,
    ) -> Result<Option<f64>, DatabaseError> {
        if exercise_ids.is_empty() {
            return Ok(None);
        }
        let placeholders: Vec<&str> = exercise_ids.iter().map(|_| "?").collect();
        let sql = format!(
            "SELECT MAX(recorded_at) as latest FROM completed_sets WHERE exercise_id IN ({}) AND recorded_at >= ? AND deleted_at IS NULL",
            placeholders.join(",")
        );
        let mut params: Vec<JsValue> = exercise_ids
            .iter()
            .map(|id| JsValue::from_str(id))
            .collect();
        params.push(JsValue::from_f64(since_ms));

        let result = self.execute(&sql, &params).await?;
        let array = match result.dyn_ref::<js_sys::Array>() {
            Some(a) if a.length() > 0 => a,
            _ => return Ok(None),
        };
        let row = array.get(0);
        let latest = js_sys::Reflect::get(&row, &JsValue::from_str("latest"))
            .ok()
            .and_then(|v| v.as_f64());
        Ok(latest)
    }

    // ── Workout Template CRUD ────────────────────────────────────────────────

    pub async fn save_template(
        &self,
        name: &str,
        exercises: &[PlanExercise],
    ) -> Result<String, DatabaseError> {
        let id = Self::generate_uuid();
        let now = js_sys::Date::now();

        self.execute(
            "INSERT INTO workout_templates (id, name, updated_at) VALUES (?, ?, ?)",
            &[
                JsValue::from_str(&id),
                JsValue::from_str(name),
                JsValue::from_f64(now),
            ],
        )
        .await?;

        for (pos, pe) in exercises.iter().enumerate() {
            let te_id = Self::generate_uuid();
            let exercise_id = pe.exercise.id.clone().unwrap_or_default();
            self.execute(
                "INSERT INTO workout_template_exercises (id, template_id, exercise_id, planned_sets, position, updated_at) VALUES (?, ?, ?, ?, ?, ?)",
                &[
                    JsValue::from_str(&te_id),
                    JsValue::from_str(&id),
                    JsValue::from_str(&exercise_id),
                    JsValue::from_f64(pe.planned_sets as f64),
                    JsValue::from_f64(pos as f64),
                    JsValue::from_f64(now),
                ],
            )
            .await?;
        }

        Ok(id)
    }

    pub async fn list_templates(&self) -> Result<Vec<WorkoutTemplate>, DatabaseError> {
        let result = self
            .execute(
                "SELECT id, name FROM workout_templates WHERE deleted_at IS NULL ORDER BY updated_at DESC",
                &[],
            )
            .await?;

        let array = match result.dyn_ref::<js_sys::Array>() {
            Some(a) => a,
            None => return Ok(Vec::new()),
        };

        let mut templates = Vec::new();
        for i in 0..array.length() {
            let row = array.get(i);
            let id = js_sys::Reflect::get(&row, &JsValue::from_str("id"))
                .ok()
                .and_then(|v| v.as_string())
                .unwrap_or_default();
            let name = js_sys::Reflect::get(&row, &JsValue::from_str("name"))
                .ok()
                .and_then(|v| v.as_string())
                .unwrap_or_default();

            let exercises = self.get_template_exercises(&id).await?;
            templates.push(WorkoutTemplate {
                id,
                name,
                exercises,
            });
        }
        Ok(templates)
    }

    async fn get_template_exercises(
        &self,
        template_id: &str,
    ) -> Result<Vec<PlanExercise>, DatabaseError> {
        let result = self
            .execute(
                r#"
                SELECT te.id, te.exercise_id, te.planned_sets, te.position,
                       e.name, e.is_weighted, e.min_weight, e.increment, e.min_reps, e.max_reps
                FROM workout_template_exercises te
                JOIN exercises e ON te.exercise_id = e.uuid
                WHERE te.template_id = ? AND te.deleted_at IS NULL
                ORDER BY te.position ASC
                "#,
                &[JsValue::from_str(template_id)],
            )
            .await?;

        let array = match result.dyn_ref::<js_sys::Array>() {
            Some(a) => a,
            None => return Ok(Vec::new()),
        };

        let mut exercises = Vec::new();
        for i in 0..array.length() {
            let row = array.get(i);
            let get_str = |key: &str| -> String {
                js_sys::Reflect::get(&row, &JsValue::from_str(key))
                    .ok()
                    .and_then(|v| v.as_string())
                    .unwrap_or_default()
            };
            let get_f64 = |key: &str| -> f64 {
                js_sys::Reflect::get(&row, &JsValue::from_str(key))
                    .ok()
                    .and_then(|v| v.as_f64())
                    .unwrap_or(0.0)
            };

            let is_weighted = get_f64("is_weighted") != 0.0;
            let set_type_config = if is_weighted {
                SetTypeConfig::Weighted {
                    min_weight: get_f64("min_weight") as f32,
                    increment: get_f64("increment") as f32,
                }
            } else {
                SetTypeConfig::Bodyweight
            };

            let exercise_id = get_str("exercise_id");
            let max_reps_val = js_sys::Reflect::get(&row, &JsValue::from_str("max_reps"))
                .ok()
                .and_then(|v| v.as_f64());

            exercises.push(PlanExercise {
                id: get_str("id"),
                exercise: ExerciseMetadata {
                    id: Some(exercise_id),
                    name: get_str("name"),
                    set_type_config,
                    min_reps: get_f64("min_reps") as i32,
                    max_reps: max_reps_val.map(|v| v as i32),
                },
                planned_sets: get_f64("planned_sets") as u32,
                position: get_f64("position") as u32,
            });
        }
        Ok(exercises)
    }

    /// Load a template's exercises into a plan, replacing current contents.
    pub async fn load_template_into_plan(
        &self,
        plan_id: &str,
        template_id: &str,
    ) -> Result<(), DatabaseError> {
        let now = js_sys::Date::now();

        // Soft-delete all current plan exercises
        self.execute(
            "UPDATE workout_plan_exercises SET deleted_at = ?, updated_at = ? WHERE plan_id = ? AND deleted_at IS NULL",
            &[
                JsValue::from_f64(now),
                JsValue::from_f64(now),
                JsValue::from_str(plan_id),
            ],
        )
        .await?;

        // Copy template exercises into plan
        let template_exercises = self.get_template_exercises(template_id).await?;
        for (pos, te) in template_exercises.iter().enumerate() {
            let pe_id = Self::generate_uuid();
            let exercise_id = te.exercise.id.clone().unwrap_or_default();
            self.execute(
                "INSERT INTO workout_plan_exercises (id, plan_id, exercise_id, planned_sets, position, updated_at) VALUES (?, ?, ?, ?, ?, ?)",
                &[
                    JsValue::from_str(&pe_id),
                    JsValue::from_str(plan_id),
                    JsValue::from_str(&exercise_id),
                    JsValue::from_f64(te.planned_sets as f64),
                    JsValue::from_f64(pos as f64),
                    JsValue::from_f64(now),
                ],
            )
            .await?;
        }

        Ok(())
    }
}

impl Default for Database {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod init_error_tests {
    use super::*;

    /// Verify that Database::new() starts with sync_unavailable = false.
    #[test]
    fn new_database_sync_available() {
        let db = Database::new();
        assert!(!db.sync_unavailable);
    }

    /// Verify that Database::new() starts uninitialized.
    #[test]
    fn new_database_not_initialized() {
        let db = Database::new();
        assert!(!db.initialized);
    }
}
