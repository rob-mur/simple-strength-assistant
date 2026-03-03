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

#[wasm_bindgen_test]
async fn test_create_session() {
    let mut db = Database::new();
    db.init(None).await.expect("Database init failed");

    let session_id = db
        .create_session("Bench Press")
        .await
        .expect("Create session failed");

    assert!(session_id > 0, "Session ID should be positive");
}

#[wasm_bindgen_test]
async fn test_insert_set_weighted() {
    let mut db = Database::new();
    db.init(None).await.expect("Database init failed");

    let session_id = db
        .create_session("Bench Press")
        .await
        .expect("Create session failed");

    let set = CompletedSet {
        set_number: 1,
        reps: 8,
        rpe: 7.5,
        set_type: SetType::Weighted { weight: 135.0 },
    };

    let set_id = db
        .insert_set(session_id, &set)
        .await
        .expect("Insert set failed");

    assert!(set_id > 0, "Set ID should be positive");
}

#[wasm_bindgen_test]
async fn test_insert_set_bodyweight() {
    let mut db = Database::new();
    db.init(None).await.expect("Database init failed");

    let session_id = db
        .create_session("Pull-ups")
        .await
        .expect("Create session failed");

    let set = CompletedSet {
        set_number: 1,
        reps: 10,
        rpe: 8.0,
        set_type: SetType::Bodyweight,
    };

    let set_id = db
        .insert_set(session_id, &set)
        .await
        .expect("Insert set failed");

    assert!(set_id > 0, "Set ID should be positive");
}

#[wasm_bindgen_test]
async fn test_complete_session() {
    let mut db = Database::new();
    db.init(None).await.expect("Database init failed");

    let session_id = db
        .create_session("Squats")
        .await
        .expect("Create session failed");

    let result = db.complete_session(session_id).await;

    assert!(result.is_ok(), "Complete session should succeed");
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

    let result = db.create_session("Test").await;

    assert!(result.is_err(), "Should return error when not initialized");
}

#[wasm_bindgen_test]
async fn test_sql_injection_protection() {
    let mut db = Database::new();
    db.init(None).await.expect("Database init failed");

    // Try exercise name with SQL injection attempt
    let malicious_name = "Test'; DROP TABLE sessions; --";

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

    // Verify we can still create a session (tables weren't dropped)
    let session_result = db.create_session("Normal Exercise").await;

    assert!(
        session_result.is_ok(),
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
    db1.save_exercise(&exercise)
        .await
        .expect("Save exercise failed");

    let session_id = db1
        .create_session("Bench Press")
        .await
        .expect("Create session failed");

    let set = CompletedSet {
        set_number: 1,
        reps: 8,
        rpe: 7.5,
        set_type: SetType::Weighted { weight: 135.0 },
    };
    db1.insert_set(session_id, &set)
        .await
        .expect("Insert set failed");

    db1.complete_session(session_id)
        .await
        .expect("Complete session failed");

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

    // Verify we can create a new session in the imported database
    let new_session = db2.create_session("Bench Press").await;

    assert!(
        new_session.is_ok(),
        "Should be able to create session in imported database"
    );
}

#[wasm_bindgen_test]
async fn test_get_last_set_for_exercise() {
    let mut db = Database::new();
    db.init(None).await.expect("Database init failed");

    // 1. Create exercise
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

    // 2. Create first session and add a set
    let session_id1 = db
        .create_session("Bench Press")
        .await
        .expect("Create session 1 failed");
    let set1 = CompletedSet {
        set_number: 1,
        reps: 8,
        rpe: 7.0,
        set_type: SetType::Weighted { weight: 100.0 },
    };
    db.insert_set(session_id1, &set1)
        .await
        .expect("Insert set 1 failed");
    db.complete_session(session_id1)
        .await
        .expect("Complete session 1 failed");

    // 3. Create second session (later) and add a set
    // Sleep a bit to ensure started_at is different (Date.now())
    // but in JS environment it might be too fast.
    // The query also orders by set_number DESC if started_at is same.

    let session_id2 = db
        .create_session("Bench Press")
        .await
        .expect("Create session 2 failed");
    let set2 = CompletedSet {
        set_number: 1,
        reps: 5,
        rpe: 8.0,
        set_type: SetType::Weighted { weight: 110.0 },
    };
    db.insert_set(session_id2, &set2)
        .await
        .expect("Insert set 2 failed");

    // 4. Test fetching last set
    let last_set = db
        .get_last_set_for_exercise(exercise_id)
        .await
        .expect("Get last set failed");

    assert!(last_set.is_some());
    let last_set = last_set.unwrap();
    assert_eq!(last_set.set_type, SetType::Weighted { weight: 110.0 });
    assert_eq!(last_set.reps, 5);
    assert_eq!(last_set.rpe, 8.0);

    // 5. Test fetching for exercise with no history
    let new_exercise = ExerciseMetadata {
        id: None,
        name: "Squat".to_string(),
        set_type_config: SetTypeConfig::Weighted {
            min_weight: 20.0,
            increment: 2.5,
        },
    };
    let new_id = db
        .save_exercise(&new_exercise)
        .await
        .expect("Save new exercise failed");

    let no_history_result = db
        .get_last_set_for_exercise(new_id)
        .await
        .expect("Get last set for new exercise failed");
    assert!(
        no_history_result.is_none(),
        "Expected None for exercise with no history"
    );
}
