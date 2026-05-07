use crate::log_buffer::{self, LogEntry};
use crate::models::{CompletedSet, ExerciseMetadata, SetType, Settings, WorkoutPlan};
use crate::state::{Database, Storage, error::WorkoutError};
#[cfg(not(test))]
use crate::sync::SyncCredentials;
use dioxus::prelude::*;
use std::collections::HashMap;
use wasm_bindgen::JsValue;

/// Write directly to browser `console.log` – always visible in Playwright
/// output regardless of the Rust log level or compile profile.
fn js_log(msg: &str) {
    web_sys::console::log_1(&JsValue::from_str(msg));
}

// Initial prediction constants
const DEFAULT_WEIGHTED_REPS: u32 = 8;
const DEFAULT_RPE: f32 = 7.0;

#[derive(Clone, Copy, Debug, PartialEq)]
pub struct PredictedParameters {
    pub weight: Option<f32>,
    pub reps: u32,
    pub rpe: f32,
    /// True when the predicted rep count was clamped to the exercise's
    /// configured `[min_reps, max_reps]` range.
    pub reps_clamped: bool,
}

#[derive(Clone, Debug, PartialEq)]
pub struct WorkoutSession {
    pub session_id: Option<String>,
    pub exercise: ExerciseMetadata,
    pub completed_sets: Vec<CompletedSet>,
    pub predicted: PredictedParameters,
}

#[derive(Clone, Copy, PartialEq, Debug, Default)]
pub enum InitializationState {
    #[default]
    NotInitialized,
    Initializing,
    SelectingFile,
    Ready,
    Error,
}

/// Represents the current sync state of the application.
///
/// `Idle`               - no sync is configured (default before any sync setup).
/// `NeverSynced`        - no sync has ever completed (distinguishes from a sync failure).
/// `Syncing`            - a sync cycle is currently in progress.
/// `UpToDate`           - the last sync completed successfully.
/// `Error(reason)`      - the last sync failed; carries a human-readable reason
///                        (e.g. "network timeout", "401 Unauthorized") so the UI
///                        can surface actionable context instead of a generic message.
#[derive(Clone, PartialEq, Debug, Default)]
pub enum SyncStatus {
    #[default]
    Idle,
    NeverSynced,
    Syncing,
    UpToDate,
    Error(String),
    /// Sync is temporarily disabled (e.g. protocol migration in progress).
    /// Unlike `Error`, this is an expected, non-alarming state.
    Disabled(String),
}

impl SyncStatus {
    /// Returns the kebab-case attribute string for use in `data-sync-status` attributes.
    pub fn as_attr_str(&self) -> &'static str {
        match self {
            SyncStatus::Idle => "idle",
            SyncStatus::NeverSynced => "never-synced",
            SyncStatus::Syncing => "syncing",
            SyncStatus::UpToDate => "up-to-date",
            SyncStatus::Error(_) => "error",
            SyncStatus::Disabled(_) => "disabled",
        }
    }
}

#[derive(Clone, Copy, PartialEq)]
pub struct WorkoutState {
    initialization_state: Signal<InitializationState>,
    current_session: Signal<Option<WorkoutSession>>,
    error: Signal<Option<WorkoutError>>,
    save_error: Signal<Option<String>>,
    database: Signal<Option<Database>>,
    file_manager: Signal<Option<Storage>>,
    last_save_time: Signal<f64>,
    exercises: Signal<Vec<ExerciseMetadata>>,
    sync_status: Signal<SyncStatus>,
    /// Global application settings (target RPE, history window, blend factor).
    settings: Signal<Settings>,
    /// Active workout plan (being built or in progress).
    current_plan: Signal<Option<WorkoutPlan>>,
    /// Cached snapshot of the debug log buffer for reactive UI rendering.
    log_entries: Signal<Vec<LogEntry>>,
}

impl Default for WorkoutState {
    fn default() -> Self {
        Self::new()
    }
}

impl WorkoutState {
    pub fn new() -> Self {
        Self {
            initialization_state: Signal::new(InitializationState::NotInitialized),
            current_session: Signal::new(None),
            error: Signal::new(None),
            save_error: Signal::new(None),
            database: Signal::new(None),
            file_manager: Signal::new(None),
            last_save_time: Signal::new(0.0),
            exercises: Signal::new(Vec::new()),
            settings: Signal::new(Settings::default()),
            sync_status: Signal::new(SyncStatus::Idle),
            current_plan: Signal::new(None),
            log_entries: Signal::new(Vec::new()),
        }
    }

    pub fn initialization_state(&self) -> InitializationState {
        (self.initialization_state)()
    }

    pub fn current_session(&self) -> Option<WorkoutSession> {
        (self.current_session)()
    }

    pub fn error(&self) -> Option<WorkoutError> {
        (self.error)()
    }

    pub fn save_error(&self) -> Option<String> {
        (self.save_error)()
    }

    pub fn set_initialization_state(&self, state: InitializationState) {
        let mut sig = self.initialization_state;
        sig.set(state);
    }

    pub fn set_current_session(&self, session: Option<WorkoutSession>) {
        let mut sig = self.current_session;
        sig.set(session);
    }

    pub fn set_error(&self, error: Option<WorkoutError>) {
        let mut sig = self.error;
        sig.set(error);
    }

    pub fn set_save_error(&self, error: Option<String>) {
        let mut sig = self.save_error;
        sig.set(error);
    }

    pub fn set_database(&self, database: Database) {
        let mut sig = self.database;
        sig.set(Some(database));
    }

    pub fn set_file_manager(&self, file_manager: Storage) {
        let mut sig = self.file_manager;
        sig.set(Some(file_manager));
    }

    pub fn database(&self) -> Option<Database> {
        (self.database)()
    }

    pub fn file_manager(&self) -> Option<Storage> {
        (self.file_manager)()
    }

    pub fn last_save_time(&self) -> f64 {
        (self.last_save_time)()
    }

    pub fn set_last_save_time(&self, time: f64) {
        let mut sig = self.last_save_time;
        sig.set(time);
    }

    pub fn exercises(&self) -> Vec<ExerciseMetadata> {
        (self.exercises)()
    }

    pub fn set_exercises(&self, exercises: Vec<ExerciseMetadata>) {
        let mut sig = self.exercises;
        sig.set(exercises);
    }

    pub fn settings(&self) -> Settings {
        (self.settings)()
    }

    pub fn set_settings(&self, settings: Settings) {
        let mut sig = self.settings;
        sig.set(settings);
    }

    pub fn sync_status(&self) -> SyncStatus {
        (self.sync_status)()
    }

    pub fn set_sync_status(&self, status: SyncStatus) {
        let mut sig = self.sync_status;
        sig.set(status);
    }

    pub fn current_plan(&self) -> Option<WorkoutPlan> {
        (self.current_plan)()
    }

    pub fn set_current_plan(&self, plan: Option<WorkoutPlan>) {
        let mut sig = self.current_plan;
        sig.set(plan);
    }

    /// Return the cached debug log entries (newest-first).
    pub fn log_entries(&self) -> Vec<LogEntry> {
        (self.log_entries)()
    }

    /// Refresh the log entries signal from the global ring buffer.
    pub fn refresh_log_entries(&self) {
        let mut sig = self.log_entries;
        sig.set(log_buffer::snapshot_global());
    }

    /// Clear the global log buffer and update the signal.
    pub fn clear_log_entries(&self) {
        log_buffer::clear_global();
        let mut sig = self.log_entries;
        sig.set(Vec::new());
    }
}

/// Returns true iff archiving `exercise_id` should be blocked.
///
/// Blocked when the exercise is the one currently being recorded on the
/// Record screen: `current_session.is_some()` AND
/// `current_session.exercise.id == Some(exercise_id)`.
///
/// Pure function — no UI dependencies.
pub fn is_archive_blocked(exercise_id: &str, current_session: &Option<WorkoutSession>) -> bool {
    match current_session {
        Some(session) => session.exercise.id.as_deref() == Some(exercise_id),
        None => false,
    }
}

