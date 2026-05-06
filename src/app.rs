use crate::components::bottom_sheet::{BottomSheet, BottomSheetItem, BottomSheetVariant};
use crate::components::confirmation_dialog::{ConfirmVariant, ConfirmationDialog};
#[cfg(debug_assertions)]
use crate::components::debug_panel::DebugPanel;
use crate::components::exercise_form::ExerciseForm;
use crate::components::history_view::HistoryView;
use crate::components::library_view::LibraryView;
use crate::components::rpe_slider::RPESlider;
use crate::components::settings_view::SettingsView;
use crate::components::sync_status_indicator::SyncStatusIndicator;
use crate::components::tab_bar::{Tab, TabBar};
use crate::components::tape_measure::TapeMeasure;
use crate::components::workout_view::WorkoutView;
use crate::models::{CompletedSet, SetType, SetTypeConfig};
use crate::state::{
    InitializationState, WorkoutError, WorkoutState, WorkoutStateManager, is_archive_blocked,
};
use dioxus::prelude::*;
use wasm_bindgen::prelude::*;

/// Write directly to browser `console.log`.
fn js_log(msg: &str) {
    web_sys::console::log_1(&JsValue::from_str(msg));
}

struct ErrorInfo {
    title: String,
    message: String,
    recovery_tip: Option<String>,
    retry_label: String,
}

fn parse_error_for_ui(error: &WorkoutError) -> ErrorInfo {
    match error {
        WorkoutError::FileSystem(crate::state::FileSystemError::InvalidFormat) => ErrorInfo {
            title: "Invalid File Format".to_string(),
            message: "The selected file is not a valid SQLite database.".to_string(),
            recovery_tip: Some(
                "Please select a .sqlite or .db file, or create a new database file.".to_string(),
            ),
            retry_label: "Select Different File".to_string(),
        },
        WorkoutError::Database(crate::state::DatabaseError::InitializationError(_)) => ErrorInfo {
            title: "Database Initialization Failed".to_string(),
            message: "Could not set up the database. The file may be corrupted.".to_string(),
            recovery_tip: Some(
                "Try selecting a different file or creating a new database.".to_string(),
            ),
            retry_label: "Try Again".to_string(),
        },
        WorkoutError::FileSystem(crate::state::FileSystemError::PermissionDenied) => ErrorInfo {
            title: "Permission Denied".to_string(),
            message: "File access permission was not granted.".to_string(),
            recovery_tip: Some(
                "Grant permission to access the file, or use browser storage instead.".to_string(),
            ),
            retry_label: "Grant Permission".to_string(),
        },
        WorkoutError::FileSystem(crate::state::FileSystemError::UserCancelled) => ErrorInfo {
            title: "File Selection Cancelled".to_string(),
            message: "No database file was selected.".to_string(),
            recovery_tip: Some(
                "Click below to select where to store your workout data.".to_string(),
            ),
            retry_label: "Select File".to_string(),
        },
        WorkoutError::FileSystem(crate::state::FileSystemError::FileTooLarge) => ErrorInfo {
            title: "File Too Large".to_string(),
            message: "The selected database file exceeds the 100 MB limit.".to_string(),
            recovery_tip: Some(
                "Try selecting a smaller file or export your data to start fresh.".to_string(),
            ),
            retry_label: "Select Different File".to_string(),
        },
        _ => ErrorInfo {
            title: "Error occurred".to_string(),
            message: error.to_string(),
            recovery_tip: Some("Check your browser console for details and try again.".to_string()),
            retry_label: "Retry".to_string(),
        },
    }
}

// ── Router ────────────────────────────────────────────────────────────────────

#[derive(Clone, Routable, Debug, PartialEq)]
pub enum Route {
    #[layout(Shell)]
    #[route("/workout")]
    WorkoutTab,
    #[route("/workout/history")]
    WorkoutHistory,
    #[route("/workout/history/:exercise_id")]
    WorkoutHistoryExercise { exercise_id: String },
    #[route("/library")]
    LibraryTab,
    #[route("/library/:exercise_id")]
    LibraryExercise { exercise_id: String },
    #[route("/analysis")]
    AnalysisTab,
    #[route("/settings")]
    SettingsTab,
    #[end_layout]
    #[route("/:..path")]
    NotFound { path: Vec<String> },
}

// ── Shell layout (tab bar + content area) ─────────────────────────────────────

#[component]
fn Shell() -> Element {
    let workout_state = consume_context::<WorkoutState>();
    let mut navigation_state = consume_context::<TabNavigationState>();
    let route = use_route::<Route>();
    let navigator = use_navigator();

    let is_settings_route = matches!(&route, Route::SettingsTab);

    let active_tab = match &route {
        Route::WorkoutTab | Route::WorkoutHistory | Route::WorkoutHistoryExercise { .. } => {
            Tab::Workout
        }
        Route::LibraryTab | Route::LibraryExercise { .. } => Tab::Library,
        Route::AnalysisTab => Tab::Analysis,
        _ => Tab::Workout,
    };

    // AC #4: Update last seen route for the current tab whenever it changes.
    // We do this in the component body so it runs on every render of Shell
    // (which re-renders whenever the route changes).
    let current_route = route.clone();
    match current_route.clone() {
        Route::WorkoutTab | Route::WorkoutHistory | Route::WorkoutHistoryExercise { .. }
            if *navigation_state.last_workout_route.peek() != current_route =>
        {
            navigation_state.last_workout_route.set(current_route);
        }
        Route::LibraryTab | Route::LibraryExercise { .. }
            if *navigation_state.last_library_route.peek() != current_route =>
        {
            navigation_state.last_library_route.set(current_route);
        }
        _ => {}
    }

    let storage_mode_banner = if let Some(fm) = workout_state.file_manager() {
        if fm.is_using_fallback() {
            Some(rsx! {
                div {
                    class: "alert alert-info mb-4",
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
                        h4 { class: "font-bold", "Browser Storage Mode" }
                        p {
                            class: "text-sm",
                            "Your data is stored in your browser's private storage (OPFS). This works offline but won't sync across devices or browsers."
                        }
                    }
                }
            })
        } else {
            None
        }
    } else {
        None
    };

    let save_error_banner = workout_state.save_error().map(|err_msg| rsx! {
        div {
            class: "alert alert-warning mb-4",
            svg {
                xmlns: "http://www.w3.org/2000/svg",
                fill: "none",
                view_box: "0 0 24 24",
                class: "stroke-current shrink-0 w-6 h-6",
                path {
                    stroke_linecap: "round",
                    stroke_linejoin: "round",
                    stroke_width: "2",
                    d: "M12 9v2m0 4h.01m-6.938 4h13.856c1.54 0 2.502-1.667 1.732-3L13.732 4c-.77-1.333-2.694-1.333-3.464 0L3.34 16c-.77 1.333.192 3 1.732 3z"
                }
            }
            div {
                h4 { class: "font-bold", "Sync Warning" }
                p { class: "text-sm", "{err_msg}" }
            }
        }
    });

    rsx! {
        div {
            class: "flex-1 flex flex-col min-h-0",
            div {
                class: "flex-1 overflow-y-auto min-h-0",
                "data-testid": "shell-content",
                div {
                    class: "container mx-auto p-2 sm:p-4",
                    if !is_settings_route {
                        {storage_mode_banner}
                        {save_error_banner}
                    }
                    Outlet::<Route> {}
                }
            }
            if !is_settings_route {
                TabBar {
                    active_tab,
                    on_change: move |tab| {
                        match tab {
                            Tab::Workout => {
                                if active_tab == Tab::Workout {
                                    navigator.push(Route::WorkoutTab);
                                } else {
                                    let target = navigation_state.last_workout_route.peek();
                                    navigator.push(target.clone());
                                }
                            }
                            Tab::Library => {
                                if active_tab == Tab::Library {
                                    navigator.push(Route::LibraryTab);
                                } else {
                                    let target = navigation_state.last_library_route.peek();
                                    navigator.push(target.clone());
                                }
                            }
                            Tab::Analysis => {
                                navigator.push(Route::AnalysisTab);
                            }
                        }
                    }
                }
            }
        }
    }
}

