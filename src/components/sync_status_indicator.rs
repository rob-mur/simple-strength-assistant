use crate::state::SyncStatus;
use dioxus::prelude::*;

/// A small UI element that shows the current sync state.
///
/// Placed in the app header so it is always visible without obscuring the
/// main workout UI.  The visual treatment follows the DaisyUI badge palette:
///
/// | State      | Badge style     | Text                |
/// |------------|-----------------|---------------------|
/// | Idle       | badge-ghost     | No sync             |
/// | NeverSynced| badge-warning   | Never synced        |
/// | Syncing    | badge-info      | Syncing…            |
/// | UpToDate   | badge-success   | Up to date          |
/// | Error      | badge-error     | Sync error          |
/// | Disabled   | badge-ghost     | Sync paused         |
#[component]
pub fn SyncStatusIndicator(status: SyncStatus) -> Element {
    let sync_attr = status.as_attr_str();
    let (badge_class, label) = match &status {
        SyncStatus::Idle => ("badge badge-ghost badge-sm", "No sync"),
        SyncStatus::NeverSynced => ("badge badge-warning badge-sm", "Never synced"),
        SyncStatus::Syncing => ("badge badge-info badge-sm", "Syncing…"),
        SyncStatus::UpToDate => ("badge badge-success badge-sm", "Up to date"),
        SyncStatus::Error(_) => ("badge badge-error badge-sm", "Sync error"),
        SyncStatus::Disabled(_) => ("badge badge-ghost badge-sm", "Sync paused"),
    };

    rsx! {
        span {
            class: "{badge_class}",
            "data-testid": "sync-status-indicator",
            "data-sync-status": sync_attr,
            "{label}"
        }
    }
}
