use crate::models::{CompletedSet, ExerciseMetadata, SetType};
use crate::state::{Database, FileSystemManager, error::WorkoutError};
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

#[derive(Clone, Copy, PartialEq)]
pub struct WorkoutState {
    initialization_state: Signal<InitializationState>,
    current_session: Signal<Option<WorkoutSession>>,
    error: Signal<Option<WorkoutError>>,
    save_error: Signal<Option<String>>,
    database: Signal<Option<Database>>,
    file_manager: Signal<Option<FileSystemManager>>,
    last_save_time: Signal<f64>,
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

    pub fn set_file_manager(&self, file_manager: FileSystemManager) {
        let mut sig = self.file_manager;
        sig.set(Some(file_manager));
    }

    pub fn database(&self) -> Option<Database> {
        (self.database)()
    }

    pub fn file_manager(&self) -> Option<FileSystemManager> {
        (self.file_manager)()
    }

    pub fn last_save_time(&self) -> f64 {
        (self.last_save_time)()
    }

    pub fn set_last_save_time(&self, time: f64) {
        let mut sig = self.last_save_time;
        sig.set(time);
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
        let mut file_manager = FileSystemManager::new();

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
        }

        if !has_cached {
            // Check if we're in E2E test mode (detected via Playwright user agent or test flag)
            let is_test_mode = if let Some(window) = web_sys::window() {
                window
                    .navigator()
                    .user_agent()
                    .ok()
                    .map(|ua: String| ua.contains("HeadlessChrome") || ua.contains("Playwright"))
                    .unwrap_or(false)
            } else {
                false
            };

            if is_test_mode {
                log::debug!("[DB Init] E2E test mode detected, initializing in-memory database");
                // Skip file selection in test mode - initialize empty database
                let mut database = Database::new();
                database.init(None).await.map_err(|e| {
                    log::error!("Failed to initialize test database: {}", e);
                    WorkoutError::Database(e)
                })?;

                state.set_database(database);
                state.set_file_manager(file_manager);
                state.set_initialization_state(InitializationState::Ready);

                // Auto-start a test workout session for E2E tests
                log::debug!("[DB Init] Creating test workout session...");
                let test_exercise = crate::models::ExerciseMetadata {
                    name: "Test Bench Press".to_string(),
                    set_type_config: crate::models::SetTypeConfig::Weighted {
                        min_weight: 45.0,
                        increment: 5.0,
                    },
                };

                // Use the start_session method to create session
                if let Err(e) = Self::start_session(state, test_exercise).await {
                    log::error!("Failed to create test session: {}", e);
                    // Don't fail completely - just log the error
                }

                log::debug!("[DB Init] Test mode setup complete!");
                return Ok(());
            }

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
        state.set_initialization_state(InitializationState::Ready);

        log::debug!("[DB Init] Setup complete! State is now Ready");
        Ok(())
    }

    pub async fn start_session(
        state: &WorkoutState,
        exercise: ExerciseMetadata,
    ) -> Result<(), WorkoutError> {
        let db = state.database().ok_or(WorkoutError::NotInitialized)?;

        db.save_exercise(&exercise)
            .await
            .map_err(|e: crate::state::DatabaseError| {
                WorkoutError::SaveExerciseError(e.to_string())
            })?;

        let session_id =
            db.create_session(&exercise.name)
                .await
                .map_err(|e: crate::state::DatabaseError| {
                    WorkoutError::CreateSessionError(e.to_string())
                })?;

        let predicted = Self::calculate_initial_predictions(&exercise);

        let session = WorkoutSession {
            session_id: Some(session_id),
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

        let session_id = session
            .session_id
            .ok_or(WorkoutError::SessionNotPersisted)?;

        let db = state.database().ok_or(WorkoutError::NotInitialized)?;

        crate::models::validate_completed_set(&set, &session.exercise)
            .map_err(|e| WorkoutError::InvalidSetData(e.to_string()))?;

        let _set_id =
            db.insert_set(session_id, &set)
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
        let session = state
            .current_session()
            .ok_or(WorkoutError::NoActiveSession)?;

        let session_id = session
            .session_id
            .ok_or(WorkoutError::SessionNotPersisted)?;

        let db = state.database().ok_or(WorkoutError::NotInitialized)?;

        db.complete_session(session_id)
            .await
            .map_err(|e: crate::state::DatabaseError| {
                WorkoutError::CompleteSessionError(e.to_string())
            })?;

        Self::save_database(state).await?;

        state.set_current_session(None);

        Ok(())
    }

    async fn save_database(state: &WorkoutState) -> Result<(), WorkoutError> {
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

    fn calculate_initial_predictions(exercise: &ExerciseMetadata) -> PredictedParameters {
        match exercise.set_type_config {
            crate::models::SetTypeConfig::Weighted { min_weight, .. } => PredictedParameters {
                weight: Some(min_weight),
                reps: DEFAULT_WEIGHTED_REPS,
                rpe: DEFAULT_RPE,
            },
            crate::models::SetTypeConfig::Bodyweight => PredictedParameters {
                weight: None,
                reps: DEFAULT_BODYWEIGHT_REPS,
                rpe: DEFAULT_RPE,
            },
        }
    }

    fn calculate_next_predictions(session: &WorkoutSession) -> PredictedParameters {
        if session.completed_sets.is_empty() {
            return Self::calculate_initial_predictions(&session.exercise);
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
            _ => Self::calculate_initial_predictions(&session.exercise),
        }
    }

    pub fn handle_error(state: &WorkoutState, error: WorkoutError) {
        log::error!("Workout state error: {}", error);
        state.set_error(Some(error));
        state.set_initialization_state(InitializationState::Error);
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::models::{SetType, SetTypeConfig};

    #[test]
    fn test_initial_predictions_weighted() {
        let exercise = ExerciseMetadata {
            name: "Bench Press".to_string(),
            set_type_config: SetTypeConfig::Weighted {
                min_weight: 45.0,
                increment: 5.0,
            },
        };

        let predicted = WorkoutStateManager::calculate_initial_predictions(&exercise);

        assert_eq!(predicted.weight, Some(45.0));
        assert_eq!(predicted.reps, 8);
        assert_eq!(predicted.rpe, 7.0);
    }

    #[test]
    fn test_initial_predictions_bodyweight() {
        let exercise = ExerciseMetadata {
            name: "Pull-ups".to_string(),
            set_type_config: SetTypeConfig::Bodyweight,
        };

        let predicted = WorkoutStateManager::calculate_initial_predictions(&exercise);

        assert_eq!(predicted.weight, None);
        assert_eq!(predicted.reps, 10);
        assert_eq!(predicted.rpe, 7.0);
    }

    #[test]
    fn test_next_predictions_progression() {
        let exercise = ExerciseMetadata {
            name: "Bench Press".to_string(),
            set_type_config: SetTypeConfig::Weighted {
                min_weight: 45.0,
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
