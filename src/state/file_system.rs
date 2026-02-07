use gloo_storage::{LocalStorage, Storage};
use serde::{Deserialize, Serialize};
use thiserror::Error;
use wasm_bindgen::prelude::*;
use wasm_bindgen_futures::JsFuture;
use web_sys::{js_sys, window};

#[derive(Error, Debug, Clone)]
pub enum FileSystemError {
    #[error("File System Access API not supported")]
    NotSupported,

    #[error("User cancelled file selection")]
    UserCancelled,

    #[error("Failed to read file: {0}")]
    ReadError(String),

    #[error("Failed to write file: {0}")]
    WriteError(String),

    #[error("Failed to access cached handle: {0}")]
    CacheError(String),

    #[error("JavaScript error: {0}")]
    JsError(String),

    #[error("No file handle available")]
    NoHandle,
}

impl From<JsValue> for FileSystemError {
    fn from(err: JsValue) -> Self {
        FileSystemError::JsError(format!("{:?}", err))
    }
}

const HANDLE_CACHE_KEY: &str = "db_file_handle";

#[derive(Clone, Serialize, Deserialize)]
pub struct FileHandle {
    cached: bool,
}

#[derive(Clone)]
pub struct FileSystemManager {
    handle: Option<JsValue>,
    use_fallback: bool,
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
            && let Ok(show_save_file_picker) =
                js_sys::Reflect::get(&window, &JsValue::from_str("showSaveFilePicker"))
        {
            return !show_save_file_picker.is_undefined();
        }
        false
    }

    pub async fn check_cached_handle(&mut self) -> Result<bool, FileSystemError> {
        if self.use_fallback {
            return Ok(false);
        }

        match self.restore_handle_from_cache().await {
            Ok(Some(handle)) => {
                if self.verify_permission(&handle).await? {
                    self.handle = Some(handle);
                    Ok(true)
                } else {
                    Ok(false)
                }
            }
            Ok(None) => Ok(false),
            Err(e) => {
                log::warn!("Failed to restore cached handle: {}", e);
                Ok(false)
            }
        }
    }

    async fn restore_handle_from_cache(&self) -> Result<Option<JsValue>, FileSystemError> {
        let window = window().ok_or(FileSystemError::NotSupported)?;
        let navigator = window.navigator();

        let storage = js_sys::Reflect::get(&navigator, &JsValue::from_str("storage"))
            .map_err(|_| FileSystemError::NotSupported)?;

        if storage.is_undefined() {
            return Ok(None);
        }

        let get_directory = js_sys::Reflect::get(&storage, &JsValue::from_str("getDirectory"))
            .map_err(|_| FileSystemError::NotSupported)?;

        if get_directory.is_undefined() {
            return Ok(None);
        }

        let get_dir_fn = get_directory
            .dyn_ref::<js_sys::Function>()
            .ok_or(FileSystemError::NotSupported)?;

        let promise = get_dir_fn
            .call0(&storage)
            .map_err(|e| FileSystemError::CacheError(format!("{:?}", e)))?;

        let opfs_root = JsFuture::from(js_sys::Promise::from(promise))
            .await
            .map_err(|e| FileSystemError::CacheError(format!("{:?}", e)))?;

        let get_file_handle_result =
            js_sys::Reflect::get(&opfs_root, &JsValue::from_str("getFileHandle"))
                .map_err(|_| FileSystemError::NotSupported)?;
        let get_file_handle = get_file_handle_result
            .dyn_ref::<js_sys::Function>()
            .ok_or(FileSystemError::NotSupported)?;

        let options = js_sys::Object::new();
        js_sys::Reflect::set(
            &options,
            &JsValue::from_str("create"),
            &JsValue::from_bool(false),
        )
        .map_err(|_| FileSystemError::CacheError("Failed to set option".to_string()))?;

        match get_file_handle.call2(&opfs_root, &JsValue::from_str(HANDLE_CACHE_KEY), &options) {
            Ok(promise) => match JsFuture::from(js_sys::Promise::from(promise)).await {
                Ok(handle) => Ok(Some(handle)),
                Err(_) => Ok(None),
            },
            Err(_) => Ok(None),
        }
    }

    async fn verify_permission(&self, handle: &JsValue) -> Result<bool, FileSystemError> {
        let query_permission_result =
            js_sys::Reflect::get(handle, &JsValue::from_str("queryPermission")).map_err(|_| {
                FileSystemError::JsError("Failed to get queryPermission".to_string())
            })?;
        let query_permission = query_permission_result
            .dyn_ref::<js_sys::Function>()
            .ok_or(FileSystemError::JsError(
                "queryPermission not a function".to_string(),
            ))?;

        let options = js_sys::Object::new();
        js_sys::Reflect::set(
            &options,
            &JsValue::from_str("mode"),
            &JsValue::from_str("readwrite"),
        )
        .map_err(|_| FileSystemError::JsError("Failed to set permission mode".to_string()))?;

        let promise = query_permission.call1(handle, &options)?;
        let result = JsFuture::from(js_sys::Promise::from(promise)).await?;

        if let Some(status) = result.as_string()
            && status == "granted"
        {
            return Ok(true);
        }

        let request_permission_result =
            js_sys::Reflect::get(handle, &JsValue::from_str("requestPermission")).map_err(
                |_| FileSystemError::JsError("Failed to get requestPermission".to_string()),
            )?;
        let request_permission = request_permission_result
            .dyn_ref::<js_sys::Function>()
            .ok_or(FileSystemError::JsError(
                "requestPermission not a function".to_string(),
            ))?;

        let promise = request_permission.call1(handle, &options)?;
        let result = JsFuture::from(js_sys::Promise::from(promise)).await?;

        if let Some(status) = result.as_string() {
            Ok(status == "granted")
        } else {
            Ok(false)
        }
    }

    pub async fn prompt_for_file(&mut self) -> Result<FileHandle, FileSystemError> {
        if self.use_fallback {
            return self.use_fallback_storage().await;
        }

        let window = window().ok_or(FileSystemError::NotSupported)?;

        let show_save_file_picker =
            js_sys::Reflect::get(&window, &JsValue::from_str("showSaveFilePicker"))
                .map_err(|_| FileSystemError::NotSupported)?;

        let picker_fn = show_save_file_picker
            .dyn_ref::<js_sys::Function>()
            .ok_or(FileSystemError::NotSupported)?;

        let options = js_sys::Object::new();
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
        js_sys::Reflect::set(
            &options,
            &JsValue::from_str("suggestedName"),
            &JsValue::from_str("workout_data.sqlite"),
        )?;

        let promise = picker_fn
            .call1(&window, &options)
            .map_err(|_| FileSystemError::UserCancelled)?;

        let handle = JsFuture::from(js_sys::Promise::from(promise))
            .await
            .map_err(|_| FileSystemError::UserCancelled)?;

        self.cache_handle(&handle).await?;
        self.handle = Some(handle);

        Ok(FileHandle { cached: true })
    }

    async fn cache_handle(&self, _handle: &JsValue) -> Result<(), FileSystemError> {
        let window = window().ok_or(FileSystemError::NotSupported)?;
        let navigator = window.navigator();

        let storage = js_sys::Reflect::get(&navigator, &JsValue::from_str("storage"))
            .map_err(|_| FileSystemError::NotSupported)?;

        if storage.is_undefined() {
            return Ok(());
        }

        Ok(())
    }

    async fn use_fallback_storage(&mut self) -> Result<FileHandle, FileSystemError> {
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
}

impl Default for FileSystemManager {
    fn default() -> Self {
        Self::new()
    }
}
