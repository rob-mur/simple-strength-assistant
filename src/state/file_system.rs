use gloo_storage::{LocalStorage, Storage};
use serde::{Deserialize, Serialize};
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
}

const MAX_FILE_SIZE: usize = 100 * 1024 * 1024; // 100MB
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

#[derive(Clone, Serialize, Deserialize)]
pub struct FileHandle {
    cached: bool,
}

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
    pub fn new() -> Self {
        Self {
            handle: None,
            use_fallback: !Self::is_file_system_api_supported(),
        }
    }

    fn is_file_system_api_supported() -> bool {
        if let Some(window) = window()
            && let Ok(show_open_file_picker) =
                js_sys::Reflect::get(&window, &JsValue::from_str("showOpenFilePicker"))
        {
            return !show_open_file_picker.is_undefined();
        }
        false
    }

    pub async fn check_cached_handle(&mut self) -> Result<bool, FileSystemError> {
        if self.use_fallback {
            web_sys::console::log_1(
                &"[FileSystem] Using fallback storage (IndexedDB/LocalStorage)".into(),
            );
            // Fallback storage doesn't need handle caching
            return Ok(true);
        }

        web_sys::console::log_1(&"[FileSystem] Checking for cached file handle...".into());
        let handle = retrieve_file_handle().await;

        if !handle.is_null() && !handle.is_undefined() {
            web_sys::console::log_1(
                &"[FileSystem] Cached handle retrieved with valid permissions".into(),
            );
            self.handle = Some(handle);
            Ok(true)
        } else {
            // Could be: (1) no handle in IndexedDB, or (2) handle exists but permission denied/requires gesture
            // Both cases require user to select file via button click
            web_sys::console::log_1(
                &"[FileSystem] No cached handle or permissions not granted".into(),
            );
            web_sys::console::log_1(&"[FileSystem] User will need to select file location".into());
            Ok(false)
        }
    }

    pub async fn prompt_for_file(&mut self) -> Result<FileHandle, FileSystemError> {
        if self.use_fallback {
            web_sys::console::log_1(
                &"[FileSystem] Using fallback storage for file operations".into(),
            );
            return self.use_fallback_storage().await;
        }

        web_sys::console::log_1(&"[FileSystem] Opening file picker dialog...".into());
        let window = window().ok_or(FileSystemError::NotSupported)?;

        let show_open_file_picker =
            js_sys::Reflect::get(&window, &JsValue::from_str("showOpenFilePicker"))
                .map_err(|_| FileSystemError::NotSupported)?;

        let picker_fn = show_open_file_picker
            .dyn_ref::<js_sys::Function>()
            .ok_or(FileSystemError::NotSupported)?;

        let options = js_sys::Object::new();

        // Set mode to 'readwrite' so we can both read existing data AND write to it
        js_sys::Reflect::set(
            &options,
            &JsValue::from_str("mode"),
            &JsValue::from_str("readwrite"),
        )?;

        let types_array = js_sys::Array::new();
        let type_obj = js_sys::Object::new();

        js_sys::Reflect::set(
            &type_obj,
            &JsValue::from_str("description"),
            &JsValue::from_str("SQLite Database"),
        )?;

        let accept_obj = js_sys::Object::new();
        let extensions_array = js_sys::Array::new();
        extensions_array.push(&JsValue::from_str(".sqlite"));
        extensions_array.push(&JsValue::from_str(".db"));

        js_sys::Reflect::set(
            &accept_obj,
            &JsValue::from_str("application/x-sqlite3"),
            &extensions_array,
        )?;

        js_sys::Reflect::set(&type_obj, &JsValue::from_str("accept"), &accept_obj)?;
        types_array.push(&type_obj);

        js_sys::Reflect::set(&options, &JsValue::from_str("types"), &types_array)?;

        let promise = picker_fn.call1(&window, &options).map_err(|e| {
            let error_string = format!("{:?}", e);
            web_sys::console::error_1(&"[FileSystem] showOpenFilePicker call failed".into());
            web_sys::console::error_1(&format!("[FileSystem] Error details: {}", error_string).into());

            // Capture stack trace for WASM-JS boundary errors (ERR-04)
            if let Ok(stack) = js_sys::Reflect::get(&e, &"stack".into()) {
                if !stack.is_undefined() {
                    web_sys::console::error_1(&format!("[FileSystem] Stack trace: {:?}", stack).into());
                }
            }

            let error_lower = error_string.to_lowercase();

            if error_lower.contains("securityerror") || error_lower.contains("user gesture") {
                web_sys::console::error_1(&"[FileSystem] CAUSE: File picker requires user gesture (must be called from button click)".into());
                FileSystemError::SecurityError
            } else if error_lower.contains("notallowederror") || error_lower.contains("permission") {
                web_sys::console::error_1(&"[FileSystem] CAUSE: User denied permission".into());
                FileSystemError::PermissionDenied
            } else if error_lower.contains("abort") {
                web_sys::console::log_1(&"[FileSystem] User cancelled file picker dialog".into());
                FileSystemError::UserCancelled
            } else {
                FileSystemError::JsError(error_string)
            }
        })?;

        let handle_array = JsFuture::from(js_sys::Promise::from(promise))
            .await
            .map_err(|e| {
                let error_string = format!("{:?}", e);
                web_sys::console::error_1(&"[FileSystem] File picker promise failed".into());
                web_sys::console::error_1(&format!("[FileSystem] Error details: {}", error_string).into());

                // Capture stack trace for WASM-JS boundary errors (ERR-04)
                if let Ok(stack) = js_sys::Reflect::get(&e, &"stack".into()) {
                    if !stack.is_undefined() {
                        web_sys::console::error_1(&format!("[FileSystem] Stack trace: {:?}", stack).into());
                    }
                }

                let error_lower = error_string.to_lowercase();

                if error_lower.contains("securityerror") || error_lower.contains("user gesture") {
                    web_sys::console::error_1(&"[FileSystem] CAUSE: File picker requires user gesture (must be called from button click)".into());
                    FileSystemError::SecurityError
                } else if error_lower.contains("notallowederror") || error_lower.contains("permission") {
                    web_sys::console::error_1(&"[FileSystem] CAUSE: User denied permission".into());
                    FileSystemError::PermissionDenied
                } else if error_lower.contains("abort") {
                    web_sys::console::log_1(&"[FileSystem] User cancelled file picker dialog".into());
                    FileSystemError::UserCancelled
                } else {
                    FileSystemError::JsError(error_string)
                }
            })?;

        // showOpenFilePicker returns an array of file handles
        // We only allow selecting a single file, so get the first element
        let handle_array = js_sys::Array::from(&handle_array);
        let handle = handle_array.get(0);

        web_sys::console::log_1(
            &"[FileSystem] File handle obtained, storing in IndexedDB...".into(),
        );
        // Store the handle in IndexedDB for persistence
        let store_result = store_file_handle(handle.clone()).await;
        if !store_result.is_truthy() {
            web_sys::console::warn_1(
                &"[FileSystem] Failed to persist file handle to IndexedDB".into(),
            );
            log::warn!("Failed to persist file handle to IndexedDB");
        } else {
            web_sys::console::log_1(&"[FileSystem] File handle stored successfully".into());
        }

        self.handle = Some(handle);

        Ok(FileHandle { cached: true })
    }

    pub async fn use_fallback_storage(&mut self) -> Result<FileHandle, FileSystemError> {
        log::info!("Using IndexedDB/OPFS fallback storage");
        self.use_fallback = true;
        Ok(FileHandle { cached: false })
    }

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

        let promise = get_file.call0(handle)?;
        let file = JsFuture::from(js_sys::Promise::from(promise)).await?;

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

        let promise = array_buffer_method.call0(&file)?;
        let array_buffer = JsFuture::from(js_sys::Promise::from(promise)).await?;

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
        match LocalStorage::get::<Vec<u8>>("workout_db_data") {
            Ok(data) => Ok(data),
            Err(_) => Ok(Vec::new()),
        }
    }

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

        let promise = create_writable.call0(handle)?;
        let writable = JsFuture::from(js_sys::Promise::from(promise)).await?;

        let uint8_array = js_sys::Uint8Array::new_with_length(data.len() as u32);
        uint8_array.copy_from(data);

        let write_result = js_sys::Reflect::get(&writable, &JsValue::from_str("write"))?;
        let write_method =
            write_result
                .dyn_ref::<js_sys::Function>()
                .ok_or(FileSystemError::WriteError(
                    "write not a function".to_string(),
                ))?;

        let promise = write_method.call1(&writable, &uint8_array)?;
        JsFuture::from(js_sys::Promise::from(promise)).await?;

        let close_result = js_sys::Reflect::get(&writable, &JsValue::from_str("close"))?;
        let close_method =
            close_result
                .dyn_ref::<js_sys::Function>()
                .ok_or(FileSystemError::WriteError(
                    "close not a function".to_string(),
                ))?;

        let promise = close_method.call0(&writable)?;
        JsFuture::from(js_sys::Promise::from(promise)).await?;

        Ok(())
    }

    async fn write_to_fallback(&self, data: &[u8]) -> Result<(), FileSystemError> {
        LocalStorage::set("workout_db_data", data.to_vec())
            .map_err(|e| FileSystemError::WriteError(e.to_string()))?;
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
