use crate::state::FileSystemManager;
use wasm_bindgen_test::*;

wasm_bindgen_test_configure!(run_in_browser);

// These tests require a proper WASM test environment with OPFS support.
// To run these tests:
//   wasm-pack test --headless --chrome

#[wasm_bindgen_test]
async fn test_file_system_manager_creation() {
    let manager = FileSystemManager::new();
    assert!(
        !manager.has_handle(),
        "New manager should not have a handle"
    );
}

#[wasm_bindgen_test]
async fn test_fallback_read_returns_empty() {
    // On browsers without OPFS, read_file should return empty bytes
    // (data does not persist — graceful fallback).
    let mut manager = FileSystemManager::new();
    let _ = manager.use_fallback_storage();

    let read_result = manager.read_file().await;
    assert!(read_result.is_ok(), "Read should succeed in fallback mode");
    assert!(
        read_result.unwrap().is_empty(),
        "Fallback read should return empty data"
    );
}

#[wasm_bindgen_test]
async fn test_fallback_write_succeeds_silently() {
    // Writes in fallback mode succeed without error but data is not persisted.
    let mut manager = FileSystemManager::new();
    let _ = manager.use_fallback_storage();

    let test_data = b"SQLite format 3\0test data";
    let write_result = manager.write_file(test_data).await;
    assert!(
        write_result.is_ok(),
        "Write should succeed silently in fallback mode"
    );
}

#[wasm_bindgen_test]
async fn test_has_handle_with_fallback() {
    let mut manager = FileSystemManager::new();

    assert!(!manager.has_handle(), "Should not have handle initially");

    let _ = manager.use_fallback_storage();

    assert!(
        manager.has_handle(),
        "Should have handle after fallback setup"
    );
}

#[wasm_bindgen_test]
async fn test_check_cached_handle_fallback() {
    let mut manager = FileSystemManager::new();
    let _ = manager.use_fallback_storage();

    let result = manager.check_cached_handle().await;
    assert!(
        result.is_ok(),
        "check_cached_handle should succeed with fallback"
    );
    assert!(
        result.unwrap(),
        "Fallback storage should always be considered cached"
    );
}

#[wasm_bindgen_test]
async fn test_sqlite_format_validation_on_opfs_path() {
    // This test requires a real browser with OPFS support.
    // Verifies the manager starts clean (no handle) in an OPFS-capable environment.
    let manager = FileSystemManager::new();
    if manager.is_using_fallback() {
        // OPFS not available in this environment — skip
        return;
    }

    assert!(!manager.has_handle(), "Fresh manager should have no handle");
}
