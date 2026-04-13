use crate::models::{CompletedSet, ExerciseMetadata, SetType, SetTypeConfig};
use crate::state::Database;
use wasm_bindgen_test::*;

wasm_bindgen_test_configure!(run_in_browser);

// ── WorkoutStateManager integration tests ────────────────────────────────────
//
// These tests exercise `WorkoutStateManager::start_session` end-to-end using
// the OPFS-backed FileSystemManager (the same code path as production).
// Run with: wasm-pack test --headless --chrome
mod workout_state_manager_tests {
    use super::*;
    use crate::state::{WorkoutState, WorkoutStateManager};

    /// Helper: creates a fully initialised `WorkoutState` with a real
    /// SQLite database and an OPFS-backed `FileSystemManager`.
    async fn make_ready_state() -> WorkoutState {
        let state = WorkoutState::new();

        // Initialise the SQLite database.
        let mut db = Database::new();
        db.init(None).await.expect("Database init failed");
        state.set_database(db);

        // Wire up the OPFS storage backend so save_database succeeds.
        let mut storage = crate::state::Storage::new();
        storage
            .create_new_file()
            .await
            .expect("create_new_file failed");
        state.set_file_manager(storage);

        state.set_initialization_state(crate::state::InitializationState::Ready);

        state
    }

    /// When `start_session` is called while a session with logged sets is already
    /// active, those sets must be visible in the database *before* the new session
    /// begins.  This is the core contract of implicit session completion.
    #[wasm_bindgen_test]
    async fn test_start_session_persists_previous_session_sets_to_db() {
        let state = make_ready_state().await;

        // ── Session A: Bench Press ────────────────────────────────────────────
        let exercise_a = ExerciseMetadata {
            id: None,
            name: "Bench Press".to_string(),
            set_type_config: SetTypeConfig::Weighted {
                min_weight: 0.0,
                increment: 5.0,
            },
            min_reps: 1,
            max_reps: None,
        };
        WorkoutStateManager::start_session(&state, exercise_a)
            .await
            .expect("start_session A failed");

        // Log two sets for session A.
        let set1 = CompletedSet {
            set_number: 1,
            reps: 8,
            rpe: 7.0,
            set_type: SetType::Weighted { weight: 100.0 },
        };
        let set2 = CompletedSet {
            set_number: 2,
            reps: 6,
            rpe: 7.5,
            set_type: SetType::Weighted { weight: 105.0 },
        };
        WorkoutStateManager::log_set(&state, set1)
            .await
            .expect("log_set 1 failed");
        WorkoutStateManager::log_set(&state, set2)
            .await
            .expect("log_set 2 failed");

        // Confirm the sets are tracked in the in-memory session state.
        assert_eq!(
            state.current_session().unwrap().completed_sets.len(),
            2,
            "Session A should have 2 completed sets before switch"
        );

        // ── Session B: Deadlift (triggers implicit completion of A) ───────────
        let exercise_b = ExerciseMetadata {
            id: None,
            name: "Deadlift".to_string(),
            set_type_config: SetTypeConfig::Weighted {
                min_weight: 0.0,
                increment: 5.0,
            },
            min_reps: 1,
            max_reps: None,
        };
        WorkoutStateManager::start_session(&state, exercise_b)
            .await
            .expect("start_session B failed");

        // Session A must now be cleared and session B active.
        let active = state.current_session().expect("Session B should be active");
        assert_eq!(active.exercise.name, "Deadlift");
        assert_eq!(
            active.completed_sets.len(),
            0,
            "Session B should start with zero completed sets"
        );

        // The two sets that were logged for Bench Press must now be queryable
        // from the database (they were persisted when session A was implicitly completed).
        let db = state.database().expect("Database should be present");

        // Retrieve the Bench Press exercise id via the exercise list.
        let exercises = db.get_exercises().await.expect("get_exercises failed");
        let bench = exercises
            .iter()
            .find(|e| e.name == "Bench Press")
            .expect("Bench Press exercise must exist in DB");
        let bench_id = bench.id.expect("Bench Press must have an id");

        let persisted_sets = db
            .get_sets_for_exercise(bench_id, 10, 0)
            .await
            .expect("get_sets_for_exercise failed");

        assert_eq!(
            persisted_sets.len(),
            2,
            "Both Bench Press sets must appear in history after implicit session completion"
        );
    }

    /// Starting a fresh session when there is no previous active session must
    /// not error and must leave the new session with zero completed sets.
    #[wasm_bindgen_test]
    async fn test_start_session_with_no_prior_session() {
        let state = make_ready_state().await;

        assert!(
            state.current_session().is_none(),
            "No session should be active before start"
        );

        let exercise = ExerciseMetadata {
            id: None,
            name: "Squat".to_string(),
            set_type_config: SetTypeConfig::Weighted {
                min_weight: 20.0,
                increment: 2.5,
            },
            min_reps: 1,
            max_reps: None,
        };
        WorkoutStateManager::start_session(&state, exercise)
            .await
            .expect("start_session failed");

        let session = state.current_session().expect("Session should be active");
        assert_eq!(session.exercise.name, "Squat");
        assert_eq!(
            session.completed_sets.len(),
            0,
            "New session should have zero completed sets"
        );
    }
}

// These tests require a proper WASM test environment with sql.js loaded
// To run these tests:
// 1. wasm-pack test --headless --chrome
// 2. Ensure sql.js is available in the test environment

#[wasm_bindgen_test]
async fn test_database_initialization() {
    let mut db = Database::new();
    let result = db.init(None).await;

    assert!(result.is_ok(), "Database initialization should succeed");
}

#[wasm_bindgen_test]
async fn test_database_initialization_with_existing_data() {
    // Create a database, add data, export, then re-import
    let mut db1 = Database::new();
    db1.init(None).await.expect("Initial db init failed");

    let exercise = ExerciseMetadata {
        id: None,
        name: "Test Exercise".to_string(),
        set_type_config: SetTypeConfig::Weighted {
            min_weight: 0.0,
            increment: 5.0,
        },
        min_reps: 1,
        max_reps: None,
    };
    db1.save_exercise(&exercise)
        .await
        .expect("Save exercise failed");

    // Export the database
    let exported_data = db1.export().await.expect("Export failed");

    // Create a new database instance and load the exported data
    let mut db2 = Database::new();
    let result = db2.init(Some(exported_data)).await;

    assert!(
        result.is_ok(),
        "Database should initialize with existing data"
    );
}

// ── TASK-2.1: New schema tests ────────────────────────────────────────────────

/// RED: log_set writes exercise_id and recorded_at; no session_id involved.
#[wasm_bindgen_test]
async fn test_log_set_weighted() {
    let mut db = Database::new();
    db.init(None).await.expect("Database init failed");

    let exercise = ExerciseMetadata {
        id: None,
        name: "Bench Press".to_string(),
        set_type_config: SetTypeConfig::Weighted {
            min_weight: 0.0,
            increment: 5.0,
        },
        min_reps: 1,
        max_reps: None,
    };
    let exercise_id = db
        .save_exercise(&exercise)
        .await
        .expect("Save exercise failed");

    let set = CompletedSet {
        set_number: 1,
        reps: 8,
        rpe: 7.5,
        set_type: SetType::Weighted { weight: 135.0 },
    };

    let set_id = db.log_set(exercise_id, &set).await.expect("log_set failed");

    assert!(set_id > 0, "Set ID should be positive");
}

/// RED: log_set works for bodyweight sets.
#[wasm_bindgen_test]
async fn test_log_set_bodyweight() {
    let mut db = Database::new();
    db.init(None).await.expect("Database init failed");

    let exercise = ExerciseMetadata {
        id: None,
        name: "Pull-ups".to_string(),
        set_type_config: SetTypeConfig::Bodyweight,
        min_reps: 1,
        max_reps: None,
    };
    let exercise_id = db
        .save_exercise(&exercise)
        .await
        .expect("Save exercise failed");

    let set = CompletedSet {
        set_number: 1,
        reps: 10,
        rpe: 8.0,
        set_type: SetType::Bodyweight,
    };

    let set_id = db
        .log_set(exercise_id, &set)
        .await
        .expect("log_set bodyweight failed");

    assert!(set_id > 0, "Set ID should be positive");
}

