mod db;
mod error;
mod file_system;
mod storage;
mod workout_state;

#[cfg(test)]
mod db_tests;
#[cfg(test)]
mod file_system_tests;

pub use crate::models::{PlanExercise, WorkoutPlan};
pub use db::{Database, DatabaseError};
pub use error::WorkoutError;
pub use file_system::FileSystemError;
pub use file_system::FileSystemManager;
pub use workout_state::{
    InitializationState, PredictedParameters, SyncStatus, WorkoutSession, WorkoutState,
    WorkoutStateManager,
};

// Storage is always the OPFS-backed FileSystemManager.
// E2E tests run against the same prod binary; the test harness injects
// window.__TEST_MODE__ = true so that JS-layer hooks are available without
// needing a compile-time feature flag.
pub type Storage = FileSystemManager;
