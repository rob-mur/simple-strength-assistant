use crate::state::db::DatabaseError;
use crate::state::file_system::FileSystemError;
use thiserror::Error;

#[derive(Error, Debug, Clone)]
pub enum WorkoutError {
    #[error("Database error: {0}")]
    Database(#[from] DatabaseError),

    #[error("File system error: {0}")]
    FileSystem(#[from] FileSystemError),

    #[error("Database initialization already in progress")]
    InitializationInProgress,

    #[error("Database already initialized")]
    AlreadyInitialized,

    #[error("Database not initialized")]
    NotInitialized,

    #[error("No active session")]
    NoActiveSession,

    #[error("Session not persisted")]
    SessionNotPersisted,

    #[error("Invalid set data: {0}")]
    InvalidSetData(String),

    #[error("Failed to save exercise: {0}")]
    SaveExerciseError(String),

    #[error("Failed to create session: {0}")]
    CreateSessionError(String),

    #[error("Failed to insert set: {0}")]
    InsertSetError(String),

    #[error("Failed to complete session: {0}")]
    CompleteSessionError(String),
}

impl From<String> for WorkoutError {
    fn from(s: String) -> Self {
        if s.contains("Database initialization already in progress") {
            WorkoutError::InitializationInProgress
        } else if s.contains("Database already initialized") {
            WorkoutError::AlreadyInitialized
        } else if s.contains("No active session") {
            WorkoutError::NoActiveSession
        } else if s.contains("Session not persisted") {
            WorkoutError::SessionNotPersisted
        } else {
            // This is a bit of a hack, but better than nothing for now
            WorkoutError::Database(DatabaseError::InitializationError(s))
        }
    }
}
