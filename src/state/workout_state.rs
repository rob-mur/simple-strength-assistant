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

    fn set_initialization_state(&self, state: InitializationState) {
        if let Ok(mut inner) = self.inner.try_borrow_mut() {
            inner.initialization_state = state;
        } else {
            log::error!("Failed to borrow WorkoutState mutably to set initialization state");
        }
    }

    fn set_current_session(&self, session: Option<WorkoutSession>) {
        if let Ok(mut inner) = self.inner.try_borrow_mut() {
            inner.current_session = session;
        } else {
            log::error!("Failed to borrow WorkoutState mutably to set current session");
        }
    }

    fn set_error_message(&self, message: Option<String>) {
        if let Ok(mut inner) = self.inner.try_borrow_mut() {
            inner.error_message = message;
        } else {
            log::error!("Failed to borrow WorkoutState mutably to set error message");
        }
    }
}

pub struct WorkoutStateManager;

impl WorkoutStateManager {
    pub async fn setup_database(state: &WorkoutState) -> Result<(), String> {
        // Check current state to prevent concurrent initialization
        let current_state = state.initialization_state();
        match current_state {
            InitializationState::Initializing => {
                return Err("Database initialization already in progress".to_string());
            }
            InitializationState::Ready => {
                return Ok(());
            }
            _ => {}
        }

        state.set_initialization_state(InitializationState::Initializing);

        let mut file_manager = FileSystemManager::new();

        let has_cached = file_manager
            .check_cached_handle()
            .await
            .map_err(|e| format!("Failed to check cached handle: {}", e))?;

        if !has_cached {
            state.set_initialization_state(InitializationState::SelectingFile);

            file_manager
                .prompt_for_file()
                .await
                .map_err(|e| format!("Failed to prompt for file: {}", e))?;
        }

        let file_data = if file_manager.has_handle() {
            match file_manager.read_file().await {
                Ok(data) if !data.is_empty() => Some(data),
                Ok(_) => None,
                Err(e) => {
                    log::warn!("Failed to read existing file: {}", e);
                    None
                }
            }
        } else {
            None
        };

        let mut database = Database::new();
        database
            .init(file_data)
            .await
            .map_err(|e| format!("Failed to initialize database: {}", e))?;

        {
            let mut inner = state
                .inner
                .try_borrow_mut()
                .map_err(|e| format!("Failed to borrow state mutably: {}", e))?;
            inner.database = Some(database);
            inner.file_manager = Some(file_manager);
        }
        state.set_initialization_state(InitializationState::Ready);

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

        // Note: Database is not saved here to avoid saving after every set.
        // The database will be saved when the session is completed.
        // If needed, implement debounced auto-save in the future.

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
