use crate::state::{ConflictChoice, ConflictRecord};
use dioxus::prelude::*;

/// Props for the ConflictResolution screen.
///
/// `conflicts`   - the list of unresolved conflicts reported by the sync client.
/// `on_resolve`  - called with the resolved conflict list when the user confirms;
///                 the caller is responsible for applying the choices, saving to
///                 OPFS, and pushing to `POST /sync/:sync_id`.
#[derive(Props, Clone, PartialEq)]
pub struct ConflictResolutionProps {
    pub conflicts: Vec<ConflictRecord>,
    pub on_resolve: EventHandler<Vec<ConflictRecord>>,
}

/// Full-screen modal shown when the sync client reports true conflicts.
///
/// The user must select one version for every conflicting record before the
/// "Resolve" button becomes active.  On confirmation, `on_resolve` is called
/// with the updated list so the parent can apply the choices and push the
/// resolved database.
#[component]
pub fn ConflictResolution(props: ConflictResolutionProps) -> Element {
    // use_memo re-evaluates whenever props.conflicts changes, ensuring the
    // local signal stays in sync if the parent re-renders with a new list.
    let mut conflicts = use_signal(|| props.conflicts.clone());
    use_effect(use_reactive!(|props| {
        conflicts.set(props.conflicts.clone());
    }));

    // If there are no conflicts, render nothing.
    if conflicts.read().is_empty() {
        return rsx! {};
    }

    let all_resolved = conflicts.read().iter().all(|c| c.choice.is_some());

    rsx! {
        div {
            "data-testid": "conflict-resolution-screen",
            class: "flex flex-col min-h-screen bg-base-200",

            // Header
            div {
                class: "navbar bg-warning text-warning-content",
                div {
                    class: "flex-1 px-4",
                    h1 {
                        class: "text-xl font-bold",
                        "Sync Conflicts Detected"
                    }
                }
            }

            // Body
            div {
                class: "flex-1 container mx-auto p-4 max-w-lg",
                p {
                    class: "mb-4 text-sm",
                    "The same records were edited on two different devices. \
                     Choose which version to keep for each one."
                }

                for (idx , conflict) in conflicts.read().iter().enumerate() {
                    div {
                        key: "{conflict.uuid}",
                        "data-testid": "conflict-record",
                        class: "card bg-base-100 shadow mb-4",
                        div {
                            class: "card-body p-4",
                            h2 {
                                class: "card-title text-base",
                                "{conflict.field_label}"
                            }

                            // Version A
                            label {
                                class: "flex items-center gap-3 p-3 rounded cursor-pointer hover:bg-base-200",
                                input {
                                    r#type: "radio",
                                    name: "conflict-{idx}",
                                    "data-testid": "version-a-radio-{idx}",
                                    class: "radio radio-primary",
                                    checked: conflict.choice == Some(ConflictChoice::VersionA),
                                    onchange: {
                                        let uuid = conflict.uuid.clone();
                                        move |_| {
                                            let mut list = conflicts.write();
                                            if let Some(rec) = list.iter_mut().find(|r| r.uuid == uuid) {
                                                rec.choice = Some(ConflictChoice::VersionA);
                                            }
                                        }
                                    },
                                }
                                div {
                                    span {
                                        class: "badge badge-outline badge-sm mb-1",
                                        "Device A"
                                    }
                                    p { class: "font-mono text-sm", "{conflict.version_a}" }
                                }
                            }

                            // Version B
                            label {
                                class: "flex items-center gap-3 p-3 rounded cursor-pointer hover:bg-base-200",
                                input {
                                    r#type: "radio",
                                    name: "conflict-{idx}",
                                    "data-testid": "version-b-radio-{idx}",
                                    class: "radio radio-primary",
                                    checked: conflict.choice == Some(ConflictChoice::VersionB),
                                    onchange: {
                                        let uuid = conflict.uuid.clone();
                                        move |_| {
                                            let mut list = conflicts.write();
                                            if let Some(rec) = list.iter_mut().find(|r| r.uuid == uuid) {
                                                rec.choice = Some(ConflictChoice::VersionB);
                                            }
                                        }
                                    },
                                }
                                div {
                                    span {
                                        class: "badge badge-outline badge-sm mb-1",
                                        "Device B"
                                    }
                                    p { class: "font-mono text-sm", "{conflict.version_b}" }
                                }
                            }
                        }
                    }
                }

                // Resolve button
                button {
                    "data-testid": "resolve-button",
                    class: if all_resolved {
                        "btn btn-primary btn-block mt-2"
                    } else {
                        "btn btn-primary btn-block mt-2 btn-disabled"
                    },
                    disabled: !all_resolved,
                    onclick: move |_| {
                        props.on_resolve.call(conflicts.read().clone());
                    },
                    "Resolve Conflicts"
                }
            }
        }
    }
}
