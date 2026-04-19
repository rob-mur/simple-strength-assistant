use crate::models::{CompletedSet, ExerciseMetadata, SetType, Settings};
use crate::state::{Database, Storage, error::WorkoutError};
use crate::sync::ConflictRecord;
#[cfg(not(test))]
use crate::sync::SyncCredentials;
use crate::sync::SyncOutcome;
use crate::sync::VectorClock;
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
/// `ConflictsDetected`  - the merge found true conflicts that require user resolution.
///                        Actual conflict data is stored in `WorkoutState::pending_conflicts`.
#[derive(Clone, PartialEq, Debug, Default)]
pub enum SyncStatus {
    #[default]
    Idle,
    NeverSynced,
    Syncing,
    UpToDate,
    Error(String),
    ConflictsDetected,
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
            SyncStatus::ConflictsDetected => "conflicts",
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
    /// Local vector clock, persisted across sync cycles
    sync_clock: Signal<VectorClock>,
    /// Global application settings (target RPE, history window, blend factor).
    settings: Signal<Settings>,
    /// Pending conflicts from a sync merge that the user needs to resolve.
    pending_conflicts: Signal<Vec<ConflictRecord>>,
    /// The merged database blob waiting for conflict resolution before being committed.
    pending_merged_blob: Signal<Option<Vec<u8>>>,
}

impl Default for WorkoutState {
    fn default() -> Self {
        Self::new()
    }
}

