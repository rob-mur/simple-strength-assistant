use crate::models::{CompletedSet, ExerciseMetadata, SetType, Settings};
use crate::state::{Database, Storage, error::WorkoutError};
#[cfg(not(test))]
use crate::sync::SyncCredentials;
use dioxus::prelude::*;
use wasm_bindgen::JsValue;

/// Write directly to browser `console.log` – always visible in Playwright
/// output regardless of the Rust log level or compile profile.
fn js_log(msg: &str) {
    web_sys::console::log_1(&JsValue::from_str(msg));
}

// Initial prediction constants
const DEFAULT_WEIGHTED_REPS: u32 = 8;
const DEFAULT_BODYWEIGHT_REPS: u32 = 10;
const DEFAULT_RPE: f32 = 7.0;
const RPE_THRESHOLD_HIGH: f32 = 8.0;
const RPE_THRESHOLD_LOW: f32 = 7.0;
const RPE_REDUCTION: f32 = 0.5;
const RPE_MINIMUM: f32 = 6.0;

#[derive(Clone, Copy, Debug, PartialEq)]
pub struct PredictedParameters {
    pub weight: Option<f32>,
    pub reps: u32,
    pub rpe: f32,
}

#[derive(Clone, Debug, PartialEq)]
pub struct WorkoutSession {
    pub session_id: Option<i64>,
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

        state.set_database(database);
        state.set_file_manager(file_manager);

        if let Err(e) = Self::sync_exercises(state).await {
            js_log(&format!("[DB Init] sync_exercises warning: {}", e));
        }

        if let Err(e) = Self::load_settings(state).await {
            js_log(&format!("[DB Init] load_settings warning: {}", e));
        }

        state.set_initialization_state(InitializationState::Ready);

