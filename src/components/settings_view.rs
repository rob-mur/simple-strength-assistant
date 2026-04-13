use crate::models::Settings;
use crate::state::{WorkoutState, WorkoutStateManager};
use dioxus::prelude::*;

/// Clamp and round a value to a given step within [min, max].
fn clamp_step(value: f64, min: f64, max: f64, step: f64) -> f64 {
    let clamped = value.clamp(min, max);
    (clamped / step).round() * step
}

#[component]
pub fn SettingsView(state: WorkoutState) -> Element {
    let settings = state.settings();

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
