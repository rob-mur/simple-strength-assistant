use crate::state::{SyncStatus, WorkoutState};
use dioxus::prelude::*;

/// A developer-only debug panel for toggling sync status during QA.
///
/// Guarded at the call site with `#[cfg(debug_assertions)]`
/// so the component is excluded from production release builds. It renders as a fixed
/// overlay in the bottom-right corner with one button per `SyncStatus` variant
/// so the sync status indicator can be verified without a live sync client.
#[component]
pub fn DebugPanel() -> Element {
    let workout_state = consume_context::<WorkoutState>();
    let current = workout_state.sync_status();

    let statuses: &[(&str, SyncStatus)] = &[
        ("Idle", SyncStatus::Idle),
        ("Never Synced", SyncStatus::NeverSynced),
        ("Syncing", SyncStatus::Syncing),
        ("Up to Date", SyncStatus::UpToDate),
        ("Error", SyncStatus::Error("debug panel test".into())),
        ("Disabled", SyncStatus::Disabled("debug panel test".into())),
    ];

    rsx! {
        div {
            class: "fixed bottom-16 right-2 z-50 bg-base-300 border border-base-content/20 rounded-lg shadow-lg p-2 pointer-events-none",
            "data-testid": "debug-panel",
            p {
                class: "text-xs font-bold text-base-content/60 uppercase tracking-wider mb-2 text-center",
                "Debug: Sync Status"
            }
            div {
                class: "flex flex-col gap-1",
                for (label, status) in statuses.iter().cloned() {
                    {
                        let status_for_click = status.clone();
                        rsx! {
                            button {
                                class: if current == status {
                                    "btn btn-xs btn-primary pointer-events-auto"
                                } else {
                                    "btn btn-xs btn-ghost pointer-events-auto"
                                },
                                "data-testid": "debug-set-{status.as_attr_str()}",
                                onclick: move |_| workout_state.set_sync_status(status_for_click.clone()),
                                "{label}"
                            }
                        }
                    }
                }
            }
        }
    }
}