impl WorkoutState {
    pub fn new() -> Self {
        // Start with an empty clock. In production, the persisted clock is
        // loaded via `load_persisted_clock()` during `setup_database()` so
        // sync resumes from the correct sequence numbers after page reloads.
        //
        // Note: calling `crate::sync::load_clock()` directly here (even
        // behind `#[cfg(not(test))]`) breaks Dioxus 0.7.x SSR rendering —
        // cross-module function calls inside `Signal::new()` constructors
        // cause the virtual DOM to produce empty output. See #95.
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
            sync_clock: Signal::new(VectorClock::new()),
            pending_conflicts: Signal::new(Vec::new()),
            pending_merged_blob: Signal::new(None),
        }
    }

    /// Load the persisted vector clock from LocalStorage (production only).
    /// Called during database setup so sync resumes from the correct state.
    pub fn load_persisted_clock(&self) {
        #[cfg(not(test))]
        {
            let clock = crate::sync::load_clock();
            self.set_sync_clock(clock);
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

    pub fn sync_clock(&self) -> VectorClock {
        (self.sync_clock)()
    }

    pub fn set_sync_clock(&self, clock: VectorClock) {
        let mut sig = self.sync_clock;
        sig.set(clock);
    }

    pub fn pending_conflicts(&self) -> Vec<ConflictRecord> {
        (self.pending_conflicts)()
    }

    pub fn set_pending_conflicts(&self, conflicts: Vec<ConflictRecord>) {
        let mut sig = self.pending_conflicts;
        sig.set(conflicts);
    }

    pub fn pending_merged_blob(&self) -> Option<Vec<u8>> {
        (self.pending_merged_blob)()
    }

    pub fn set_pending_merged_blob(&self, blob: Option<Vec<u8>>) {
        let mut sig = self.pending_merged_blob;
        sig.set(blob);
    }

    /// Returns true if there are unresolved sync conflicts pending.
    pub fn has_pending_conflicts(&self) -> bool {
        !self.pending_conflicts().is_empty()
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

        state.load_persisted_clock();

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

        state.load_persisted_clock();
        state.set_initialization_state(InitializationState::Ready);

        js_log("[UI] complete_file_initialization done → Ready");
    }

    pub fn handle_error(state: &WorkoutState, error: WorkoutError) {
        js_log(&format!("[ERROR] Workout state error: {}", error));
        state.set_error(Some(error));
        state.set_initialization_state(InitializationState::Error);
    }

    /// Map a `SyncOutcome` to the corresponding `SyncStatus` for the UI indicator.
    ///
    /// This is a pure function so it can be unit-tested without wasm dependencies.
    pub fn map_sync_outcome_to_status(outcome: &SyncOutcome) -> SyncStatus {
        match outcome {
            SyncOutcome::Pushed => SyncStatus::UpToDate,
            SyncOutcome::Pulled(_) => SyncStatus::UpToDate,
            SyncOutcome::Merged(_) => SyncStatus::UpToDate,
            SyncOutcome::Offline => SyncStatus::Error("Server unreachable".into()),
            SyncOutcome::ConflictDetected { .. } => SyncStatus::ConflictsDetected,
            SyncOutcome::Skipped => SyncStatus::Idle,
        }
    }

    /// Trigger a background sync cycle.  Non-blocking: errors are swallowed
    /// except for `ConflictDetected`, which surfaces the conflict resolution UI.
    ///
    /// This is a no-op when `sync_id` is not configured in LocalStorage
    /// (i.e. the pairing flow has not been run yet).
    #[cfg(not(test))]
    pub async fn trigger_background_sync(state: &WorkoutState) {
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

        // Sync is disabled: the old blob-level protocol is incompatible with
        // the new CRR-backed database.  Rather than silently running no-op
        // network calls, we short-circuit here and surface the state to the
        // user.  Remove this gate once CRR changeset exchange is implemented.
        let _ = credentials; // validated above; will be used by CRR sync
        log::info!(
            "[Sync] Sync is temporarily disabled while the protocol is migrated to CRR changesets"
        );
        state.set_sync_status(SyncStatus::Error(
            "Sync is temporarily unavailable — update in progress".to_string(),
        ));
        // TODO(sync): Once CRR changeset exchange is implemented, replace this
        // early return with:
        // 1. Extract local changesets via `crsql_changes()`
        // 2. Send changesets to server via SyncClient
        // 3. Apply server changesets locally
        // 4. Update vector clock and sync status
        // See git history for the previous blob-level sync implementation.
    }

    /// Shared helper for `Pulled` and `Merged` sync outcomes: initialise a new
    /// database from the given blob and sync exercises.
    ///
    /// With crsqlite-wasm, persistence is handled automatically via IndexedDB.
    /// On failure the existing in-memory database is left untouched so the
    /// session remains functional.
    // TODO(sync): Re-enable when CRR changeset exchange is implemented.
    #[cfg(not(test))]
    #[allow(dead_code)]
    async fn apply_synced_blob(state: &WorkoutState, blob: &[u8], label: &str) {
        // Rust-layer defence: reject empty blobs before handing them to the JS
        // layer.  An empty blob is never a valid SQLite database.
        if blob.is_empty() {
            log::warn!(
                "[Sync] Ignoring empty {} blob — existing database left intact",
                label
            );
            return;
        }

        let mut new_db = Database::new();
        match new_db.init(Some(blob.to_vec())).await {
            Ok(_) => {
                // crsqlite-wasm auto-persists via IndexedDB — no OPFS write needed.
                state.set_database(new_db);
                if let Err(e) = Self::sync_exercises(state).await {
                    log::warn!("[Sync] Failed to sync exercises after {}: {}", label, e);
                }
            }
            Err(e) => {
                log::warn!(
                    "[Sync] Failed to init DB from {} blob (existing DB left intact): {}",
                    label,
                    e
                );
            }
        }
    }

    /// Store pending conflicts from a sync merge so the UI can display the
    /// conflict resolution screen.
    pub fn set_conflict_state(
        state: &WorkoutState,
        conflicts: Vec<ConflictRecord>,
        merged_blob: Vec<u8>,
    ) {
        state.set_pending_conflicts(conflicts);
        state.set_pending_merged_blob(Some(merged_blob));
    }

    /// Apply the user's conflict resolutions to the merged database blob.
    ///
    /// `choices` maps each conflict's `row_id` to a boolean:
    /// - `true`  = keep version A (local)
    /// - `false` = keep version B (remote)
    ///
    /// The method rewrites the conflicting rows in the merged blob according
    /// to the user's choices, saves the result to OPFS, and pushes to the server.
    pub async fn apply_conflict_resolutions(
        state: &WorkoutState,
        choices: &std::collections::HashMap<String, bool>,
    ) -> Result<(), WorkoutError> {
        let merged_blob = state
            .pending_merged_blob()
            .ok_or(WorkoutError::NotInitialized)?;
        let conflicts = state.pending_conflicts();

        // Build a JSON array of resolution instructions for the JS side.
        // Each entry tells the DB module which version's data to write for a given UUID.
        let mut resolutions = Vec::new();
        for conflict in &conflicts {
            let keep_a = choices.get(&conflict.row_id).copied().unwrap_or(true);
            let chosen_version = if keep_a {
                &conflict.version_a
            } else {
                &conflict.version_b
            };
            resolutions.push(serde_json::json!({
                "uuid": conflict.row_id,
                "table": conflict.table,
                "chosen_version": chosen_version,
            }));
        }

        log::info!("[Conflict] Applying {} resolutions", resolutions.len());

        // Initialize a new database from the merged blob
        let mut resolved_db = Database::new();
        resolved_db
            .init(Some(merged_blob.clone()))
            .await
            .map_err(|e| {
                log::error!("[Conflict] Failed to init DB from merged blob: {}", e);
                WorkoutError::Database(e)
            })?;

        // Whitelist of allowed table and column names matching the known schema.
        // This prevents SQL injection via server-controlled ConflictRecord data.
        use std::collections::HashSet;

        let allowed_tables: HashSet<&str> =
            ["exercises", "completed_sets"].iter().copied().collect();
        let allowed_columns: HashSet<&str> = [
            "id",
            "name",
            "is_weighted",
            "min_weight",
            "increment",
            "exercise_id",
            "set_number",
            "reps",
            "rpe",
            "weight",
            "is_bodyweight",
            "recorded_at",
            "uuid",
            "updated_at",
            "deleted_at",
        ]
        .iter()
        .copied()
        .collect();

        // Apply each resolution by updating the row in the database
        for resolution in &resolutions {
            let uuid = resolution["uuid"].as_str().unwrap_or_default();
            let table = resolution["table"].as_str().unwrap_or_default();
            let chosen_json = resolution["chosen_version"].as_str().unwrap_or_default();

            // Reject unknown table names
            if !allowed_tables.contains(table) {
                log::warn!(
                    "[Conflict] Rejecting unknown table '{}' for row {}",
                    table,
                    uuid
                );
                continue;
            }

            // Parse the chosen version and build an UPDATE statement
            let fields: std::collections::HashMap<String, serde_json::Value> =
                serde_json::from_str(chosen_json).unwrap_or_default();

            // Build SET clause from the chosen version's fields (excluding uuid)
            let mut set_parts = Vec::new();
            let mut values = Vec::new();
            for (key, val) in &fields {
                if key == "uuid" {
                    continue;
                }
                // Reject unknown column names
                if !allowed_columns.contains(key.as_str()) {
                    log::warn!(
                        "[Conflict] Rejecting unknown column '{}' in table '{}' for row {}",
                        key,
                        table,
                        uuid
                    );
                    continue;
                }
                set_parts.push(format!("{} = ?", key));
                match val {
                    serde_json::Value::String(s) => values.push(s.clone()),
                    serde_json::Value::Number(n) => values.push(n.to_string()),
                    serde_json::Value::Null => values.push("NULL".to_string()),
                    other => values.push(other.to_string()),
                }
            }

            if !set_parts.is_empty() {
                let sql = format!(
                    "UPDATE {} SET {} WHERE uuid = ?",
                    table,
                    set_parts.join(", ")
                );
                values.push(uuid.to_string());

                log::debug!("[Conflict] Executing: {} with {} params", sql, values.len());
                if let Err(e) = resolved_db.execute_raw(&sql, &values).await {
                    log::warn!("[Conflict] Failed to apply resolution for {}: {}", uuid, e);
                }
            }
        }

        // crsqlite-wasm auto-persists via IndexedDB — no manual export/write needed.

        // Update the active database
        state.set_database(resolved_db);

        // Sync exercises from the resolved database
        if let Err(e) = Self::sync_exercises(state).await {
            log::warn!(
                "[Conflict] Failed to sync exercises after resolution: {}",
                e
            );
        }

        log::info!("[Conflict] All conflicts resolved successfully");
        Ok(())
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

    // ── Sync outcome → status mapping tests ───────────────────────────────

    #[test]
    fn test_pushed_outcome_maps_to_up_to_date() {
        let status = WorkoutStateManager::map_sync_outcome_to_status(&SyncOutcome::Pushed);
        assert_eq!(status, SyncStatus::UpToDate);
    }

    #[test]
    fn test_pulled_outcome_maps_to_up_to_date() {
        let status =
            WorkoutStateManager::map_sync_outcome_to_status(&SyncOutcome::Pulled(vec![1, 2, 3]));
        assert_eq!(status, SyncStatus::UpToDate);
    }

    #[test]
    fn test_merged_outcome_maps_to_up_to_date() {
        let status =
            WorkoutStateManager::map_sync_outcome_to_status(&SyncOutcome::Merged(vec![4, 5, 6]));
        assert_eq!(status, SyncStatus::UpToDate);
    }

    #[test]
    fn test_offline_outcome_maps_to_error() {
        let status = WorkoutStateManager::map_sync_outcome_to_status(&SyncOutcome::Offline);
        assert_eq!(status, SyncStatus::Error("Server unreachable".to_string()));
    }

    #[test]
    fn test_skipped_outcome_maps_to_idle() {
        let status = WorkoutStateManager::map_sync_outcome_to_status(&SyncOutcome::Skipped);
        assert_eq!(status, SyncStatus::Idle);
    }

    #[test]
    fn test_conflict_detected_outcome_maps_to_conflicts_detected() {
        use crate::sync::ConflictRecord;
        let status =
            WorkoutStateManager::map_sync_outcome_to_status(&SyncOutcome::ConflictDetected {
                merged: vec![7, 8, 9],
                conflicts: vec![ConflictRecord {
                    table: "exercises".into(),
                    row_id: "row-1".into(),
                    version_a: "{}".into(),
                    version_b: "{}".into(),
                }],
            });
        assert_eq!(status, SyncStatus::ConflictsDetected);
    }

    #[test]
    fn test_error_then_success_transitions_back_to_up_to_date() {
        // After an error, a subsequent successful sync should transition back
        let error_status = WorkoutStateManager::map_sync_outcome_to_status(&SyncOutcome::Offline);
        assert_eq!(
            error_status,
            SyncStatus::Error("Server unreachable".to_string())
        );

        // Next sync succeeds
        let success_status = WorkoutStateManager::map_sync_outcome_to_status(&SyncOutcome::Pushed);
        assert_eq!(success_status, SyncStatus::UpToDate);
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