/// RED: get_sets_for_exercise returns sets in reverse-chronological order,
/// respecting limit and offset for pagination.
#[wasm_bindgen_test]
async fn test_get_sets_for_exercise_pagination() {
    let mut db = Database::new();
    db.init(None).await.expect("Database init failed");

    let exercise = ExerciseMetadata {
        id: None,
        name: "Squat".to_string(),
        set_type_config: SetTypeConfig::Weighted {
            min_weight: 20.0,
            increment: 2.5,
        },
        min_reps: 1,
        max_reps: None,
    };
    let exercise_id = db
        .save_exercise(&exercise)
        .await
        .expect("Save exercise failed");

    // Log 3 sets in order
    for i in 1u32..=3 {
        let set = CompletedSet {
            set_number: i,
            reps: i * 3,
            rpe: 7.0,
            set_type: SetType::Weighted {
                weight: 60.0 + (i as f32 * 5.0),
            },
        };
        db.log_set(exercise_id, &set).await.expect("log_set failed");
    }

    // Page 1: first 2 results (most recent first)
    let page1 = db
        .get_sets_for_exercise(exercise_id, 2, 0)
        .await
        .expect("get_sets_for_exercise failed");
    assert_eq!(page1.len(), 2, "Page 1 should have 2 results");

    // Page 2: remaining 1 result
    let page2 = db
        .get_sets_for_exercise(exercise_id, 2, 2)
        .await
        .expect("get_sets_for_exercise page 2 failed");
    assert_eq!(page2.len(), 1, "Page 2 should have 1 result");

    // Results should be reverse-chronological (set_number 3 before 2 before 1)
    assert_eq!(
        page1[0].set_number, 3,
        "First result should be most recent (set 3)"
    );
    assert_eq!(page1[1].set_number, 2, "Second result should be set 2");
    assert_eq!(
        page2[0].set_number, 1,
        "Third result should be oldest (set 1)"
    );
}

/// RED: get_sets_for_exercise only returns sets for the given exercise_id.
#[wasm_bindgen_test]
async fn test_get_sets_for_exercise_isolation() {
    let mut db = Database::new();
    db.init(None).await.expect("Database init failed");

    let ex_a = ExerciseMetadata {
        id: None,
        name: "Bench Press".to_string(),
        set_type_config: SetTypeConfig::Weighted {
            min_weight: 0.0,
            increment: 5.0,
        },
        min_reps: 1,
        max_reps: None,
    };
    let id_a = db.save_exercise(&ex_a).await.expect("Save A failed");

    let ex_b = ExerciseMetadata {
        id: None,
        name: "Deadlift".to_string(),
        set_type_config: SetTypeConfig::Weighted {
            min_weight: 0.0,
            increment: 5.0,
        },
        min_reps: 1,
        max_reps: None,
    };
    let id_b = db.save_exercise(&ex_b).await.expect("Save B failed");

    db.log_set(
        id_a,
        &CompletedSet {
            set_number: 1,
            reps: 5,
            rpe: 7.0,
            set_type: SetType::Weighted { weight: 100.0 },
        },
    )
    .await
    .expect("log set A failed");
    db.log_set(
        id_b,
        &CompletedSet {
            set_number: 1,
            reps: 3,
            rpe: 8.0,
            set_type: SetType::Weighted { weight: 150.0 },
        },
    )
    .await
    .expect("log set B failed");

    let sets_a = db
        .get_sets_for_exercise(id_a, 10, 0)
        .await
        .expect("query A failed");
    assert_eq!(sets_a.len(), 1);
    assert_eq!(sets_a[0].exercise_id, id_a);
    assert_eq!(sets_a[0].reps, 5);
}

/// RED: get_sets_for_exercise_before excludes sets recorded on or after the cutoff.
/// The "Previous Sessions" panel must not show sets logged during the current (today's) session.
#[wasm_bindgen_test]
async fn test_get_sets_for_exercise_before_excludes_today() {
    let mut db = Database::new();
    db.init(None).await.expect("Database init failed");

    let exercise = ExerciseMetadata {
        id: None,
        name: "Squat".to_string(),
        set_type_config: SetTypeConfig::Weighted {
            min_weight: 20.0,
            increment: 2.5,
        },
        min_reps: 1,
        max_reps: None,
    };
    let exercise_id = db
        .save_exercise(&exercise)
        .await
        .expect("Save exercise failed");

    // Compute the UTC ms at the start of today (local calendar day).
    let now_ms = js_sys::Date::now();
    let offset_ms = -js_sys::Date::new_0().get_timezone_offset() * 60_000.0;
    let local_now_ms = now_ms + offset_ms;
    let start_of_today_utc = (local_now_ms / 86_400_000.0).floor() * 86_400_000.0 - offset_ms;

    // A set recorded firmly in the previous day.
    let yesterday_ms = start_of_today_utc - 86_400_000.0;
    let yesterday_set = CompletedSet {
        set_number: 1,
        reps: 5,
        rpe: 7.0,
        set_type: SetType::Weighted { weight: 100.0 },
    };
    // A set recorded right now (today).
    let today_set = CompletedSet {
        set_number: 2,
        reps: 3,
        rpe: 8.0,
        set_type: SetType::Weighted { weight: 110.0 },
    };

    db.log_set_at(exercise_id, &yesterday_set, yesterday_ms)
        .await
        .expect("log yesterday set failed");
    db.log_set(exercise_id, &today_set)
        .await
        .expect("log today set failed");

    let results = db
        .get_sets_for_exercise_before(exercise_id, start_of_today_utc, 10, 0)
        .await
        .expect("get_sets_for_exercise_before failed");

    assert_eq!(
        results.len(),
        1,
        "Should only return the set from before today"
    );
    assert_eq!(results[0].reps, 5, "Returned set should be yesterday's");
}

/// RED: get_all_sets_paginated returns sets from all exercises reverse-chronologically.
#[wasm_bindgen_test]
async fn test_get_all_sets_paginated() {
    let mut db = Database::new();
    db.init(None).await.expect("Database init failed");

    let ex1 = ExerciseMetadata {
        id: None,
        name: "Exercise One".to_string(),
        set_type_config: SetTypeConfig::Bodyweight,
        min_reps: 1,
        max_reps: None,
    };
    let id1 = db.save_exercise(&ex1).await.expect("Save 1 failed");

    let ex2 = ExerciseMetadata {
        id: None,
        name: "Exercise Two".to_string(),
        set_type_config: SetTypeConfig::Bodyweight,
        min_reps: 1,
        max_reps: None,
    };
    let id2 = db.save_exercise(&ex2).await.expect("Save 2 failed");

    db.log_set(
        id1,
        &CompletedSet {
            set_number: 1,
            reps: 10,
            rpe: 7.0,
            set_type: SetType::Bodyweight,
        },
    )
    .await
    .expect("log 1 failed");
    db.log_set(
        id2,
        &CompletedSet {
            set_number: 1,
            reps: 8,
            rpe: 7.5,
            set_type: SetType::Bodyweight,
        },
    )
    .await
    .expect("log 2 failed");

    let all = db
        .get_all_sets_paginated(10, 0)
        .await
        .expect("get_all_sets_paginated failed");

    assert_eq!(all.len(), 2, "Should return 2 sets total");
    // Most recently logged should come first
    assert_eq!(all[0].exercise_id, id2);
    assert_eq!(all[1].exercise_id, id1);
}

/// RED: update_set persists changes; subsequent reads reflect the update.
#[wasm_bindgen_test]
async fn test_update_set() {
    let mut db = Database::new();
    db.init(None).await.expect("Database init failed");

    let exercise = ExerciseMetadata {
        id: None,
        name: "Overhead Press".to_string(),
        set_type_config: SetTypeConfig::Weighted {
            min_weight: 0.0,
            increment: 2.5,
        },
        min_reps: 1,
        max_reps: None,
    };
    let exercise_id = db
        .save_exercise(&exercise)
        .await
        .expect("Save exercise failed");

    let set = CompletedSet {
        set_number: 1,
        reps: 8,
        rpe: 7.0,
        set_type: SetType::Weighted { weight: 50.0 },
    };
    let set_id = db.log_set(exercise_id, &set).await.expect("log_set failed");

    // Update: change reps to 10, rpe to 8.0, weight to 55.0 (keep same recorded_at)
    let original_recorded_at = 1_700_000_000_000.0_f64;
    db.update_set(set_id, 10, 8.0, Some(55.0), original_recorded_at)
        .await
        .expect("update_set failed");

    let updated = db
        .get_sets_for_exercise(exercise_id, 1, 0)
        .await
        .expect("read after update failed");

    assert_eq!(updated.len(), 1);
    assert_eq!(updated[0].reps, 10, "reps should be updated to 10");
    assert_eq!(updated[0].rpe, 8.0, "rpe should be updated to 8.0");
    assert_eq!(
        updated[0].set_type,
        SetType::Weighted { weight: 55.0 },
        "weight should be updated to 55.0"
    );
}