        js_log("[DB Init] Setup complete! State is now Ready");
        Ok(())
    }

    pub async fn save_exercise(
        state: &WorkoutState,
        exercise: ExerciseMetadata,
    ) -> Result<i64, WorkoutError> {
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
        exercise.id = Some(id);

        // Fetch last set for suggestions (only for weighted exercises)
        let last_set = match exercise.set_type_config {
            crate::models::SetTypeConfig::Weighted { .. } => {
                db.get_last_set_for_exercise(id).await.unwrap_or_else(|e| {
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

        let predicted = Self::calculate_initial_predictions(&exercise, last_set.as_ref());

        // Use exercise_id as session_id so the UI can detect a new session started
        let session = WorkoutSession {
            session_id: exercise.id,
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
            .ok_or(WorkoutError::SessionNotPersisted)?;

        let db = state.database().ok_or(WorkoutError::NotInitialized)?;

        crate::models::validate_completed_set(&set, &session.exercise)
            .map_err(|e| WorkoutError::InvalidSetData(e.to_string()))?;

        let _set_id =
            db.log_set(exercise_id, &set)
                .await
                .map_err(|e: crate::state::DatabaseError| {
                    WorkoutError::InsertSetError(e.to_string())
                })?;

        session.completed_sets.push(set.clone());
        session.predicted = Self::calculate_next_predictions(&session);

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

    fn calculate_initial_predictions(
        exercise: &ExerciseMetadata,
        last_set: Option<&CompletedSet>,
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
                }
            }
            crate::models::SetTypeConfig::Bodyweight => PredictedParameters {
                weight: None,
                reps: DEFAULT_BODYWEIGHT_REPS,
                rpe: DEFAULT_RPE,
            },
        }
    }

    fn calculate_next_predictions(session: &WorkoutSession) -> PredictedParameters {
        if session.completed_sets.is_empty() {
            // Note: This path is unreachable in normal UI flow because initial prediction from start_session
            // is stored in session.predicted, and calculate_next_predictions is only called after a set is completed.
            // If it is ever called with 0 sets, it returns min_weight (ignoring history).
            return Self::calculate_initial_predictions(&session.exercise, None);
        }

        let last_set = &session.completed_sets[session.completed_sets.len() - 1];

        let predicted_rpe = if last_set.rpe < RPE_THRESHOLD_HIGH {
            last_set.rpe
        } else {
            (last_set.rpe - RPE_REDUCTION).max(RPE_MINIMUM)
        };

        match (&last_set.set_type, &session.exercise.set_type_config) {
            (
                SetType::Weighted { weight },
                crate::models::SetTypeConfig::Weighted { increment, .. },
            ) => {
                let predicted_weight = if last_set.rpe < RPE_THRESHOLD_LOW {
                    weight + increment
                } else {
                    *weight
                };

                PredictedParameters {
                    weight: Some(predicted_weight),
                    reps: last_set.reps,
                    rpe: predicted_rpe,
                }
            }
            (SetType::Bodyweight, _) => PredictedParameters {
                weight: None,
                reps: last_set.reps,
                rpe: predicted_rpe,
            },
            // Fallback for unexpected state: Within an active session, suggestions
            // should come from the session's own sets. If this fails, we fall back
            // to initial predictions ignoring previous session history to maintain
            // focus on the current session's performance.
            _ => Self::calculate_initial_predictions(&session.exercise, None),
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
        state.set_database(database);
        state.set_file_manager(file_manager);

        if let Err(e) = Self::sync_exercises(state).await {
            js_log(&format!("[UI] sync_exercises warning: {}", e));
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
            log::debug!("[Sync] No credentials configured — skipping sync");
            return;
        };
        if !credentials.is_valid() {
            log::debug!("[Sync] Skipped — credentials failed validation");
            return;
        }

        state.set_sync_status(SyncStatus::Syncing);

        let outcome = ws_bridge::run_ws_sync(&credentials.sync_id, &credentials.sync_secret).await;

        match outcome {
            crate::sync::WsSyncOutcome::Synced => {
                log::info!("[Sync] CRR changeset sync completed — changes exchanged");
                state.set_sync_status(SyncStatus::UpToDate);

                // Re-read exercises from the database since remote changes may
                // have added or modified exercise rows.
                if let Err(e) = Self::sync_exercises(state).await {
                    log::warn!("[Sync] Failed to refresh exercises after sync: {}", e);
                }
            }
            crate::sync::WsSyncOutcome::NoChanges => {
                log::debug!("[Sync] CRR changeset sync — no changes to exchange");
                state.set_sync_status(SyncStatus::UpToDate);
            }
            crate::sync::WsSyncOutcome::Offline => {
                log::warn!("[Sync] Server unreachable — continuing offline");
                state.set_sync_status(SyncStatus::Error("Server unreachable".to_string()));
            }
            crate::sync::WsSyncOutcome::Error(msg) => {
                log::warn!("[Sync] Sync error: {}", msg);
                state.set_sync_status(SyncStatus::Error(msg));
            }
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::models::{SetType, SetTypeConfig};

    #[test]
    fn test_initial_predictions_weighted() {
        let exercise = ExerciseMetadata {
            id: Some(1),
            name: "Bench Press".to_string(),
            set_type_config: SetTypeConfig::Weighted {
                min_weight: 0.0,
                increment: 5.0,
            },
            min_reps: 1,
            max_reps: None,
        };

        let predicted = WorkoutStateManager::calculate_initial_predictions(&exercise, None);

        assert_eq!(predicted.weight, Some(0.0));
        assert_eq!(predicted.reps, 8);
        assert_eq!(predicted.rpe, 7.0);
    }

    #[test]
    fn test_initial_predictions_weighted_with_history() {
        let exercise = ExerciseMetadata {
            id: Some(1),
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
            WorkoutStateManager::calculate_initial_predictions(&exercise, Some(&last_set));

        assert_eq!(predicted.weight, Some(100.0));
        assert_eq!(predicted.reps, 8);
        assert_eq!(predicted.rpe, 7.0);
    }

    #[test]
    fn test_initial_predictions_bodyweight() {
        let exercise = ExerciseMetadata {
            id: Some(2),
            name: "Pull-ups".to_string(),
            set_type_config: SetTypeConfig::Bodyweight,
            min_reps: 1,
            max_reps: None,
        };

        let predicted = WorkoutStateManager::calculate_initial_predictions(&exercise, None);

        assert_eq!(predicted.weight, None);
        assert_eq!(predicted.reps, 10);
        assert_eq!(predicted.rpe, 7.0);
    }

    #[test]
    fn test_next_predictions_progression() {
        let exercise = ExerciseMetadata {
            id: Some(3),
            name: "Bench Press".to_string(),
            set_type_config: SetTypeConfig::Weighted {
                min_weight: 0.0,
                increment: 5.0,
            },
            min_reps: 1,
            max_reps: None,
        };

        let session = WorkoutSession {
            session_id: Some(1),
            exercise,
            completed_sets: vec![CompletedSet {
                set_number: 1,
                reps: 8,
                rpe: 6.5,
                set_type: SetType::Weighted { weight: 100.0 },
            }],
            predicted: PredictedParameters {
                weight: Some(100.0),
                reps: 8,
                rpe: 7.0,
            },
        };

        let predicted = WorkoutStateManager::calculate_next_predictions(&session);

        assert_eq!(predicted.weight, Some(105.0));
        assert_eq!(predicted.reps, 8);
        assert_eq!(predicted.rpe, 6.5);
    }
}
