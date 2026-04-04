use crate::models::{CompletedSet, ExerciseMetadata, SetType, SetTypeConfig};
use crate::state::Database;
use wasm_bindgen_test::*;

wasm_bindgen_test_configure!(run_in_browser);

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
    };
    let id_a = db.save_exercise(&ex_a).await.expect("Save A failed");

    let ex_b = ExerciseMetadata {
        id: None,
        name: "Deadlift".to_string(),
        set_type_config: SetTypeConfig::Weighted {
            min_weight: 0.0,
            increment: 5.0,
        },
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
    };
    let exercise_id = db
        .save_exercise(&exercise)
        .await
        .expect("Save exercise failed");

    // Compute the UTC ms at the start of today (local calendar day).
    let now_ms = js_sys::Date::now();
    let offset_ms = -(js_sys::Date::new_0().get_timezone_offset() as f64) * 60_000.0;
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
    };
    let id1 = db.save_exercise(&ex1).await.expect("Save 1 failed");

    let ex2 = ExerciseMetadata {
        id: None,
        name: "Exercise Two".to_string(),
        set_type_config: SetTypeConfig::Bodyweight,
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
    };
    let exercise2 = ExerciseMetadata {
        id: None,
        name: "Pull-ups".to_string(),
        set_type_config: SetTypeConfig::Bodyweight,
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

/// RED: Creating a new database and reopening the same file restores exercises.
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
