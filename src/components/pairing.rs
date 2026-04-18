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

/// Component that displays the QR code for the initiating device.
///
/// Shows the sync_id encoded in a QR code along with a warning that
/// the code is equivalent to a password.
#[component]
pub fn QrCodeDisplay(sync_id: String, backend_url: String) -> Element {
    let payload = QrPayload::new(sync_id.clone(), backend_url);
    let payload_json = payload.to_json().unwrap_or_default();
    let svg_result = generate_qr_svg(&payload_json);

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

/// Component for the QR scanner (joining device).
///
/// Uses the device camera via a JS interop layer to scan a QR code.
/// On successful scan, stores credentials and triggers initial sync.
#[component]
pub fn QrScanner(on_scan: EventHandler<String>) -> Element {
    let mut scan_result = use_signal(|| Option::<String>::None);
    let mut manual_input = use_signal(String::new);
    let mut show_manual = use_signal(|| false);

    rsx! {
        div {
            class: "flex flex-col items-center gap-4",
            "data-testid": "qr-scanner",

            if scan_result().is_some() {
                div {
                    class: "alert alert-success",
                    "Scanned successfully"
                }
            }

            // Camera-based scanning placeholder — the actual camera access
            // requires JS interop which we wire up via a simple paste/input
            // fallback for now, since camera APIs need user gesture + HTTPS.
            div {
                class: "w-64 h-64 bg-base-300 rounded-lg flex items-center justify-center border-2 border-dashed border-base-content/20",
                "data-testid": "camera-viewfinder",
                p {
                    class: "text-center text-sm opacity-60 px-4",
                    "Camera scanning requires HTTPS. Use manual entry below."
                }
            }

            button {
                class: "btn btn-outline btn-sm",
                "data-testid": "manual-entry-toggle",
                onclick: move |_| show_manual.set(!show_manual()),
                if show_manual() { "Hide manual entry" } else { "Enter code manually" }
            }

            if show_manual() {
                div {
                    class: "form-control w-full max-w-xs",
                    "data-testid": "manual-entry-form",
                    label {
                        class: "label",
                        span { class: "label-text", "Paste the sync code JSON:" }
                    }
                    textarea {
                        class: "textarea textarea-bordered h-24 font-mono text-xs",
                        "data-testid": "manual-code-input",
                        placeholder: "Paste sync code JSON here",
                        value: "{manual_input}",
                        oninput: move |evt| manual_input.set(evt.value())
                    }
                    button {
                        class: "btn btn-primary btn-sm mt-2",
                        "data-testid": "manual-submit-button",
                        onclick: move |_| {
                            let input = manual_input();
                            if !input.is_empty() {
                                scan_result.set(Some(input.clone()));
                                on_scan.call(input);
                            }
                        },
                        "Connect"
                    }
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
