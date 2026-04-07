#![cfg_attr(feature = "test-mode", allow(dead_code, unused_imports))]
use super::storage::StorageBackend;
use async_trait::async_trait;
use thiserror::Error;
use wasm_bindgen::prelude::*;
use wasm_bindgen_futures::JsFuture;
use web_sys::{js_sys, window};

#[wasm_bindgen(module = "/public/file-handle-storage.js")]
extern "C" {
    #[wasm_bindgen(js_name = storeFileHandle)]
    async fn store_file_handle(handle: JsValue) -> JsValue;

    #[wasm_bindgen(js_name = retrieveFileHandle)]
    async fn retrieve_file_handle() -> JsValue;

    #[wasm_bindgen(js_name = clearFileHandle)]
    async fn clear_file_handle() -> JsValue;

    #[wasm_bindgen(js_name = requestWritePermissionAndStore)]
    async fn request_write_permission_and_store(handle: JsValue) -> JsValue;

    #[wasm_bindgen(js_name = createNewDatabaseFile)]
    async fn create_new_database_file() -> JsValue;
}

/// Maximum allowed size for the database file (100MB).
/// Prevents excessive memory consumption when reading files.
const MAX_FILE_SIZE: usize = 100 * 1024 * 1024;

/// Standard SQLite file header magic number.
/// Used to validate that the selected file is indeed a SQLite database.
const SQLITE_MAGIC_NUMBER: &[u8] = b"SQLite format 3\0";

#[derive(Error, Debug, Clone)]
pub enum FileSystemError {
    #[error("File System Access API not supported")]
    NotSupported,

    #[error("User cancelled file selection")]
    UserCancelled,

    #[error("Permission denied. Please grant file access to continue.")]
    PermissionDenied,

    #[error("Security error: File picker requires user gesture (button click)")]
    SecurityError,

    #[error("Failed to read file: {0}")]
    ReadError(String),

    #[error("Failed to write file: {0}")]
    WriteError(String),

    #[error("JavaScript error: {0}")]
    JsError(String),

    #[error("No file handle available")]
    NoHandle,

    #[error("File is too large (max {} MB)", MAX_FILE_SIZE / 1024 / 1024)]
    FileTooLarge,

    #[error("File is not a valid SQLite database")]
    InvalidFormat,
}

impl From<JsValue> for FileSystemError {
    fn from(err: JsValue) -> Self {
        FileSystemError::JsError(format!("{:?}", err))
    }
}

/// Manages file system operations using OPFS (Origin Private File System).
/// On browsers without OPFS support (iOS Safari < 16.4), the app loads but
/// data is not persisted across sessions (graceful fallback).
#[derive(Clone)]
pub struct FileSystemManager {
    handle: Option<JsValue>,
    use_fallback: bool,
}

impl PartialEq for FileSystemManager {
    fn eq(&self, other: &Self) -> bool {
        self.use_fallback == other.use_fallback && self.handle.is_some() == other.handle.is_some()
    }
}

impl FileSystemManager {
    /// Creates a new FileSystemManager, automatically detecting whether OPFS is
    /// available. Browsers without OPFS (iOS Safari < 16.4) use a no-persistence
    /// fallback: the app loads but data does not persist across sessions.
    pub fn new() -> Self {
        Self {
            handle: None,
            use_fallback: !Self::is_opfs_supported(),
        }
    }

    fn is_opfs_supported() -> bool {
        if let Some(nav) = window().and_then(|w| w.navigator().into())
            && let Ok(storage) = js_sys::Reflect::get(&nav, &JsValue::from_str("storage"))
            && let Ok(get_dir) = js_sys::Reflect::get(&storage, &JsValue::from_str("getDirectory"))
        {
            return get_dir.is_function();
        }
        false
    }

    /// Checks whether the OPFS database file already exists from a prior session.
    /// Returns true if a valid handle was retrieved and stored in this manager.
    /// On the fallback path (no OPFS), always returns true so the app proceeds.
    pub async fn check_cached_handle(&mut self) -> Result<bool, FileSystemError> {
        if self.use_fallback {
            log::debug!("[FileSystem] OPFS not available — using no-persistence fallback mode");
            return Ok(true);
        }

        log::debug!("[FileSystem] Checking for existing OPFS database file...");
        let handle = retrieve_file_handle().await;

        if !handle.is_null() && !handle.is_undefined() {
            log::debug!("[FileSystem] Existing OPFS database file found");
            self.handle = Some(handle);
            Ok(true)
        } else {
            // No prior OPFS file — user needs to create or open a database
            log::debug!("[FileSystem] No existing OPFS database file found");
            Ok(false)
        }
    }

