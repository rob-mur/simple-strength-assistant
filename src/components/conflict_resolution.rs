use crate::state::{WorkoutState, WorkoutStateManager};
use crate::sync::ConflictRecord;
use dioxus::prelude::*;
use std::collections::HashMap;

/// Parsed key-value pairs from a JSON-encoded row version.
/// Used to display human-readable field differences in the conflict UI.
fn parse_version_fields(json_str: &str) -> Vec<(String, String)> {
    // Parse the JSON string into key-value pairs, filtering out internal fields
    let map: Result<HashMap<String, serde_json::Value>, _> = serde_json::from_str(json_str);
    match map {
        Ok(m) => {
            let mut fields: Vec<(String, String)> = m
                .into_iter()
                .filter(|(k, _)| !matches!(k.as_str(), "uuid" | "updated_at" | "deleted_at"))
                .map(|(k, v)| {
                    let display = match &v {
                        serde_json::Value::String(s) => s.clone(),
                        serde_json::Value::Null => "(empty)".to_string(),
                        other => other.to_string(),
                    };
                    (k, display)
                })
                .collect();
            fields.sort_by(|a, b| a.0.cmp(&b.0));
            fields
        }
        Err(_) => vec![("raw".to_string(), json_str.to_string())],
    }
}

/// Identifies which fields differ between two versions.
fn differing_fields(version_a: &str, version_b: &str) -> Vec<String> {
    let a: HashMap<String, serde_json::Value> = serde_json::from_str(version_a).unwrap_or_default();
    let b: HashMap<String, serde_json::Value> = serde_json::from_str(version_b).unwrap_or_default();

    let mut diffs = Vec::new();
    for (key, val_a) in &a {
        if matches!(key.as_str(), "uuid" | "updated_at" | "deleted_at") {
            continue;
        }
        if b.get(key) != Some(val_a) {
            diffs.push(key.clone());
        }
    }
    diffs
}

/// A human-readable label for a conflict record, extracted from its version data.
fn conflict_label(conflict: &ConflictRecord) -> String {
    // Try to extract a "name" field from version_a for context
    let parsed: Result<HashMap<String, serde_json::Value>, _> =
        serde_json::from_str(&conflict.version_a);
    if let Ok(map) = parsed
        && let Some(serde_json::Value::String(name)) = map.get("name")
    {
        return format!("{} ({})", conflict.table, name);
    }
    format!(
        "{} [{}]",
        conflict.table,
        &conflict.row_id[..8.min(conflict.row_id.len())]
    )
}

#[derive(Clone, PartialEq)]
enum VersionChoice {
    A,
    B,
}

