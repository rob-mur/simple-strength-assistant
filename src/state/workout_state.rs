use crate::models::{CompletedSet, ExerciseMetadata, SetType};
#[cfg(feature = "test-mode")]
use crate::state::StorageBackend;
use crate::state::{Database, Storage, error::WorkoutError};
use crate::sync::VectorClock;
#[cfg(all(not(feature = "test-mode"), not(test)))]
use crate::sync::{SyncCredentials, SyncOutcome, save_clock};
use dioxus::prelude::*;

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

/// Which version of a conflicting record the user has chosen to keep.
#[derive(Clone, Copy, PartialEq, Debug)]
pub enum ConflictChoice {
    VersionA,
    VersionB,
}

/// A single record that has a true conflict: same UUID, same `updated_at`,
/// but different field values on the two devices.
///
/// `uuid`       - stable record identifier used to apply the resolution.
/// `field_label`- human-readable description of the record (e.g. exercise name).
/// `version_a`  - string representation of the value on device A.
/// `version_b`  - string representation of the value on device B.
/// `choice`     - `None` until the user selects a version.
#[derive(Clone, Debug, PartialEq)]
pub struct ConflictRecord {
    pub uuid: String,
    pub field_label: String,
    pub version_a: String,
    pub version_b: String,
    pub choice: Option<ConflictChoice>,
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
#[derive(Clone, PartialEq, Debug, Default)]
pub enum SyncStatus {
    #[default]
    Idle,
    NeverSynced,
    Syncing,
    UpToDate,
    Error(String),
    ConflictsDetected(Vec<ConflictRecord>),
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
            SyncStatus::ConflictsDetected(_) => "conflicts",
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
    /// Stores the user's resolved conflict choices after `ConflictResolution`
    /// fires `on_resolve`.  The sync client (#91) reads this to perform the
    /// OPFS merge write and push to `POST /sync/:sync_id`.
    resolved_conflicts: Signal<Vec<ConflictRecord>>,
    /// Local vector clock, persisted across sync cycles
    sync_clock: Signal<VectorClock>,
}

impl Default for WorkoutState {
    fn default() -> Self {
        Self::new()
    }
}

impl WorkoutState {
    pub fn new() -> Self {
        // Load persisted vector clock so sync resumes from the correct
        // sequence numbers across page reloads.  In test mode there is
        // no LocalStorage, so we start with an empty clock.
        let initial_clock = {
            #[cfg(all(not(feature = "test-mode"), not(test)))]
            {
                crate::sync::load_clock()
            }
            #[cfg(any(feature = "test-mode", test))]
            {
                VectorClock::new()
            }
        };

        Self {
            initialization_state: Signal::new(InitializationState::NotInitialized),
            current_session: Signal::new(None),
            error: Signal::new(None),
            save_error: Signal::new(None),
            database: Signal::new(None),
            file_manager: Signal::new(None),
            last_save_time: Signal::new(0.0),
            exercises: Signal::new(Vec::new()),
            sync_status: Signal::new(SyncStatus::Idle),
            resolved_conflicts: Signal::new(Vec::new()),
            sync_clock: Signal::new(initial_clock),
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

    pub fn sync_status(&self) -> SyncStatus {
        (self.sync_status)()
    }

    pub fn set_sync_status(&self, status: SyncStatus) {
        let mut sig = self.sync_status;
        sig.set(status);
    }

    /// Returns the conflict choices recorded by the last call to `set_resolved_conflicts`.
    pub fn resolved_conflicts(&self) -> Vec<ConflictRecord> {
        (self.resolved_conflicts)()
    }

    /// Stores the user's conflict resolution choices so the sync client (#91)
    /// can read them when performing the OPFS merge write and server push.
    pub fn set_resolved_conflicts(&self, conflicts: Vec<ConflictRecord>) {
        let mut sig = self.resolved_conflicts;
        sig.set(conflicts);
    }

    pub fn sync_clock(&self) -> VectorClock {
        (self.sync_clock)()
    }

    pub fn set_sync_clock(&self, clock: VectorClock) {
        let mut sig = self.sync_clock;
        sig.set(clock);
    }
}

pub struct WorkoutStateManager;

impl WorkoutStateManager {
    pub async fn setup_database(state: &WorkoutState) -> Result<(), WorkoutError> {
        log::debug!("[DB Init] Starting database setup...");

        match state.initialization_state() {
            InitializationState::Initializing => {
                log::debug!("[DB Init] Already in progress, skipping");
                return Err(WorkoutError::InitializationInProgress);
            }
            InitializationState::Ready => {
                log::debug!("[DB Init] Already initialized, skipping");
                return Ok(());
            }
            _ => {}
        }

        state.set_initialization_state(InitializationState::Initializing);

        log::debug!("[DB Init] Creating file manager...");
        let mut file_manager = Storage::new();

        log::debug!("[DB Init] Checking for cached file handle...");
        let has_cached = file_manager.check_cached_handle().await.map_err(|e| {
            log::error!("Failed to check cached handle: {}", e);
            WorkoutError::FileSystem(e)
        })?;

        log::debug!("[DB Init] Has cached handle: {}", has_cached);

        if has_cached {
            // Store it even if we might fail later (e.g. permission check)
            // This allows the Error UI to see we have a handle and re-request permission.
            state.set_file_manager(file_manager.clone());
        } else {
            log::debug!("[DB Init] No cached handle, transitioning to SelectingFile state");
            log::debug!("[DB Init] File picker requires user gesture - waiting for button click");
            state.set_initialization_state(InitializationState::SelectingFile);

            // Return OK - UI will call prompt_for_file from button onclick
            return Ok(());
        }

        let file_data = if file_manager.has_handle() {
            log::debug!("[DB Init] Reading file contents...");
            match file_manager.read_file().await {
                Ok(data) if data.is_empty() => {
                    log::debug!("[DB Init] File is empty (0 bytes), will create new database");
                    None
                }
                Ok(data) => {
                    log::debug!(
                        "[DB Init] Read {} bytes from file, loading existing database",
                        data.len()
                    );
                    Some(data)
                }
                Err(e) => {
                    // Don't silently treat read errors as "empty file"
                    // If we can't read the cached file handle, return error
                    log::error!("Failed to read cached file handle: {}", e);

                    // If the format is invalid, clear the cached handle from IndexedDB
                    // This prevents the loop where "Retry" keeps finding the same bad handle.
                    if matches!(e, crate::state::FileSystemError::InvalidFormat) {
                        let _ = file_manager.clear_handle().await;
                    }

                    return Err(WorkoutError::FileSystem(e));
                }
            }
        } else {
            log::debug!("[DB Init] No file handle, creating new database");
            None
        };

        log::debug!("[DB Init] Initializing database...");
        let mut database = Database::new();
        database.init(file_data).await.map_err(|e| {
            log::error!("Failed to initialize database: {}", e);
            WorkoutError::Database(e)
        })?;
        log::debug!("[DB Init] Database initialized successfully");

        state.set_database(database);
        state.set_file_manager(file_manager);

        // Load exercises from database
        if let Err(e) = Self::sync_exercises(state).await {
            log::warn!("Failed to load exercises after DB setup: {}", e);
        }

        state.set_initialization_state(InitializationState::Ready);

        log::debug!("[DB Init] Setup complete! State is now Ready");
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

    pub async fn save_database(state: &WorkoutState) -> Result<(), WorkoutError> {
        let db = state.database().ok_or(WorkoutError::NotInitialized)?;

        let file_manager = state.file_manager().ok_or(WorkoutError::FileSystem(
            crate::state::FileSystemError::NoHandle,
        ))?;

        let data = db.export().await.map_err(WorkoutError::Database)?;

        file_manager
            .write_file(&data)
            .await
            .map_err(WorkoutError::FileSystem)?;

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
        state.set_database(database);
        state.set_file_manager(file_manager);

        if let Err(e) = Self::sync_exercises(state).await {
            log::warn!("Failed to sync exercises after file initialization: {}", e);
        }

        state.set_initialization_state(InitializationState::Ready);

        log::debug!("[UI] Setup complete! State is now Ready");
    }

    pub fn handle_error(state: &WorkoutState, error: WorkoutError) {
        log::error!("Workout state error: {}", error);
        state.set_error(Some(error));
        state.set_initialization_state(InitializationState::Error);
    }

    /// Trigger a background sync cycle.  Non-blocking: errors are swallowed
    /// except for `ConflictDetected`, which updates the save_error banner so
    /// the user is informed they need to resolve conflicts.
    ///
    /// This is a no-op when `sync_id` is not configured in LocalStorage
    /// (i.e. the pairing flow has not been run yet).
    #[cfg(all(not(feature = "test-mode"), not(test)))]
    pub async fn trigger_background_sync(state: &WorkoutState) {
        use crate::sync::client::SyncClient;
        use crate::sync::http::wasm::FetchClient;
        use crate::sync::stub_merge;

        // Check credentials first to short-circuit before the expensive
        // database export when sync is not configured.
        let credentials = SyncCredentials::load();
        if credentials.as_ref().is_none_or(|c| !c.is_valid()) {
            log::debug!("[Sync] Skipped — no valid sync_id configured");
            return;
        }

        let db = match state.database() {
            Some(db) => db,
            None => {
                log::debug!("[Sync] Database not ready, skipping sync");
                return;
            }
        };

        let local_blob = match db.export().await {
            Ok(b) => b,
            Err(e) => {
                log::warn!("[Sync] Failed to export database for sync: {}", e);
                return;
            }
        };

        let mut clock = state.sync_clock();

        let client = SyncClient::new(FetchClient);
        let outcome = client
            .run(credentials.as_ref(), &local_blob, &mut clock, &stub_merge)
            .await;

        // Only persist the updated clock when the sync cycle actually reached
        // the server.  Persisting after Offline would accumulate meaningless
        // sequence numbers (the server never saw the increment).
        let should_persist_clock = !matches!(outcome, SyncOutcome::Skipped | SyncOutcome::Offline);
        if should_persist_clock {
            state.set_sync_clock(clock.clone());
            if let Err(e) = save_clock(&clock) {
                log::warn!(
                    "[Sync] Failed to persist vector clock to LocalStorage: {}",
                    e
                );
            }
        }

        match outcome {
            SyncOutcome::Skipped => {
                log::debug!("[Sync] Skipped — no sync_id configured");
            }
            SyncOutcome::Offline => {
                log::debug!("[Sync] Server unreachable, continuing offline");
            }
            SyncOutcome::Pushed => {
                log::debug!("[Sync] Push complete");
            }
            SyncOutcome::Pulled(blob) => {
                log::info!("[Sync] Fast-forward pull complete, reloading database");
                Self::apply_synced_blob(state, &blob, "pull").await;
            }
            SyncOutcome::Merged(blob) => {
                log::info!("[Sync] Merge complete, reloading database");
                Self::apply_synced_blob(state, &blob, "merge").await;
            }
            // TODO(#89): this arm is unreachable while `stub_merge` always
            // reports a conflict.  Once the real union-merge lands, genuine
            // conflict-free merges will flow through `Merged` above, and only
            // true row-level conflicts will reach here.
            SyncOutcome::ConflictDetected(_) => {
                log::warn!("[Sync] Conflicts detected — user action required");
                state.set_save_error(Some(
                    "Sync conflicts detected. Your data has been preserved locally. Conflict resolution coming in a future update.".to_string(),
                ));
            }
        }
    }

    /// Shared helper for `Pulled` and `Merged` sync outcomes: initialise a new
    /// database from the given blob, persist it to OPFS, and sync exercises.
    #[cfg(all(not(feature = "test-mode"), not(test)))]
    async fn apply_synced_blob(state: &WorkoutState, blob: &[u8], label: &str) {
        let mut new_db = Database::new();
        match new_db.init(Some(blob.to_vec())).await {
            Ok(_) => {
                if let Some(fm) = state.file_manager()
                    && let Err(e) = fm.write_file(blob).await
                {
                    log::warn!("[Sync] Failed to persist {} blob to OPFS: {}", label, e);
                }
                state.set_database(new_db);
                if let Err(e) = Self::sync_exercises(state).await {
                    log::warn!("[Sync] Failed to sync exercises after {}: {}", label, e);
                }
            }
            Err(e) => {
                log::warn!("[Sync] Failed to init DB from {} blob: {}", label, e);
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
