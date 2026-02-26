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
