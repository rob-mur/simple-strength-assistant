use dioxus::prelude::*;

/// Generates a QR code as an SVG string from the given data.
///
/// The QR code encodes a JSON payload containing the `sync_id` and backend URL,
/// allowing the joining device to establish sync without manual entry.
/// The `sync_id` is never transmitted in a URL query parameter.
pub fn generate_qr_svg(data: &str) -> Result<String, String> {
    use qrcode::QrCode;
    use qrcode::render::svg;

    let code = QrCode::new(data.as_bytes()).map_err(|e| format!("QR generation failed: {}", e))?;
    let svg_str = code
        .render::<svg::Color>()
        .min_dimensions(200, 200)
        .max_dimensions(300, 300)
        .quiet_zone(true)
        .build();
    Ok(svg_str)
}

/// The QR code payload format.  Encoded as JSON inside the QR code.
/// Contains only the `sync_id` and backend URL — never the `sync_secret`.
#[derive(serde::Serialize, serde::Deserialize, Debug, Clone)]
pub struct QrPayload {
    pub sync_id: String,
    pub backend_url: String,
}

impl QrPayload {
    pub fn new(sync_id: String, backend_url: String) -> Self {
        Self {
            sync_id,
            backend_url,
        }
    }

    pub fn to_json(&self) -> Result<String, String> {
        serde_json::to_string(self).map_err(|e| e.to_string())
    }

    pub fn from_json(json: &str) -> Result<Self, String> {
        serde_json::from_str(json).map_err(|e| format!("Invalid QR payload: {}", e))
    }
}

/// Pairing flow state machine.
#[derive(Clone, Debug, PartialEq)]
pub enum PairingStep {
    /// Idle — no pairing in progress.
    Idle,
    /// Showing QR code for another device to scan.
    ShowingQr,
    /// Scanning a QR code from another device.
    Scanning,
    /// Scan succeeded, performing initial sync.
    Syncing,
    /// Pairing complete.
    Done,
    /// An error occurred during pairing.
    Error(String),
}

/// Copy text to clipboard via the Web Clipboard API.
#[cfg(not(test))]
fn copy_to_clipboard(text: &str) {
    use wasm_bindgen::JsCast;
    use wasm_bindgen::JsValue;
    if let Some(window) = web_sys::window()
        && let Ok(clipboard) =
            js_sys::Reflect::get(&window.navigator(), &JsValue::from_str("clipboard"))
        && let Ok(write_fn) = js_sys::Reflect::get(&clipboard, &JsValue::from_str("writeText"))
        && let Some(f) = write_fn.dyn_ref::<js_sys::Function>()
    {
        let _: Result<JsValue, JsValue> = f.call1(&clipboard, &JsValue::from_str(text));
    }
}

/// Read the app's origin URL (e.g. `https://app.example.com`).
fn read_app_origin() -> String {
    #[cfg(not(test))]
    {
        web_sys::window()
            .and_then(|w| w.location().origin().ok())
            .unwrap_or_else(|| "https://localhost".to_string())
    }
    #[cfg(test)]
    {
        "https://localhost".to_string()
    }
}