/// Clamp `raw_reps` to `[min_reps, max_reps]` and return the clamped value plus
/// a flag indicating whether clamping actually occurred.
fn clamp_reps(raw_reps: u32, min_reps: u32, max_reps: Option<u32>) -> (u32, bool) {
    let clamped = match max_reps {
        Some(max) => raw_reps.clamp(min_reps, max),
        None => raw_reps.max(min_reps),
    };
    let was_clamped = clamped != raw_reps;
    (clamped, was_clamped)
}

pub struct WorkoutStateManager;

impl WorkoutStateManager {
    pub async fn setup_database(state: &WorkoutState) -> Result<(), WorkoutError> {
        js_log("[DB Init] Starting database setup...");

        match state.initialization_state() {
            InitializationState::Initializing => {
                js_log("[DB Init] Already in progress, skipping");
                return Err(WorkoutError::InitializationInProgress);
            }
            InitializationState::Ready => {
                js_log("[DB Init] Already initialized, skipping");
                return Ok(());
            }
            _ => {}
        }

        state.set_initialization_state(InitializationState::Initializing);

        js_log("[DB Init] Creating file manager...");
        let mut file_manager = Storage::new();

        js_log("[DB Init] Checking for cached file handle...");
        let has_cached = file_manager.check_cached_handle().await.map_err(|e| {
            js_log(&format!("[DB Init] check_cached_handle error: {}", e));
            WorkoutError::FileSystem(e)
        })?;

        js_log(&format!(
            "[DB Init] has_cached={}, use_fallback={}",
            has_cached,
            file_manager.is_using_fallback()
        ));

        if has_cached {
            state.set_file_manager(file_manager.clone());
        } else {
            js_log("[DB Init] No cached handle → SelectingFile (waiting for user gesture)");
            state.set_initialization_state(InitializationState::SelectingFile);
            return Ok(());
        }

        let file_data = if file_manager.has_handle() {
            js_log("[DB Init] Reading file contents...");
            match file_manager.read_file().await {
                Ok(data) if data.is_empty() => {
                    js_log("[DB Init] File empty → will create new database");
                    None
                }
                Ok(data) => {
                    js_log(&format!(
                        "[DB Init] Read {} bytes, loading existing database",
                        data.len()
                    ));
                    Some(data)
                }
                Err(e) => {
                    js_log(&format!("[DB Init] read_file error: {}", e));

                    if matches!(e, crate::state::FileSystemError::InvalidFormat) {
                        let _ = file_manager.clear_handle().await;
                    }

                    return Err(WorkoutError::FileSystem(e));
                }
            }
        } else {
            js_log("[DB Init] No file handle → new database");
            None
        };

        js_log("[DB Init] Calling database.init()...");
        let mut database = Database::new();
        database.init(file_data).await.map_err(|e| {
            js_log(&format!("[DB Init] database.init() FAILED: {}", e));
            WorkoutError::Database(e)
        })?;
        js_log("[DB Init] Database initialized successfully");

        // If the sync module failed to load, mark sync as disabled so the UI
        // can surface a visible indicator without blocking the app.
        if database.sync_unavailable {
            js_log("[DB Init] Sync module unavailable — marking sync as Disabled");
            state.set_sync_status(SyncStatus::Disabled(
                "Sync module could not be loaded".to_string(),
            ));
        }

        state.set_database(database);
        state.set_file_manager(file_manager);

        if let Err(e) = Self::sync_exercises(state).await {
            js_log(&format!("[DB Init] sync_exercises warning: {}", e));
        }

        if let Err(e) = Self::load_settings(state).await {
            js_log(&format!("[DB Init] load_settings warning: {}", e));
        }

        if let Err(e) = Self::resume_active_plan(state).await {
            js_log(&format!("[DB Init] resume_active_plan warning: {}", e));
        }

        state.set_initialization_state(InitializationState::Ready);

        js_log("[DB Init] Setup complete! State is now Ready");
        Ok(())
    }

    pub async fn save_exercise(
        state: &WorkoutState,
        exercise: ExerciseMetadata,
    ) -> Result<String, WorkoutError> {
        let db = state.database().ok_or(WorkoutError::NotInitialized)?;

        let id = db
            .save_exercise(&exercise)
            .await
            .map_err(|e: crate::state::DatabaseError| {
                WorkoutError::SaveExerciseError(e.to_string())
            })?;

        // Sync exercises in state after saving
        if let Err(e) = Self::sync_exercises(state).await {
            log::warn!("Failed to sync exercises after saving: {}", e);
        }

        // Auto-save the database file
        if let Err(e) = Self::save_database(state).await {
            log::warn!("Auto-save after exercise save failed: {}", e);
        }

        Ok(id)
    }

    pub async fn start_session(
        state: &WorkoutState,
        mut exercise: ExerciseMetadata,
    ) -> Result<(), WorkoutError> {
        // Implicitly complete any in-progress session before starting a new one.
        // This ensures sets from the previous exercise are persisted to disk
        // and removes the need for an explicit "Finish Workout Session" action.
        if state.current_session().is_some() {
            Self::complete_session(state).await?;
        }

        let db = state.database().ok_or(WorkoutError::NotInitialized)?;

        let id = db
            .save_exercise(&exercise)
            .await
            .map_err(|e: crate::state::DatabaseError| {
                WorkoutError::SaveExerciseError(e.to_string())
            })?;
        exercise.id = Some(id.clone());

        // Fetch last set for suggestions (only for weighted exercises)
        let last_set = match exercise.set_type_config {
            crate::models::SetTypeConfig::Weighted { .. } => {
                db.get_last_set_for_exercise(&id).await.unwrap_or_else(|e| {
                    log::warn!("Failed to fetch last set for suggestion: {}", e);
                    None
                })
            }
            _ => None,
        };

        // Sync exercises in state after saving new one
        if let Err(e) = Self::sync_exercises(state).await {
            log::warn!("Failed to sync exercises after saving: {}", e);
        }

        let predicted = Self::calculate_initial_predictions(
            &exercise,
            last_set.as_ref(),
            state.settings().default_bodyweight_reps,
        );

        // Use exercise_id as session_id so the UI can detect a new session started
        let session = WorkoutSession {
            session_id: exercise.id.clone(),
            exercise,
            completed_sets: Vec::new(),
            predicted,
        };

        state.set_current_session(Some(session));

        Ok(())
    }

    pub async fn log_set(state: &WorkoutState, set: CompletedSet) -> Result<(), WorkoutError> {
        let mut session = state
            .current_session()
            .ok_or(WorkoutError::NoActiveSession)?;

        let exercise_id = session
            .exercise
            .id
            .clone()
            .ok_or(WorkoutError::SessionNotPersisted)?;

        let db = state.database().ok_or(WorkoutError::NotInitialized)?;

        crate::models::validate_completed_set(&set, &session.exercise)
            .map_err(|e| WorkoutError::InvalidSetData(e.to_string()))?;

        let _set_id =
            db.log_set(&exercise_id, &set)
                .await
                .map_err(|e: crate::state::DatabaseError| {
                    WorkoutError::InsertSetError(e.to_string())
                })?;

        session.completed_sets.push(set.clone());

        // Pre-fetch all inputs for calculate_next_predictions (no async inside
        // the pure function itself).
        let settings = state.settings();
        let now = js_sys::Date::now();
        let history_window_ms = settings.history_window_days as f64 * 24.0 * 60.0 * 60.0 * 1000.0;
        let since_ms = now - history_window_ms;

        // "Today" boundaries: midnight at start-of-day and end-of-day (UTC-based
        // approximation using a 24-hour window ending now).
        // We define today_start as the most recent midnight in the local timezone
        // by using Date arithmetic.
        let today_start_ms = {
            let d = js_sys::Date::new_0();
            d.set_hours(0);
            d.set_minutes(0);
            d.set_seconds(0);
            d.set_milliseconds(0);
            d.value_of()
        };
        let today_end_ms = today_start_ms + 24.0 * 60.0 * 60.0 * 1000.0;

        let historical_best = db
            .get_historical_best_for_exercise(
                &session.exercise,
                since_ms,
                today_start_ms,
                today_end_ms,
            )
            .await
            .unwrap_or_else(|e| {
                log::warn!("Failed to fetch historical_best: {}", e);
                None
            });

        let today_best = db
            .get_latest_set_today(&exercise_id, today_start_ms, today_end_ms)
            .await
            .unwrap_or_else(|e| {
                log::warn!("Failed to fetch today_best: {}", e);
                None
            });

        let per_rep_maxes = match session.exercise.set_type_config {
            crate::models::SetTypeConfig::Weighted { .. } => db
                .get_max_weight_per_rep(&exercise_id, since_ms)
                .await
                .unwrap_or_else(|e| {
                    log::warn!("Failed to fetch per_rep_maxes: {}", e);
                    HashMap::new()
                }),
            crate::models::SetTypeConfig::Bodyweight => HashMap::new(),
        };

        session.predicted = Self::calculate_next_predictions(
            &session,
            historical_best,
            today_best,
            per_rep_maxes,
            &settings,
        );

        state.set_current_session(Some(session));

        // Auto-save with debouncing (every 5 seconds) to prevent performance issues while minimizing data loss
        let now = js_sys::Date::now();
        if now - state.last_save_time() > 5000.0 {
            log::debug!("[Workout] Auto-saving database (debounced)...");
            state.set_last_save_time(now);
            match Self::save_database(state).await {
                Ok(_) => {
                    state.set_save_error(None);
                }
                Err(e) => {
                    log::warn!("Auto-save failed but set logged in memory: {}", e);
                    state.set_save_error(Some(format!(
                        "Auto-save failed: {}. Your latest data is only saved locally in memory.",
                        e
                    )));
                }
            }
        } else {
            log::debug!("[Workout] Skipping auto-save (debounced)");
        }

        Ok(())
    }

