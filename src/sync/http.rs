/// Real WASM HTTP implementation of `HttpClient` using `web_sys::fetch`.
///
/// Authentication: `sync_secret` is passed as a raw bearer token in the
/// `X-Sync-Secret` header.  It is **never** placed in any URL.
/// TODO: upgrade to HMAC-SHA256 of the request body if the server supports it.
#[cfg(not(test))]
pub mod wasm {
    use super::super::client::{HttpClient, PushRequest, SyncError, SyncMetadata};
    use wasm_bindgen::JsCast;
    use wasm_bindgen_futures::JsFuture;
    use web_sys::{Request, RequestInit, RequestMode, Response, js_sys};

    fn build_url(sync_id: &str, path: &str) -> String {
        // sync_id is the only path component; sync_secret is NEVER in the URL.
        // Read `window.SYNC_BASE_URL` which is injected into index.html at build
        // time by scripts/inject-sync-url.sh.  Falls back to "/api" when the
        // value is absent, empty, or still contains the un-replaced placeholder.
        let base = js_sys::Reflect::get(
            &web_sys::window().unwrap(),
            &wasm_bindgen::JsValue::from_str("SYNC_BASE_URL"),
        )
        .ok()
        .and_then(|v| v.as_string())
        .filter(|s| !s.is_empty() && !s.contains("%%"))
        .unwrap_or_else(|| "/api".to_string());

        format!("{}/sync/{}{}", base.trim_end_matches('/'), sync_id, path)
    }

    pub struct FetchClient;

    #[async_trait::async_trait(?Send)]
    impl HttpClient for FetchClient {
        async fn push(
            &self,
            sync_id: &str,
            sync_secret: &str,
            body: &PushRequest,
        ) -> Result<(), SyncError> {
            let body_json = serde_json::to_string(body)
                .map_err(|e| SyncError::SerializationError(e.to_string()))?;

            let url = build_url(sync_id, "");
            let opts = RequestInit::new();
            opts.set_method("POST");
            opts.set_mode(RequestMode::Cors);
            opts.set_body(&wasm_bindgen::JsValue::from_str(&body_json));

            let request = Request::new_with_str_and_init(&url, &opts)
                .map_err(|e| SyncError::NetworkError(format!("{:?}", e)))?;

            request
                .headers()
                .set("Content-Type", "application/json")
                .map_err(|e| SyncError::NetworkError(format!("{:?}", e)))?;

            // Pass sync_secret as a header, never in the URL
            request
                .headers()
                .set("X-Sync-Secret", sync_secret)
                .map_err(|e| SyncError::NetworkError(format!("{:?}", e)))?;

            let window = web_sys::window()
                .ok_or_else(|| SyncError::NetworkError("No window object".to_string()))?;

            let resp_value = JsFuture::from(window.fetch_with_request(&request))
                .await
                .map_err(|e| SyncError::NetworkError(format!("{:?}", e)))?;

            let resp: Response = resp_value
                .dyn_into()
                .map_err(|_| SyncError::NetworkError("Not a Response".to_string()))?;

            if resp.ok() {
                Ok(())
            } else {
                Err(SyncError::ServerError(resp.status()))
            }
        }

        async fn get_metadata(
            &self,
            sync_id: &str,
            sync_secret: &str,
        ) -> Result<SyncMetadata, SyncError> {
            let url = build_url(sync_id, "/metadata");
            let opts = RequestInit::new();
            opts.set_method("GET");
            opts.set_mode(RequestMode::Cors);

            let request = Request::new_with_str_and_init(&url, &opts)
                .map_err(|e| SyncError::NetworkError(format!("{:?}", e)))?;

            request
                .headers()
                .set("X-Sync-Secret", sync_secret)
                .map_err(|e| SyncError::NetworkError(format!("{:?}", e)))?;

            let window = web_sys::window()
                .ok_or_else(|| SyncError::NetworkError("No window object".to_string()))?;

            let resp_value = JsFuture::from(window.fetch_with_request(&request))
                .await
                .map_err(|e| SyncError::NetworkError(format!("{:?}", e)))?;

            let resp: Response = resp_value
                .dyn_into()
                .map_err(|_| SyncError::NetworkError("Not a Response".to_string()))?;

            if !resp.ok() {
                return Err(SyncError::ServerError(resp.status()));
            }

            // Use resp.text() → serde_json::from_str to avoid a double
            // round-trip (JS parse → JS stringify → Rust parse).
            let text_promise = resp
                .text()
                .map_err(|e| SyncError::SerializationError(format!("{:?}", e)))?;

            let text_val = JsFuture::from(text_promise)
                .await
                .map_err(|e| SyncError::SerializationError(format!("{:?}", e)))?;

            let json_str = text_val.as_string().ok_or_else(|| {
                SyncError::SerializationError("Response text was not a string".to_string())
            })?;

            serde_json::from_str(&json_str)
                .map_err(|e| SyncError::SerializationError(e.to_string()))
        }

        async fn pull_blob(&self, sync_id: &str, sync_secret: &str) -> Result<Vec<u8>, SyncError> {
            let url = build_url(sync_id, "");
            let opts = RequestInit::new();
            opts.set_method("GET");
            opts.set_mode(RequestMode::Cors);

            let request = Request::new_with_str_and_init(&url, &opts)
                .map_err(|e| SyncError::NetworkError(format!("{:?}", e)))?;

            request
                .headers()
                .set("X-Sync-Secret", sync_secret)
                .map_err(|e| SyncError::NetworkError(format!("{:?}", e)))?;

            let window = web_sys::window()
                .ok_or_else(|| SyncError::NetworkError("No window object".to_string()))?;

            let resp_value = JsFuture::from(window.fetch_with_request(&request))
                .await
                .map_err(|e| SyncError::NetworkError(format!("{:?}", e)))?;

            let resp: Response = resp_value
                .dyn_into()
                .map_err(|_| SyncError::NetworkError("Not a Response".to_string()))?;

            if !resp.ok() {
                return Err(SyncError::ServerError(resp.status()));
            }

            let array_buf_promise = resp
                .array_buffer()
                .map_err(|e| SyncError::NetworkError(format!("{:?}", e)))?;

            let array_buf = JsFuture::from(array_buf_promise)
                .await
                .map_err(|e| SyncError::NetworkError(format!("{:?}", e)))?;

            let uint8 = js_sys::Uint8Array::new(&array_buf);
            let mut buf = vec![0u8; uint8.length() as usize];
            uint8.copy_to(&mut buf);
            Ok(buf)
        }
    }
}