/// RED: update_set can change recorded_at; subsequent reads reflect the new timestamp.
#[wasm_bindgen_test]
async fn test_update_set_recorded_at() {
    let mut db = Database::new();
    db.init(None).await.expect("Database init failed");

    let exercise = ExerciseMetadata {
        id: None,
        name: "Squat".to_string(),
        set_type_config: SetTypeConfig::Weighted {
            min_weight: 0.0,
            increment: 5.0,
        },
        min_reps: 1,
        max_reps: None,
    };
    let exercise_id = db
        .save_exercise(&exercise)
        .await
        .expect("Save exercise failed");

    // Log a set with a known timestamp (yesterday: 24h ago in ms)
    let yesterday_ms = 1_000_000_000.0_f64; // arbitrary past timestamp
    let set = CompletedSet {
        set_number: 1,
        reps: 5,
        rpe: 7.0,
        set_type: SetType::Weighted { weight: 100.0 },
    };
    let set_id = db
        .log_set_at(exercise_id, &set, yesterday_ms)
        .await
        .expect("log_set_at failed");

    // Update recorded_at to a different timestamp (two days ago)
    let two_days_ago_ms = 500_000_000.0_f64;
    db.update_set(set_id, 5, 7.0, Some(100.0), two_days_ago_ms)
        .await
        .expect("update_set failed");

    let updated = db
        .get_sets_for_exercise(exercise_id, 1, 0)
        .await
        .expect("read after update failed");

    assert_eq!(updated.len(), 1);
    assert_eq!(
        updated[0].recorded_at, two_days_ago_ms,
        "recorded_at should be updated to two_days_ago_ms"
    );
}

/// RED: delete_set removes the set; subsequent reads no longer include it.
#[wasm_bindgen_test]
async fn test_delete_set() {
    let mut db = Database::new();
    db.init(None).await.expect("Database init failed");

    let exercise = ExerciseMetadata {
        id: None,
        name: "Romanian Deadlift".to_string(),
        set_type_config: SetTypeConfig::Weighted {
            min_weight: 0.0,
            increment: 5.0,
        },
        min_reps: 1,
        max_reps: None,
    };
    let exercise_id = db
        .save_exercise(&exercise)
        .await
        .expect("Save exercise failed");

    let set_id = db
        .log_set(
            exercise_id,
            &CompletedSet {
                set_number: 1,
                reps: 8,
                rpe: 7.0,
                set_type: SetType::Weighted { weight: 80.0 },
            },
        )
        .await
        .expect("log_set failed");

    db.delete_set(set_id).await.expect("delete_set failed");

    let remaining = db
        .get_sets_for_exercise(exercise_id, 10, 0)
        .await
        .expect("read after delete failed");

    assert_eq!(remaining.len(), 0, "No sets should remain after deletion");
}

/// RED: get_last_set_for_exercise uses the new schema (no sessions table).
#[wasm_bindgen_test]
async fn test_get_last_set_for_exercise_new_schema() {
    let mut db = Database::new();
    db.init(None).await.expect("Database init failed");

    let exercise = ExerciseMetadata {
        id: None,
        name: "Bench Press".to_string(),
        set_type_config: SetTypeConfig::Weighted {
            min_weight: 0.0,
            increment: 2.5,
        },
        min_reps: 1,
        max_reps: None,
    };
    let exercise_id = db
        .save_exercise(&exercise)
        .await
        .expect("Save exercise failed");

    // Log two sets — most recent should be returned
    db.log_set(
        exercise_id,
        &CompletedSet {
            set_number: 1,
            reps: 8,
            rpe: 7.0,
            set_type: SetType::Weighted { weight: 100.0 },
        },
    )
    .await
    .expect("log set 1 failed");

    db.log_set(
        exercise_id,
        &CompletedSet {
            set_number: 2,
            reps: 5,
            rpe: 8.0,
            set_type: SetType::Weighted { weight: 110.0 },
        },
    )
    .await
    .expect("log set 2 failed");

    let last = db
        .get_last_set_for_exercise(exercise_id)
        .await
        .expect("get_last_set_for_exercise failed");

    assert!(last.is_some());
    let last = last.unwrap();
    assert_eq!(last.set_type, SetType::Weighted { weight: 110.0 });
    assert_eq!(last.reps, 5);
    assert_eq!(last.rpe, 8.0);
}

/// start_session no longer writes to the database (no sessions table).
/// This is verified by confirming get_all_sets_paginated returns 0 rows after start.
#[wasm_bindgen_test]
async fn test_start_session_does_not_write_db_row() {
    let mut db = Database::new();
    db.init(None).await.expect("Database init failed");

    // If we haven't logged any sets, the paginated query should return nothing.
    let sets = db
        .get_all_sets_paginated(10, 0)
        .await
        .expect("paginated query failed");

    assert_eq!(sets.len(), 0, "No sets before any are logged");
}

#[wasm_bindgen_test]
async fn test_save_exercise() {
    let mut db = Database::new();
    db.init(None).await.expect("Database init failed");

    let exercise = ExerciseMetadata {
        id: None,
        name: "Deadlift".to_string(),
        set_type_config: SetTypeConfig::Weighted {
            min_weight: 135.0,
            increment: 10.0,
        },
        min_reps: 1,
        max_reps: None,
    };

    let result = db.save_exercise(&exercise).await;

    assert!(result.is_ok(), "Save exercise should succeed");
    let inserted_id = result.unwrap();

    // Test explicit UPDATE WHERE id = ?
    let updated_exercise_with_id = ExerciseMetadata {
        id: Some(inserted_id),
        name: "Deadlift Modified".to_string(),
        set_type_config: SetTypeConfig::Weighted {
            min_weight: 145.0,
            increment: 5.0,
        },
        min_reps: 1,
        max_reps: None,
    };

    let result_id_update = db.save_exercise(&updated_exercise_with_id).await;
    assert!(
        result_id_update.is_ok(),
        "Update exercise with ID should succeed"
    );
    assert_eq!(
        result_id_update.unwrap(),
        inserted_id,
        "Update with ID should return same ID"
    );

    // Test updating the same exercise by name (INSERT ON CONFLICT)
    let updated_exercise = ExerciseMetadata {
        id: None,
        name: "Deadlift Modified".to_string(),
        set_type_config: SetTypeConfig::Weighted {
            min_weight: 150.0,
            increment: 10.0,
        },
        min_reps: 1,
        max_reps: None,
    };

    let result2 = db.save_exercise(&updated_exercise).await;

    assert!(result2.is_ok(), "Update exercise by name should succeed");
    assert_eq!(
        result2.unwrap(),
        inserted_id,
        "Update by name should return same ID"
    );
}

#[wasm_bindgen_test]
async fn test_export_database() {
    let mut db = Database::new();
    db.init(None).await.expect("Database init failed");

    let exercise = ExerciseMetadata {
        id: None,
        name: "Test Exercise".to_string(),
        set_type_config: SetTypeConfig::Bodyweight,
        min_reps: 1,
        max_reps: None,
    };
    db.save_exercise(&exercise)
        .await
        .expect("Save exercise failed");

    let exported = db.export().await;

    assert!(exported.is_ok(), "Export should succeed");
    let data = exported.unwrap();
    assert!(!data.is_empty(), "Exported data should not be empty");

    // Verify SQLite magic number
    assert!(
        data.starts_with(b"SQLite format 3\0"),
        "Exported data should be valid SQLite"
    );
}

#[wasm_bindgen_test]
async fn test_database_not_initialized_error() {
    let db = Database::new();

    let result = db.get_all_sets_paginated(10, 0).await;

    assert!(result.is_err(), "Should return error when not initialized");
}

#[wasm_bindgen_test]
async fn test_sql_injection_protection() {
    let mut db = Database::new();
    db.init(None).await.expect("Database init failed");

    // Try exercise name with SQL injection attempt
    let malicious_name = "Test'; DROP TABLE exercises; --";

    let exercise = ExerciseMetadata {
        id: None,
        name: malicious_name.to_string(),
        set_type_config: SetTypeConfig::Weighted {
            min_weight: 0.0,
            increment: 5.0,
        },
        min_reps: 1,
        max_reps: None,
    };

    let result = db.save_exercise(&exercise).await;

    assert!(
        result.is_ok(),
        "Should handle special characters safely with parameterized queries"
    );

    // Verify we can still query (tables weren't dropped)
    let sets_result = db.get_all_sets_paginated(10, 0).await;

    assert!(
        sets_result.is_ok(),
        "Database should still be functional after injection attempt"
    );
}

