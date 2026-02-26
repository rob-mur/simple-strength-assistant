use crate::models::{CompletedSet, ExerciseMetadata, SetType};
use crate::state::{Database, FileSystemManager};
use std::cell::RefCell;
use std::rc::Rc;

// Initial prediction constants
const DEFAULT_WEIGHTED_REPS: u32 = 8;
const DEFAULT_BODYWEIGHT_REPS: u32 = 10;
const DEFAULT_RPE: f32 = 7.0;
const RPE_THRESHOLD_HIGH: f32 = 8.0;
const RPE_THRESHOLD_LOW: f32 = 7.0;
const RPE_REDUCTION: f32 = 0.5;
const RPE_MINIMUM: f32 = 6.0;

#[derive(Clone, Debug, PartialEq)]
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

#[derive(Clone, Copy, PartialEq, Debug)]
pub enum InitializationState {
    NotInitialized,
    Initializing,
    SelectingFile,
    Ready,
    Error,
}

#[derive(Clone)]
pub struct WorkoutState {
    inner: Rc<RefCell<WorkoutStateInner>>,
}

impl PartialEq for WorkoutState {
    fn eq(&self, other: &Self) -> bool {
        Rc::ptr_eq(&self.inner, &other.inner)
    }
}

struct WorkoutStateInner {
    pub initialization_state: InitializationState,
    pub current_session: Option<WorkoutSession>,
    pub error_message: Option<String>,
    database: Option<Database>,
    file_manager: Option<FileSystemManager>,
}

impl WorkoutState {
    pub fn new() -> Self {
        Self {
            inner: Rc::new(RefCell::new(WorkoutStateInner {
                initialization_state: InitializationState::NotInitialized,
                current_session: None,
                error_message: None,
                database: None,
                file_manager: None,
            })),
        }
    }

    pub fn initialization_state(&self) -> InitializationState {
        self.inner
            .try_borrow()
            .map(|state| state.initialization_state)
            .unwrap_or(InitializationState::Error)
    }

    pub fn current_session(&self) -> Option<WorkoutSession> {
        self.inner
            .try_borrow()
            .ok()
            .and_then(|state| state.current_session.clone())
    }

    pub fn error_message(&self) -> Option<String> {
        self.inner
            .try_borrow()
            .ok()
            .and_then(|state| state.error_message.clone())
    }

    pub fn set_initialization_state(&self, state: InitializationState) {
        if let Ok(mut inner) = self.inner.try_borrow_mut() {
            inner.initialization_state = state;
        } else {
            log::error!("Failed to borrow WorkoutState mutably to set initialization state");
        }
    }

    pub fn set_current_session(&self, session: Option<WorkoutSession>) {
        if let Ok(mut inner) = self.inner.try_borrow_mut() {
            inner.current_session = session;
        } else {
            log::error!("Failed to borrow WorkoutState mutably to set current session");
        }
    }

    pub fn set_error_message(&self, message: Option<String>) {
        if let Ok(mut inner) = self.inner.try_borrow_mut() {
            inner.error_message = message;
        } else {
            log::error!("Failed to borrow WorkoutState mutably to set error message");
        }
    }

    pub fn set_database(&self, database: Database) {
        if let Ok(mut inner) = self.inner.try_borrow_mut() {
            inner.database = Some(database);
        } else {
            log::error!("Failed to borrow WorkoutState mutably to set database");
        }
    }

    pub fn set_file_manager(&self, file_manager: FileSystemManager) {
        if let Ok(mut inner) = self.inner.try_borrow_mut() {
            inner.file_manager = Some(file_manager);
        } else {
            log::error!("Failed to borrow WorkoutState mutably to set file manager");
        }
    }
}

pub struct WorkoutStateManager;

