use crate::state::WorkoutState;
use dioxus::prelude::*;

/// The SQLite magic number used to validate imported files.
const SQLITE_MAGIC: &[u8] = b"SQLite format 3\0";

/// Returns `true` if `data` begins with the SQLite magic header.
pub(crate) fn is_valid_sqlite(data: &[u8]) -> bool {
    data.len() >= SQLITE_MAGIC.len() && data.starts_with(SQLITE_MAGIC)
}

/// Panel with Import button for the workout database.
///
/// - Import: presents a file picker, validates the file, calls `initDatabase()`,
///   and persists via crsqlite-wasm's IndexedDB backend.
#[component]
pub fn DataManagementPanel(state: WorkoutState) -> Element {
    let mut import_error = use_signal(|| Option::<String>::None);
    let mut is_importing = use_signal(|| false);

    rsx! {
        div {
            class: "flex gap-2 mt-4",

            // NOTE: Export button removed — crsqlite-wasm uses IndexedDB
            // persistence and does not support byte-level database export (#179).

            // ── Import button (triggers hidden file input) ─────────────────────
            div {
                class: "relative",
                label {
                    class: if *is_importing.read() {
                        "btn btn-outline btn-sm loading cursor-pointer"
                    } else {
                        "btn btn-outline btn-sm cursor-pointer"
                    },
                    "data-testid": "import-db-btn",
                    r#for: "import-db-file-input",
                    svg {
                        xmlns: "http://www.w3.org/2000/svg",
                        fill: "none",
                        view_box: "0 0 24 24",
                        stroke_width: "1.5",
                        stroke: "currentColor",
                        class: "w-4 h-4 mr-1",
                        path {
                            stroke_linecap: "round",
                            stroke_linejoin: "round",
                            d: "M3 16.5v2.25A2.25 2.25 0 005.25 21h13.5A2.25 2.25 0 0021 18.75V16.5M16.5 12L12 16.5m0 0L7.5 12m4.5 4.5V3"
                        }
                        path {
                            stroke_linecap: "round",
                            stroke_linejoin: "round",
                            d: "M12 3v13.5M7.5 12L12 16.5l4.5-4.5"
                        }
                    }
                    "Import"
                    input {
                        id: "import-db-file-input",
                        r#type: "file",
                        accept: ".sqlite,.db",
                        class: "hidden",
                        disabled: *is_importing.read(),
                        onchange: move |event| {
                            spawn(async move {
                                import_error.set(None);

                                let files = event.files();
                                if files.is_empty() {
                                    log::warn!("[DataManagement] No file selected");
                                    return;
                                }

                                is_importing.set(true);

                                let file = &files[0];
                                let file_name = file.name();

                                let data: Vec<u8> = match file.read_bytes().await {
                                    Ok(bytes) => bytes.to_vec(),
                                    Err(e) => {
                                        log::error!("[DataManagement] Failed to read file {}: {:?}", file_name, e);
                                        import_error.set(Some("Failed to read the selected file.".to_string()));
                                        is_importing.set(false);
                                        return;
                                    }
                                };

                                // Validate SQLite magic number
                                if !is_valid_sqlite(&data) {
                                    log::warn!("[DataManagement] Import rejected: not a valid SQLite file");
                                    import_error.set(Some(
                                        "The selected file is not a valid SQLite database.".to_string(),
                                    ));
                                    is_importing.set(false);
                                    return;
                                }

                                // Re-initialize the database with the imported data
                                let mut database = crate::state::Database::new();
                                match database.init(Some(data.clone())).await {
                                    Ok(_) => {
                                        log::debug!("[DataManagement] Database re-initialized from import");

                                        // crsqlite-wasm auto-persists via IndexedDB — no OPFS write needed.

                                        // Update app state with new database
                                        state.set_database(database);

                                        // Refresh exercise list
                                        if let Err(e) = crate::state::WorkoutStateManager::sync_exercises(&state).await {
                                            log::error!("[DataManagement] Failed to sync exercises after import: {}", e);
                                        }

                                        log::debug!("[DataManagement] Import complete");
                                    }
                                    Err(e) => {
                                        log::error!("[DataManagement] Database init from import failed: {}", e);
                                        import_error.set(Some(format!(
                                            "Failed to load the imported database: {}",
                                            e
                                        )));
                                    }
                                }

                                is_importing.set(false);
                            });
                        }
                    }
                }
            }

            // ── Import error alert ─────────────────────────────────────────────
            if let Some(err) = import_error() {
                div {
                    class: "alert alert-error mt-2 text-sm py-2",
                    "data-testid": "import-error",
                    svg {
                        xmlns: "http://www.w3.org/2000/svg",
                        class: "stroke-current shrink-0 h-5 w-5",
                        fill: "none",
                        view_box: "0 0 24 24",
                        path {
                            stroke_linecap: "round",
                            stroke_linejoin: "round",
                            stroke_width: "2",
                            d: "M10 14l2-2m0 0l2-2m-2 2l-2-2m2 2l2 2m7-2a9 9 0 11-18 0 9 9 0 0118 0z"
                        }
                    }
                    span { {err} }
                }
            }
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    /// Constructs a minimal valid SQLite header (just the magic bytes).
    fn sqlite_magic() -> Vec<u8> {
        SQLITE_MAGIC.to_vec()
    }

    #[test]
    fn accepts_data_starting_with_sqlite_magic() {
        let mut data = sqlite_magic();
        data.extend_from_slice(&[0u8; 100]); // rest of the file
        assert!(is_valid_sqlite(&data));
    }

    #[test]
    fn rejects_empty_data() {
        assert!(!is_valid_sqlite(&[]));
    }

    #[test]
    fn rejects_data_shorter_than_magic() {
        let truncated = &SQLITE_MAGIC[..8];
        assert!(!is_valid_sqlite(truncated));
    }

    #[test]
    fn rejects_non_sqlite_content() {
        let data = b"this is not a sqlite file at all";
        assert!(!is_valid_sqlite(data));
    }

    #[test]
    fn rejects_data_with_wrong_first_byte() {
        let mut data = sqlite_magic();
        data[0] = b'X'; // corrupt the first byte
        assert!(!is_valid_sqlite(&data));
    }
}