// ── Route components ──────────────────────────────────────────────────────────

#[component]
fn WorkoutTab() -> Element {
    let state = consume_context::<WorkoutState>();
    rsx! { WorkoutView { state } }
}

#[component]
fn WorkoutHistory() -> Element {
    let state = consume_context::<WorkoutState>();
    let navigator = use_navigator();
    rsx! { HistoryView { state, exercise_id: None, on_back: move |_| { navigator.push(Route::WorkoutTab); } } }
}

#[component]
fn WorkoutHistoryExercise(exercise_id: String) -> Element {
    let state = consume_context::<WorkoutState>();
    let navigator = use_navigator();
    rsx! { HistoryView { state, exercise_id: Some(exercise_id), on_back: move |_| { navigator.push(Route::WorkoutTab); } } }
}

#[component]
fn LibraryTab() -> Element {
    rsx! { LibraryView {} }
}

#[component]
fn AnalysisTab() -> Element {
    let state = consume_context::<WorkoutState>();
    rsx! { HistoryView { state, exercise_id: None } }
}

#[component]
fn SettingsTab() -> Element {
    let state = consume_context::<WorkoutState>();
    rsx! {
        div {
            // Back arrow header for Settings
            div {
                class: "flex items-center gap-2 mb-4",
                button {
                    class: "btn btn-ghost btn-sm btn-circle",
                    "data-testid": "settings-back-button",
                    onclick: move |_| {
                        // Use browser history back so user returns to whichever tab they came from
                        if let Some(window) = web_sys::window() {
                            let _ = window.history().and_then(|h| h.back());
                        }
                    },
                    svg {
                        xmlns: "http://www.w3.org/2000/svg",
                        fill: "none",
                        view_box: "0 0 24 24",
                        stroke_width: "2.5",
                        stroke: "currentColor",
                        class: "w-6 h-6",
                        path {
                            stroke_linecap: "round",
                            stroke_linejoin: "round",
                            d: "M15.75 19.5L8.25 12l7.5-7.5"
                        }
                    }
                }
                h2 {
                    class: "text-xl font-bold",
                    "Settings"
                }
            }
            SettingsView { state }
        }
    }
}