impl WorkoutStateManager {
    pub async fn setup_database(state: &WorkoutState) -> Result<(), String> {
        web_sys::console::log_1(&"[DB Init] Starting database setup...".into());

        // Atomically check and set initialization state to prevent race conditions
        {
            let mut inner = state
                .inner
                .try_borrow_mut()
                .map_err(|_| "Failed to access state: already borrowed".to_string())?;

            match inner.initialization_state {
                InitializationState::Initializing => {
                    web_sys::console::log_1(&"[DB Init] Already in progress, skipping".into());
                    return Err("Database initialization already in progress".to_string());
                }
                InitializationState::Ready => {
                    web_sys::console::log_1(&"[DB Init] Already initialized, skipping".into());
                    return Ok(());
                }
                _ => {}
            }

            inner.initialization_state = InitializationState::Initializing;
        }

        web_sys::console::log_1(&"[DB Init] Creating file manager...".into());
        let mut file_manager = FileSystemManager::new();

        web_sys::console::log_1(&"[DB Init] Checking for cached file handle...".into());
        let has_cached = file_manager.check_cached_handle().await.map_err(|e| {
            let msg = format!("Failed to check cached handle: {}", e);
            web_sys::console::error_1(&msg.clone().into());
            msg
        })?;

        web_sys::console::log_1(&format!("[DB Init] Has cached handle: {}", has_cached).into());

        if !has_cached {
            web_sys::console::log_1(
                &"[DB Init] No cached handle, transitioning to SelectingFile state".into(),
            );
            web_sys::console::log_1(
                &"[DB Init] File picker requires user gesture - waiting for button click".into(),
            );
            state.set_initialization_state(InitializationState::SelectingFile);

            // Return early - UI will call prompt_for_file from button onclick
            // which provides the required user gesture (transient activation)
            return Err("Waiting for user to select file - not an error, normal flow".to_string());
        }

        let file_data = if file_manager.has_handle() {
            web_sys::console::log_1(&"[DB Init] Reading existing file...".into());
            match file_manager.read_file().await {
                Ok(data) if !data.is_empty() => {
                    web_sys::console::log_1(
                        &format!("[DB Init] Read {} bytes from file", data.len()).into(),
                    );
                    Some(data)
                }
                Ok(_) => {
                    web_sys::console::log_1(
                        &"[DB Init] File is empty, creating new database".into(),
                    );
                    None
                }
                Err(e) => {
                    let msg = format!("Failed to read existing file: {}", e);
                    web_sys::console::warn_1(&msg.clone().into());
                    log::warn!("{}", msg);
                    None
                }
            }
        } else {
            web_sys::console::log_1(&"[DB Init] No file handle, creating new database".into());
            None
        };

        web_sys::console::log_1(&"[DB Init] Initializing database...".into());
        let mut database = Database::new();
        database.init(file_data).await.map_err(|e| {
            let msg = format!("Failed to initialize database: {}", e);
            web_sys::console::error_1(&msg.clone().into());
            msg
        })?;
        web_sys::console::log_1(&"[DB Init] Database initialized successfully".into());

        {
            let mut inner = state.inner.try_borrow_mut().map_err(|e| {
                let msg = format!("Failed to borrow state mutably: {}", e);
                web_sys::console::error_1(&msg.clone().into());
                msg
            })?;
            inner.database = Some(database);
            inner.file_manager = Some(file_manager);
        }
        state.set_initialization_state(InitializationState::Ready);

        web_sys::console::log_1(&"[DB Init] Setup complete! State is now Ready".into());
        Ok(())
    }

    pub async fn start_session(
        state: &WorkoutState,
        exercise: ExerciseMetadata,
    ) -> Result<(), String> {
        let db = state
            .inner
            .try_borrow()
            .map_err(|e| format!("Failed to borrow state: {}", e))?
            .database
            .clone()
            .ok_or("Database not initialized".to_string())?;

        db.save_exercise(&exercise)
            .await
            .map_err(|e| format!("Failed to save exercise: {}", e))?;

        let session_id = db
            .create_session(&exercise.name)
            .await
            .map_err(|e| format!("Failed to create session: {}", e))?;

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

    pub async fn log_set(state: &WorkoutState, set: CompletedSet) -> Result<(), String> {
        let mut session = state.current_session().ok_or("No active session")?;

        let session_id = session.session_id.ok_or("Session not persisted")?;

        let db = state
            .inner
            .try_borrow()
            .map_err(|e| format!("Failed to borrow state: {}", e))?
            .database
            .clone()
            .ok_or("Database not initialized".to_string())?;

        crate::models::validate_completed_set(&set, &session.exercise)
            .map_err(|e| format!("Invalid set data: {}", e))?;

        let _set_id = db
            .insert_set(session_id, &set)
            .await
            .map_err(|e| format!("Failed to insert set: {}", e))?;

        session.completed_sets.push(set.clone());
        session.predicted = Self::calculate_next_predictions(&session);

        state.set_current_session(Some(session));

        // Auto-save after each set to prevent data loss if browser closes
        web_sys::console::log_1(&"[Workout] Auto-saving database after set...".into());
        Self::save_database(state)
            .await
            .map_err(|e| {
                web_sys::console::warn_1(&format!("[Workout] Auto-save failed: {}", e).into());
                // Don't fail the entire operation if auto-save fails
                // The set is still recorded in memory and will be saved on session completion
                log::warn!("Auto-save failed but set logged in memory: {}", e);
            })
            .ok();

        Ok(())
    }

    pub async fn complete_session(state: &WorkoutState) -> Result<(), String> {
        let session = state.current_session().ok_or("No active session")?;

        let session_id = session.session_id.ok_or("Session not persisted")?;

        let db = state
            .inner
            .try_borrow()
            .map_err(|e| format!("Failed to borrow state: {}", e))?
            .database
            .clone()
            .ok_or("Database not initialized".to_string())?;

        db.complete_session(session_id)
            .await
            .map_err(|e| format!("Failed to complete session: {}", e))?;

        Self::save_database(state).await?;

        state.set_current_session(None);

        Ok(())
    }

    async fn save_database(state: &WorkoutState) -> Result<(), String> {
        let (db, file_manager) = {
            let inner = state
                .inner
                .try_borrow()
                .map_err(|e| format!("Failed to borrow state: {}", e))?;

            let db = inner
                .database
                .clone()
                .ok_or("Database not initialized".to_string())?;

            let file_manager = inner
                .file_manager
                .clone()
                .ok_or("File manager not initialized".to_string())?;

            (db, file_manager)
        };

        let data = db
            .export()
            .await
            .map_err(|e| format!("Failed to export database: {}", e))?;

        file_manager
            .write_file(&data)
            .await
            .map_err(|e| format!("Failed to write file: {}", e))?;

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

    pub fn handle_error(state: &WorkoutState, error: String) {
        log::error!("Workout state error: {}", error);
        state.set_error_message(Some(error));
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