    /// Prompts the user to create a new database file using the browser's save file picker.
    /// The resulting handle is persisted for future sessions.
    pub async fn create_new_file(&mut self) -> Result<(), FileSystemError> {
        if self.use_fallback {
            log::debug!("[FileSystem] Using fallback storage for new database");
            return self.use_fallback_storage();
        }

        log::debug!("[FileSystem] Creating new database file...");

        // create_new_database_file returns { success: bool, handle?: FileHandle, error?: string, message?: string }
        let result = create_new_database_file().await;

        // Check success field
        let success = js_sys::Reflect::get(&result, &JsValue::from_str("success"))
            .map(|v| v.as_bool().unwrap_or(false))
            .unwrap_or(false);

        if !success {
            // Extract error details
            let error_name = js_sys::Reflect::get(&result, &JsValue::from_str("error"))
                .ok()
                .and_then(|v| v.as_string())
                .unwrap_or_else(|| "Unknown".to_string());

            let error_message = js_sys::Reflect::get(&result, &JsValue::from_str("message"))
                .ok()
                .and_then(|v| v.as_string())
                .unwrap_or_else(|| "Unknown error".to_string());

            log::error!(
                "[FileSystem] createNewDatabaseFile failed: {} - {}",
                error_name,
                error_message
            );

            let error_lower = format!("{} {}", error_name, error_message).to_lowercase();

            if error_lower.contains("securityerror") || error_lower.contains("user gesture") {
                log::error!(
                    "[FileSystem] CAUSE: File picker requires user gesture (must be called from button click)"
                );
                return Err(FileSystemError::SecurityError);
            } else if error_lower.contains("notallowederror") || error_lower.contains("permission")
            {
                log::error!("[FileSystem] CAUSE: User denied permission");
                return Err(FileSystemError::PermissionDenied);
            } else if error_lower.contains("abort") {
                log::debug!("[FileSystem] User cancelled file creation dialog");
                return Err(FileSystemError::UserCancelled);
            } else {
                return Err(FileSystemError::JsError(format!(
                    "{}: {}",
                    error_name, error_message
                )));
            }
        }

        // Extract handle
        let handle = js_sys::Reflect::get(&result, &JsValue::from_str("handle"))
            .map_err(|_| FileSystemError::JsError("No handle in response".to_string()))?;

        if handle.is_undefined() || handle.is_null() {
            log::error!("[FileSystem] No handle returned despite success=true");
            return Err(FileSystemError::JsError(
                "No handle in response".to_string(),
            ));
        }

        log::debug!("[FileSystem] New database file created successfully");
        self.handle = Some(handle);

        Ok(())
    }

    /// Opens the existing OPFS database file, or creates a new one if none exists.
    /// With OPFS no user gesture or file picker is required.
    pub async fn prompt_for_file(&mut self) -> Result<(), FileSystemError> {
        if self.use_fallback {
            log::debug!("[FileSystem] Using fallback storage for file operations");
            return self.use_fallback_storage();
        }

        log::debug!("[FileSystem] Opening OPFS database file...");

        // Use createNewDatabaseFile which creates-or-opens the OPFS file
        let result = create_new_database_file().await;

        let success = js_sys::Reflect::get(&result, &JsValue::from_str("success"))
            .map(|v| v.as_bool().unwrap_or(false))
            .unwrap_or(false);

        if !success {
            let error_name = js_sys::Reflect::get(&result, &JsValue::from_str("error"))
                .ok()
                .and_then(|v| v.as_string())
                .unwrap_or_else(|| "Unknown".to_string());

            let error_message = js_sys::Reflect::get(&result, &JsValue::from_str("message"))
                .ok()
                .and_then(|v| v.as_string())
                .unwrap_or_else(|| "Unknown error".to_string());

            log::error!(
                "[FileSystem] OPFS file open failed: {} - {}",
                error_name,
                error_message
            );

            return Err(FileSystemError::JsError(format!(
                "{}: {}",
                error_name, error_message
            )));
        }

        let handle = js_sys::Reflect::get(&result, &JsValue::from_str("handle"))
            .map_err(|_| FileSystemError::JsError("No handle in response".to_string()))?;

        if handle.is_undefined() || handle.is_null() {
            return Err(FileSystemError::JsError(
                "No handle in OPFS response".to_string(),
            ));
        }

        log::debug!("[FileSystem] OPFS database file opened successfully");
        self.handle = Some(handle);

        Ok(())
    }

    /// Switches the manager to fallback (no-persistence) mode.
    /// Used when OPFS is not available (iOS Safari < 16.4): the app loads
    /// and runs normally but data is not persisted across sessions.
    pub fn use_fallback_storage(&mut self) -> Result<(), FileSystemError> {
        log::info!("Using fallback mode (OPFS unavailable — data will not persist)");
        self.use_fallback = true;
        Ok(())
    }

