use crate::components::pairing::PairingStep;
use crate::models::Settings;
use crate::state::{SyncStatus, WorkoutState, WorkoutStateManager};
use crate::sync::SyncCredentials;
use dioxus::prelude::*;

/// Write directly to browser `console.log` — always visible in Playwright
/// regardless of log level.
#[cfg(not(test))]
fn js_log(msg: &str) {
    use wasm_bindgen::JsValue;
    web_sys::console::log_1(&JsValue::from_str(msg));
}

/// Clamp and round a value to a given step within [min, max].
fn clamp_step(value: f64, min: f64, max: f64, step: f64) -> f64 {
    let clamped = value.clamp(min, max);
    (clamped / step).round() * step
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

/// Truncate a sync_id to `n` chars + ellipsis for display.
fn truncate_id(id: &str, n: usize) -> String {
    if id.len() <= n {
        id.to_string()
    } else {
        format!("{}...", &id[..n])
    }
}

#[component]
pub fn SettingsView(state: WorkoutState) -> Element {
    let settings = state.settings();

    // Load current credentials for the sync section.
    let mut credentials = use_signal(SyncCredentials::load);
    let mut pairing_step = use_signal(|| PairingStep::Idle);
    let mut join_input = use_signal(String::new);

    // Persist a single-field change immediately.
    let persist = move |updated: Settings| {
        spawn(async move {
            if let Err(e) = WorkoutStateManager::update_settings(&state, updated).await {
                log::warn!("Failed to persist settings: {}", e);
            }
        });
    };

    rsx! {
        div {
            class: "max-w-md mx-auto py-6",
            "data-testid": "settings-view",
            h2 { class: "text-xl font-black uppercase tracking-tight mb-6", "Settings" }

            // ── Sync section ──────────────────────────────────────────────────
            div {
                class: "card bg-base-100 shadow-xl mb-6",
                "data-testid": "sync-section",
                div {
                    class: "card-body",
                    h3 { class: "card-title text-base font-bold mb-4", "Sync" }

                    match (credentials(), pairing_step()) {
                        // ── Paired state ──────────────────────────────────────
                        (Some(creds), PairingStep::Idle) => rsx! {
                            div {
                                "data-testid": "sync-paired-status",
                                div {
                                    class: "flex items-center gap-2 mb-3",
                                    span {
                                        class: "badge badge-success badge-sm",
                                        "Paired"
                                    }
                                    span {
                                        class: "text-sm font-mono opacity-70",
                                        "data-testid": "sync-id-display",
                                        "{truncate_id(&creds.sync_id, 12)}"
                                    }
                                }

                                div {
                                    class: "flex flex-col gap-2",
                                    button {
                                        class: "btn btn-outline btn-sm gap-2",
                                        "data-testid": "copy-sync-id-button",
                                        onclick: {
                                            let _sync_id = creds.sync_id.clone();
                                            move |_| {
                                                #[cfg(not(test))]
                                                copy_to_clipboard(&_sync_id);
                                            }
                                        },
                                        "Copy sync code"
                                    }
                                    button {
                                        class: "btn btn-outline btn-error btn-sm",
                                        "data-testid": "unpair-button",
                                        onclick: move |_| {
                                            #[cfg(not(test))]
                                            {
                                                SyncCredentials::delete();
                                            }
                                            credentials.set(None);
                                            state.set_sync_status(SyncStatus::Idle);
                                        },
                                        "Unpair"
                                    }
                                }
                            }
                        },

                        // ── Showing sync code after setup ──────────────────
                        (Some(creds), PairingStep::ShowingCode) => rsx! {
                            div {
                                "data-testid": "sync-code-display-section",
                                p {
                                    class: "text-sm text-base-content/60 mb-4",
                                    "Your sync code is ready. Share it with your other device to sync workout data."
                                }
                                div {
                                    class: "bg-base-200 rounded-lg p-4 mb-4 text-center",
                                    p {
                                        class: "font-mono text-sm select-all break-all",
                                        "data-testid": "sync-code-value",
                                        "{creds.sync_id}"
                                    }
                                }
                                button {
                                    class: "btn btn-outline btn-sm gap-2 mb-4 w-full",
                                    "data-testid": "copy-sync-id-button",
                                    onclick: {
                                        let _sync_id = creds.sync_id.clone();
                                        move |_| {
                                            #[cfg(not(test))]
                                            copy_to_clipboard(&_sync_id);
                                        }
                                    },
                                    "Copy sync code"
                                }
                                div {
                                    class: "alert alert-info text-sm mb-4",
                                    "data-testid": "sync-backup-reminder",
                                    span {
                                        "Save this code somewhere safe. If you lose your device and haven't exported your data, this code is the only way to recover your workouts."
                                    }
                                }
                                button {
                                    class: "btn btn-ghost btn-sm",
                                    "data-testid": "done-setup-button",
                                    onclick: move |_| pairing_step.set(PairingStep::Idle),
                                    "Done"
                                }
                            }
                        },

                        // ── Unpaired: set up sync ────────────────────────────
                        (None, PairingStep::Idle) => rsx! {
                            div {
                                "data-testid": "sync-unpaired-status",
                                p {
                                    class: "text-sm text-base-content/60 mb-4",
                                    "Sync your workouts across devices. No account required."
                                }
                                div {
                                    class: "flex flex-col gap-2",
                                    button {
                                        class: "btn btn-primary btn-sm",
                                        "data-testid": "setup-sync-button",
                                        onclick: move |_| {
                                            let new_creds = SyncCredentials::generate();
                                            #[cfg(not(test))]
                                            {
                                                if let Err(e) = new_creds.save() {
                                                    log::warn!("Failed to save new credentials: {}", e);
                                                }
                                            }
                                            credentials.set(Some(new_creds));
                                            state.set_sync_status(SyncStatus::NeverSynced);
                                            pairing_step.set(PairingStep::ShowingCode);

                                            #[cfg(not(test))]
                                            {
                                                let state = state;
                                                spawn(async move {
                                                    js_log("[Sync] Initial sync after setup — starting");
                                                    WorkoutStateManager::trigger_background_sync(&state).await;
                                                    js_log("[Sync] Initial sync after setup complete");
                                                });
                                            }
                                        },
                                        "Set up sync"
                                    }
                                    button {
                                        class: "btn btn-outline btn-sm",
                                        "data-testid": "scan-code-button",
                                        onclick: move |_| {
                                            pairing_step.set(PairingStep::Joining);
                                        },
                                        "Join with a code"
                                    }
                                }
                            }
                        },

                        // ── Joining: enter sync code ────────────────────────
                        (_, PairingStep::Joining) => rsx! {
                            div {
                                "data-testid": "qr-scan-section",
                                div {
                                    class: "form-control w-full",
                                    "data-testid": "manual-entry-form",
                                    p {
                                        class: "text-sm text-base-content/60 mb-2",
                                        "Enter the sync code from your other device."
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
                                        value: "{join_input}",
                                        oninput: move |evt| join_input.set(evt.value())
                                    }
                                    button {
                                        class: "btn btn-primary btn-sm mt-2",
                                        "data-testid": "manual-submit-button",
                                        onclick: move |_| {
                                            let sync_id = join_input().trim().to_string();
                                            if sync_id.is_empty() {
                                                pairing_step.set(PairingStep::Error(
                                                    "Sync code cannot be empty".to_string()
                                                ));
                                                return;
                                            }

                                            let new_creds = SyncCredentials::from_sync_id(sync_id);
                                            if !new_creds.is_valid() {
                                                pairing_step.set(PairingStep::Error(
                                                    "Invalid sync code".to_string()
                                                ));
                                                return;
                                            }
                                            #[cfg(not(test))]
                                            {
                                                if let Err(e) = new_creds.save() {
                                                    pairing_step.set(PairingStep::Error(
                                                        format!("Failed to save credentials: {}", e)
                                                    ));
                                                    return;
                                                }
                                            }
                                            credentials.set(Some(new_creds));
                                            state.set_sync_status(SyncStatus::NeverSynced);
                                            pairing_step.set(PairingStep::Syncing);

                                            #[cfg(not(test))]
                                            {
                                                let state = state;
                                                spawn(async move {
                                                    js_log("[Sync] Join complete — triggering initial sync");
                                                    WorkoutStateManager::trigger_background_sync(&state).await;
                                                    js_log("[Sync] Initial sync after pairing complete");
                                                    pairing_step.set(PairingStep::Done);
                                                });
                                            }
                                            #[cfg(test)]
                                            {
                                                pairing_step.set(PairingStep::Done);
                                            }
                                        },
                                        "Connect"
                                    }
                                }
                                button {
                                    class: "btn btn-ghost btn-sm mt-4",
                                    "data-testid": "cancel-scan-button",
                                    onclick: move |_| pairing_step.set(PairingStep::Idle),
                                    "Cancel"
                                }
                            }
                        },

                        // ── Syncing after join ─────────────────────────────
                        (_, PairingStep::Syncing) => rsx! {
                            div {
                                class: "flex flex-col items-center gap-3",
                                "data-testid": "pairing-syncing",
                                div { class: "loading loading-spinner loading-md text-primary" }
                                p { class: "text-sm", "Performing initial sync..." }
                            }
                        },

                        // ── Pairing complete ─────────────────────────────────
                        (_, PairingStep::Done) => rsx! {
                            div {
                                "data-testid": "pairing-done",
                                div {
                                    class: "alert alert-success mb-3",
                                    "Pairing complete. Your devices are now synced."
                                }
                                button {
                                    class: "btn btn-primary btn-sm",
                                    "data-testid": "pairing-done-button",
                                    onclick: move |_| pairing_step.set(PairingStep::Idle),
                                    "Done"
                                }
                            }
                        },

                        // ── Error state ──────────────────────────────────────
                        (_, PairingStep::Error(ref msg)) => {
                            let msg = msg.clone();
                            rsx! {
                                div {
                                    "data-testid": "pairing-error",
                                    div {
                                        class: "alert alert-error mb-3",
                                        "{msg}"
                                    }
                                    button {
                                        class: "btn btn-ghost btn-sm",
                                        "data-testid": "pairing-retry-button",
                                        onclick: move |_| pairing_step.set(PairingStep::Idle),
                                        "Try again"
                                    }
                                }
                            }
                        },

                        // ── Fallback ─────────────────────────────────────────
                        (None, PairingStep::ShowingCode) => rsx! {
                            div {
                                class: "alert alert-warning",
                                "No credentials available. Please set up sync first."
                            }
                        },
                    }
                }
            }

            // ── Algorithm settings card ──────────────────────────────────────
            div {
                class: "card bg-base-100 shadow-xl mb-6",
                div {
                    class: "card-body",
                    h3 { class: "card-title text-base font-bold mb-4", "Algorithm Tuning" }

                    // Target RPE slider (6.0 – 10.0, step 0.5)
                    div {
                        class: "form-control mb-6",
                        label {
                            class: "label",
                            span { class: "label-text font-semibold", "Target RPE" }
                            span {
                                class: "label-text-alt font-mono text-lg",
                                "data-testid": "target-rpe-value",
                                "{settings.target_rpe:.1}"
                            }
                        }
                        input {
                            r#type: "range",
                            min: "6.0",
                            max: "10.0",
                            step: "0.5",
                            value: "{settings.target_rpe}",
                            class: "range range-primary",
                            "data-testid": "target-rpe-slider",
                            oninput: move |evt| {
                                if let Ok(val) = evt.value().parse::<f64>() {
                                    let val = clamp_step(val, 6.0, 10.0, 0.5);
                                    let mut s = settings;
                                    s.target_rpe = val;
                                    persist(s);
                                }
                            }
                        }
                        div {
                            class: "flex justify-between text-xs opacity-60 px-1 mt-1",
                            span { "6" }
                            span { "7" }
                            span { "8" }
                            span { "9" }
                            span { "10" }
                        }
                    }

                    // History window days (numeric input)
                    div {
                        class: "form-control mb-6",
                        label {
                            class: "label",
                            span { class: "label-text font-semibold", "History Window (days)" }
                        }
                        input {
                            r#type: "number",
                            min: "1",
                            max: "365",
                            value: "{settings.history_window_days}",
                            class: "input input-bordered w-full",
                            "data-testid": "history-window-input",
                            oninput: move |evt| {
                                if let Ok(val) = evt.value().parse::<i32>()
                                    && val > 0
                                {
                                    let mut s = settings;
                                    s.history_window_days = val;
                                    persist(s);
                                }
                            }
                        }
                        label {
                            class: "label",
                            span {
                                class: "label-text-alt opacity-60",
                                "Number of past days to consider for suggestions"
                            }
                        }
                    }

                    // Today blend factor slider (0.0 – 1.0, step 0.1)
                    div {
                        class: "form-control mb-2",
                        label {
                            class: "label",
                            span { class: "label-text font-semibold", "Today Blend Factor" }
                            span {
                                class: "label-text-alt font-mono text-lg",
                                "data-testid": "blend-factor-value",
                                "{settings.today_blend_factor:.1}"
                            }
                        }
                        input {
                            r#type: "range",
                            min: "0.0",
                            max: "1.0",
                            step: "0.1",
                            value: "{settings.today_blend_factor}",
                            class: "range range-secondary",
                            "data-testid": "blend-factor-slider",
                            oninput: move |evt| {
                                if let Ok(val) = evt.value().parse::<f64>() {
                                    let val = clamp_step(val, 0.0, 1.0, 0.1);
                                    let mut s = settings;
                                    s.today_blend_factor = val;
                                    persist(s);
                                }
                            }
                        }
                        div {
                            class: "flex justify-between text-xs opacity-60 px-1 mt-1",
                            span { "History" }
                            span { "Balanced" }
                            span { "Today" }
                        }
                    }
                }
            }

            // ── Data management card (existing) ─────────────────────────────
            div {
                class: "card bg-base-100 shadow-xl",
                div {
                    class: "card-body",
                    h3 { class: "card-title text-base font-bold mb-2", "Data Management" }
                    p {
                        class: "text-sm text-base-content/60 mb-4",
                        "Export your workout database for backup or transfer to another device. Import a previously exported database to restore your data."
                    }
                    crate::components::data_management::DataManagementPanel { state }
                }
            }
        }
    }
}