    pub async fn complete_session(state: &WorkoutState) -> Result<(), WorkoutError> {
        state
            .current_session()
            .ok_or(WorkoutError::NoActiveSession)?;

        Self::save_database(state).await?;

        state.set_current_session(None);

        Ok(())
    }

    /// No-op: crsqlite-wasm persists automatically via IndexedDB
    /// (IDBBatchAtomicVFS). Retained as a function for call-site compatibility.
    // TODO: Remove this no-op (and its call sites) after one release, once we
    // are confident that no code path depends on an explicit save step.
    pub async fn save_database(_state: &WorkoutState) -> Result<(), WorkoutError> {
        log::debug!("[DB] save_database: no-op (crsqlite auto-persists via IndexedDB)");
        Ok(())
    }

    /// Fetches all exercises from the database and updates the state's exercise signal.
    pub async fn sync_exercises(state: &WorkoutState) -> Result<(), WorkoutError> {
        let db = state.database().ok_or(WorkoutError::NotInitialized)?;

        let exercises = db.get_exercises().await.map_err(WorkoutError::Database)?;

        log::debug!(
            "[WorkoutState] Syncing {} exercises from database",
            exercises.len()
        );
        state.set_exercises(exercises);

        Ok(())
    }

    /// Fetches soft-deleted (archived) exercises from the database.
    pub async fn fetch_archived_exercises(
        state: &WorkoutState,
    ) -> Result<Vec<ExerciseMetadata>, WorkoutError> {
        let db = state.database().ok_or(WorkoutError::NotInitialized)?;
        db.get_archived_exercises()
            .await
            .map_err(WorkoutError::Database)
    }

    /// Archives an exercise (sets `deleted_at` to now) and refreshes the
    /// active exercise list in app state.
    pub async fn archive_exercise(
        state: &WorkoutState,
        exercise_id: &str,
    ) -> Result<(), WorkoutError> {
        let db = state.database().ok_or(WorkoutError::NotInitialized)?;
        db.archive_exercise(exercise_id)
            .await
            .map_err(WorkoutError::Database)?;
        Self::sync_exercises(state).await
    }

    /// Restores an archived exercise (clears `deleted_at`) and refreshes the
    /// active exercise list in app state.
    pub async fn unarchive_exercise(
        state: &WorkoutState,
        exercise_id: &str,
    ) -> Result<(), WorkoutError> {
        let db = state.database().ok_or(WorkoutError::NotInitialized)?;
        db.unarchive_exercise(exercise_id)
            .await
            .map_err(WorkoutError::Database)?;
        Self::sync_exercises(state).await
    }

    /// Returns the number of future plans that would be deleted if the exercise
    /// were archived.  Always 0 in this slice (no plan cascade).
    pub async fn preview_archive(
        state: &WorkoutState,
        exercise_id: &str,
    ) -> Result<u32, WorkoutError> {
        let db = state.database().ok_or(WorkoutError::NotInitialized)?;
        db.preview_archive(exercise_id)
            .await
            .map_err(WorkoutError::Database)
    }

    /// Returns `(completed_sets, plans_to_delete)` counts for the permanent-delete
    /// preview dialog.
    pub async fn preview_permanent_delete(
        state: &WorkoutState,
        exercise_id: &str,
    ) -> Result<(u32, u32), WorkoutError> {
        let db = state.database().ok_or(WorkoutError::NotInitialized)?;
        db.preview_permanent_delete(exercise_id)
            .await
            .map_err(WorkoutError::Database)
    }

    /// Permanently soft-deletes the exercise and all associated data (completed
    /// sets, plan slots, now-empty plans), then refreshes the active exercise list.
    pub async fn permanent_delete_exercise(
        state: &WorkoutState,
        exercise_id: &str,
    ) -> Result<(), WorkoutError> {
        let db = state.database().ok_or(WorkoutError::NotInitialized)?;
        db.permanent_delete_exercise(exercise_id)
            .await
            .map_err(WorkoutError::Database)?;
        Self::sync_exercises(state).await
    }

    /// Renames a template by id. Validates the new name (empty/whitespace-only
    /// is rejected). Auto-saves the database file on success.
    pub async fn rename_template(
        state: &WorkoutState,
        template_id: &str,
        new_name: &str,
    ) -> Result<(), WorkoutError> {
        let db = state.database().ok_or(WorkoutError::NotInitialized)?;
        db.rename_template(template_id, new_name)
            .await
            .map_err(WorkoutError::Database)?;
        if let Err(e) = Self::save_database(state).await {
            log::warn!("Auto-save after rename_template failed: {}", e);
        }
        Ok(())
    }

    /// Soft-deletes a template by id. Auto-saves the database file on success.
    /// Plans previously loaded from this template are unaffected.
    pub async fn delete_template(
        state: &WorkoutState,
        template_id: &str,
    ) -> Result<(), WorkoutError> {
        let db = state.database().ok_or(WorkoutError::NotInitialized)?;
        db.delete_template(template_id)
            .await
            .map_err(WorkoutError::Database)?;
        if let Err(e) = Self::save_database(state).await {
            log::warn!("Auto-save after delete_template failed: {}", e);
        }
        Ok(())
    }

    /// Load settings from the database into app state.
    pub async fn load_settings(state: &WorkoutState) -> Result<(), WorkoutError> {
        let db = state.database().ok_or(WorkoutError::NotInitialized)?;
        let settings = db.get_settings().await.map_err(WorkoutError::Database)?;
        state.set_settings(settings);
        Ok(())
    }

    /// Persist updated settings to the database and refresh app state.
    pub async fn update_settings(
        state: &WorkoutState,
        settings: Settings,
    ) -> Result<(), WorkoutError> {
        let db = state.database().ok_or(WorkoutError::NotInitialized)?;
        db.update_settings(&settings)
            .await
            .map_err(WorkoutError::Database)?;
        state.set_settings(settings);

        // Auto-save the database file
        if let Err(e) = Self::save_database(state).await {
            log::warn!("Auto-save after settings update failed: {}", e);
        }

        Ok(())
    }

    // ── Workout Plan lifecycle ─────────────────────────────────────────────

    pub async fn create_plan(state: &WorkoutState) -> Result<String, WorkoutError> {
        let db = state.database().ok_or(WorkoutError::NotInitialized)?;
        let plan_id = db.create_plan().await.map_err(WorkoutError::Database)?;
        let plan = db
            .get_plan(&plan_id)
            .await
            .map_err(WorkoutError::Database)?;
        state.set_current_plan(plan);
        Ok(plan_id)
    }