/// The conflict resolution screen, shown when the sync client reports unresolved conflicts.
///
/// For each conflicting record, shows both versions side by side and lets the
/// user pick one. After all conflicts are resolved, the merged database is
/// saved to OPFS and pushed to the server.
#[component]
pub fn ConflictResolutionScreen(state: WorkoutState) -> Element {
    let conflicts = state.pending_conflicts();
    let mut selections = use_signal(HashMap::<String, VersionChoice>::new);
    let mut resolving = use_signal(|| false);
    let mut resolve_error = use_signal(|| None::<String>);

    let all_resolved = conflicts
        .iter()
        .all(|c| selections().contains_key(&c.row_id));

    let conflict_count = conflicts.len();

    rsx! {
        div {
            class: "flex flex-col min-h-screen bg-base-200",
            "data-testid": "conflict-resolution-screen",

            // Header
            header {
                class: "navbar bg-warning text-warning-content flex-none",
                div {
                    class: "flex-1",
                    h1 {
                        class: "text-xl font-bold px-4",
                        "Resolve Sync Conflicts"
                    }
                }
                div {
                    class: "flex-none px-4",
                    span {
                        class: "badge badge-lg",
                        {
                            let suffix = if conflict_count != 1 { "s" } else { "" };
                            format!("{conflict_count} conflict{suffix}")
                        }
                    }
                }
            }

            // Explanation banner
            div {
                class: "container mx-auto p-4",
                div {
                    class: "alert alert-info mb-6",
                    svg {
                        xmlns: "http://www.w3.org/2000/svg",
                        fill: "none",
                        view_box: "0 0 24 24",
                        class: "stroke-current shrink-0 w-6 h-6",
                        path {
                            stroke_linecap: "round",
                            stroke_linejoin: "round",
                            stroke_width: "2",
                            d: "M13 16h-1v-4h-1m1-4h.01M21 12a9 9 0 11-18 0 9 9 0 0118 0z"
                        }
                    }
                    div {
                        p {
                            class: "font-semibold",
                            "Your data was edited on multiple devices at the same time."
                        }
                        p {
                            class: "text-sm mt-1",
                            "For each conflict below, choose which version to keep. The other version will be discarded."
                        }
                    }
                }

                // Error banner
                if let Some(err) = resolve_error() {
                    div {
                        class: "alert alert-error mb-4",
                        p { "{err}" }
                    }
                }

                // Conflict cards
                for conflict in conflicts.iter() {
                    {
                        let row_id = conflict.row_id.clone();
                        let label = conflict_label(conflict);
                        let fields_a = parse_version_fields(&conflict.version_a);
                        let fields_b = parse_version_fields(&conflict.version_b);
                        let diffs = differing_fields(&conflict.version_a, &conflict.version_b);
                        let current_selection = selections().get(&row_id).cloned();

                        rsx! {
                            div {
                                class: "card bg-base-100 shadow-xl mb-4",
                                "data-testid": "conflict-card",
                                div {
                                    class: "card-body",
                                    h3 {
                                        class: "card-title text-lg mb-4",
                                        "{label}"
                                    }

                                    // Two-column layout for the versions
                                    div {
                                        class: "grid grid-cols-1 md:grid-cols-2 gap-4",

                                        // Version A (local / device A)
                                        {
                                            let is_selected = current_selection.as_ref() == Some(&VersionChoice::A);
                                            let row_id_a = row_id.clone();
                                            rsx! {
                                                div {
                                                    class: if is_selected {
                                                        "card bg-success/10 border-2 border-success cursor-pointer"
                                                    } else {
                                                        "card bg-base-200 border-2 border-base-300 cursor-pointer hover:border-primary"
                                                    },
                                                    "data-testid": "version-a",
                                                    onclick: move |_| {
                                                        let mut s = selections();
                                                        s.insert(row_id_a.clone(), VersionChoice::A);
                                                        selections.set(s);
                                                    },
                                                    div {
                                                        class: "card-body p-4",
                                                        div {
                                                            class: "flex items-center justify-between mb-2",
                                                            span {
                                                                class: "badge badge-outline",
                                                                "Device A (Local)"
                                                            }
                                                            if is_selected {
                                                                span {
                                                                    class: "badge badge-success",
                                                                    "Selected"
                                                                }
                                                            }
                                                        }
                                                        div {
                                                            class: "space-y-1",
                                                            for (key, value) in fields_a.iter() {
                                                                {
                                                                    let is_diff = diffs.contains(key);
                                                                    rsx! {
                                                                        div {
                                                                            class: if is_diff { "text-sm font-semibold text-warning" } else { "text-sm" },
                                                                            span {
                                                                                class: "text-base-content/60 mr-1",
                                                                                "{key}:"
                                                                            }
                                                                            span { "{value}" }
                                                                        }
                                                                    }
                                                                }
                                                            }
                                                        }
                                                    }
                                                }
                                            }
                                        }

                                        // Version B (remote / device B)
                                        {
                                            let is_selected = current_selection.as_ref() == Some(&VersionChoice::B);
                                            let row_id_b = row_id.clone();
                                            rsx! {
                                                div {
                                                    class: if is_selected {
                                                        "card bg-success/10 border-2 border-success cursor-pointer"
                                                    } else {
                                                        "card bg-base-200 border-2 border-base-300 cursor-pointer hover:border-primary"
                                                    },
                                                    "data-testid": "version-b",
                                                    onclick: move |_| {
                                                        let mut s = selections();
                                                        s.insert(row_id_b.clone(), VersionChoice::B);
                                                        selections.set(s);
                                                    },
                                                    div {
                                                        class: "card-body p-4",
                                                        div {
                                                            class: "flex items-center justify-between mb-2",
                                                            span {
                                                                class: "badge badge-outline",
                                                                "Device B (Remote)"
                                                            }
                                                            if is_selected {
                                                                span {
                                                                    class: "badge badge-success",
                                                                    "Selected"
                                                                }
                                                            }
                                                        }
                                                        div {
                                                            class: "space-y-1",
                                                            for (key, value) in fields_b.iter() {
                                                                {
                                                                    let is_diff = diffs.contains(key);
                                                                    rsx! {
                                                                        div {
                                                                            class: if is_diff { "text-sm font-semibold text-warning" } else { "text-sm" },
                                                                            span {
                                                                                class: "text-base-content/60 mr-1",
                                                                                "{key}:"
                                                                            }
                                                                            span { "{value}" }
                                                                        }
                                                                    }
                                                                }
                                                            }
                                                        }
                                                    }
                                                }
                                            }
                                        }
                                    }
                                }
                            }
                        }
                    }
                }

                // Resolve button
                div {
                    class: "sticky bottom-0 bg-base-200 py-4 mt-4",
                    button {
                        class: "btn btn-primary btn-lg btn-block font-bold shadow-lg",
                        disabled: !all_resolved || resolving(),
                        "data-testid": "resolve-conflicts-btn",
                        onclick: move |_| {
                            let state = state;
                            let sels = selections();
                            resolving.set(true);
                            resolve_error.set(None);

                            spawn(async move {
                                match WorkoutStateManager::apply_conflict_resolutions(
                                    &state,
                                    &sels.iter().map(|(k, v)| {
                                        (k.clone(), matches!(v, VersionChoice::A))
                                    }).collect::<HashMap<String, bool>>(),
                                ).await {
                                    Ok(_) => {
                                        // Conflicts resolved — clear conflict state
                                        state.set_pending_conflicts(Vec::new());
                                        state.set_pending_merged_blob(None);
                                        state.set_save_error(None);
                                    }
                                    Err(e) => {
                                        resolve_error.set(Some(format!("Failed to resolve conflicts: {}", e)));
                                    }
                                }
                                resolving.set(false);
                            });
                        },
                        if resolving() {
                            span {
                                class: "loading loading-spinner loading-sm mr-2"
                            }
                            "Resolving..."
                        } else if all_resolved {
                            "Apply Resolutions"
                        } else {
                            "Select a version for each conflict"
                        }
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
    fn test_parse_version_fields_valid_json() {
        let json = r#"{"uuid":"abc","name":"Bench Press","reps":8,"updated_at":"2025-01-01"}"#;
        let fields = parse_version_fields(json);
        // Should exclude uuid and updated_at
        assert_eq!(fields.len(), 2);
        assert!(
            fields
                .iter()
                .any(|(k, v)| k == "name" && v == "Bench Press")
        );
        assert!(fields.iter().any(|(k, v)| k == "reps" && v == "8"));
    }

    #[test]
    fn test_parse_version_fields_invalid_json() {
        let json = "not json";
        let fields = parse_version_fields(json);
        assert_eq!(fields.len(), 1);
        assert_eq!(fields[0].0, "raw");
    }

    #[test]
    fn test_differing_fields() {
        let a = r#"{"uuid":"abc","name":"Bench Press","reps":8}"#;
        let b = r#"{"uuid":"abc","name":"Flat Bench Press","reps":8}"#;
        let diffs = differing_fields(a, b);
        assert_eq!(diffs, vec!["name"]);
    }

    #[test]
    fn test_differing_fields_no_diffs() {
        let a = r#"{"uuid":"abc","name":"Bench Press","reps":8}"#;
        let b = r#"{"uuid":"abc","name":"Bench Press","reps":8}"#;
        let diffs = differing_fields(a, b);
        assert!(diffs.is_empty());
    }

    #[test]
    fn test_conflict_label_with_name() {
        let conflict = ConflictRecord {
            table: "exercises".to_string(),
            row_id: "abc123".to_string(),
            version_a: r#"{"name":"Bench Press"}"#.to_string(),
            version_b: r#"{"name":"Flat Bench"}"#.to_string(),
        };
        assert_eq!(conflict_label(&conflict), "exercises (Bench Press)");
    }

    #[test]
    fn test_conflict_label_without_name() {
        let conflict = ConflictRecord {
            table: "completed_sets".to_string(),
            row_id: "abcdef12-3456-7890".to_string(),
            version_a: r#"{"reps":8}"#.to_string(),
            version_b: r#"{"reps":10}"#.to_string(),
        };
        assert_eq!(conflict_label(&conflict), "completed_sets [abcdef12]");
    }
}