    /// Reads the entire contents of the managed file into a Vec<u8>.
    /// Performs size and format validation (magic number check).
    pub async fn read_file(&self) -> Result<Vec<u8>, FileSystemError> {
        if self.use_fallback {
            return self.read_from_fallback().await;
        }

        let handle = self.handle.as_ref().ok_or(FileSystemError::NoHandle)?;

        let get_file_result = js_sys::Reflect::get(handle, &JsValue::from_str("getFile"))
            .map_err(|_| FileSystemError::ReadError("Failed to get getFile method".to_string()))?;
        let get_file =
            get_file_result
                .dyn_ref::<js_sys::Function>()
                .ok_or(FileSystemError::ReadError(
                    "getFile not a function".to_string(),
                ))?;

        let promise = get_file.call0(handle).map_err(|e| {
            let err_str = format!("{:?}", e);
            if err_str.contains("NotAllowedError") {
                FileSystemError::PermissionDenied
            } else {
                FileSystemError::from(e)
            }
        })?;
        let file = JsFuture::from(js_sys::Promise::from(promise))
            .await
            .map_err(|e| {
                let err_str = format!("{:?}", e);
                if err_str.contains("NotAllowedError") {
                    FileSystemError::PermissionDenied
                } else {
                    FileSystemError::from(e)
                }
            })?;

        // Check file size before reading
        let size_result = js_sys::Reflect::get(&file, &JsValue::from_str("size"))?;
        let size_f64 = size_result.as_f64().ok_or(FileSystemError::ReadError(
            "Failed to get file size".to_string(),
        ))?;

        // Validate size is within valid range before converting to usize
        if size_f64 < 0.0 || size_f64 > usize::MAX as f64 {
            return Err(FileSystemError::ReadError(
                "File size out of valid range".to_string(),
            ));
        }

        let size = size_f64 as usize;

        if size > MAX_FILE_SIZE {
            return Err(FileSystemError::FileTooLarge);
        }

        let array_buffer_result = js_sys::Reflect::get(&file, &JsValue::from_str("arrayBuffer"))?;
        let array_buffer_method =
            array_buffer_result
                .dyn_ref::<js_sys::Function>()
                .ok_or(FileSystemError::ReadError(
                    "arrayBuffer not a function".to_string(),
                ))?;

        let promise = array_buffer_method.call0(&file).map_err(|e| {
            let err_str = format!("{:?}", e);
            if err_str.contains("NotAllowedError") {
                FileSystemError::PermissionDenied
            } else {
                FileSystemError::from(e)
            }
        })?;
        let array_buffer = JsFuture::from(js_sys::Promise::from(promise))
            .await
            .map_err(|e| {
                let err_str = format!("{:?}", e);
                if err_str.contains("NotAllowedError") {
                    FileSystemError::PermissionDenied
                } else {
                    FileSystemError::from(e)
                }
            })?;

        let uint8_array = js_sys::Uint8Array::new(&array_buffer);
        let mut buffer = vec![0; uint8_array.length() as usize];
        uint8_array.copy_to(&mut buffer);

        // Validate SQLite format if file is not empty
        if !buffer.is_empty()
            && buffer.len() >= SQLITE_MAGIC_NUMBER.len()
            && !buffer.starts_with(SQLITE_MAGIC_NUMBER)
        {
            return Err(FileSystemError::InvalidFormat);
        }

        Ok(buffer)
    }

    async fn read_from_fallback(&self) -> Result<Vec<u8>, FileSystemError> {
        // OPFS is not available on this browser (iOS Safari < 16.4).
        // Data does not persist across sessions — return empty so the app
        // starts fresh without crashing. This matches the documented graceful
        // fallback behaviour for unsupported platforms.
        log::debug!(
            "[FileSystem] Fallback read: OPFS unavailable, returning empty (no persistence)"
        );
        Ok(Vec::new())
    }