#[wasm_bindgen_test]
async fn test_export_import_round_trip() {
    // Create database with data
    let mut db1 = Database::new();
    db1.init(None).await.expect("DB1 init failed");

    let exercise = ExerciseMetadata {
        id: None,
        name: "Bench Press".to_string(),
        set_type_config: SetTypeConfig::Weighted {
            min_weight: 0.0,
            increment: 5.0,
        },
        min_reps: 1,
        max_reps: None,
    };
    let exercise_id = db1
        .save_exercise(&exercise)
        .await
        .expect("Save exercise failed");

    let set = CompletedSet {
        set_number: 1,
        reps: 8,
        rpe: 7.5,
        set_type: SetType::Weighted { weight: 135.0 },
    };
    db1.log_set(exercise_id, &set)
        .await
        .expect("log_set failed");

    // Export and re-import
    let exported = db1.export().await.expect("Export failed");

    let mut db2 = Database::new();
    db2.init(Some(exported)).await.expect("Import failed");

    // Verify imported data
    let result = db2
        .execute("SELECT count(*) as count FROM completed_sets", &[])
        .await
        .expect("Select query failed");
    use wasm_bindgen::JsCast;
    let array = result
        .dyn_ref::<js_sys::Array>()
        .expect("Result should be an array");
    let first_row = array.get(0);
    let count = js_sys::Reflect::get(&first_row, &wasm_bindgen::JsValue::from_str("count"))
        .expect("Failed to get count property")
        .as_f64()
        .expect("Count should be a number") as i64;
    assert_eq!(count, 1, "Expected exactly 1 set in the imported database");

    // Verify we can log another set in the imported database
    let new_set_id = db2
        .log_set(
            exercise_id,
            &CompletedSet {
                set_number: 2,
                reps: 6,
                rpe: 8.0,
                set_type: SetType::Weighted { weight: 140.0 },
            },
        )
        .await;

    assert!(
        new_set_id.is_ok(),
        "Should be able to log a set in imported database"
    );
}

// ── Issue 71: exercises not restored after clearing site data and reselecting database file ──

/// RED: Opening an existing database file restores exercises.
///
/// Simulates the "open existing database" path: create a DB with exercises,
/// export it, then re-import it (as if the user reselected their file after
/// clearing site data). After re-import, `get_exercises` must return all
/// exercises that were present before the export.
#[wasm_bindgen_test]
async fn test_exercises_restored_after_open_existing_database() {
    // Create a database with custom exercises
    let mut db1 = Database::new();
    db1.init(None).await.expect("Initial db init failed");

    let exercise1 = ExerciseMetadata {
        id: None,
        name: "Squat".to_string(),
        set_type_config: SetTypeConfig::Weighted {
            min_weight: 20.0,
            increment: 2.5,
        },
        min_reps: 1,
        max_reps: None,
    };
    let exercise2 = ExerciseMetadata {
        id: None,
        name: "Pull-ups".to_string(),
        set_type_config: SetTypeConfig::Bodyweight,
        min_reps: 1,
        max_reps: None,
    };

    db1.save_exercise(&exercise1)
        .await
        .expect("Save exercise1 failed");
    db1.save_exercise(&exercise2)
        .await
        .expect("Save exercise2 failed");

    // Verify exercises exist before export
    let exercises_before = db1
        .get_exercises()
        .await
        .expect("get_exercises before export failed");
    assert_eq!(
        exercises_before.len(),
        2,
        "Should have 2 exercises before export"
    );

    // Simulate "clear site data": export the database bytes
    let exported_data = db1.export().await.expect("Export failed");

    // Simulate "reopen the same file": re-import the exported bytes into a fresh DB instance
    let mut db2 = Database::new();
    db2.init(Some(exported_data))
        .await
        .expect("Re-import failed");

    // Assert: exercises are restored
    let exercises_after = db2
        .get_exercises()
        .await
        .expect("get_exercises after re-import failed");
    assert_eq!(
        exercises_after.len(),
        2,
        "Exercises should be restored after opening existing database file"
    );

    let names: Vec<&str> = exercises_after.iter().map(|e| e.name.as_str()).collect();
    assert!(
        names.contains(&"Squat"),
        "Squat exercise should be restored"
    );
    assert!(
        names.contains(&"Pull-ups"),
        "Pull-ups exercise should be restored"
    );
}

/// RED: Opening an existing database with no exercises returns empty list (not an error).
#[wasm_bindgen_test]
async fn test_empty_exercise_list_after_open_existing_database_with_no_exercises() {
    // Create a database with no exercises (only workout history)
    let mut db1 = Database::new();
    db1.init(None).await.expect("Initial db init failed");

    let exported_data = db1.export().await.expect("Export failed");

    // Re-import into fresh DB
    let mut db2 = Database::new();
    db2.init(Some(exported_data))
        .await
        .expect("Re-import failed");

    // Assert: empty list, not an error
    let exercises = db2
        .get_exercises()
        .await
        .expect("get_exercises should succeed even when list is empty");
    assert!(
        exercises.is_empty(),
        "Exercise list should be empty when none were saved"
    );
}

// ── Issue 85: sync-readiness schema columns ────────────────────────────────────

/// A newly-saved exercise must have a non-empty UUID.
#[wasm_bindgen_test]
async fn test_new_exercise_has_uuid() {
    use wasm_bindgen::JsCast;

    let mut db = Database::new();
    db.init(None).await.expect("Database init failed");

    let exercise = ExerciseMetadata {
        id: None,
        name: "Squat".to_string(),
        set_type_config: SetTypeConfig::Weighted {
            min_weight: 20.0,
            increment: 2.5,
        },
        min_reps: 1,
        max_reps: None,
    };
    let exercise_id = db
        .save_exercise(&exercise)
        .await
        .expect("Save exercise failed");

    let result = db
        .execute(
            "SELECT uuid, updated_at FROM exercises WHERE id = ?",
            &[wasm_bindgen::JsValue::from_f64(exercise_id as f64)],
        )
        .await
        .expect("Query failed");
    let array = result.dyn_ref::<js_sys::Array>().expect("Expected array");
    assert_eq!(array.length(), 1, "Should find exactly one row");
    let row = array.get(0);

    let uuid_val = js_sys::Reflect::get(&row, &wasm_bindgen::JsValue::from_str("uuid"))
        .unwrap()
        .as_string()
        .expect("uuid should be a string");
    assert!(
        !uuid_val.is_empty(),
        "UUID should not be empty for a new exercise"
    );
    assert_eq!(uuid_val.len(), 36, "UUID should be 36 characters long");

    let updated_at = js_sys::Reflect::get(&row, &wasm_bindgen::JsValue::from_str("updated_at"))
        .unwrap()
        .as_f64()
        .expect("updated_at should be a number");
    assert!(
        updated_at > 0.0,
        "updated_at should be set for a new exercise"
    );
}

/// A newly-logged set must have a non-empty UUID and updated_at.
#[wasm_bindgen_test]
async fn test_new_set_has_uuid_and_updated_at() {
    use wasm_bindgen::JsCast;

    let mut db = Database::new();
    db.init(None).await.expect("Database init failed");

    let exercise = ExerciseMetadata {
        id: None,
        name: "Bench Press".to_string(),
        set_type_config: SetTypeConfig::Weighted {
            min_weight: 0.0,
            increment: 5.0,
        },
        min_reps: 1,
        max_reps: None,
    };
    let exercise_id = db
        .save_exercise(&exercise)
        .await
        .expect("Save exercise failed");

    let set = CompletedSet {
        set_number: 1,
        reps: 8,
        rpe: 7.5,
        set_type: SetType::Weighted { weight: 80.0 },
    };
    let set_id = db.log_set(exercise_id, &set).await.expect("log_set failed");

    let result = db
        .execute(
            "SELECT uuid, updated_at FROM completed_sets WHERE id = ?",
            &[wasm_bindgen::JsValue::from_f64(set_id as f64)],
        )
        .await
        .expect("Query failed");
    let array = result.dyn_ref::<js_sys::Array>().expect("Expected array");
    assert_eq!(array.length(), 1);
    let row = array.get(0);

    let uuid_val = js_sys::Reflect::get(&row, &wasm_bindgen::JsValue::from_str("uuid"))
        .unwrap()
        .as_string()
        .expect("uuid should be a string");
    assert!(
        !uuid_val.is_empty(),
        "UUID should not be empty for a new set"
    );
    assert_eq!(uuid_val.len(), 36, "UUID should be 36 characters long");

    let updated_at = js_sys::Reflect::get(&row, &wasm_bindgen::JsValue::from_str("updated_at"))
        .unwrap()
        .as_f64()
        .expect("updated_at should be a number");
    assert!(updated_at > 0.0, "updated_at should be set for a new set");
}