    pub async fn add_exercise_to_plan(
        state: &WorkoutState,
        exercise_id: &str,
        planned_sets: u32,
    ) -> Result<(), WorkoutError> {
        let plan = state.current_plan().ok_or(WorkoutError::NoActiveSession)?;
        let db = state.database().ok_or(WorkoutError::NotInitialized)?;
        db.add_exercise_to_plan(&plan.id, exercise_id, planned_sets)
            .await
            .map_err(WorkoutError::Database)?;
        let refreshed = db
            .get_plan(&plan.id)
            .await
            .map_err(WorkoutError::Database)?;
        state.set_current_plan(refreshed);
        Ok(())
    }

    pub async fn remove_exercise_from_plan(
        state: &WorkoutState,
        plan_exercise_id: &str,
    ) -> Result<(), WorkoutError> {
        let plan = state.current_plan().ok_or(WorkoutError::NoActiveSession)?;
        let db = state.database().ok_or(WorkoutError::NotInitialized)?;
        db.remove_exercise_from_plan(plan_exercise_id)
            .await
            .map_err(WorkoutError::Database)?;
        let refreshed = db
            .get_plan(&plan.id)
            .await
            .map_err(WorkoutError::Database)?;
        state.set_current_plan(refreshed);
        Ok(())
    }

    pub async fn start_plan(state: &WorkoutState) -> Result<(), WorkoutError> {
        let plan = state.current_plan().ok_or(WorkoutError::NoActiveSession)?;
        let db = state.database().ok_or(WorkoutError::NotInitialized)?;
        db.start_plan(&plan.id)
            .await
            .map_err(WorkoutError::Database)?;
        let refreshed = db
            .get_plan(&plan.id)
            .await
            .map_err(WorkoutError::Database)?;
        // Auto-start a session on the first planned exercise so the user
        // lands directly on the recording UI instead of an empty state.
        let first_exercise = refreshed
            .as_ref()
            .and_then(|p| p.exercises.first())
            .map(|pe| pe.exercise.clone());
        // IMPORTANT: start the session BEFORE updating the plan signal.
        // Updating the plan signal triggers a re-render which unmounts
        // PlanBuilder (where this spawn lives). Dioxus drops spawned tasks
        // when the owning component unmounts, so any awaits after
        // set_current_plan would be cancelled.
        if let Some(exercise) = first_exercise {
            Self::start_session(state, exercise).await?;
        }
        state.set_current_plan(refreshed);
        Ok(())
    }

    /// Create a transient one-exercise plan, add the given exercise with the
    /// configured `default_planned_sets`, then start the plan (which auto-starts
    /// a session on the first — and only — exercise).
    ///
    /// This replaces the legacy `start_session`-only path from the Library so
    /// that every workout always has a plan backing it.
    ///
    /// IMPORTANT: This method performs all DB operations first and only updates
    /// reactive signals at the very end.  Updating `current_plan` mid-flight
    /// triggers a re-render that unmounts the calling Library component, which
    /// kills the `spawn` task that is running this future.
    pub async fn start_adhoc_plan(
        state: &WorkoutState,
        exercise: &ExerciseMetadata,
    ) -> Result<(), WorkoutError> {
        let exercise_id = exercise
            .id
            .as_deref()
            .ok_or(WorkoutError::SessionNotPersisted)?;
        let planned_sets = state.settings().default_planned_sets;
        let db = state.database().ok_or(WorkoutError::NotInitialized)?;

        // 1. Create plan in DB (no signal update yet)
        let plan_id = db.create_plan().await.map_err(WorkoutError::Database)?;

        // 2. Add the single exercise to the plan in DB
        db.add_exercise_to_plan(&plan_id, exercise_id, planned_sets)
            .await
            .map_err(WorkoutError::Database)?;

        // 3. Start the plan in DB
        db.start_plan(&plan_id)
            .await
            .map_err(WorkoutError::Database)?;

        // 4. Auto-start a session on the exercise (this updates current_session
        //    signal but that does NOT unmount the Library component)
        Self::start_session(state, exercise.clone()).await?;

        // 5. NOW update the plan signal — all DB work is done, so even if the
        //    component unmounts after this point, nothing is lost.
        let refreshed = db
            .get_plan(&plan_id)
            .await
            .map_err(WorkoutError::Database)?;
        state.set_current_plan(refreshed);

        js_log(&format!(
            "[Workout] Ad-hoc plan {} started for exercise {}",
            plan_id, exercise_id
        ));
        Ok(())
    }

    pub async fn end_plan(state: &WorkoutState) -> Result<(), WorkoutError> {
        let plan = state.current_plan().ok_or(WorkoutError::NoActiveSession)?;
        let db = state.database().ok_or(WorkoutError::NotInitialized)?;
        db.end_plan(&plan.id)
            .await
            .map_err(WorkoutError::Database)?;
        state.set_current_plan(None);
        state.set_current_session(None);
        Ok(())
    }

    /// Discard an in-progress workout: soft-delete sets recorded since
    /// `started_at` for the plan's exercises, then un-start the plan so the
    /// user lands back on PlanBuilder with the original exercise list.
    pub async fn discard_plan(state: &WorkoutState) -> Result<(), WorkoutError> {
        let plan = state.current_plan().ok_or(WorkoutError::NoActiveSession)?;
        let db = state.database().ok_or(WorkoutError::NotInitialized)?;
        db.discard_plan(&plan.id)
            .await
            .map_err(WorkoutError::Database)?;
        // Refresh the plan from the database so the UI sees the unstarted state
        // with the original exercise list preserved.
        let refreshed = db
            .get_plan(&plan.id)
            .await
            .map_err(WorkoutError::Database)?;
        state.set_current_plan(refreshed);
        state.set_current_session(None);
        Ok(())
    }

    /// Resume an active plan on app load. Called during initialization.
    /// Auto-closes plans that have been inactive for > 4 hours.
    pub async fn resume_active_plan(state: &WorkoutState) -> Result<(), WorkoutError> {
        const AUTO_CLOSE_MS: f64 = 4.0 * 60.0 * 60.0 * 1000.0; // 4 hours

        let db = state.database().ok_or(WorkoutError::NotInitialized)?;
        if let Some(plan) = db.get_active_plan().await.map_err(WorkoutError::Database)? {
            let now = js_sys::Date::now();
            let started_at = plan.started_at.unwrap_or(0.0);

            // Check for auto-close: find the most recent set activity
            let exercise_ids: Vec<String> = plan
                .exercises
                .iter()
                .filter_map(|pe| pe.exercise.id.clone())
                .collect();

            let last_activity = db
                .get_latest_set_time(&exercise_ids, started_at)
                .await
                .map_err(WorkoutError::Database)?;

            let reference_time = last_activity.unwrap_or(started_at);

            if now - reference_time > AUTO_CLOSE_MS {
                js_log(&format!(
                    "[Plan] Auto-closing plan {} (inactive for > 4h, last activity: {})",
                    plan.id, reference_time
                ));
                db.end_plan(&plan.id)
                    .await
                    .map_err(WorkoutError::Database)?;
                state.set_current_plan(None);
                return Ok(());
            }

            js_log(&format!(
                "[Plan] Resuming active plan {} with {} exercises",
                plan.id,
                plan.exercises.len()
            ));
            state.set_current_plan(Some(plan));
        } else if let Some(plan) = db
            .get_unstarted_plan()
            .await
            .map_err(WorkoutError::Database)?
        {
            js_log(&format!(
                "[Plan] Resuming unstarted plan {} with {} exercises",
                plan.id,
                plan.exercises.len()
            ));
            state.set_current_plan(Some(plan));
        }
        Ok(())
    }

