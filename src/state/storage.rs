use super::file_system::FileSystemError;
use async_trait::async_trait;

/// Storage backend trait that abstracts away the underlying storage mechanism.
/// This allows switching between OPFS (production) and in-memory storage (tests).
#[async_trait(?Send)]
pub trait StorageBackend: Clone + PartialEq {
    /// Creates a new storage backend instance
    fn new() -> Self;

    /// Checks if there is a previously used file handle or cached data
    async fn check_cached_handle(&mut self) -> Result<bool, FileSystemError>;

    /// Creates a new database file or storage location
    async fn create_new_file(&mut self) -> Result<(), FileSystemError>;

    /// Prompts for or opens an existing file
    async fn prompt_for_file(&mut self) -> Result<(), FileSystemError>;

    /// Reads the database contents
    async fn read_file(&self) -> Result<Vec<u8>, FileSystemError>;

    /// Writes data to the database
    async fn write_file(&self, data: &[u8]) -> Result<(), FileSystemError>;

    /// Checks if storage is ready (has handle or is using fallback)
    fn has_handle(&self) -> bool;

    /// Checks if using fallback storage
    fn is_using_fallback(&self) -> bool;

    /// Requests permissions if needed
    async fn request_permission(&self) -> Result<(), FileSystemError>;

    /// Clears the current handle/storage
    async fn clear_handle(&mut self) -> Result<(), FileSystemError>;
}

use std::cell::RefCell;
use std::rc::Rc;

/// In-memory storage backend for E2E tests and environments where OPFS isn't available.
/// This implementation bypasses all file picker dialogs and stores data in memory.
#[derive(Clone)]
pub struct InMemoryStorage {
    data: Rc<RefCell<Option<Vec<u8>>>>,
    initialized: Rc<RefCell<bool>>,
}

impl PartialEq for InMemoryStorage {
    fn eq(&self, other: &Self) -> bool {
        *self.initialized.borrow() == *other.initialized.borrow()
    }
}

#[async_trait(?Send)]
impl StorageBackend for InMemoryStorage {
    fn new() -> Self {
        log::debug!("[InMemoryStorage] Creating new in-memory storage backend");
        Self {
            data: Rc::new(RefCell::new(None)),
            initialized: Rc::new(RefCell::new(false)),
        }
    }

    async fn check_cached_handle(&mut self) -> Result<bool, FileSystemError> {
        log::debug!(
            "[InMemoryStorage] check_cached_handle: returning false (no cache in memory mode)"
        );
        Ok(false)
    }

    async fn create_new_file(&mut self) -> Result<(), FileSystemError> {
        log::debug!("[InMemoryStorage] create_new_file: initializing empty in-memory database");
        *self.data.borrow_mut() = Some(Vec::new());
        *self.initialized.borrow_mut() = true;
        Ok(())
    }

    async fn prompt_for_file(&mut self) -> Result<(), FileSystemError> {
        log::debug!("[InMemoryStorage] prompt_for_file: initializing empty in-memory database");
        *self.data.borrow_mut() = Some(Vec::new());
        *self.initialized.borrow_mut() = true;
        Ok(())
    }

    async fn read_file(&self) -> Result<Vec<u8>, FileSystemError> {
        log::debug!("[InMemoryStorage] read_file: returning in-memory data");
        Ok(self.data.borrow().clone().unwrap_or_default())
    }

    async fn write_file(&self, data: &[u8]) -> Result<(), FileSystemError> {
        log::debug!(
            "[InMemoryStorage] write_file: {} bytes (stored in memory, not persistent across page reloads)",
            data.len()
        );
        *self.data.borrow_mut() = Some(data.to_vec());
        Ok(())
    }

    fn has_handle(&self) -> bool {
        *self.initialized.borrow()
    }

    fn is_using_fallback(&self) -> bool {
        true // In-memory storage is always "fallback"
    }

    async fn request_permission(&self) -> Result<(), FileSystemError> {
        // No permissions needed for in-memory storage
        Ok(())
    }

    async fn clear_handle(&mut self) -> Result<(), FileSystemError> {
        log::debug!("[InMemoryStorage] clear_handle: clearing in-memory data");
        *self.data.borrow_mut() = None;
        *self.initialized.borrow_mut() = false;
        Ok(())
    }
}