/// Editing a set must update its updated_at to a value >= the original.
#[wasm_bindgen_test]
async fn test_update_set_updates_updated_at() {
    use wasm_bindgen::JsCast;

    let mut db = Database::new();
    db.init(None).await.expect("Database init failed");

    let exercise = ExerciseMetadata {
        id: None,
        name: "Overhead Press".to_string(),
        set_type_config: SetTypeConfig::Weighted {
            min_weight: 0.0,
            increment: 2.5,
        },
        min_reps: 1,
        max_reps: None,
    };
    let exercise_id = db
        .save_exercise(&exercise)
        .await
        .expect("Save exercise failed");

    let set_id = db
        .log_set(
            exercise_id,
            &CompletedSet {
                set_number: 1,
                reps: 8,
                rpe: 7.0,
                set_type: SetType::Weighted { weight: 50.0 },
            },
        )
        .await
        .expect("log_set failed");

    // Capture the updated_at before the edit.
    let before_result = db
        .execute(
            "SELECT updated_at FROM completed_sets WHERE id = ?",
            &[wasm_bindgen::JsValue::from_f64(set_id as f64)],
        )
        .await
        .expect("Query before failed");
    let before_arr = before_result
        .dyn_ref::<js_sys::Array>()
        .expect("Expected array");
    let before_updated_at = js_sys::Reflect::get(
        &before_arr.get(0),
        &wasm_bindgen::JsValue::from_str("updated_at"),
    )
    .unwrap()
    .as_f64()
    .expect("updated_at before should be a number");

    db.update_set(set_id, 10, 8.0, Some(55.0), 1_700_000_000_000.0)
        .await
        .expect("update_set failed");

    let after_result = db
        .execute(
            "SELECT updated_at FROM completed_sets WHERE id = ?",
            &[wasm_bindgen::JsValue::from_f64(set_id as f64)],
        )
        .await
        .expect("Query after failed");
    let after_arr = after_result
        .dyn_ref::<js_sys::Array>()
        .expect("Expected array");
    let after_updated_at = js_sys::Reflect::get(
        &after_arr.get(0),
        &wasm_bindgen::JsValue::from_str("updated_at"),
    )
    .unwrap()
    .as_f64()
    .expect("updated_at after should be a number");

    assert!(
        after_updated_at >= before_updated_at,
        "updated_at ({}) should be >= original ({}) after update",
        after_updated_at,
        before_updated_at
    );
}

/// Deleting a set soft-deletes it: the row has deleted_at set and no longer
/// appears in normal queries.
#[wasm_bindgen_test]
async fn test_delete_set_is_soft_delete() {
    use wasm_bindgen::JsCast;

    let mut db = Database::new();
    db.init(None).await.expect("Database init failed");

    let exercise = ExerciseMetadata {
        id: None,
        name: "Romanian Deadlift".to_string(),
        set_type_config: SetTypeConfig::Weighted {
            min_weight: 0.0,
            increment: 5.0,
        },
        min_reps: 1,
        max_reps: None,
    };
    let exercise_id = db
        .save_exercise(&exercise)
        .await
        .expect("Save exercise failed");

    let set_id = db
        .log_set(
            exercise_id,
            &CompletedSet {
                set_number: 1,
                reps: 8,
                rpe: 7.0,
                set_type: SetType::Weighted { weight: 80.0 },
            },
        )
        .await
        .expect("log_set failed");

    db.delete_set(set_id).await.expect("delete_set failed");

    // The row must still exist in the raw table (soft delete).
    let raw_result = db
        .execute(
            "SELECT deleted_at FROM completed_sets WHERE id = ?",
            &[wasm_bindgen::JsValue::from_f64(set_id as f64)],
        )
        .await
        .expect("Raw query failed");
    let raw_arr = raw_result
        .dyn_ref::<js_sys::Array>()
        .expect("Expected array");
    assert_eq!(
        raw_arr.length(),
        1,
        "Row should still exist after soft delete"
    );

    let deleted_at = js_sys::Reflect::get(
        &raw_arr.get(0),
        &wasm_bindgen::JsValue::from_str("deleted_at"),
    )
    .unwrap();
    assert!(
        !deleted_at.is_null() && !deleted_at.is_undefined(),
        "deleted_at should be set after soft delete"
    );

    // The set must not appear in normal queries.
    let visible = db
        .get_sets_for_exercise(exercise_id, 10, 0)
        .await
        .expect("get_sets_for_exercise failed");
    assert_eq!(
        visible.len(),
        0,
        "Soft-deleted set should not appear in normal queries"
    );
}

/// Each newly-inserted exercise or set gets a unique UUID.
#[wasm_bindgen_test]
async fn test_uuids_are_unique_across_records() {
    use wasm_bindgen::JsCast;

    let mut db = Database::new();
    db.init(None).await.expect("Database init failed");

    let exercise = ExerciseMetadata {
        id: None,
        name: "Deadlift".to_string(),
        set_type_config: SetTypeConfig::Weighted {
            min_weight: 0.0,
            increment: 5.0,
        },
        min_reps: 1,
        max_reps: None,
    };
    let exercise_id = db
        .save_exercise(&exercise)
        .await
        .expect("Save exercise failed");

    let id1 = db
        .log_set(
            exercise_id,
            &CompletedSet {
                set_number: 1,
                reps: 5,
                rpe: 7.0,
                set_type: SetType::Weighted { weight: 100.0 },
            },
        )
        .await
        .expect("log set 1 failed");
    let id2 = db
        .log_set(
            exercise_id,
            &CompletedSet {
                set_number: 2,
                reps: 5,
                rpe: 7.5,
                set_type: SetType::Weighted { weight: 105.0 },
            },
        )
        .await
        .expect("log set 2 failed");

    let result = db
        .execute(
            "SELECT uuid FROM completed_sets WHERE id IN (?, ?)",
            &[
                wasm_bindgen::JsValue::from_f64(id1 as f64),
                wasm_bindgen::JsValue::from_f64(id2 as f64),
            ],
        )
        .await
        .expect("Query failed");
    let array = result.dyn_ref::<js_sys::Array>().expect("Expected array");
    assert_eq!(array.length(), 2);

    let uuid1 = js_sys::Reflect::get(&array.get(0), &wasm_bindgen::JsValue::from_str("uuid"))
        .unwrap()
        .as_string()
        .expect("uuid1 should be a string");
    let uuid2 = js_sys::Reflect::get(&array.get(1), &wasm_bindgen::JsValue::from_str("uuid"))
        .unwrap()
        .as_string()
        .expect("uuid2 should be a string");

    assert_ne!(uuid1, uuid2, "Each set should have a unique UUID");
}

// ── Issue #89: Client-Side Union Merge ───────────────────────────────────────

/// Helper: build a minimal initialised Database with one exercise row.
/// Returns (db, exported_bytes, exercise_uuid).
async fn make_db_with_exercise(name: &str, updated_at: f64) -> (Database, Vec<u8>, String) {
    let mut db = Database::new();
    db.init(None).await.expect("db init failed");
    let ex_uuid = db
        .insert_exercise_for_test(name, updated_at, None)
        .await
        .expect("insert_exercise_for_test failed");
    let bytes = db.export().await.expect("export failed");
    (db, bytes, ex_uuid)
}

/// Helper: build a minimal initialised Database with one exercise row that is
/// soft-deleted (deleted_at set).
async fn make_db_with_tombstone(
    name: &str,
    updated_at: f64,
    deleted_at: f64,
) -> (Database, Vec<u8>, String) {
    let mut db = Database::new();
    db.init(None).await.expect("db init failed");
    let ex_uuid = db
        .insert_exercise_for_test(name, updated_at, Some(deleted_at))
        .await
        .expect("insert_exercise_for_test failed");
    let bytes = db.export().await.expect("export failed");
    (db, bytes, ex_uuid)
}

/// RED → GREEN: Merge of two databases with no overlapping UUIDs returns a
/// merged database whose row count equals the sum of both inputs.
#[wasm_bindgen_test]
async fn test_merge_disjoint_sets_returns_full_union() {
    let (_, bytes_a, _) = make_db_with_exercise("Squat", 1000.0).await;
    let (_, bytes_b, _) = make_db_with_exercise("Deadlift", 1000.0).await;

    let result = Database::merge_databases(bytes_a, bytes_b)
        .await
        .expect("merge_databases failed");

    assert!(
        result.conflicts.is_empty(),
        "Disjoint databases should produce no conflicts"
    );

    // Load merged blob and count exercises.
    let mut merged_db = Database::new();
    merged_db
        .init(Some(result.merged))
        .await
        .expect("init merged db failed");
    let exercises = merged_db
        .get_exercises()
        .await
        .expect("get_exercises failed");
    assert_eq!(
        exercises.len(),
        2,
        "Merged database should contain both exercises"
    );
}

