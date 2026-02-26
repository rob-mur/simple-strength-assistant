#![cfg(test)]

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
        name: "Test Exercise".to_string(),
        set_type_config: SetTypeConfig::Weighted {
            min_weight: 45.0,
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
        name: "Deadlift".to_string(),
        set_type_config: SetTypeConfig::Weighted {
            min_weight: 135.0,
            increment: 10.0,
        },
    };

    let result = db.save_exercise(&exercise).await;

    assert!(result.is_ok(), "Save exercise should succeed");

    // Test updating the same exercise (should replace)
    let updated_exercise = ExerciseMetadata {
        name: "Deadlift".to_string(),
        set_type_config: SetTypeConfig::Weighted {
            min_weight: 145.0,
            increment: 10.0,
        },
    };

    let result2 = db.save_exercise(&updated_exercise).await;

    assert!(result2.is_ok(), "Update exercise should succeed");
}

#[wasm_bindgen_test]
async fn test_export_database() {
    let mut db = Database::new();
    db.init(None).await.expect("Database init failed");

    let exercise = ExerciseMetadata {
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
        name: malicious_name.to_string(),
        set_type_config: SetTypeConfig::Weighted {
            min_weight: 45.0,
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
        name: "Bench Press".to_string(),
        set_type_config: SetTypeConfig::Weighted {
            min_weight: 45.0,
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