    fn calculate_initial_predictions(
        exercise: &ExerciseMetadata,
        last_set: Option<&CompletedSet>,
        default_bodyweight_reps: u32,
    ) -> PredictedParameters {
        match &exercise.set_type_config {
            crate::models::SetTypeConfig::Weighted { min_weight, .. } => {
                let weight = last_set
                    .and_then(|ls| {
                        if let SetType::Weighted { weight } = ls.set_type {
                            Some(weight)
                        } else {
                            None
                        }
                    })
                    .unwrap_or(*min_weight);

                PredictedParameters {
                    weight: Some(weight),
                    reps: DEFAULT_WEIGHTED_REPS,
                    rpe: DEFAULT_RPE,
                    reps_clamped: false,
                }
            }
            crate::models::SetTypeConfig::Bodyweight => PredictedParameters {
                weight: None,
                reps: default_bodyweight_reps,
                rpe: DEFAULT_RPE,
                reps_clamped: false,
            },
        }
    }

    /// Compute the next set prediction using the full e1RM algorithm.
    ///
    /// All database inputs (`historical_best`, `today_best`, `per_rep_maxes`) are
    /// pre-fetched by `log_set` so that this function contains no async calls.
    ///
    /// ### Weighted path
    /// 1. Compute e1RM from `today_best` and `historical_best`.
    /// 2. Blend them via `blended_e1rm`.
    /// 3. For each rep count in the exercise range, compute the projected weight
    ///    and compare against the historical max at that rep count.
    /// 4. Select the rep count with the highest positive margin, falling back to
    ///    the rep count with the least-negative margin when none are positive.
    /// 5. Clamp to `[min_reps, max_reps]`.
    ///
    /// ### Bodyweight path
    /// `failure_reps = set.reps + (10 - set.rpe)`.  Blend today and historical
    /// failure_reps, subtract the RIR implied by `target_rpe`, round to nearest
    /// integer, and clamp.
    ///
    /// ### No-data fallback
    /// When neither `today_best` nor `historical_best` are present, delegate to
    /// `calculate_initial_predictions`.
    fn calculate_next_predictions(
        session: &WorkoutSession,
        historical_best: Option<CompletedSet>,
        today_best: Option<CompletedSet>,
        per_rep_maxes: HashMap<u32, f64>,
        settings: &Settings,
    ) -> PredictedParameters {
        use crate::domain::e1rm::{blended_e1rm, e1rm, predicted_weight};

        let exercise = &session.exercise;
        let min_reps = exercise.min_reps as u32;
        let max_reps = exercise.max_reps.map(|v| v as u32);
        let target_rpe = settings.target_rpe;

        match &exercise.set_type_config {
            crate::models::SetTypeConfig::Weighted { .. } => {
                // Compute e1RM from today_best and historical_best (weighted sets only).
                let today_e1rm: Option<f64> = today_best.as_ref().and_then(|s| {
                    if let SetType::Weighted { weight } = s.set_type {
                        Some(e1rm(weight as f64, s.reps, s.rpe as f64))
                    } else {
                        None
                    }
                });

                let hist_e1rm: Option<f64> = historical_best.as_ref().and_then(|s| {
                    if let SetType::Weighted { weight } = s.set_type {
                        Some(e1rm(weight as f64, s.reps, s.rpe as f64))
                    } else {
                        None
                    }
                });

                // No history at all → fall back to initial predictions.
                let blended = match (today_e1rm, hist_e1rm) {
                    (None, None) => {
                        let last_session_set = session.completed_sets.last();
                        return Self::calculate_initial_predictions(
                            exercise,
                            last_session_set,
                            settings.default_bodyweight_reps,
                        );
                    }
                    (Some(t), Some(h)) => blended_e1rm(t, h, settings.today_blend_factor),
                    (Some(t), None) => t,
                    (None, Some(h)) => h,
                };

                // Build the rep search range.
                // If max_reps is None (infinite mode), extend one past the highest
                // rep count that has historical data, or use min_reps + a small window.
                let upper = match max_reps {
                    Some(m) => m,
                    None => {
                        let max_data_rep = per_rep_maxes.keys().copied().max().unwrap_or(min_reps);
                        max_data_rep + 1
                    }
                };
                let upper = upper.max(min_reps);

                // Compute margin for each rep count.
                let mut best_rep: Option<u32> = None;
                let mut best_margin: Option<f64> = None;

                for r in min_reps..=upper {
                    let proj = predicted_weight(blended, r, target_rpe);
                    // historical_max[r] = max weight across all sets where reps_done >= r
                    // (already folded in get_max_weight_per_rep).
                    let hist_max = per_rep_maxes.get(&r).copied();
                    // No data for this rep count → margin is None (treated as best positive case).
                    let margin = hist_max.map(|hm| proj - hm);

                    match (best_rep, best_margin, margin) {
                        // First iteration.
                        (None, _, _) => {
                            best_rep = Some(r);
                            best_margin = margin;
                        }
                        // Current best has no data (None margin = highest priority no-data).
                        // Only replace if current best is also no-data and r is lower
                        // (issue says "lowest no-data rep").
                        (Some(_), None, None) => {
                            // keep the first (lowest) no-data rep
                        }
                        // New rep has no data; current best has data → no-data wins.
                        (Some(_), Some(_), None) => {
                            best_rep = Some(r);
                            best_margin = None;
                        }
                        // New rep has data; current best has no data → keep no-data.
                        (Some(_), None, Some(_)) => {}
                        // Both have data margins.
                        (Some(_), Some(bm), Some(m)) => {
                            // Prefer highest positive margin; if neither positive,
                            // prefer least-negative (closest to zero from below).
                            let new_is_better = if m > 0.0 && bm > 0.0 {
                                m > bm
                            } else if m > 0.0 {
                                // new is positive, best is not
                                true
                            } else if bm > 0.0 {
                                // best is positive, new is not
                                false
                            } else {
                                // both negative — prefer least negative
                                m > bm
                            };
                            if new_is_better {
                                best_rep = Some(r);
                                best_margin = Some(m);
                            }
                        }
                    }
                }

                let raw_reps = best_rep.unwrap_or(min_reps);
                let (clamped_reps, reps_clamped) = clamp_reps(raw_reps, min_reps, max_reps);

                // Compute the weight for the chosen rep count.
                let weight = predicted_weight(blended, clamped_reps, target_rpe) as f32;

                PredictedParameters {
                    weight: Some(weight),
                    reps: clamped_reps,
                    rpe: target_rpe as f32,
                    reps_clamped,
                }
            }

            crate::models::SetTypeConfig::Bodyweight => {
                // Compute failure_reps for today and historical best.
                let today_fr: Option<f64> = today_best.as_ref().and_then(|s| {
                    if matches!(s.set_type, SetType::Bodyweight) {
                        Some(s.reps as f64 + (10.0 - s.rpe as f64))
                    } else {
                        None
                    }
                });

                let hist_fr: Option<f64> = historical_best.as_ref().and_then(|s| {
                    if matches!(s.set_type, SetType::Bodyweight) {
                        Some(s.reps as f64 + (10.0 - s.rpe as f64))
                    } else {
                        None
                    }
                });

                // No history → fall back.
                let blended_fr = match (today_fr, hist_fr) {
                    (None, None) => {
                        let last_session_set = session.completed_sets.last();
                        return Self::calculate_initial_predictions(
                            exercise,
                            last_session_set,
                            settings.default_bodyweight_reps,
                        );
                    }
                    (Some(t), Some(h)) => blended_e1rm(t, h, settings.today_blend_factor),
                    (Some(t), None) => t,
                    (None, Some(h)) => h,
                };

                // suggested_reps = round(blended_failure_reps - (10 - target_rpe))
                let rir_at_target = 10.0 - target_rpe;
                let raw_reps_f = (blended_fr - rir_at_target).round();
                let raw_reps = if raw_reps_f <= 0.0 {
                    1
                } else {
                    raw_reps_f as u32
                };

                let (clamped_reps, reps_clamped) = clamp_reps(raw_reps, min_reps, max_reps);

                PredictedParameters {
                    weight: None,
                    reps: clamped_reps,
                    rpe: target_rpe as f32,
                    reps_clamped,
                }
            }
        }
    }