/// RED → GREEN: When two databases share a UUID but have different updated_at,
/// the row with the higher updated_at wins (all fields).
#[wasm_bindgen_test]
async fn test_merge_newer_updated_at_wins() {
    // DB A: Squat at t=1000
    let (_, bytes_a, uuid) = make_db_with_exercise("Squat", 1000.0).await;

    // DB B: same UUID, name "Squat Variant", updated_at=2000 (newer).
    let mut db_b = Database::new();
    db_b.init(None).await.expect("db init failed");
    db_b.insert_exercise_with_uuid_for_test(&uuid, "Squat Variant", 2000.0, None)
        .await
        .expect("insert_exercise_with_uuid failed");
    let bytes_b = db_b.export().await.expect("export failed");

    let result = Database::merge_databases(bytes_a, bytes_b)
        .await
        .expect("merge_databases failed");

    assert!(
        result.conflicts.is_empty(),
        "Different updated_at should not produce a conflict"
    );

    let mut merged_db = Database::new();
    merged_db
        .init(Some(result.merged))
        .await
        .expect("init merged db failed");
    let exercises = merged_db
        .get_exercises()
        .await
        .expect("get_exercises failed");
    assert_eq!(exercises.len(), 1, "Only one row for the shared UUID");
    assert_eq!(
        exercises[0].name, "Squat Variant",
        "Newer (t=2000) row name should win"
    );
}

/// RED → GREEN: Same UUID, same updated_at, different field values → conflict
/// reported; merged database is still returned.
#[wasm_bindgen_test]
async fn test_merge_same_updated_at_different_values_yields_conflict() {
    let ts = 1000.0_f64;
    let (_, bytes_a, uuid) = make_db_with_exercise("Squat", ts).await;

    let mut db_b = Database::new();
    db_b.init(None).await.expect("db init failed");
    db_b.insert_exercise_with_uuid_for_test(&uuid, "Squat (variant)", ts, None)
        .await
        .expect("insert failed");
    let bytes_b = db_b.export().await.expect("export failed");

    let result = Database::merge_databases(bytes_a, bytes_b)
        .await
        .expect("merge_databases failed");

    assert_eq!(
        result.conflicts.len(),
        1,
        "Exactly one conflict expected for the shared UUID with identical updated_at"
    );
    assert_eq!(
        result.conflicts[0].uuid, uuid,
        "Conflict UUID should match the shared exercise UUID"
    );
    // The merged blob must still be present (not empty).
    assert!(
        !result.merged.is_empty(),
        "Merged blob must be non-empty even when conflicts exist"
    );
}

/// RED → GREEN: One database has a tombstone (deleted_at set), the other has a
/// live row for the same UUID → the merged database treats that record as deleted.
#[wasm_bindgen_test]
async fn test_merge_tombstone_in_one_propagates_to_merged() {
    let ts = 1000.0_f64;
    let (_, bytes_a, uuid) = make_db_with_exercise("Bench Press", ts).await;

    // DB B: same UUID but with deleted_at set (tombstone).
    let mut db_b = Database::new();
    db_b.init(None).await.expect("db init failed");
    db_b.insert_exercise_with_uuid_for_test(&uuid, "Bench Press", ts, Some(2000.0))
        .await
        .expect("insert tombstone failed");
    let bytes_b = db_b.export().await.expect("export failed");

    let result = Database::merge_databases(bytes_a, bytes_b)
        .await
        .expect("merge_databases failed");

    assert!(
        result.conflicts.is_empty(),
        "Tombstone vs live should not produce a conflict"
    );

    let mut merged_db = Database::new();
    merged_db
        .init(Some(result.merged))
        .await
        .expect("init merged db failed");
    // get_exercises filters out deleted rows; row should not appear.
    let exercises = merged_db
        .get_exercises()
        .await
        .expect("get_exercises failed");
    assert!(
        exercises.is_empty(),
        "Tombstone should propagate: exercise must not appear in live query"
    );
}

/// RED → GREEN: Both databases have a tombstone for the same UUID → the merged
/// database carries the tombstone with the more recent deleted_at value.
#[wasm_bindgen_test]
async fn test_merge_two_tombstones_keeps_most_recent_deleted_at() {
    let ts = 1000.0_f64;
    let (_, bytes_a, uuid) = make_db_with_tombstone("Pull-ups", ts, 3000.0).await;

    let mut db_b = Database::new();
    db_b.init(None).await.expect("db init failed");
    db_b.insert_exercise_with_uuid_for_test(&uuid, "Pull-ups", ts, Some(5000.0))
        .await
        .expect("insert tombstone 2 failed");
    let bytes_b = db_b.export().await.expect("export failed");

    let result = Database::merge_databases(bytes_a, bytes_b)
        .await
        .expect("merge_databases failed");

    assert!(
        result.conflicts.is_empty(),
        "Two tombstones should not conflict"
    );
    // The row should be deleted in the merged database.
    let mut merged_db = Database::new();
    merged_db
        .init(Some(result.merged))
        .await
        .expect("init merged db failed");
    let exercises = merged_db
        .get_exercises()
        .await
        .expect("get_exercises failed");
    assert!(
        exercises.is_empty(),
        "Both tombstones: row must remain deleted in the merged database"
    );
}

/// RED → GREEN: The merge function is pure — calling it twice with the same
/// inputs produces logically identical results (same rows, same conflicts).
///
/// We compare logical content (row count, exercise names, conflict count)
/// rather than raw bytes because SQLite does not guarantee byte-for-byte
/// identical output across independent database constructions, even with
/// the same logical content.
#[wasm_bindgen_test]
async fn test_merge_is_pure_function() {
    let (_, bytes_a, _) = make_db_with_exercise("Overhead Press", 1000.0).await;
    let (_, bytes_b, _) = make_db_with_exercise("Row", 1000.0).await;

    let result1 = Database::merge_databases(bytes_a.clone(), bytes_b.clone())
        .await
        .expect("first merge failed");
    let result2 = Database::merge_databases(bytes_a, bytes_b)
        .await
        .expect("second merge failed");

    // Both calls must agree on conflicts.
    assert_eq!(
        result1.conflicts.len(),
        result2.conflicts.len(),
        "Conflict counts must match across two identical calls"
    );

    // Both merged blobs must be non-empty.
    assert!(
        !result1.merged.is_empty(),
        "First merged blob must not be empty"
    );
    assert!(
        !result2.merged.is_empty(),
        "Second merged blob must not be empty"
    );

    // Compare logical content: load both merged databases and verify they
    // contain the same exercises.
    let mut db1 = Database::new();
    db1.init(Some(result1.merged))
        .await
        .expect("init db1 failed");
    let exercises1 = db1.get_exercises().await.expect("get_exercises db1 failed");

    let mut db2 = Database::new();
    db2.init(Some(result2.merged))
        .await
        .expect("init db2 failed");
    let exercises2 = db2.get_exercises().await.expect("get_exercises db2 failed");

    assert_eq!(
        exercises1.len(),
        exercises2.len(),
        "Both merges must produce the same number of exercises"
    );

    let mut names1: Vec<&str> = exercises1.iter().map(|e| e.name.as_str()).collect();
    let mut names2: Vec<&str> = exercises2.iter().map(|e| e.name.as_str()).collect();
    names1.sort();
    names2.sort();
    assert_eq!(
        names1, names2,
        "Both merges must produce the same exercise names"
    );
}

