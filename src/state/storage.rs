// Storage abstraction layer.
//
// Historically this module contained a `StorageBackend` trait and an
// `InMemoryStorage` implementation used by the `test-mode` Cargo feature.
// That feature was removed in #136: E2E tests now run against the production
// binary (OPFS-backed `FileSystemManager`), and the JS test harness injects
// `window.__TEST_MODE__ = true` for any runtime hooks the tests need.
//
// The `StorageBackend` trait and `InMemoryStorage` struct are no longer needed.
// `FileSystemManager` is used directly as the sole storage implementation.
