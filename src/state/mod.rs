mod db;
mod error;
mod file_system;
mod storage;
mod workout_state;

#[cfg(test)]
mod db_tests;
#[cfg(test)]
mod file_system_tests;

pub use db::{Database, DatabaseError};
pub use error::WorkoutError;
pub use file_system::FileSystemError;
#[cfg(not(feature = "test-mode"))]
pub use file_system::FileSystemManager;
pub use workout_state::{InitializationState, WorkoutSession, WorkoutState, WorkoutStateManager};

#[cfg(feature = "test-mode")]
pub use storage::StorageBackend;

// Type alias that switches between OPFS and in-memory storage based on test-mode feature
#[cfg(not(feature = "test-mode"))]
pub type Storage = FileSystemManager;

#[cfg(feature = "test-mode")]
pub type Storage = storage::InMemoryStorage;
