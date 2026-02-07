#![cfg(test)]

use crate::state::FileSystemManager;
use gloo_storage::{LocalStorage, Storage};
use wasm_bindgen_test::*;

wasm_bindgen_test_configure!(run_in_browser);

// These tests require a proper WASM test environment with File System Access API
// To run these tests:
// 1. wasm-pack test --headless --chrome
// 2. Mock the File System Access API or use a test harness

#[wasm_bindgen_test]
async fn test_file_system_manager_creation() {
    let manager = FileSystemManager::new();
    assert!(
        !manager.has_handle(),
        "New manager should not have a handle"
    );
}

#[wasm_bindgen_test]
async fn test_fallback_storage_write_read() {
    // Clear any existing data
    let _ = LocalStorage::delete("workout_db_data");

    // Create a manager that uses fallback
    let mut manager = FileSystemManager::new();

    // Force fallback usage
    let _ = manager.use_fallback_storage().await;

    // Test data
    let test_data = b"SQLite format 3\0test data";

    // Write data
    let write_result = manager.write_file(test_data).await;
    assert!(write_result.is_ok(), "Write should succeed with fallback");

    // Read data back
    let read_result = manager.read_file().await;
    assert!(read_result.is_ok(), "Read should succeed with fallback");

    let read_data = read_result.unwrap();
    assert_eq!(read_data, test_data, "Read data should match written data");

    // Cleanup
    let _ = LocalStorage::delete("workout_db_data");
}

#[wasm_bindgen_test]
async fn test_sqlite_format_validation() {
    // Clear any existing data
    let _ = LocalStorage::delete("workout_db_data");

    let mut manager = FileSystemManager::new();
    let _ = manager.use_fallback_storage().await;

    // Write invalid SQLite data
    let invalid_data = b"Not a SQLite file";
    let _ = manager.write_file(invalid_data).await;

    // Try to read - should fail validation
    let read_result = manager.read_file().await;

    assert!(
        read_result.is_err(),
        "Should reject files without SQLite magic number"
    );

    // Cleanup
    let _ = LocalStorage::delete("workout_db_data");
}

#[wasm_bindgen_test]
async fn test_empty_file_handling() {
    // Clear any existing data
    let _ = LocalStorage::delete("workout_db_data");

    let mut manager = FileSystemManager::new();
    let _ = manager.use_fallback_storage().await;

    // Write empty data
    let empty_data = b"";
    let write_result = manager.write_file(empty_data).await;
    assert!(write_result.is_ok(), "Should handle empty files");

    // Read back - should work for empty files
    let read_result = manager.read_file().await;
    assert!(read_result.is_ok(), "Should read empty files");

    let read_data = read_result.unwrap();
    assert!(read_data.is_empty(), "Empty file should return empty data");

    // Cleanup
    let _ = LocalStorage::delete("workout_db_data");
}

#[wasm_bindgen_test]
async fn test_has_handle_with_fallback() {
    let mut manager = FileSystemManager::new();

    assert!(!manager.has_handle(), "Should not have handle initially");

    // Set up fallback
    let _ = manager.use_fallback_storage().await;

    assert!(
        manager.has_handle(),
        "Should have handle after fallback setup"
    );
}

#[wasm_bindgen_test]
async fn test_valid_sqlite_file_acceptance() {
    // Clear any existing data
    let _ = LocalStorage::delete("workout_db_data");

    let mut manager = FileSystemManager::new();
    let _ = manager.use_fallback_storage().await;

    // Write valid SQLite data (with proper magic number)
    let valid_data = b"SQLite format 3\0\x00\x00\x00\x00";
    let write_result = manager.write_file(valid_data).await;
    assert!(write_result.is_ok(), "Should write valid SQLite data");

    // Read back - should succeed
    let read_result = manager.read_file().await;
    assert!(
        read_result.is_ok(),
        "Should accept files with valid SQLite magic number"
    );

    let read_data = read_result.unwrap();
    assert!(
        read_data.starts_with(b"SQLite format 3\0"),
        "Data should have SQLite magic number"
    );

    // Cleanup
    let _ = LocalStorage::delete("workout_db_data");
}

#[wasm_bindgen_test]
async fn test_check_cached_handle_fallback() {
    // Clear any existing data
    let _ = LocalStorage::delete("workout_db_data");

    let mut manager = FileSystemManager::new();

    // For fallback, check_cached_handle should return true
    let _ = manager.use_fallback_storage().await;

    let result = manager.check_cached_handle().await;
    assert!(
        result.is_ok(),
        "check_cached_handle should succeed with fallback"
    );
    assert!(
        result.unwrap(),
        "Fallback storage should always be considered cached"
    );

    // Cleanup
    let _ = LocalStorage::delete("workout_db_data");
}

#[wasm_bindgen_test]
async fn test_multiple_write_operations() {
    // Clear any existing data
    let _ = LocalStorage::delete("workout_db_data");

    let mut manager = FileSystemManager::new();
    let _ = manager.use_fallback_storage().await;

    // Write data multiple times
    let data1 = b"SQLite format 3\0data1";
    let data2 = b"SQLite format 3\0data2";

    manager.write_file(data1).await.expect("First write failed");
    manager
        .write_file(data2)
        .await
        .expect("Second write failed");

    // Read back - should get the latest data
    let read_result = manager.read_file().await.expect("Read failed");

    assert_eq!(
        read_result, data2,
        "Should get the most recent written data"
    );

    // Cleanup
    let _ = LocalStorage::delete("workout_db_data");
}

#[wasm_bindgen_test]
async fn test_large_data_handling() {
    // Clear any existing data
    let _ = LocalStorage::delete("workout_db_data");

    let mut manager = FileSystemManager::new();
    let _ = manager.use_fallback_storage().await;

    // Create data larger than typical but still reasonable (1MB)
    let mut large_data = b"SQLite format 3\0".to_vec();
    large_data.extend(vec![0u8; 1024 * 1024]);

    let write_result = manager.write_file(&large_data).await;
    assert!(write_result.is_ok(), "Should handle reasonably large files");

    let read_result = manager.read_file().await;
    assert!(read_result.is_ok(), "Should read large files");

    let read_data = read_result.unwrap();
    assert_eq!(
        read_data.len(),
        large_data.len(),
        "Should preserve data size"
    );

    // Cleanup
    let _ = LocalStorage::delete("workout_db_data");
}