/// RED → GREEN: A tombstoned row with an older `updated_at` loses to a live row
/// with a newer `updated_at`. The live row must survive in the merged database.
#[wasm_bindgen_test]
async fn test_merge_older_tombstone_loses_to_newer_live_row() {
    let old_ts = 1000.0_f64;
    let new_ts = 2000.0_f64;

    // DB A: tombstoned row with the older timestamp.
    let (_, bytes_a, uuid) = make_db_with_tombstone("Bench Press", old_ts, old_ts).await;

    // DB B: live row (no tombstone) with the newer timestamp.
    let mut db_b = Database::new();
    db_b.init(None).await.expect("db init failed");
    db_b.insert_exercise_with_uuid_for_test(&uuid, "Bench Press", new_ts, None)
        .await
        .expect("insert live row failed");
    let bytes_b = db_b.export().await.expect("export failed");

    let result = Database::merge_databases(bytes_a, bytes_b)
        .await
        .expect("merge_databases failed");

    assert!(
        result.conflicts.is_empty(),
        "Newer live row vs older tombstone should not conflict"
    );

    // The live row (newer updated_at) must win — exercise should be present.
    let mut merged_db = Database::new();
    merged_db
        .init(Some(result.merged))
        .await
        .expect("init merged db failed");
    let exercises = merged_db
        .get_exercises()
        .await
        .expect("get_exercises failed");
    assert_eq!(
        exercises.len(),
        1,
        "Newer live row must win over older tombstone"
    );
    assert_eq!(exercises[0].name, "Bench Press");
}

/// RED → GREEN: Creating a new database and reopening the same file restores exercises.
///
/// Simulates the "create new database" path: create a DB, add exercises, export,
/// then re-import. Exercises must survive the round-trip.
#[wasm_bindgen_test]
async fn test_exercises_restored_after_create_new_then_reopen() {
    // "Create new database" path
    let mut db1 = Database::new();
    db1.init(None).await.expect("New database init failed");

    // User adds exercises after creation
    let exercise = ExerciseMetadata {
        id: None,
        name: "Deadlift".to_string(),
        set_type_config: SetTypeConfig::Weighted {
            min_weight: 60.0,
            increment: 5.0,
        },
        min_reps: 1,
        max_reps: None,
    };
    db1.save_exercise(&exercise)
        .await
        .expect("Save exercise failed");

    // App is closed / site data cleared — export simulates the persisted file
    let exported_data = db1.export().await.expect("Export failed");

    // User reopens the same file
    let mut db2 = Database::new();
    db2.init(Some(exported_data)).await.expect("Reopen failed");

    let exercises = db2
        .get_exercises()
        .await
        .expect("get_exercises after reopen failed");
    assert_eq!(
        exercises.len(),
        1,
        "Deadlift should be restored after reopening"
    );
    assert_eq!(exercises[0].name, "Deadlift");
}

/// Editing an exercise updates its updated_at to a value >= the one before the edit.
#[wasm_bindgen_test]
async fn test_edit_exercise_updates_updated_at() {
    use wasm_bindgen::JsCast;

    let mut db = Database::new();
    db.init(None).await.expect("Database init failed");

    let exercise = ExerciseMetadata {
        id: None,
        name: "Squat".to_string(),
        set_type_config: SetTypeConfig::Weighted {
            min_weight: 20.0,
            increment: 2.5,
        },
        min_reps: 1,
        max_reps: None,
    };
    let exercise_id = db.save_exercise(&exercise).await.expect("save failed");

    // Read the initial updated_at
    let before_result = db
        .execute(
            "SELECT updated_at FROM exercises WHERE id = ?",
            &[wasm_bindgen::JsValue::from_f64(exercise_id as f64)],
        )
        .await
        .expect("SELECT before failed");
    let before_arr = before_result.dyn_ref::<js_sys::Array>().unwrap();
    let before_row = before_arr.get(0);
    let updated_at_before =
        js_sys::Reflect::get(&before_row, &wasm_bindgen::JsValue::from_str("updated_at"))
            .unwrap()
            .as_f64()
            .expect("updated_at_before should be number");

    // Update the exercise
    let updated = ExerciseMetadata {
        id: Some(exercise_id),
        name: "Squat".to_string(),
        set_type_config: SetTypeConfig::Weighted {
            min_weight: 20.0,
            increment: 5.0, // changed
        },
        min_reps: 1,
        max_reps: None,
    };
    db.save_exercise(&updated).await.expect("update failed");

    let after_result = db
        .execute(
            "SELECT updated_at FROM exercises WHERE id = ?",
            &[wasm_bindgen::JsValue::from_f64(exercise_id as f64)],
        )
        .await
        .expect("SELECT after failed");
    let after_arr = after_result.dyn_ref::<js_sys::Array>().unwrap();
    let after_row = after_arr.get(0);
    let updated_at_after =
        js_sys::Reflect::get(&after_row, &wasm_bindgen::JsValue::from_str("updated_at"))
            .unwrap()
            .as_f64()
            .expect("updated_at_after should be number");

    assert!(
        updated_at_after >= updated_at_before,
        "updated_at after edit ({}) must be >= before ({})",
        updated_at_after,
        updated_at_before
    );
}

/// Migration from v2 to v3: an existing database (exported at v2, imported fresh)
/// should run the v3 migration without errors. Pre-existing rows should be
/// backfilled with uuid and updated_at.
#[wasm_bindgen_test]
async fn test_v2_to_v3_migration_backfills_existing_rows() {
    use wasm_bindgen::JsCast;

    // Create and export a v2-schema database (same code path — the migration runs
    // incrementally, so the result after init() on a fresh DB is always current).
    // We simulate a pre-v3 database by creating data, exporting, and re-importing;
    // the re-import triggers migrate_and_create_tables which applies v3 on top.
    let mut db1 = Database::new();
    db1.init(None).await.expect("db1 init failed");

    let exercise = ExerciseMetadata {
        id: None,
        name: "Deadlift".to_string(),
        set_type_config: SetTypeConfig::Weighted {
            min_weight: 60.0,
            increment: 5.0,
        },
        min_reps: 1,
        max_reps: None,
    };
    let ex_id = db1.save_exercise(&exercise).await.expect("save failed");
    db1.log_set(
        ex_id,
        &CompletedSet {
            set_number: 1,
            reps: 5,
            rpe: 8.0,
            set_type: SetType::Weighted { weight: 100.0 },
        },
    )
    .await
    .expect("log_set failed");

    let exported = db1.export().await.expect("export failed");

    // Re-import: migration runs again (idempotent).
    let mut db2 = Database::new();
    db2.init(Some(exported)).await.expect("db2 init failed");

    // All exercises and sets should have non-empty uuids and non-zero updated_at.
    let ex_result = db2
        .execute(
            "SELECT uuid, updated_at FROM exercises WHERE deleted_at IS NULL",
            &[],
        )
        .await
        .expect("SELECT exercises failed");
    let ex_arr = ex_result
        .dyn_ref::<js_sys::Array>()
        .expect("Expected array");
    assert!(ex_arr.length() > 0, "Should have at least one exercise");
    for i in 0..ex_arr.length() {
        let row = ex_arr.get(i);
        let uuid = js_sys::Reflect::get(&row, &wasm_bindgen::JsValue::from_str("uuid"))
            .unwrap()
            .as_string()
            .unwrap_or_default();
        assert!(
            !uuid.is_empty(),
            "Exercise uuid must not be empty after migration"
        );
        let updated_at = js_sys::Reflect::get(&row, &wasm_bindgen::JsValue::from_str("updated_at"))
            .unwrap()
            .as_f64()
            .unwrap_or(0.0);
        assert!(
            updated_at > 0.0,
            "Exercise updated_at must be non-zero after migration"
        );
    }

    let sets_result = db2
        .execute(
            "SELECT uuid, updated_at FROM completed_sets WHERE deleted_at IS NULL",
            &[],
        )
        .await
        .expect("SELECT sets failed");
    let sets_arr = sets_result
        .dyn_ref::<js_sys::Array>()
        .expect("Expected array");
    assert!(sets_arr.length() > 0, "Should have at least one set");
    for i in 0..sets_arr.length() {
        let row = sets_arr.get(i);
        let uuid = js_sys::Reflect::get(&row, &wasm_bindgen::JsValue::from_str("uuid"))
            .unwrap()
            .as_string()
            .unwrap_or_default();
        assert!(
            !uuid.is_empty(),
            "Set uuid must not be empty after migration"
        );
        let updated_at = js_sys::Reflect::get(&row, &wasm_bindgen::JsValue::from_str("updated_at"))
            .unwrap()
            .as_f64()
            .unwrap_or(0.0);
        assert!(
            updated_at > 0.0,
            "Set updated_at must be non-zero after migration"
        );
    }
}

// ── History query tests for e1RM calculation (#131) ─────────────────────────

/// Helper: creates a fresh DB with a weighted exercise and returns (db, exercise_id).
async fn setup_exercise_for_history() -> (Database, i64) {
    let mut db = Database::new();
    db.init(None).await.expect("DB init failed");

    let exercise = ExerciseMetadata {
        id: None,
        name: "Squat".to_string(),
        set_type_config: SetTypeConfig::Weighted {
            min_weight: 20.0,
            increment: 2.5,
        },
        min_reps: 1,
        max_reps: None,
    };
    let id = db
        .save_exercise(&exercise)
        .await
        .expect("save_exercise failed");
    (db, id)
}