/// Component that displays the QR code for the initiating device.
///
/// Shows the sync_id encoded in a QR code along with a warning that
/// the code is equivalent to a password.
#[component]
pub fn QrCodeDisplay(sync_id: String, backend_url: String) -> Element {
    // Encode a deeplink URL so native phone QR scanners open the app directly.
    let deeplink = format!("{}/join/{}", read_app_origin(), sync_id);
    let svg_result = generate_qr_svg(&deeplink);

    rsx! {
        div {
            class: "flex flex-col items-center gap-4",
            "data-testid": "qr-code-display",

            match &svg_result {
                Ok(svg) => rsx! {
                    div {
                        class: "bg-white p-4 rounded-lg",
                        dangerous_inner_html: "{svg}"
                    }
                },
                Err(e) => rsx! {
                    div {
                        class: "alert alert-error",
                        "Failed to generate QR code: {e}"
                    }
                },
            }

            button {
                class: "btn btn-outline btn-sm gap-2",
                "data-testid": "copy-sync-id-button",
                onclick: {
                    let sync_id = sync_id.clone();
                    move |_| {
                        #[cfg(not(test))]
                        copy_to_clipboard(&sync_id);
                    }
                },
                "Copy sync code"
            }

            div {
                class: "alert alert-warning text-sm max-w-xs",
                "data-testid": "qr-security-warning",
                svg {
                    xmlns: "http://www.w3.org/2000/svg",
                    fill: "none",
                    view_box: "0 0 24 24",
                    class: "stroke-current shrink-0 w-5 h-5",
                    path {
                        stroke_linecap: "round",
                        stroke_linejoin: "round",
                        stroke_width: "2",
                        d: "M12 9v2m0 4h.01m-6.938 4h13.856c1.54 0 2.502-1.667 1.732-3L13.732 4c-.77-1.333-2.694-1.333-3.464 0L3.34 16c-.77 1.333.192 3 1.732 3z"
                    }
                }
                span {
                    "This code grants full access to your workout data. Treat it like a password — do not share it publicly."
                }
            }
        }
    }
}

/// Component for joining sync from another device.
///
/// Users either scan the QR with their phone's native scanner (which opens a
/// deeplink) or paste the sync code manually here.
#[component]
pub fn QrScanner(on_scan: EventHandler<String>) -> Element {
    let mut manual_input = use_signal(String::new);

    rsx! {
        div {
            class: "flex flex-col items-center gap-4",
            "data-testid": "qr-scanner",

            div {
                class: "form-control w-full max-w-xs",
                "data-testid": "manual-entry-form",
                p {
                    class: "text-sm text-base-content/60 mb-2",
                    "Scan the QR code with your phone's camera to open the app automatically, or paste the sync code below."
                }
                label {
                    class: "label",
                    span { class: "label-text", "Sync code" }
                }
                input {
                    r#type: "text",
                    class: "input input-bordered w-full font-mono text-sm",
                    "data-testid": "manual-code-input",
                    placeholder: "e.g. a1b2c3d4-e5f6-...",
                    value: "{manual_input}",
                    oninput: move |evt| manual_input.set(evt.value())
                }
                button {
                    class: "btn btn-primary btn-sm mt-2",
                    "data-testid": "manual-submit-button",
                    onclick: move |_| {
                        let input = manual_input().trim().to_string();
                        if !input.is_empty() {
                            on_scan.call(input);
                        }
                    },
                    "Connect"
                }
            }
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_qr_payload_roundtrip() {
        let payload = QrPayload::new("test-sync-id".into(), "https://sync.example.com".into());
        let json = payload.to_json().unwrap();
        let parsed = QrPayload::from_json(&json).unwrap();
        assert_eq!(parsed.sync_id, "test-sync-id");
        assert_eq!(parsed.backend_url, "https://sync.example.com");
    }

    #[test]
    fn test_qr_payload_invalid_json() {
        let result = QrPayload::from_json("not json");
        assert!(result.is_err());
    }

    #[test]
    fn test_generate_qr_svg_produces_svg() {
        let svg = generate_qr_svg("hello").unwrap();
        assert!(svg.contains("<svg"), "Should produce SVG markup");
        assert!(svg.contains("</svg>"), "Should be a complete SVG element");
    }

    #[test]
    fn test_generate_qr_svg_with_payload() {
        let payload = QrPayload::new("abc-123".into(), "https://example.com".into());
        let json = payload.to_json().unwrap();
        let svg = generate_qr_svg(&json).unwrap();
        assert!(svg.contains("<svg"));
    }

    #[test]
    fn test_qr_payload_does_not_contain_secret() {
        let payload = QrPayload::new("sync-id".into(), "https://example.com".into());
        let json = payload.to_json().unwrap();
        // The payload struct has no sync_secret field — verify it's absent from JSON
        assert!(!json.contains("sync_secret"));
        assert!(!json.contains("secret"));
    }
}