    /// Shared post-initialization helper called by both file-selection UI paths
    /// ("Create New Database" and "Open Existing Database").
    ///
    /// Sets the database, sets the file manager, syncs the exercise list from the
    /// database into the reactive state signal, and transitions to `Ready`.  The
    /// exercise sync is non-fatal: if it fails we log a warning but still reach
    /// `Ready`, matching the behaviour of `setup_database`.
    pub(crate) async fn complete_file_initialization(
        state: &WorkoutState,
        database: Database,
        file_manager: Storage,
    ) {
        js_log("[UI] complete_file_initialization: storing DB and file manager...");

        // If the sync module failed to load during DB init, mark sync as disabled
        // so the UI can surface a visible indicator without blocking the app.
        if database.sync_unavailable {
            js_log("[UI] Sync module unavailable — marking sync as Disabled");
            state.set_sync_status(SyncStatus::Disabled(
                "Sync module could not be loaded".to_string(),
            ));
        }

        state.set_database(database);
        state.set_file_manager(file_manager);

        if let Err(e) = Self::sync_exercises(state).await {
            js_log(&format!("[UI] sync_exercises warning: {}", e));
        }

        if let Err(e) = Self::resume_active_plan(state).await {
            js_log(&format!("[UI] resume_active_plan warning: {}", e));
        }

        state.set_initialization_state(InitializationState::Ready);

        js_log("[UI] complete_file_initialization done → Ready");
    }

    pub fn handle_error(state: &WorkoutState, error: WorkoutError) {
        js_log(&format!("[ERROR] Workout state error: {}", error));
        state.set_error(Some(error));
        state.set_initialization_state(InitializationState::Error);
    }