#[component]
fn LibraryExercise(exercise_id: String) -> Element {
    let workout_state = consume_context::<WorkoutState>();
    let navigator = use_navigator();
    let mut show_edit_form = use_signal(|| false);
    let mut show_archive_dialog = use_signal(|| false);

    // Look in active exercises first; fall back to archived exercises.
    let active_exercises = workout_state.exercises();
    let active_match = active_exercises
        .iter()
        .find(|e| e.id.as_deref() == Some(exercise_id.as_str()))
        .cloned();
    let is_archived = active_match.is_none();

    // For archived exercises we need to fetch separately.  We store the
    // result in a signal so the async fetch can update the UI.
    let mut archived_exercise: Signal<Option<crate::models::ExerciseMetadata>> =
        use_signal(|| None);

    {
        let eid = exercise_id.clone();
        use_effect(move || {
            if is_archived {
                let eid2 = eid.clone();
                spawn(async move {
                    match WorkoutStateManager::fetch_archived_exercises(&workout_state).await {
                        Ok(list) => {
                            let found = list
                                .into_iter()
                                .find(|e| e.id.as_deref() == Some(eid2.as_str()));
                            archived_exercise.set(found);
                        }
                        Err(e) => log::warn!("Failed to fetch archived exercises: {}", e),
                    }
                });
            }
        });
    }

    let exercise = if is_archived {
        archived_exercise()
    } else {
        active_match
    };

    let Some(exercise) = exercise else {
        return rsx! {
            div {
                class: "max-w-md mx-auto p-4",
                p { "Exercise not found." }
                button {
                    class: "btn btn-primary",
                    onclick: move |_| { navigator.push(Route::LibraryTab); },
                    "Back to Library"
                }
            }
        };
    };

    if show_edit_form() {
        return rsx! {
            div {
                class: "max-w-md mx-auto p-4",
                ExerciseForm {
                    initial_exercise: Some(exercise.clone()),
                    on_cancel: move |_| show_edit_form.set(false),
                    on_save: move |updated_exercise| {
                        let state = workout_state;
                        spawn(async move {
                            if let Err(e) = WorkoutStateManager::save_exercise(&state, updated_exercise).await {
                                WorkoutStateManager::handle_error(&state, e);
                            }
                            show_edit_form.set(false);
                        });
                    }
                }
            }
        };
    }

    let exercise_name = exercise.name.clone();
    let exercise_id_for_archive = exercise_id.clone();
    let exercise_id_for_unarchive = exercise_id.clone();
    let archive_blocked = is_archive_blocked(&exercise_id, &workout_state.current_session());

    rsx! {
        div {
            class: "max-w-md mx-auto",
            "data-testid": "exercise-detail-view",

            // Archive dialog (active exercises only)
            if show_archive_dialog() {
                ConfirmationDialog {
                    title: format!("Archive {}?", exercise_name),
                    body: "Hidden from library, removed from upcoming plans. 0 future plans will be deleted.".to_string(),
                    confirm_label: "Archive".to_string(),
                    cancel_label: "Cancel".to_string(),
                    variant: ConfirmVariant::Default,
                    on_cancel: move |_| show_archive_dialog.set(false),
                    on_confirm: move |_| {
                        let state = workout_state;
                        let eid = exercise_id_for_archive.clone();
                        show_archive_dialog.set(false);
                        spawn(async move {
                            if let Err(e) = WorkoutStateManager::archive_exercise(&state, &eid).await {
                                WorkoutStateManager::handle_error(&state, e);
                            } else {
                                navigator.push(Route::LibraryTab);
                            }
                        });
                    }
                }
            }

            // Header
            div {
                class: "flex items-center justify-between mb-4 sticky top-0 bg-base-200 z-20 py-2",
                div {
                    class: "flex items-center gap-1",
                    button {
                        class: "btn btn-ghost btn-sm btn-circle",
                        "data-testid": "back-button",
                        onclick: move |_| { navigator.push(Route::LibraryTab); },
                        svg {
                            xmlns: "http://www.w3.org/2000/svg",
                            fill: "none",
                            view_box: "0 0 24 24",
                            stroke_width: "2.5",
                            stroke: "currentColor",
                            class: "w-6 h-6",
                            path {
                                stroke_linecap: "round",
                                stroke_linejoin: "round",
                                d: "M15.75 19.5L8.25 12l7.5-7.5"
                            }
                        }
                    }
                    h2 {
                        class: "text-xl font-black tracking-tight truncate max-w-[150px] sm:max-w-xs",
                        "{exercise.name.to_uppercase()}"
                    }
                }
                div {
                    class: "flex gap-2",
                    // Edit pencil — always available (rename allowed on archived too)
                    button {
                        class: "btn btn-ghost btn-sm btn-circle",
                        "data-testid": "edit-button",
                        onclick: move |_| show_edit_form.set(true),
                        svg {
                            xmlns: "http://www.w3.org/2000/svg",
                            fill: "none",
                            view_box: "0 0 24 24",
                            stroke_width: "2",
                            stroke: "currentColor",
                            class: "w-5 h-5",
                            path {
                                stroke_linecap: "round",
                                stroke_linejoin: "round",
                                d: "m16.862 4.487 1.687-1.688a1.875 1.875 0 1 1 2.652 2.652L10.582 16.07a4.5 4.5 0 0 1-1.897 1.13L6 18l.8-2.685a4.5 4.5 0 0 1 1.13-1.897l8.932-8.931Zm0 0L19.5 7.125M18 14v4.75A2.25 2.25 0 0 1 15.75 21H5.25A2.25 2.25 0 0 1 3 18.75V8.25A2.25 2.25 0 0 1 5.25 6H10"
                            }
                        }
                    }

                    if is_archived {
                        // Archived: show Unarchive button instead of START/trash
                        button {
                            class: "btn btn-primary btn-sm px-4 font-bold shadow-sm",
                            "data-testid": "unarchive-button",
                            onclick: move |_| {
                                let state = workout_state;
                                let eid = exercise_id_for_unarchive.clone();
                                spawn(async move {
                                    if let Err(e) = WorkoutStateManager::unarchive_exercise(&state, &eid).await {
                                        WorkoutStateManager::handle_error(&state, e);
                                    } else {
                                        navigator.push(Route::LibraryTab);
                                    }
                                });
                            },
                            "Unarchive"
                        }
                    } else if archive_blocked {
                        // Active exercise currently being recorded: trash is disabled.
                        // Long-press shows a tooltip explaining why.
                        div {
                            class: "tooltip tooltip-left",
                            "data-tip": "In current set — finish first",
                            "data-testid": "archive-blocked-tooltip",
                            button {
                                class: "btn btn-ghost btn-sm btn-circle opacity-30 cursor-not-allowed",
                                "data-testid": "archive-button",
                                "aria-disabled": "true",
                                "aria-label": "Archive (disabled — exercise is currently being recorded)",
                                disabled: true,
                                svg {
                                    xmlns: "http://www.w3.org/2000/svg",
                                    fill: "none",
                                    view_box: "0 0 24 24",
                                    stroke_width: "2",
                                    stroke: "currentColor",
                                    class: "w-5 h-5",
                                    path {
                                        stroke_linecap: "round",
                                        stroke_linejoin: "round",
                                        d: "m14.74 9-.346 9m-4.788 0L9.26 9m9.968-3.21c.342.052.682.107 1.022.166m-1.022-.165L18.16 19.673a2.25 2.25 0 0 1-2.244 2.077H8.084a2.25 2.25 0 0 1-2.244-2.077L4.772 5.79m14.456 0a48.108 48.108 0 0 0-3.478-.397m-12 .562c.34-.059.68-.114 1.022-.165m0 0a48.11 48.11 0 0 1 3.478-.397m7.5 0v-.916c0-1.18-.91-2.164-2.09-2.201a51.964 51.964 0 0 0-3.32 0c-1.18.037-2.09 1.022-2.09 2.201v.916m7.5 0a48.667 48.667 0 0 0-7.5 0"
                                    }
                                }
                            }
                        }
                        // START button still shown (but exercise is active — user may want to navigate back)
                        button {
                            class: "btn btn-primary btn-sm px-4 font-bold shadow-sm",
                            "data-testid": "start-button",
                            onclick: move |_| {
                                navigator.push(Route::WorkoutTab);
                            },
                            "GO TO WORKOUT"
                        }
                    } else {
                        // Active: trash icon to open archive dialog
                        button {
                            class: "btn btn-ghost btn-sm btn-circle",
                            "data-testid": "archive-button",
                            onclick: move |_| show_archive_dialog.set(true),
                            svg {
                                xmlns: "http://www.w3.org/2000/svg",
                                fill: "none",
                                view_box: "0 0 24 24",
                                stroke_width: "2",
                                stroke: "currentColor",
                                class: "w-5 h-5",
                                path {
                                    stroke_linecap: "round",
                                    stroke_linejoin: "round",
                                    d: "m14.74 9-.346 9m-4.788 0L9.26 9m9.968-3.21c.342.052.682.107 1.022.166m-1.022-.165L18.16 19.673a2.25 2.25 0 0 1-2.244 2.077H8.084a2.25 2.25 0 0 1-2.244-2.077L4.772 5.79m14.456 0a48.108 48.108 0 0 0-3.478-.397m-12 .562c.34-.059.68-.114 1.022-.165m0 0a48.11 48.11 0 0 1 3.478-.397m7.5 0v-.916c0-1.18-.91-2.164-2.09-2.201a51.964 51.964 0 0 0-3.32 0c-1.18.037-2.09 1.022-2.09 2.201v.916m7.5 0a48.667 48.667 0 0 0-7.5 0"
                                }
                            }
                        }
                        // START button
                        button {
                            class: "btn btn-primary btn-sm px-4 font-bold shadow-sm",
                            "data-testid": "start-button",
                            onclick: move |_| {
                                let state = workout_state;
                                let ex = exercise.clone();
                                spawn(async move {
                                    if let Err(e) = WorkoutStateManager::start_adhoc_plan(&state, &ex).await {
                                        WorkoutStateManager::handle_error(&state, e);
                                    } else {
                                        navigator.push(Route::WorkoutTab);
                                    }
                                });
                            },
                            "START"
                        }
                    }
                }
            }

            // Body
            HistoryView {
                state: workout_state,
                exercise_id: Some(exercise_id)
            }
        }
    }
}