    /// Writes the provided data to the managed file.
    /// For the File System Access API, it uses a writable stream to ensure atomic-like writes.
    pub async fn write_file(&self, data: &[u8]) -> Result<(), FileSystemError> {
        if self.use_fallback {
            return self.write_to_fallback(data).await;
        }

        let handle = self.handle.as_ref().ok_or(FileSystemError::NoHandle)?;

        let create_writable_result =
            js_sys::Reflect::get(handle, &JsValue::from_str("createWritable")).map_err(|_| {
                FileSystemError::WriteError("Failed to get createWritable".to_string())
            })?;
        let create_writable = create_writable_result.dyn_ref::<js_sys::Function>().ok_or(
            FileSystemError::WriteError("createWritable not a function".to_string()),
        )?;

        let promise = create_writable.call0(handle).map_err(|e| {
            let err_str = format!("{:?}", e);
            if err_str.contains("NotAllowedError") {
                FileSystemError::PermissionDenied
            } else {
                FileSystemError::from(e)
            }
        })?;
        let writable = JsFuture::from(js_sys::Promise::from(promise))
            .await
            .map_err(|e| {
                let err_str = format!("{:?}", e);
                if err_str.contains("NotAllowedError") {
                    FileSystemError::PermissionDenied
                } else {
                    FileSystemError::from(e)
                }
            })?;

        let uint8_array = js_sys::Uint8Array::new_with_length(data.len() as u32);
        uint8_array.copy_from(data);

        let write_result = js_sys::Reflect::get(&writable, &JsValue::from_str("write"))
            .map_err(FileSystemError::from)?;
        let write_method =
            write_result
                .dyn_ref::<js_sys::Function>()
                .ok_or(FileSystemError::WriteError(
                    "write not a function".to_string(),
                ))?;

        let promise = write_method.call1(&writable, &uint8_array).map_err(|e| {
            let err_str = format!("{:?}", e);
            if err_str.contains("NotAllowedError") {
                FileSystemError::PermissionDenied
            } else {
                FileSystemError::from(e)
            }
        })?;
        JsFuture::from(js_sys::Promise::from(promise))
            .await
            .map_err(|e| {
                let err_str = format!("{:?}", e);
                if err_str.contains("NotAllowedError") {
                    FileSystemError::PermissionDenied
                } else {
                    FileSystemError::from(e)
                }
            })?;

        let close_result = js_sys::Reflect::get(&writable, &JsValue::from_str("close"))
            .map_err(FileSystemError::from)?;
        let close_method =
            close_result
                .dyn_ref::<js_sys::Function>()
                .ok_or(FileSystemError::WriteError(
                    "close not a function".to_string(),
                ))?;

        let promise = close_method.call0(&writable).map_err(|e| {
            let err_str = format!("{:?}", e);
            if err_str.contains("NotAllowedError") {
                FileSystemError::PermissionDenied
            } else {
                FileSystemError::from(e)
            }
        })?;
        JsFuture::from(js_sys::Promise::from(promise))
            .await
            .map_err(|e| {
                let err_str = format!("{:?}", e);
                if err_str.contains("NotAllowedError") {
                    FileSystemError::PermissionDenied
                } else {
                    FileSystemError::from(e)
                }
            })?;

        Ok(())
    }

    pub async fn request_permission(&self) -> Result<(), FileSystemError> {
        let handle = self.handle.as_ref().ok_or(FileSystemError::NoHandle)?;
        let result = request_write_permission_and_store(handle.clone()).await;
        if result.is_truthy() {
            Ok(())
        } else {
            Err(FileSystemError::PermissionDenied)
        }
    }

    pub async fn clear_handle(&mut self) -> Result<(), FileSystemError> {
        clear_file_handle().await;
        self.handle = None;
        Ok(())
    }

    async fn write_to_fallback(&self, _data: &[u8]) -> Result<(), FileSystemError> {
        // OPFS is not available on this browser (iOS Safari < 16.4).
        // Writes are silently dropped — the app remains functional but data
        // does not persist across sessions.
        log::debug!(
            "[FileSystem] Fallback write: OPFS unavailable, discarding data (no persistence)"
        );
        Ok(())
    }

    pub fn has_handle(&self) -> bool {
        self.handle.is_some() || self.use_fallback
    }

    pub fn is_using_fallback(&self) -> bool {
        self.use_fallback
    }
}

impl Default for FileSystemManager {
    fn default() -> Self {
        Self::new()
    }
}

// Implement StorageBackend trait for FileSystemManager (OPFS-based storage)
#[async_trait(?Send)]
impl StorageBackend for FileSystemManager {
    fn new() -> Self {
        FileSystemManager::new()
    }

    async fn check_cached_handle(&mut self) -> Result<bool, FileSystemError> {
        FileSystemManager::check_cached_handle(self).await
    }

    async fn create_new_file(&mut self) -> Result<(), FileSystemError> {
        FileSystemManager::create_new_file(self).await
    }

    async fn prompt_for_file(&mut self) -> Result<(), FileSystemError> {
        FileSystemManager::prompt_for_file(self).await
    }

    async fn read_file(&self) -> Result<Vec<u8>, FileSystemError> {
        FileSystemManager::read_file(self).await
    }

    async fn write_file(&self, data: &[u8]) -> Result<(), FileSystemError> {
        FileSystemManager::write_file(self, data).await
    }

    fn has_handle(&self) -> bool {
        FileSystemManager::has_handle(self)
    }

    fn is_using_fallback(&self) -> bool {
        FileSystemManager::is_using_fallback(self)
    }

    async fn request_permission(&self) -> Result<(), FileSystemError> {
        FileSystemManager::request_permission(self).await
    }

    async fn clear_handle(&mut self) -> Result<(), FileSystemError> {
        FileSystemManager::clear_handle(self).await
    }
}