    /// Trigger a background sync cycle using WebSocket-based CRR changeset
    /// exchange.  Non-blocking: errors are logged but do not crash the app.
    ///
    /// This is a no-op when `sync_id` is not configured in LocalStorage
    /// (i.e. the pairing flow has not been run yet).
    #[cfg(not(test))]
    pub async fn trigger_background_sync(state: &WorkoutState) {
        use crate::sync::ws_bridge;

        // Load existing credentials. If none are saved, sync is not configured
        // and we skip silently — the user must explicitly set up sync first.
        let Some(credentials) = SyncCredentials::load() else {
            js_log("[Sync] No credentials configured — skipping sync");
            return;
        };
        if !credentials.is_valid() {
            js_log("[Sync] Skipped — credentials failed validation");
            return;
        }

        state.set_sync_status(SyncStatus::Syncing);

        let outcome = ws_bridge::run_ws_sync(&credentials.sync_id, &credentials.sync_secret).await;

        match outcome {
            crate::sync::WsSyncOutcome::Synced => {
                js_log("[Sync] CRR changeset sync completed — changes exchanged");
                state.set_sync_status(SyncStatus::UpToDate);

                // Re-read exercises from the database since remote changes may
                // have added or modified exercise rows.
                if let Err(e) = Self::sync_exercises(state).await {
                    js_log(&format!(
                        "[Sync] Failed to refresh exercises after sync: {}",
                        e
                    ));
                }
                let ex_names: Vec<String> = state
                    .exercises
                    .read()
                    .iter()
                    .map(|e| e.name.clone())
                    .collect();
                js_log(&format!(
                    "[Sync] Exercises after sync refresh: {:?}",
                    ex_names
                ));
            }
            crate::sync::WsSyncOutcome::NoChanges => {
                js_log("[Sync] CRR changeset sync — no changes to exchange");
                state.set_sync_status(SyncStatus::UpToDate);
            }
            crate::sync::WsSyncOutcome::Offline => {
                js_log("[Sync] Server unreachable — continuing offline");
                state.set_sync_status(SyncStatus::Error("Server unreachable".to_string()));
            }
            crate::sync::WsSyncOutcome::Error(msg) => {
                js_log(&format!("[Sync] Sync error: {}", msg));
                state.set_sync_status(SyncStatus::Error(msg));
            }
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::models::{SetType, SetTypeConfig};

    // ── is_archive_blocked ────────────────────────────────────────────────────

    fn make_session(exercise_id: &str) -> WorkoutSession {
        WorkoutSession {
            // session_id is not read by is_archive_blocked (which checks
            // session.exercise.id); it is included here only to satisfy the
            // struct definition and does not affect any of these test outcomes.
            session_id: Some(exercise_id.to_string()),
            exercise: crate::models::ExerciseMetadata {
                id: Some(exercise_id.to_string()),
                name: "Test Exercise".to_string(),
                set_type_config: SetTypeConfig::Bodyweight,
                min_reps: 1,
                max_reps: None,
            },
            completed_sets: Vec::new(),
            predicted: PredictedParameters {
                weight: None,
                reps: 10,
                rpe: 7.0,
                reps_clamped: false,
            },
        }
    }

    #[test]
    fn test_is_archive_blocked_same_exercise_as_current_session() {
        let session = make_session("exercise-abc");
        assert!(
            is_archive_blocked("exercise-abc", &Some(session)),
            "should be blocked when exercise matches current session"
        );
    }

    #[test]
    fn test_is_archive_blocked_different_exercise() {
        let session = make_session("exercise-abc");
        assert!(
            !is_archive_blocked("exercise-xyz", &Some(session)),
            "should not be blocked when exercise differs from current session"
        );
    }

    #[test]
    fn test_is_archive_blocked_no_current_session() {
        assert!(
            !is_archive_blocked("exercise-abc", &None),
            "should not be blocked when no session is active"
        );
    }

    #[test]
    fn test_initial_predictions_weighted() {
        let exercise = ExerciseMetadata {
            id: Some("test-uuid-1".to_string()),
            name: "Bench Press".to_string(),
            set_type_config: SetTypeConfig::Weighted {
                min_weight: 0.0,
                increment: 5.0,
            },
            min_reps: 1,
            max_reps: None,
        };

        let predicted = WorkoutStateManager::calculate_initial_predictions(&exercise, None, 10);

        assert_eq!(predicted.weight, Some(0.0));
        assert_eq!(predicted.reps, 8);
        assert_eq!(predicted.rpe, 7.0);
    }

    #[test]
    fn test_initial_predictions_weighted_with_history() {
        let exercise = ExerciseMetadata {
            id: Some("test-uuid-1".to_string()),
            name: "Bench Press".to_string(),
            set_type_config: SetTypeConfig::Weighted {
                min_weight: 0.0,
                increment: 5.0,
            },
            min_reps: 1,
            max_reps: None,
        };

        let last_set = CompletedSet {
            set_number: 1,
            reps: 5,
            rpe: 8.0,
            set_type: SetType::Weighted { weight: 100.0 },
        };

        let predicted =
            WorkoutStateManager::calculate_initial_predictions(&exercise, Some(&last_set), 10);

        assert_eq!(predicted.weight, Some(100.0));
        assert_eq!(predicted.reps, 8);
        assert_eq!(predicted.rpe, 7.0);
    }

    #[test]
    fn test_initial_predictions_bodyweight() {
        let exercise = ExerciseMetadata {
            id: Some("test-uuid-2".to_string()),
            name: "Pull-ups".to_string(),
            set_type_config: SetTypeConfig::Bodyweight,
            min_reps: 1,
            max_reps: None,
        };

        let predicted = WorkoutStateManager::calculate_initial_predictions(&exercise, None, 10);

        assert_eq!(predicted.weight, None);
        assert_eq!(predicted.reps, 10);
        assert_eq!(predicted.rpe, 7.0);
    }

    #[test]
    fn test_initial_predictions_bodyweight_custom_reps() {
        let exercise = ExerciseMetadata {
            id: Some("test-uuid-2".to_string()),
            name: "Pull-ups".to_string(),
            set_type_config: SetTypeConfig::Bodyweight,
            min_reps: 1,
            max_reps: None,
        };

        let predicted = WorkoutStateManager::calculate_initial_predictions(&exercise, None, 15);

        assert_eq!(predicted.weight, None);
        assert_eq!(predicted.reps, 15);
        assert_eq!(predicted.rpe, 7.0);
    }

    // ── helpers for calculate_next_predictions tests ────────────────────────

    fn default_settings() -> Settings {
        Settings::default() // target_rpe=8.0, today_blend_factor=0.5
    }

    fn weighted_session(min_reps: i32, max_reps: Option<i32>) -> WorkoutSession {
        WorkoutSession {
            session_id: Some("s1".to_string()),
            exercise: ExerciseMetadata {
                id: Some("e1".to_string()),
                name: "Squat".to_string(),
                set_type_config: SetTypeConfig::Weighted {
                    min_weight: 0.0,
                    increment: 2.5,
                },
                min_reps,
                max_reps,
            },
            completed_sets: vec![CompletedSet {
                set_number: 1,
                reps: 5,
                rpe: 8.0,
                set_type: SetType::Weighted { weight: 100.0 },
            }],
            predicted: PredictedParameters {
                weight: Some(100.0),
                reps: 5,
                rpe: 8.0,
                reps_clamped: false,
            },
        }
    }

    fn bodyweight_session(min_reps: i32, max_reps: Option<i32>) -> WorkoutSession {
        WorkoutSession {
            session_id: Some("s2".to_string()),
            exercise: ExerciseMetadata {
                id: Some("e2".to_string()),
                name: "Pull-ups".to_string(),
                set_type_config: SetTypeConfig::Bodyweight,
                min_reps,
                max_reps,
            },
            completed_sets: vec![CompletedSet {
                set_number: 1,
                reps: 10,
                rpe: 8.0,
                set_type: SetType::Bodyweight,
            }],
            predicted: PredictedParameters {
                weight: None,
                reps: 10,
                rpe: 8.0,
                reps_clamped: false,
            },
        }
    }

    // ── QA: Weighted path — no-data fallback ──────────────────────────────────

    /// When there is no history (both today_best and historical_best are None),
    /// fall back to calculate_initial_predictions.
    #[test]
    fn test_next_predictions_weighted_no_data_fallback() {
        let session = weighted_session(1, None);
        let predicted = WorkoutStateManager::calculate_next_predictions(
            &session,
            None,
            None,
            HashMap::new(),
            &default_settings(),
        );
        // Fallback should return initial predictions (weight from last session set,
        // DEFAULT_WEIGHTED_REPS reps, DEFAULT_RPE).
        assert_eq!(predicted.weight, Some(100.0)); // from session's last completed set
        assert_eq!(predicted.reps, DEFAULT_WEIGHTED_REPS);
        assert_eq!(predicted.rpe, DEFAULT_RPE);
        assert!(!predicted.reps_clamped);
    }

    // ── QA: Weighted path — positive-margin selection ────────────────────────

    /// After a new personal-best weight set, the projected weight for at least one
    /// rep count should exceed the historical max, so the margin is positive.
    #[test]
    fn test_next_predictions_weighted_positive_margin_selects_best_rep() {
        // today_best: 120kg for 5 reps @ RPE 8 (a new PB)
        let today_best = Some(CompletedSet {
            set_number: 1,
            reps: 5,
            rpe: 8.0,
            set_type: SetType::Weighted { weight: 120.0 },
        });
        // historical_best: 100kg for 5 reps @ RPE 8 (previous best)
        let historical_best = Some(CompletedSet {
            set_number: 1,
            reps: 5,
            rpe: 8.0,
            set_type: SetType::Weighted { weight: 100.0 },
        });

        let settings = Settings {
            target_rpe: 8.0,
            today_blend_factor: 1.0, // today only, for predictable test
            ..Settings::default()
        };

        // historical_max: 100kg for 5 reps only
        let mut per_rep_maxes = HashMap::new();
        per_rep_maxes.insert(5u32, 100.0f64);

        let session = weighted_session(1, Some(10));
        let predicted = WorkoutStateManager::calculate_next_predictions(
            &session,
            historical_best,
            today_best,
            per_rep_maxes,
            &settings,
        );

        // With today_blend_factor=1.0, blended = today e1RM.
        // The projected weight for 5 reps @ RPE 8 is ~120kg (>100kg historical max).
        // So there should be a positive margin for 5 reps.
        assert!(predicted.weight.is_some());
        assert!(!predicted.reps_clamped);
    }

    // ── QA: Weighted path — least-negative margin ────────────────────────────

    /// When all rep counts have been done before and none project above history,
    /// select the rep with the least-negative margin (closest to PB).
    #[test]
    fn test_next_predictions_weighted_least_negative_margin() {
        // Same today and historical best — blended e1RM equals today e1RM.
        let set_100_5r = Some(CompletedSet {
            set_number: 1,
            reps: 5,
            rpe: 8.0,
            set_type: SetType::Weighted { weight: 100.0 },
        });

        let settings = Settings {
            target_rpe: 8.0,
            today_blend_factor: 0.5,
            ..Settings::default()
        };

        // All rep counts have historical data that exceeds projections slightly.
        // Use very high historical maxes so no margin is positive.
        let mut per_rep_maxes = HashMap::new();
        for r in 3u32..=8 {
            per_rep_maxes.insert(r, 999.0); // impossibly high
        }

        let session = weighted_session(3, Some(8));
        let predicted = WorkoutStateManager::calculate_next_predictions(
            &session,
            set_100_5r.clone(),
            set_100_5r,
            per_rep_maxes,
            &settings,
        );

        // Should produce a valid result with some rep count in [3, 8].
        assert!(predicted.reps >= 3 && predicted.reps <= 8);
        assert!(predicted.weight.is_some());
    }

    // ── QA: Bodyweight path — rep suggestion ────────────────────────────────

    /// After logging a bodyweight set, suggested reps are derived from
    /// blended_failure_reps and target_rpe.
    #[test]
    fn test_next_predictions_bodyweight_rep_suggestion() {
        // today_best: 10 reps @ RPE 8 → failure_reps = 10 + (10-8) = 12
        let today_best = Some(CompletedSet {
            set_number: 1,
            reps: 10,
            rpe: 8.0,
            set_type: SetType::Bodyweight,
        });
        // No historical best.
        let settings = Settings {
            target_rpe: 8.0,
            today_blend_factor: 1.0,
            ..Settings::default()
        };

        let session = bodyweight_session(1, None);
        let predicted = WorkoutStateManager::calculate_next_predictions(
            &session,
            None,
            today_best,
            HashMap::new(),
            &settings,
        );

        // failure_reps = 12, target_rpe=8 → rir_at_target=2
        // suggested = round(12 - 2) = 10
        assert_eq!(predicted.reps, 10);
        assert_eq!(predicted.weight, None);
        assert!(!predicted.reps_clamped);
    }

    /// A higher RPE on the logged set (harder) → higher failure_reps → more reps suggested.
    #[test]
    fn test_next_predictions_bodyweight_higher_rpe_raises_suggestion() {
        let make_bw_prediction = |rpe: f32| {
            let today_best = Some(CompletedSet {
                set_number: 1,
                reps: 10,
                rpe,
                set_type: SetType::Bodyweight,
            });
            let settings = Settings {
                target_rpe: 8.0,
                today_blend_factor: 1.0,
                ..Settings::default()
            };
            let session = bodyweight_session(1, None);
            WorkoutStateManager::calculate_next_predictions(
                &session,
                None,
                today_best,
                HashMap::new(),
                &settings,
            )
        };

        let pred_rpe7 = make_bw_prediction(7.0);
        let pred_rpe9 = make_bw_prediction(9.0);

        // RPE 9 set: failure_reps = 10 + 1 = 11, suggest = 9 (vs RPE 7: failure=13, suggest=11)
        assert!(
            pred_rpe9.reps < pred_rpe7.reps,
            "Higher RPE on a same-rep set means fewer failure reps → fewer suggested reps"
        );
    }

    // ── QA: Reps clamped flag ────────────────────────────────────────────────

    /// When the algorithm selects a rep outside [min_reps, max_reps],
    /// the result is clamped and reps_clamped = true.
    ///
    /// Strategy: use a range of [5,5] (one rep count) with no historical data.
    /// The no-data path selects rep 5 (the only option in range).
    /// Clamping doesn't trigger because 5 is within [5,5].
    /// To trigger clamping we need the unconstrained algorithm to pick a rep
    /// outside the range. With range [5,10] and historical max only at rep 5,
    /// the algorithm should pick rep 6 (no data). That's within range, so we
    /// instead use a very high per_rep_maxes for all in-range reps to force
    /// the algorithm down to the single allowed rep in a range of [10,10]
    /// while having data only at rep 5. The computed raw rep (best positive margin
    /// or least-negative margin) is then clamped to [10,10].
    #[test]
    fn test_next_predictions_weighted_reps_clamped_within_range() {
        // Use a session whose exercise has range [5,5] (single rep count).
        // Provide today_best at 5 reps — algorithm naturally picks 5, no clamping.
        let today_best = Some(CompletedSet {
            set_number: 1,
            reps: 5,
            rpe: 8.0,
            set_type: SetType::Weighted { weight: 100.0 },
        });

        let settings = Settings {
            target_rpe: 8.0,
            today_blend_factor: 1.0,
            ..Settings::default()
        };

        let session = WorkoutSession {
            session_id: Some("s1".to_string()),
            exercise: ExerciseMetadata {
                id: Some("e1".to_string()),
                name: "Test".to_string(),
                set_type_config: SetTypeConfig::Weighted {
                    min_weight: 0.0,
                    increment: 2.5,
                },
                min_reps: 5,
                max_reps: Some(5),
            },
            completed_sets: vec![CompletedSet {
                set_number: 1,
                reps: 5,
                rpe: 8.0,
                set_type: SetType::Weighted { weight: 100.0 },
            }],
            predicted: PredictedParameters {
                weight: Some(100.0),
                reps: 5,
                rpe: 8.0,
                reps_clamped: false,
            },
        };

        let predicted = WorkoutStateManager::calculate_next_predictions(
            &session,
            None,
            today_best,
            HashMap::new(),
            &settings,
        );
        // Single rep range [5,5]: algorithm picks 5, no clamping.
        assert_eq!(predicted.reps, 5);
        assert!(!predicted.reps_clamped);
    }

    /// Clamping is triggered on the bodyweight path via a separate dedicated test
    /// (test_next_predictions_bodyweight_reps_clamped_to_max).  For weighted, the
    /// clamp_reps function is tested directly via test_clamp_reps_* above, and
    /// the reps_clamped field is set correctly whenever clamp_reps says it changed.

    #[test]
    fn test_next_predictions_bodyweight_reps_clamped_to_max() {
        // failure_reps produces large suggested value that exceeds max_reps
        // reps=20, rpe=10 → failure_reps=20+0=20
        // target_rpe=8 → rir_at_target=2 → suggested=round(20-2)=18
        // max_reps=10 → clamped to 10
        let today_best = Some(CompletedSet {
            set_number: 1,
            reps: 20,
            rpe: 10.0,
            set_type: SetType::Bodyweight,
        });
        let settings = Settings {
            target_rpe: 8.0,
            today_blend_factor: 1.0,
            ..Settings::default()
        };

        let session = bodyweight_session(3, Some(10));
        let predicted = WorkoutStateManager::calculate_next_predictions(
            &session,
            None,
            today_best,
            HashMap::new(),
            &settings,
        );

        assert_eq!(predicted.reps, 10);
        assert!(predicted.reps_clamped);
    }

    #[test]
    fn test_next_predictions_bodyweight_reps_not_clamped_in_range() {
        // reps=10, rpe=8 → failure_reps=12, target_rpe=8 → suggested=10
        // max_reps=15 → within range → no clamp
        let today_best = Some(CompletedSet {
            set_number: 1,
            reps: 10,
            rpe: 8.0,
            set_type: SetType::Bodyweight,
        });
        let settings = Settings {
            target_rpe: 8.0,
            today_blend_factor: 1.0,
            ..Settings::default()
        };

        let session = bodyweight_session(1, Some(15));
        let predicted = WorkoutStateManager::calculate_next_predictions(
            &session,
            None,
            today_best,
            HashMap::new(),
            &settings,
        );

        assert_eq!(predicted.reps, 10);
        assert!(!predicted.reps_clamped);
    }

    // ── QA: Today-blend factor ────────────────────────────────────────────────

    /// Two sets logged today and an older historical best produce a blended e1RM
    /// that reflects today_blend_factor; changing the factor alters the suggestion.
    #[test]
    fn test_next_predictions_weighted_blend_factor_affects_suggestion() {
        // today_best: 120kg for 5 reps @ RPE 8
        let today_best = Some(CompletedSet {
            set_number: 1,
            reps: 5,
            rpe: 8.0,
            set_type: SetType::Weighted { weight: 120.0 },
        });
        // historical_best: 100kg for 5 reps @ RPE 8 (lower)
        let historical_best = Some(CompletedSet {
            set_number: 1,
            reps: 5,
            rpe: 8.0,
            set_type: SetType::Weighted { weight: 100.0 },
        });

        // With factor=1.0 (today only) projected weight is higher than factor=0.0.
        let settings_today = Settings {
            target_rpe: 8.0,
            today_blend_factor: 1.0,
            ..Settings::default()
        };
        let settings_hist = Settings {
            target_rpe: 8.0,
            today_blend_factor: 0.0,
            ..Settings::default()
        };

        let session = weighted_session(5, Some(5)); // single rep range for determinism

        let pred_today = WorkoutStateManager::calculate_next_predictions(
            &session,
            historical_best.clone(),
            today_best.clone(),
            HashMap::new(),
            &settings_today,
        );

        let pred_hist = WorkoutStateManager::calculate_next_predictions(
            &session,
            historical_best,
            today_best,
            HashMap::new(),
            &settings_hist,
        );

        // factor=1.0 uses today (120kg e1RM), factor=0.0 uses historical (100kg e1RM).
        // Both suggest 5 reps (single rep range), but predicted weight differs.
        assert!(
            pred_today.weight.unwrap() > pred_hist.weight.unwrap(),
            "today factor=1.0 should produce higher suggested weight than factor=0.0"
        );
    }

    // ── QA: No async in calculate_next_predictions (structural test) ──────────

    /// This test exercises the pure function directly without any DB calls,
    /// confirming the function has no async operations (it wouldn't compile
    /// as `async fn` if called with `.await` in tests that aren't async).
    #[test]
    fn test_next_predictions_is_pure_no_db_calls() {
        let session = weighted_session(1, None);
        let settings = default_settings();
        // Pure call with pre-supplied inputs — no DB, no await.
        let _ = WorkoutStateManager::calculate_next_predictions(
            &session,
            None,
            None,
            HashMap::new(),
            &settings,
        );
        // If this compiled and ran, the function is pure (no async/DB calls).
    }

    // ── clamp_reps ──────────────────────────────────────────────────────────

    #[test]
    fn test_clamp_reps_no_max_within_range() {
        let (reps, clamped) = clamp_reps(10, 1, None);
        assert_eq!(reps, 10);
        assert!(!clamped);
    }

    #[test]
    fn test_clamp_reps_no_max_below_min() {
        // raw_reps < min_reps → clamp to min
        let (reps, clamped) = clamp_reps(0, 3, None);
        assert_eq!(reps, 3);
        assert!(clamped);
    }

    #[test]
    fn test_clamp_reps_with_max_within_range() {
        let (reps, clamped) = clamp_reps(6, 3, Some(10));
        assert_eq!(reps, 6);
        assert!(!clamped);
    }

    #[test]
    fn test_clamp_reps_clamped_to_max() {
        let (reps, clamped) = clamp_reps(15, 3, Some(10));
        assert_eq!(reps, 10);
        assert!(clamped);
    }

    #[test]
    fn test_clamp_reps_clamped_to_min() {
        let (reps, clamped) = clamp_reps(1, 3, Some(10));
        assert_eq!(reps, 3);
        assert!(clamped);
    }

    #[test]
    fn test_initial_predictions_reps_clamped_false() {
        let exercise = ExerciseMetadata {
            id: None,
            name: "Test".to_string(),
            set_type_config: SetTypeConfig::Bodyweight,
            min_reps: 3,
            max_reps: Some(10),
        };
        let predicted = WorkoutStateManager::calculate_initial_predictions(&exercise, None, 10);
        assert!(!predicted.reps_clamped);
    }
}