#[component]
fn NotFound(path: Vec<String>) -> Element {
    let nav = use_navigator();
    use_effect(move || {
        nav.replace(Route::WorkoutTab);
    });
    rsx! { div {} }
}

// ── App root ──────────────────────────────────────────────────────────────────

#[derive(Clone, Copy)]
pub struct TabNavigationState {
    pub last_workout_route: Signal<Route>,
    pub last_library_route: Signal<Route>,
}

#[component]
pub fn App() -> Element {
    let workout_state = use_context_provider(WorkoutState::new);
    use_context_provider(|| TabNavigationState {
        last_workout_route: Signal::new(Route::WorkoutTab),
        last_library_route: Signal::new(Route::LibraryTab),
    });

    use_effect(move || {
        spawn(async move {
            if let Err(e) = WorkoutStateManager::setup_database(&workout_state).await {
                WorkoutStateManager::handle_error(&workout_state, e);
            }
        });
    });

    // Set data-hydrated attribute after the app is past the initial loading state
    use_effect(move || {
        let state = workout_state.initialization_state();
        if state != InitializationState::NotInitialized
            && state != InitializationState::Initializing
        {
            spawn(async move {
                if let Some(window) = web_sys::window()
                    && let Some(document) = window.document()
                    && let Some(body) = document.body()
                {
                    if let Err(e) = body.set_attribute("data-hydrated", "true") {
                        log::error!("Failed to set data-hydrated attribute: {:?}", e);
                    } else {
                        log::debug!("WASM hydration complete - data-hydrated attribute set");
                    }
                }
            });
        }
    });

    // Trigger background sync exactly once when the database transitions to Ready.
    // Sync is non-blocking: the app is fully usable while sync runs.
    // Sync short-circuits if no credentials are configured (see SyncCredentials::load),
    // so it is safe to run even when sync is not set up or in E2E test environments.
    // A one-shot `sync_attempted` flag ensures the sync fires at most once per
    // app load, preventing the infinite re-trigger loop that occurs when a
    // completion callback resets a guard signal the effect subscribes to.
    //
    // The `#[cfg(not(test))]` guard is needed because `trigger_background_sync`
    // depends on the real HTTP client module which is excluded from test builds.
    #[cfg(not(test))]
    {
        let mut sync_attempted = use_signal(|| false);
        use_effect(move || {
            if workout_state.initialization_state() == InitializationState::Ready
                && !sync_attempted()
            {
                // Skip sync in test mode — the test harness sets __TEST_MODE__
                // and there is no sync server to talk to. Running sync here would
                // attempt a network request that never completes, freezing the
                // page for Playwright.
                let is_fallback = workout_state
                    .file_manager()
                    .map(|fm| fm.is_using_fallback())
                    .unwrap_or(false);
                if is_fallback {
                    return;
                }
                sync_attempted.set(true);
                spawn(async move {
                    js_log("[Sync] App ready — starting background sync");
                    WorkoutStateManager::trigger_background_sync(&workout_state).await;
                    js_log("[Sync] Background sync complete");

                    // Periodic sync every 30 seconds while the app is open.
                    loop {
                        gloo_timers::future::sleep(std::time::Duration::from_secs(30)).await;
                        js_log("[Sync] Periodic sync tick");
                        WorkoutStateManager::trigger_background_sync(&workout_state).await;
                        js_log("[Sync] Periodic sync complete");
                    }
                });
            }
        });
    }

    rsx! {
        div {
            class: "flex flex-col h-[100dvh] bg-base-200",
            header {
                class: "navbar min-h-0 py-1 bg-primary text-primary-content flex-none",
                div {
                    class: "flex-1",
                    h1 {
                        class: "text-lg sm:text-2xl font-bold px-4",
                        "Simple Strength Assistant"
                    }
                }
                div {
                    class: "flex items-center justify-end pr-4 gap-2",
                    SyncStatusIndicator {
                        status: workout_state.sync_status()
                    }
                    button {
                        class: "btn btn-ghost btn-sm btn-circle text-primary-content",
                        "data-testid": "gear-icon-button",
                        "aria-label": "Settings",
                        onclick: move |_| {
                            // Navigate to settings using pushState + popstate event
                            // so the Dioxus router picks up the navigation without a full reload
                            if let Some(window) = web_sys::window() {
                                let history = window.history().unwrap();
                                let _ = history.push_state_with_url(
                                    &wasm_bindgen::JsValue::NULL,
                                    "",
                                    Some("/settings"),
                                );
                                // Dispatch popstate so the Dioxus router reacts to the URL change
                                let event = web_sys::Event::new("popstate").unwrap();
                                let _ = window.dispatch_event(&event);
                            }
                        },
                        svg {
                            xmlns: "http://www.w3.org/2000/svg",
                            fill: "none",
                            view_box: "0 0 24 24",
                            stroke_width: "1.5",
                            stroke: "currentColor",
                            class: "w-6 h-6",
                            path {
                                stroke_linecap: "round",
                                stroke_linejoin: "round",
                                d: "M9.594 3.94c.09-.542.56-.94 1.11-.94h2.593c.55 0 1.02.398 1.11.94l.213 1.281c.063.374.313.686.645.87.074.04.147.083.22.127.325.196.72.257 1.075.124l1.217-.456a1.125 1.125 0 011.37.49l1.296 2.247a1.125 1.125 0 01-.26 1.431l-1.003.827c-.293.241-.438.613-.43.992a7.723 7.723 0 010 .255c-.008.378.137.75.43.991l1.004.827c.424.35.534.955.26 1.43l-1.298 2.247a1.125 1.125 0 01-1.369.491l-1.217-.456c-.355-.133-.75-.072-1.076.124a6.47 6.47 0 01-.22.128c-.331.183-.581.495-.644.869l-.213 1.281c-.09.543-.56.94-1.11.94h-2.594c-.55 0-1.019-.398-1.11-.94l-.213-1.281c-.062-.374-.312-.686-.644-.87a6.52 6.52 0 01-.22-.127c-.325-.196-.72-.257-1.076-.124l-1.217.456a1.125 1.125 0 01-1.369-.49l-1.297-2.247a1.125 1.125 0 01.26-1.431l1.004-.827c.292-.24.437-.613.43-.991a6.932 6.932 0 010-.255c.007-.38-.138-.751-.43-.992l-1.004-.827a1.125 1.125 0 01-.26-1.43l1.297-2.247a1.125 1.125 0 011.37-.491l1.216.456c.356.133.751.072 1.076-.124.072-.044.146-.086.22-.128.332-.183.582-.495.644-.869l.214-1.28z"
                            }
                            path {
                                stroke_linecap: "round",
                                stroke_linejoin: "round",
                                d: "M15 12a3 3 0 11-6 0 3 3 0 016 0z"
                            }
                        }
                    }
                }
            }
            match workout_state.initialization_state() {
                InitializationState::NotInitialized | InitializationState::Initializing => {
                    rsx! {
                        main {
                            class: "flex-1 container mx-auto p-4 flex items-center justify-center",
                            div {
                                class: "text-center",
                                div {
                                    class: "loading loading-spinner loading-lg text-primary"
                                }
                                p {
                                    class: "mt-4 text-lg",
                                    "Initializing database..."
                                }
                            }
                        }
                    }
                }
                InitializationState::SelectingFile => {
                    // Check if mobile and not installed (for PWA banner)
                    let mut show_pwa_banner = use_signal(|| {
                        if let Some(window) = web_sys::window() {
                            // Check if mobile
                            let is_mobile = window.inner_width().unwrap_or(1920.0.into()).as_f64().unwrap_or(1920.0) < 768.0;
                            // Check if already installed
                            let is_installed = js_sys::Reflect::get(&window, &"isPWAInstalled".into())
                                .ok()
                                .and_then(|v| v.as_bool())
                                .unwrap_or(false);
                            // Check if install prompt available
                            let install_available = js_sys::Reflect::get(&window, &"pwaInstallAvailable".into())
                                .ok()
                                .and_then(|v| v.as_bool())
                                .unwrap_or(false);

                            is_mobile && !is_installed && install_available
                        } else {
                            false
                        }
                    });

                    rsx! {
                        main {
                            class: "flex-1 container mx-auto p-4 flex items-center justify-center",
                            div {
                                class: "card bg-base-100 shadow-xl max-w-md",
                                div {
                                    class: "card-body",
                                    h2 {
                                        class: "card-title text-xl mb-2",
                                        "Choose Database Setup"
                                    }

                                    // PWA Install Banner (mobile only)
                                    if show_pwa_banner() {
                                        div {
                                            class: "alert alert-info mb-4",
                                            svg {
                                                xmlns: "http://www.w3.org/2000/svg",
                                                fill: "none",
                                                view_box: "0 0 24 24",
                                                class: "stroke-current shrink-0 w-6 h-6",
                                                path {
                                                    stroke_linecap: "round",
                                                    stroke_linejoin: "round",
                                                    stroke_width: "2",
                                                    d: "M12 18h.01M8 21h8a2 2 0 002-2V5a2 2 0 00-2-2H8a2 2 0 00-2 2v14a2 2 0 002 2z"
                                                }
                                            }
                                            div {
                                                class: "flex-1",
                                                h4 {
                                                    class: "font-bold",
                                                    "Install for Best Mobile Experience"
                                                }
                                                p {
                                                    class: "text-sm",
                                                    "Install this app to your home screen for better file access permissions."
                                                }
                                            }
                                            button {
                                                class: "btn btn-sm btn-primary",
                                                onclick: move |_| {
                                                    if let Some(window) = web_sys::window() {
                                                        let install_fn = js_sys::Reflect::get(&window, &"installPWA".into()).ok();
                                                        if let Some(func) = install_fn.and_then(|f| f.dyn_into::<js_sys::Function>().ok()) {
                                                            let _ = func.call0(&window);
                                                            // Hide banner after attempting install
                                                            show_pwa_banner.set(false);
                                                        }
                                                    }
                                                },
                                                "Install"
                                            }
                                        }
                                    }

                                    p {
                                        class: "text-sm text-gray-600 mb-6",
                                        "Your data will be stored locally on your device and remain completely private."
                                    }

                                    // Create New Database Button
                                    div {
                                        class: "mb-4",
                                        button {
                                            class: "btn btn-primary btn-block justify-start h-auto py-4",
                                            onclick: move |_| {
                                                spawn(async move {
                                                    js_log("[UI] Create New Database clicked");
                                                    let mut file_manager = crate::state::Storage::new();

                                                    match file_manager.create_new_file().await {
                                                        Ok(_) => {
                                                            js_log("[UI] File created, initializing DB...");
                                                            workout_state.set_initialization_state(InitializationState::Initializing);

                                                            let mut database = crate::state::Database::new();
                                                            match database.init(None).await {
                                                                Ok(_) => {
                                                                    js_log("[UI] DB initialized, transitioning to Ready...");
                                                                    workout_state.set_current_session(None);
                                                                    WorkoutStateManager::complete_file_initialization(&workout_state, database, file_manager).await;
                                                                }
                                                                Err(e) => {
                                                                    js_log(&format!("[UI] DB init failed: {}", e));
                                                                    WorkoutStateManager::handle_error(&workout_state, WorkoutError::Database(e));
                                                                }
                                                            }
                                                        }
                                                        Err(e) => {
                                                            js_log(&format!("[UI] create_new_file failed: {}", e));
                                                            WorkoutStateManager::handle_error(&workout_state, WorkoutError::FileSystem(e));
                                                        }
                                                    }
                                                });
                                            },
                                            div {
                                                class: "flex items-start gap-3",
                                                svg {
                                                    xmlns: "http://www.w3.org/2000/svg",
                                                    fill: "none",
                                                    view_box: "0 0 24 24",
                                                    class: "w-6 h-6 flex-shrink-0 mt-1",
                                                    stroke: "currentColor",
                                                    stroke_width: "2",
                                                    path {
                                                        stroke_linecap: "round",
                                                        stroke_linejoin: "round",
                                                        d: "M12 4v16m8-8H4"
                                                    }
                                                }
                                                div {
                                                    class: "text-left",
                                                    div {
                                                        class: "font-bold text-base",
                                                        "Create New Database"
                                                    }
                                                    div {
                                                        class: "text-sm opacity-90 mt-1",
                                                        "Start fresh with an empty workout database"
                                                    }
                                                }
                                            }
                                        }
                                    }

                                    // Open Existing Database Button
                                    div {
                                        button {
                                            class: "btn btn-outline btn-block justify-start h-auto py-4",
                                            onclick: move |_| {
                                                spawn(async move {
                                                    log::debug!("[UI] User clicked open existing database - has user gesture");
                                                    let mut file_manager = crate::state::Storage::new();

                                                    match file_manager.prompt_for_file().await {
                                                        Ok(_) => {
                                                            log::debug!("[UI] File selected successfully");

                                                            // Continue initialization inline
                                                            workout_state.set_initialization_state(InitializationState::Initializing);

                                                            // Read file data if handle exists
                                                            let file_data = if file_manager.has_handle() {
                                                                log::debug!("[UI] Reading file contents...");
                                                                match file_manager.read_file().await {
                                                                    Ok(data) if data.is_empty() => {
                                                                        log::debug!("[UI] File is empty (0 bytes), will create new database");
                                                                        None
                                                                    }
                                                                    Ok(data) => {
                                                                        log::debug!("[UI] Read {} bytes from file, loading existing database", data.len());
                                                                        Some(data)
                                                                    }
                                                                    Err(e) => {
                                                                        log::error!("Failed to read selected file: {}", e);

                                                                        // Clear the handle if it's invalid so it doesn't stay cached
                                                                        if matches!(e, crate::state::FileSystemError::InvalidFormat) {
                                                                            let mut fm_clone = file_manager.clone();
                                                                            spawn(async move {
                                                                                let _ = fm_clone.clear_handle().await;
                                                                            });
                                                                        }

                                                                        WorkoutStateManager::handle_error(&workout_state, WorkoutError::FileSystem(e));
                                                                        return;
                                                                    }
                                                                }
                                                            } else {
                                                                log::debug!("[UI] No file handle, will create new database in memory");
                                                                None
                                                            };

                                                            // Initialize database
                                                            log::debug!("[UI] Initializing database...");
                                                            let mut database = crate::state::Database::new();
                                                            match database.init(file_data).await {
                                                                Ok(_) => {
                                                                    log::debug!("[UI] Database initialized successfully");

                                                                    // Store database and file manager in state, sync exercises, transition to Ready
                                                                    WorkoutStateManager::complete_file_initialization(&workout_state, database, file_manager).await;
                                                                }
                                                                Err(e) => {
                                                                    log::error!("Database initialization failed: {}", e);
                                                                    WorkoutStateManager::handle_error(&workout_state, WorkoutError::Database(e));
                                                                }
                                                            }
                                                        }
                                                        Err(e) => {
                                                            log::error!("File selection failed: {}", e);
                                                            WorkoutStateManager::handle_error(&workout_state, WorkoutError::FileSystem(e));
                                                        }
                                                    }
                                                });
                                            },
                                            div {
                                                class: "flex items-start gap-3",
                                                svg {
                                                    xmlns: "http://www.w3.org/2000/svg",
                                                    fill: "none",
                                                    view_box: "0 0 24 24",
                                                    class: "w-6 h-6 flex-shrink-0 mt-1",
                                                    stroke: "currentColor",
                                                    stroke_width: "2",
                                                    path {
                                                        stroke_linecap: "round",
                                                        stroke_linejoin: "round",
                                                        d: "M3 7v10a2 2 0 002 2h14a2 2 0 002-2V9a2 2 0 00-2-2h-6l-2-2H5a2 2 0 00-2 2z"
                                                    }
                                                }
                                                div {
                                                    class: "text-left",
                                                    div {
                                                        class: "font-bold text-base",
                                                        "Open Existing Database"
                                                    }
                                                    div {
                                                        class: "text-sm opacity-90 mt-1",
                                                        "Continue with your existing workout data"
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
                InitializationState::Ready => {
                    rsx! {
                        main {
                            class: "flex-1 flex flex-col min-h-0 w-full",
                            Router::<Route> {}
                        }
                    }
                }
                InitializationState::Error => {
                    let error = workout_state.error().unwrap_or(WorkoutError::Database(crate::state::DatabaseError::NotInitialized));
                    let error_info = parse_error_for_ui(&error);

                    // Check if we have a handle but need permission
                    let has_handle = workout_state.file_manager().map(|fm| fm.has_handle()).unwrap_or(false);
                    let is_permission_error = matches!(error, WorkoutError::FileSystem(crate::state::FileSystemError::PermissionDenied));

                    rsx! {
                        main {
                            class: "flex-1 container mx-auto p-4 flex items-center justify-center",
                            div {
                                class: "card bg-base-100 shadow-xl max-w-md",
                                div {
                                    class: "card-body",
                                    div {
                                        class: "alert alert-error mb-4",
                                        svg {
                                            xmlns: "http://www.w3.org/2000/svg",
                                            class: "stroke-current shrink-0 h-6 w-6",
                                            fill: "none",
                                            view_box: "0 0 24 24",
                                            path {
                                                stroke_linecap: "round",
                                                stroke_linejoin: "round",
                                                stroke_width: "2",
                                                d: "M10 14l2-2m0 0l2-2m-2 2l-2-2m2 2l2 2m7-2a9 9 0 11-18 0 9 9 0 0118 0z"
                                            }
                                        }
                                        div {
                                            h3 {
                                                class: "font-bold",
                                                {error_info.title}
                                            }
                                            p {
                                                class: "text-sm mt-2",
                                                {error_info.message}
                                            }
                                            if let Some(tip) = error_info.recovery_tip {
                                                p {
                                                    class: "text-sm mt-3 flex items-start gap-2",
                                                    span {
                                                        class: "text-base",
                                                        "💡"
                                                    }
                                                    span { {tip} }
                                                }
                                            }
                                        }
                                    }
                                    div {
                                        class: "card-actions justify-end",
                                        button {
                                            class: "btn btn-primary",
                                            onclick: move |_| {
                                                spawn(async move {
                                                    if has_handle && is_permission_error {
                                                        log::debug!("[UI] Re-requesting permission for existing handle...");
                                                        if let Some(fm) = workout_state.file_manager() {
                                                            if let Err(e) = fm.request_permission().await {
                                                                log::error!("[UI] Permission re-request failed: {:?}", e);
                                                                WorkoutStateManager::handle_error(&workout_state, WorkoutError::FileSystem(e));
                                                                return;
                                                            }
                                                            log::debug!("[UI] Permission granted! Retrying initialization...");
                                                        }
                                                    }

                                                    // Reset error state
                                                    workout_state.set_error(None);
                                                    workout_state.set_initialization_state(InitializationState::NotInitialized);
                                                    // Retry initialization
                                                    if let Err(e) = WorkoutStateManager::setup_database(&workout_state).await {
                                                        WorkoutStateManager::handle_error(&workout_state, e);
                                                    }
                                                });
                                            },
                                            {
                                                if has_handle && is_permission_error {
                                                    "Grant Permission"
                                                } else {
                                                    error_info.retry_label.as_str()
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
            {render_debug_panel()}
        }
    }
}

#[cfg(debug_assertions)]
fn render_debug_panel() -> Element {
    rsx! { DebugPanel {} }
}

#[cfg(not(debug_assertions))]
fn render_debug_panel() -> Element {
    rsx! {}
}

#[component]
pub fn ActiveSession(state: WorkoutState, session: crate::state::WorkoutSession) -> Element {
    let session_clone = session.clone();
    let session_for_display = session_clone.clone();
    let mut reps_input = use_signal(|| session.predicted.reps as f64);
    let mut rpe_input = use_signal(|| session.predicted.rpe as f64);
    let mut weight_input = use_signal(|| session.predicted.weight.map(|w| w as f64).unwrap_or(0.0));

    // Sync inputs when session or predicted changes (e.g., after logging a set or starting a new session)
    let mut last_session_id = use_signal(|| session.session_id.clone());
    let mut last_predicted = use_signal(|| session.predicted);

    if *last_session_id.peek() != session.session_id || *last_predicted.peek() != session.predicted
    {
        last_session_id.set(session.session_id.clone());
        last_predicted.set(session.predicted);
        reps_input.set(session.predicted.reps as f64);
        rpe_input.set(session.predicted.rpe as f64);
        weight_input.set(session.predicted.weight.map(|w| w as f64).unwrap_or(0.0));
    }

    let state_for_log = state;
    let session_for_log = session_clone.clone();
    let log_set = move |_| {
        let session = &session_for_log;
        let reps = reps_input() as u32;
        let rpe = rpe_input() as f32;
        let weight = if session.predicted.weight.is_some() {
            Some(weight_input() as f32)
        } else {
            None
        };

        let set = CompletedSet {
            set_number: (session.completed_sets.len() + 1) as u32,
            reps,
            rpe,
            set_type: if let Some(w) = weight {
                SetType::Weighted { weight: w }
            } else {
                SetType::Bodyweight
            },
        };

        let state_clone = state_for_log;
        spawn(async move {
            if let Err(e) = WorkoutStateManager::log_set(&state_clone, set).await {
                WorkoutStateManager::handle_error(&state_clone, e);
            }
        });
    };

    let navigator = use_navigator();
    let history_exercise_id = session_for_display.exercise.id.clone().unwrap_or_default();
    let mut show_action_menu = use_signal(|| false);
    let mut show_complete_confirm = use_signal(|| false);
    let mut show_discard_confirm = use_signal(|| false);

    rsx! {
        div {
            class: "max-w-md mx-auto space-y-2",

            // Input Section
            div {
                class: "card bg-base-100 shadow-xl",
                div {
                    class: "card-body p-2 sm:p-6",
                    div {
                        class: "flex flex-col gap-2 items-stretch w-full",

                        // Weight Input (compact: inline header + tape measure)
                        if let SetTypeConfig::Weighted { min_weight, increment } = session_for_display.exercise.set_type_config {
                            div {
                                class: "form-control w-full",
                                // Row 1: [−10] Weight 80kg [+10]
                                div {
                                    class: "flex items-center justify-between w-full px-1",
                                    button {
                                        "data-testid": "weight-step-down",
                                        class: "btn btn-circle btn-sm glass border border-error/30 hover:border-error text-error transition-all",
                                        onclick: {
                                            let min = min_weight as f64;
                                            move |_| {
                                                let new_val = (weight_input() - 10.0).clamp(min, 500.0);
                                                if (new_val - weight_input()).abs() > 0.001 {
                                                    weight_input.set(new_val);
                                                }
                                            }
                                        },
                                        "−10"
                                    }
                                    div {
                                        class: "flex items-baseline gap-2",
                                        span {
                                            class: "text-sm font-semibold text-base-content/60 uppercase",
                                            "data-testid": "weight-label",
                                            "Weight"
                                        }
                                        span {
                                            class: "text-2xl font-black text-primary",
                                            "data-testid": "weight-readout",
                                            "{crate::format::fmt_weight(weight_input())} kg"
                                        }
                                    }
                                    button {
                                        "data-testid": "weight-step-up",
                                        class: "btn btn-circle btn-sm glass border border-success/30 hover:border-success text-success transition-all",
                                        onclick: {
                                            let min = min_weight as f64;
                                            move |_| {
                                                let new_val = (weight_input() + 10.0).clamp(min, 500.0);
                                                if (new_val - weight_input()).abs() > 0.001 {
                                                    weight_input.set(new_val);
                                                }
                                            }
                                        },
                                        "+10"
                                    }
                                }
                                // Row 2: TapeMeasure
                                TapeMeasure {
                                    value: weight_input(),
                                    min: min_weight as f64,
                                    max: 500.0,
                                    step: increment as f64,
                                    on_change: move |val| weight_input.set(val)
                                }
                            }
                        }

                        // Reps Input (compact: inline header + tape measure)
                        div {
                            class: "form-control w-full",
                            // Row 1: [−1] Reps 5 [+5]
                            div {
                                class: "flex items-center justify-between w-full px-1",
                                button {
                                    "data-testid": "reps-step-down",
                                    class: "btn btn-circle btn-sm glass border border-error/30 hover:border-error text-error transition-all",
                                    onclick: move |_| {
                                        let new_val = (reps_input() - 5.0).clamp(1.0, 100.0);
                                        if (new_val - reps_input()).abs() > 0.001 {
                                            reps_input.set(new_val);
                                        }
                                    },
                                    "−5"
                                }
                                div {
                                    class: "flex items-baseline gap-2",
                                    span {
                                        class: "text-sm font-semibold text-base-content/60 uppercase",
                                        "data-testid": "reps-label",
                                        "Reps"
                                    }
                                    span {
                                        class: "text-2xl font-black text-primary",
                                        "data-testid": "reps-readout",
                                        "{reps_input}"
                                    }
                                }
                                button {
                                    "data-testid": "reps-step-up",
                                    class: "btn btn-circle btn-sm glass border border-success/30 hover:border-success text-success transition-all",
                                    onclick: move |_| {
                                        let new_val = (reps_input() + 5.0).clamp(1.0, 100.0);
                                        if (new_val - reps_input()).abs() > 0.001 {
                                            reps_input.set(new_val);
                                        }
                                    },
                                    "+5"
                                }
                            }
                            // Row 2: TapeMeasure
                            TapeMeasure {
                                value: reps_input(),
                                min: 1.0,
                                max: 100.0,
                                step: 1.0,
                                on_change: move |val| reps_input.set(val)
                            }
                        }

                        // RPE Input (compact: header with description + slider)
                        div {
                            class: "form-control w-full",
                            // Row 1: RPE value description
                            div {
                                class: "flex items-baseline justify-center gap-2 px-1",
                                span {
                                    class: "text-sm font-semibold text-base-content/60 uppercase",
                                    "data-testid": "rpe-label",
                                    "RPE"
                                }
                                span {
                                    class: "text-2xl font-black text-primary",
                                    "data-testid": "rpe-readout",
                                    "{rpe_input:.1}"
                                }
                                span {
                                    class: "text-sm font-medium text-base-content/50",
                                    "data-testid": "rpe-description",
                                    {crate::domain::rpe::rpe_description(rpe_input())}
                                }
                            }
                            // Row 2: RPESlider (value shown in header above)
                            RPESlider {
                                value: rpe_input(),
                                on_change: move |val| rpe_input.set(val),
                                hide_value: true
                            }
                        }
                    }

                    // Log Set Button + Action Menu Trigger
                    div {
                        class: "mt-2 flex gap-2",
                        button {
                            class: "btn btn-primary flex-1 h-12 text-lg font-black shadow-lg",
                            onclick: log_set,
                            "LOG SET"
                        }
                        button {
                            class: "btn btn-ghost w-12 h-12",
                            "aria-label": "More actions",
                            "data-testid": "action-menu-trigger",
                            onclick: move |_| show_action_menu.set(true),
                            // Vertical ellipsis icon — larger size with filled discs so it's
                            // clearly visible inside its 48px touch target on mobile.
                            // Stroke is disabled because the path is closed discs; combining
                            // fill and stroke would double-paint and inflate the visual radius.
                            svg {
                                xmlns: "http://www.w3.org/2000/svg",
                                fill: "currentColor",
                                view_box: "0 0 24 24",
                                stroke: "none",
                                class: "w-8 h-8",
                                path {
                                    d: "M12 8a2 2 0 1 1 0-4 2 2 0 0 1 0 4Zm0 6a2 2 0 1 1 0-4 2 2 0 0 1 0 4Zm0 6a2 2 0 1 1 0-4 2 2 0 0 1 0 4Z"
                                }
                            }
                        }
                    }
                }
            }

            // Action Menu Bottom Sheet
            if show_action_menu() {
                {
                    let exercise_id_for_menu = history_exercise_id.clone();
                    rsx! {
                        BottomSheet {
                            items: vec![
                                BottomSheetItem {
                                    label: "View History".to_string(),
                                    icon: None,
                                    variant: BottomSheetVariant::Default,
                                    testid: None,
                                },
                                BottomSheetItem {
                                    label: String::new(),
                                    icon: None,
                                    variant: BottomSheetVariant::Divider,
                                    testid: None,
                                },
                                BottomSheetItem {
                                    label: "Complete Workout".to_string(),
                                    icon: None,
                                    variant: BottomSheetVariant::Default,
                                    testid: Some("complete-workout-menu-item".to_string()),
                                },
                                BottomSheetItem {
                                    label: "Discard Workout".to_string(),
                                    icon: None,
                                    variant: BottomSheetVariant::Danger,
                                    testid: Some("discard-workout-menu-item".to_string()),
                                },
                                BottomSheetItem {
                                    label: "Cancel".to_string(),
                                    icon: None,
                                    variant: BottomSheetVariant::Default,
                                    testid: None,
                                },
                            ],
                            on_select: move |idx: usize| {
                                show_action_menu.set(false);
                                match idx {
                                    // View History
                                    0 => {
                                        navigator.push(Route::WorkoutHistoryExercise {
                                            exercise_id: exercise_id_for_menu.clone(),
                                        });
                                    }
                                    // Complete Workout
                                    2 => {
                                        show_complete_confirm.set(true);
                                    }
                                    // Discard Workout
                                    3 => {
                                        show_discard_confirm.set(true);
                                    }
                                    // Cancel (4) or Divider (1) — no-op
                                    _ => {}
                                }
                            },
                            on_dismiss: move |_| show_action_menu.set(false),
                        }
                    }
                }
            }

            // Complete Workout Confirmation Dialog
            if show_complete_confirm() {
                ConfirmationDialog {
                    title: "Complete this workout?".to_string(),
                    body: "End this workout? Your recorded sets will be saved.".to_string(),
                    confirm_label: "Complete".to_string(),
                    cancel_label: "Cancel".to_string(),
                    on_confirm: move |_| {
                        show_complete_confirm.set(false);
                        spawn(async move {
                            if let Err(e) = WorkoutStateManager::end_plan(&state).await {
                                log::warn!("Failed to complete workout: {}", e);
                            }
                        });
                    },
                    on_cancel: move |_| show_complete_confirm.set(false),
                }
            }

            // Discard Workout Confirmation Dialog
            if show_discard_confirm() {
                ConfirmationDialog {
                    title: "Discard this workout?".to_string(),
                    body: "Discard this workout? All sets recorded in this session will be permanently deleted.".to_string(),
                    confirm_label: "Discard".to_string(),
                    cancel_label: "Cancel".to_string(),
                    variant: ConfirmVariant::Danger,
                    on_confirm: move |_| {
                        show_discard_confirm.set(false);
                        spawn(async move {
                            if let Err(e) = WorkoutStateManager::discard_plan(&state).await {
                                log::warn!("Failed to discard plan: {}", e);
                            }
                        });
                    },
                    on_cancel: move |_| {
                        show_discard_confirm.set(false);
                    },
                }
            }

        }
    }
}
