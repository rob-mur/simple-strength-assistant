mod db;
mod error;
mod file_system;
mod storage;
mod vector_clock;
mod workout_state;

#[cfg(test)]
mod db_tests;
#[cfg(all(test, not(feature = "test-mode")))]
mod file_system_tests;

pub use db::{Database, DatabaseError, MergeConflict, MergeResult};
pub use error::WorkoutError;
pub use file_system::FileSystemError;
#[cfg(not(feature = "test-mode"))]
pub use file_system::FileSystemManager;
// VectorClock, ClockRelationship, and compare_vector_clocks are pub(crate)
// until the sync client (#91) wires them up.
pub use workout_state::{
    ConflictChoice, ConflictRecord, InitializationState, PredictedParameters, SyncStatus,
    WorkoutSession, WorkoutState, WorkoutStateManager,
};

#[cfg(feature = "test-mode")]
pub use storage::StorageBackend;

// Type alias that switches between OPFS and in-memory storage based on test-mode feature
#[cfg(not(feature = "test-mode"))]
pub type Storage = FileSystemManager;

#[cfg(feature = "test-mode")]
pub type Storage = storage::InMemoryStorage;