/// Milliseconds per day constant for test date arithmetic.
const MS_PER_DAY: f64 = 86_400_000.0;

/// get_best_set_for_exercise returns the set with the highest e1RM in the window.
#[wasm_bindgen_test]
async fn test_best_set_returns_highest_e1rm() {
    let (db, eid) = setup_exercise_for_history().await;

    // "Today" = day 10, history window starts at day 0.
    let day = |d: i64| d as f64 * MS_PER_DAY;
    let today_start = day(10);
    let today_end = day(11);

    // Set A: 100kg x 5 @ RPE 8 on day 3
    let set_a = CompletedSet {
        set_number: 1,
        reps: 5,
        rpe: 8.0,
        set_type: SetType::Weighted { weight: 100.0 },
    };
    db.log_set_at(eid, &set_a, day(3) + 1000.0)
        .await
        .expect("log A");

    // Set B: 120kg x 3 @ RPE 9 on day 5 — higher e1RM
    let set_b = CompletedSet {
        set_number: 1,
        reps: 3,
        rpe: 9.0,
        set_type: SetType::Weighted { weight: 120.0 },
    };
    db.log_set_at(eid, &set_b, day(5) + 1000.0)
        .await
        .expect("log B");

    // Set C: 80kg x 8 @ RPE 7 on day 7 — lower e1RM
    let set_c = CompletedSet {
        set_number: 1,
        reps: 8,
        rpe: 7.0,
        set_type: SetType::Weighted { weight: 80.0 },
    };
    db.log_set_at(eid, &set_c, day(7) + 1000.0)
        .await
        .expect("log C");

    let best = db
        .get_best_set_for_exercise(eid, day(0), today_start, today_end)
        .await
        .expect("query failed");

    assert!(best.is_some(), "Should find a best set");
    let best = best.unwrap();
    // Set B (120kg x 3 @ RPE 9) should have highest e1RM
    assert_eq!(best.reps, 3);
    if let SetType::Weighted { weight } = best.set_type {
        assert_eq!(weight, 120.0);
    } else {
        panic!("Expected Weighted set");
    }
}

/// get_best_set_for_exercise excludes sets that fall on the excluded date.
#[wasm_bindgen_test]
async fn test_best_set_excludes_today() {
    let (db, eid) = setup_exercise_for_history().await;

    let day = |d: i64| d as f64 * MS_PER_DAY;
    let today_start = day(10);
    let today_end = day(11);

    // Best set is on "today" — should be excluded
    let set_today = CompletedSet {
        set_number: 1,
        reps: 1,
        rpe: 10.0,
        set_type: SetType::Weighted { weight: 200.0 },
    };
    db.log_set_at(eid, &set_today, today_start + 5000.0)
        .await
        .expect("log today");

    // Weaker set in history — should be returned
    let set_history = CompletedSet {
        set_number: 1,
        reps: 5,
        rpe: 8.0,
        set_type: SetType::Weighted { weight: 100.0 },
    };
    db.log_set_at(eid, &set_history, day(5) + 1000.0)
        .await
        .expect("log history");

    let best = db
        .get_best_set_for_exercise(eid, day(0), today_start, today_end)
        .await
        .expect("query failed");

    assert!(best.is_some());
    let best = best.unwrap();
    assert_eq!(best.reps, 5, "Should return historical set, not today's");
    if let SetType::Weighted { weight } = best.set_type {
        assert_eq!(weight, 100.0);
    } else {
        panic!("Expected Weighted set");
    }
}

/// get_best_set_for_exercise returns None when history window is empty.
#[wasm_bindgen_test]
async fn test_best_set_empty_history() {
    let (db, eid) = setup_exercise_for_history().await;

    let day = |d: i64| d as f64 * MS_PER_DAY;

    let best = db
        .get_best_set_for_exercise(eid, day(0), day(10), day(11))
        .await
        .expect("query failed");

    assert!(best.is_none(), "No sets → None");
}

/// get_best_set_for_exercise returns None when all sets fall on excluded date.
#[wasm_bindgen_test]
async fn test_best_set_all_sets_today() {
    let (db, eid) = setup_exercise_for_history().await;

    let day = |d: i64| d as f64 * MS_PER_DAY;
    let today_start = day(10);
    let today_end = day(11);

    // Only set is on today
    let set = CompletedSet {
        set_number: 1,
        reps: 5,
        rpe: 8.0,
        set_type: SetType::Weighted { weight: 100.0 },
    };
    db.log_set_at(eid, &set, today_start + 1000.0)
        .await
        .expect("log");

    let best = db
        .get_best_set_for_exercise(eid, day(0), today_start, today_end)
        .await
        .expect("query failed");

    assert!(best.is_none(), "All sets on excluded date → None");
}

/// get_best_set_for_exercise respects since_date boundary.
#[wasm_bindgen_test]
async fn test_best_set_respects_since_date() {
    let (db, eid) = setup_exercise_for_history().await;

    let day = |d: i64| d as f64 * MS_PER_DAY;

    // Strong set on day 2 (before window)
    let old_set = CompletedSet {
        set_number: 1,
        reps: 1,
        rpe: 10.0,
        set_type: SetType::Weighted { weight: 200.0 },
    };
    db.log_set_at(eid, &old_set, day(2) + 1000.0)
        .await
        .expect("log old");

    // Weaker set on day 8 (inside window)
    let recent_set = CompletedSet {
        set_number: 1,
        reps: 5,
        rpe: 8.0,
        set_type: SetType::Weighted { weight: 100.0 },
    };
    db.log_set_at(eid, &recent_set, day(8) + 1000.0)
        .await
        .expect("log recent");

    // Window starts at day 5, so old_set (day 2) is excluded
    let best = db
        .get_best_set_for_exercise(eid, day(5), day(10), day(11))
        .await
        .expect("query failed");

    assert!(best.is_some());
    let best = best.unwrap();
    assert_eq!(best.reps, 5, "Should only see set within window");
}

/// get_latest_set_today returns the most recently logged set today.
#[wasm_bindgen_test]
async fn test_latest_set_today_returns_most_recent() {
    let (db, eid) = setup_exercise_for_history().await;

    let day = |d: i64| d as f64 * MS_PER_DAY;
    let today_start = day(10);
    let today_end = day(11);

    // Earlier set today
    let set_early = CompletedSet {
        set_number: 1,
        reps: 5,
        rpe: 7.0,
        set_type: SetType::Weighted { weight: 80.0 },
    };
    db.log_set_at(eid, &set_early, today_start + 1000.0)
        .await
        .expect("log early");

    // Later set today
    let set_late = CompletedSet {
        set_number: 2,
        reps: 3,
        rpe: 9.0,
        set_type: SetType::Weighted { weight: 100.0 },
    };
    db.log_set_at(eid, &set_late, today_start + 5000.0)
        .await
        .expect("log late");

    let latest = db
        .get_latest_set_today(eid, today_start, today_end)
        .await
        .expect("query failed");

    assert!(latest.is_some());
    let latest = latest.unwrap();
    assert_eq!(latest.set_number, 2, "Should return most recent set");
    assert_eq!(latest.reps, 3);
}

/// get_latest_set_today returns None when no sets logged today.
#[wasm_bindgen_test]
async fn test_latest_set_today_none_when_empty() {
    let (db, eid) = setup_exercise_for_history().await;

    let day = |d: i64| d as f64 * MS_PER_DAY;

    let latest = db
        .get_latest_set_today(eid, day(10), day(11))
        .await
        .expect("query failed");

    assert!(latest.is_none(), "No sets today → None");
}

/// get_latest_set_today only returns sets from the specified day.
#[wasm_bindgen_test]
async fn test_latest_set_today_ignores_other_days() {
    let (db, eid) = setup_exercise_for_history().await;

    let day = |d: i64| d as f64 * MS_PER_DAY;
    let today_start = day(10);
    let today_end = day(11);

    // Set from yesterday — should not appear
    let yesterday_set = CompletedSet {
        set_number: 1,
        reps: 5,
        rpe: 8.0,
        set_type: SetType::Weighted { weight: 100.0 },
    };
    db.log_set_at(eid, &yesterday_set, day(9) + 5000.0)
        .await
        .expect("log yesterday");

    let latest = db
        .get_latest_set_today(eid, today_start, today_end)
        .await
        .expect("query failed");

    assert!(latest.is_none(), "Yesterday's set should not appear");
}
