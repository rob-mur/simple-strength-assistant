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
async fn test_no_sessions_table() {
    // The sessions table must not exist in the new schema
    let mut db = Database::new();
    db.init(None).await.expect("Database init failed");

    // Attempt to query sessions table should fail
    let result = db.execute("SELECT count(*) FROM sessions", &[]).await;

    assert!(
        result.is_err(),
        "sessions table should not exist in the new schema"
    );
}

#[wasm_bindgen_test]
async fn test_completed_sets_has_exercise_id_and_recorded_at() {
    // completed_sets must have exercise_id and recorded_at columns
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

    // insert_set now takes exercise_id — if the column doesn't exist this will fail
    let result = db.insert_set(exercise_id, &set).await;
    assert!(
        result.is_ok(),
        "insert_set with exercise_id should succeed: {:?}",
        result
    );

    // Verify recorded_at was written
    use wasm_bindgen::JsValue;
    let rows = db
        .execute(
            "SELECT exercise_id, recorded_at FROM completed_sets WHERE id = ?",
            &[JsValue::from_f64(result.unwrap() as f64)],
        )
        .await
        .expect("Select query failed");
    use wasm_bindgen::JsCast;
    let array = rows.dyn_ref::<js_sys::Array>().expect("Expected array");
    let row = array.get(0);
    let ex_id = js_sys::Reflect::get(&row, &JsValue::from_str("exercise_id"))
        .unwrap()
        .as_f64()
        .expect("exercise_id should be a number") as i64;
    let rec_at = js_sys::Reflect::get(&row, &JsValue::from_str("recorded_at"))
        .unwrap()
        .as_f64()
        .expect("recorded_at should be a number");

    assert_eq!(ex_id, exercise_id, "exercise_id should match");
    assert!(rec_at > 0.0, "recorded_at should be a positive timestamp");
}

#[wasm_bindgen_test]
async fn test_insert_set_weighted() {
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

    let set_id = db
        .insert_set(exercise_id, &set)
        .await
        .expect("Insert set failed");

    assert!(set_id > 0, "Set ID should be positive");
}

#[wasm_bindgen_test]
async fn test_insert_set_bodyweight() {
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
        .insert_set(exercise_id, &set)
        .await
        .expect("Insert set failed");

    assert!(set_id > 0, "Set ID should be positive");
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

    let result = db.execute("SELECT 1", &[]).await;

    assert!(result.is_err(), "Should return error when not initialized");
}

#[wasm_bindgen_test]
async fn test_sql_injection_protection() {
    let mut db = Database::new();
    db.init(None).await.expect("Database init failed");

    // Try exercise name with SQL injection attempt
    let malicious_name = "Test'; DROP TABLE completed_sets; --";

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

    // Verify the exercises table is still accessible
    let exercises_result = db.get_exercises().await;
    assert!(
        exercises_result.is_ok(),
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
    db1.insert_set(exercise_id, &set)
        .await
        .expect("Insert set failed");

    // Export and re-import
    let exported = db1.export().await.expect("Export failed");

    let mut db2 = Database::new();
    db2.init(Some(exported)).await.expect("Import failed");

    // Verify imported data
    use wasm_bindgen::JsCast;
    use wasm_bindgen::JsValue;
    let result = db2
        .execute("SELECT count(*) as count FROM completed_sets", &[])
        .await
        .expect("Select query failed");
    let array = result
        .dyn_ref::<js_sys::Array>()
        .expect("Result should be an array");
    let first_row = array.get(0);
    let count = js_sys::Reflect::get(&first_row, &JsValue::from_str("count"))
        .expect("Failed to get count property")
        .as_f64()
        .expect("Count should be a number") as i64;
    assert_eq!(count, 1, "Expected exactly 1 set in the imported database");

    // Verify we can still insert into the imported database
    let exercise2 = ExerciseMetadata {
        id: None,
        name: "Squat".to_string(),
        set_type_config: SetTypeConfig::Weighted {
            min_weight: 0.0,
            increment: 5.0,
        },
    };
    let new_ex_id = db2.save_exercise(&exercise2).await;
    assert!(
        new_ex_id.is_ok(),
        "Should be able to save exercise in imported database"
    );
}

#[wasm_bindgen_test]
async fn test_db_version_reset_on_old_schema() {
    // Simulate an old database (user_version = 0, has sessions table)
    // by creating a fresh DB with old schema manually, then exporting it
    // and verifying that init() on that data produces a clean new schema.
    let mut old_db = Database::new();
    old_db.init(None).await.expect("Old DB init failed");

    // Manually create the old sessions table (simulating old schema)
    old_db
        .execute(
            r#"CREATE TABLE IF NOT EXISTS sessions (
                id INTEGER PRIMARY KEY AUTOINCREMENT,
                exercise_name TEXT NOT NULL,
                started_at INTEGER NOT NULL
            )"#,
            &[],
        )
        .await
        .expect("Failed to create old sessions table");

    // Export this "old" database
    let old_data = old_db.export().await.expect("Export of old DB failed");

    // Now init a new Database instance with this old data
    let mut new_db = Database::new();
    new_db
        .init(Some(old_data))
        .await
        .expect("init with old data should succeed (with reset)");

    // The sessions table should NOT exist after reset
    let sessions_result = new_db.execute("SELECT count(*) FROM sessions", &[]).await;
    assert!(
        sessions_result.is_err(),
        "sessions table should not exist after DB version reset"
    );

    // The exercises table should exist (fresh schema)
    let exercises_result = new_db.get_exercises().await;
    assert!(
        exercises_result.is_ok(),
        "exercises table should exist after reset"
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

    // 2. Insert first set (earlier recorded_at via direct SQL to control ordering)
    use wasm_bindgen::JsValue;
    db.execute(
        r#"INSERT INTO completed_sets (exercise_id, set_number, reps, rpe, weight, is_bodyweight, recorded_at)
           VALUES (?, 1, 8, 7.0, 100.0, 0, 1000)"#,
        &[JsValue::from_f64(exercise_id as f64)],
    )
    .await
    .expect("Insert first set failed");

    // 3. Insert second set with later recorded_at
    db.execute(
        r#"INSERT INTO completed_sets (exercise_id, set_number, reps, rpe, weight, is_bodyweight, recorded_at)
           VALUES (?, 1, 5, 8.0, 110.0, 0, 2000)"#,
        &[JsValue::from_f64(exercise_id as f64)],
    )
    .await
    .expect("Insert second set failed");

    // 4. Test fetching last set — should return the one with recorded_at=2000 (weight 110)
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
