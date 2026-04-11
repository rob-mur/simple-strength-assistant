use crate::components::conflict_resolution::ConflictResolutionScreen;
use crate::components::data_management::DataManagementPanel;
#[cfg(debug_assertions)]
use crate::components::debug_panel::DebugPanel;
use crate::components::exercise_form::ExerciseForm;
use crate::components::history_view::HistoryView;
use crate::components::library_view::LibraryView;
use crate::components::previous_sessions::PreviousSessions;
use crate::components::rpe_slider::RPESlider;
use crate::components::step_controls::StepControls;
use crate::components::sync_status_indicator::SyncStatusIndicator;
use crate::components::tab_bar::{Tab, TabBar};
use crate::components::tape_measure::TapeMeasure;
use crate::components::workout_view::WorkoutView;
use crate::models::{CompletedSet, SetType, SetTypeConfig};
use crate::state::{InitializationState, WorkoutError, WorkoutState, WorkoutStateManager};
use dioxus::prelude::*;
use wasm_bindgen::JsCast;

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
    WorkoutHistoryExercise { exercise_id: i64 },
    #[route("/library")]
    LibraryTab,
    #[route("/library/:exercise_id")]
    LibraryExercise { exercise_id: i64 },
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

    let active_tab = match &route {
        Route::WorkoutTab | Route::WorkoutHistory | Route::WorkoutHistoryExercise { .. } => {
            Tab::Workout
        }
        Route::LibraryTab | Route::LibraryExercise { .. } => Tab::Library,
        Route::SettingsTab => Tab::Settings,
        _ => Tab::Workout,
    };

    // AC #4: Update last seen route for the current tab whenever it changes.
    // We do this in the component body so it runs on every render of Shell
    // (which re-renders whenever the route changes).
    let current_route = route.clone();
    match current_route.clone() {
        Route::WorkoutTab | Route::WorkoutHistory | Route::WorkoutHistoryExercise { .. } => {
            if *navigation_state.last_workout_route.peek() != current_route {
                navigation_state.last_workout_route.set(current_route);
            }
        }
        Route::LibraryTab | Route::LibraryExercise { .. } => {
            if *navigation_state.last_library_route.peek() != current_route {
                navigation_state.last_library_route.set(current_route);
            }
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
                    class: "container mx-auto p-4",
                    {storage_mode_banner}
                    {save_error_banner}
                    Outlet::<Route> {}
                }
            }
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
                        Tab::Settings => {
                            navigator.push(Route::SettingsTab);
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
fn WorkoutHistoryExercise(exercise_id: i64) -> Element {
    let state = consume_context::<WorkoutState>();
    let navigator = use_navigator();
    rsx! { HistoryView { state, exercise_id: Some(exercise_id), on_back: move |_| { navigator.push(Route::WorkoutHistory); } } }
}

#[component]
fn LibraryTab() -> Element {
    rsx! { LibraryView {} }
}

#[component]
fn SettingsTab() -> Element {
    let state = consume_context::<WorkoutState>();
    rsx! {
        div {
            class: "max-w-md mx-auto py-6",
            h2 { class: "text-xl font-black uppercase tracking-tight mb-6", "Settings" }
            div {
                class: "card bg-base-100 shadow-xl",
                div {
                    class: "card-body",
                    h3 { class: "card-title text-base font-bold mb-2", "Data Management" }
                    p {
                        class: "text-sm text-base-content/60 mb-4",
                        "Export your workout database for backup or transfer to another device. Import a previously exported database to restore your data."
                    }
                    DataManagementPanel { state }
                }
            }
        }
    }
}

#[component]
fn LibraryExercise(exercise_id: i64) -> Element {
    let workout_state = consume_context::<WorkoutState>();
    let navigator = use_navigator();
    let mut show_edit_form = use_signal(|| false);

    let exercises = workout_state.exercises();
    let exercise = exercises
        .iter()
        .find(|e| e.id == Some(exercise_id))
        .cloned();

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

    rsx! {
        div {
            class: "max-w-md mx-auto",
            "data-testid": "exercise-detail-view",
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
                    button {
                        class: "btn btn-primary btn-sm px-4 font-bold shadow-sm",
                        "data-testid": "start-button",
                        onclick: move |_| {
                            let state = workout_state;
                            let ex = exercise.clone();
                            spawn(async move {
                                if let Err(e) = WorkoutStateManager::start_session(&state, ex).await {
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

    // Trigger background sync once the database transitions to Ready.
    // Sync is non-blocking: the app is fully usable while sync runs.
    // Sync short-circuits if no credentials are configured (see SyncCredentials::load),
    // so it is safe to run even when sync is not set up or in E2E test environments.
    // A `sync_in_progress` guard prevents duplicate sync cycles if the
    // effect re-fires (e.g. due to re-renders or state transitions).
    //
    // The `#[cfg(not(test))]` guard is needed because `trigger_background_sync`
    // depends on the real HTTP client module which is excluded from test builds.
    #[cfg(not(test))]
    {
        let mut sync_in_progress = use_signal(|| false);
        use_effect(move || {
            if workout_state.initialization_state() == InitializationState::Ready
                && !sync_in_progress()
            {
                sync_in_progress.set(true);
                spawn(async move {
                    log::debug!("[Sync] App ready — starting background sync");
                    WorkoutStateManager::trigger_background_sync(&workout_state).await;
                    sync_in_progress.set(false);
                });
            }
        });
    }

    rsx! {
        div {
            class: "flex flex-col h-[100dvh] bg-base-200",
            header {
                class: "navbar bg-primary text-primary-content flex-none",
                div {
                    class: "flex-1",
                    h1 {
                        class: "text-2xl font-bold px-4",
                        "Simple Strength Assistant"
                    }
                }
                div {
                    class: "flex items-center justify-end pr-4",
                    SyncStatusIndicator {
                        status: workout_state.sync_status()
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
                                                    log::debug!("[UI] User clicked create new database - has user gesture");
                                                    let mut file_manager = crate::state::Storage::new();

                                                    match file_manager.create_new_file().await {
                                                        Ok(_) => {
                                                            log::debug!("[UI] New database file created successfully");

                                                            // Continue initialization inline
                                                            workout_state.set_initialization_state(InitializationState::Initializing);

                                                            // New file is always empty
                                                            log::debug!("[UI] Initializing new database...");
                                                            let mut database = crate::state::Database::new();
                                                            match database.init(None).await {
                                                                Ok(_) => {
                                                                    log::debug!("[UI] New database initialized successfully");

                                                                    // Clear any existing session state to ensure clean slate
                                                                    log::debug!("[UI] Clearing current_session to ensure fresh start");
                                                                    workout_state.set_current_session(None);
                                                                    log::debug!("[UI] current_session cleared, should show StartSessionView");

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
                                                            log::error!("Failed to create new database: {}", e);
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
                    if workout_state.has_pending_conflicts() {
                        rsx! {
                            ConflictResolutionScreen { state: workout_state }
                        }
                    } else {
                        rsx! {
                            main {
                                class: "flex-1 flex flex-col min-h-0 w-full",
                                Router::<Route> {}
                            }
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
    let mut last_session_id = use_signal(|| session.session_id);
    let mut last_predicted = use_signal(|| session.predicted);

    if *last_session_id.peek() != session.session_id || *last_predicted.peek() != session.predicted
    {
        last_session_id.set(session.session_id);
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
    let history_exercise_id = session_for_display.exercise.id.unwrap_or(0);

    rsx! {
        div {
            class: "max-w-md mx-auto space-y-8 pb-10",
            // Exercise Header
            div {
                class: "card bg-base-100 shadow-xl border-t-4 border-primary",
                div {
                    class: "card-body p-6",
                    div {
                        class: "flex justify-between items-center",
                        h2 {
                            class: "card-title text-2xl font-black",
                            {session_for_display.exercise.name.clone()}
                        }
                        div {
                            class: "flex items-center gap-2",
                            div {
                                class: "badge badge-primary badge-lg font-bold",
                                "Set {session_for_display.completed_sets.len() + 1}"
                            }
                            // History icon — AC #7
                            button {
                                class: "btn btn-ghost btn-sm btn-circle",
                                "aria-label": "View exercise history",
                                "data-testid": "history-icon-btn",
                                onclick: move |_| {
                                    navigator.push(Route::WorkoutHistoryExercise { exercise_id: history_exercise_id });
                                },
                                svg {
                                    xmlns: "http://www.w3.org/2000/svg",
                                    fill: "none",
                                    view_box: "0 0 24 24",
                                    stroke_width: "1.5",
                                    stroke: "currentColor",
                                    class: "w-5 h-5",
                                    path {
                                        stroke_linecap: "round",
                                        stroke_linejoin: "round",
                                        d: "M12 6v6h4.5m4.5 0a9 9 0 1 1-18 0 9 9 0 0 1 18 0Z"
                                    }
                                }
                            }
                        }
                    }
                }
            }

            // Input Section
            div {
                class: "card bg-base-100 shadow-xl",
                div {
                    class: "card-body p-4 sm:p-6",
                    div {
                        class: "flex flex-col gap-12 items-stretch w-full",

                        // Weight Input
                        if let SetTypeConfig::Weighted { min_weight, increment } = session_for_display.exercise.set_type_config {
                            div {
                                class: "form-control w-full",
                                label {
                                    class: "label justify-center mb-2",
                                    span {
                                        class: "label-text font-black text-xl text-base-content/70 uppercase tracking-widest",
                                        "Weight"
                                    }
                                }
                                TapeMeasure {
                                    value: weight_input(),
                                    min: min_weight as f64,
                                    max: 500.0,
                                    step: increment as f64,
                                    on_change: move |val| weight_input.set(val)
                                }
                                div {
                                    class: "text-center text-5xl font-black text-primary my-4",
                                    "{crate::format::fmt_weight(weight_input())} kg"
                                }
                                StepControls {
                                    value: weight_input(),
                                    steps: vec![-10.0, 10.0],
                                    min: min_weight as f64,
                                    max: 500.0,
                                    on_change: move |val| weight_input.set(val)
                                }
                            }
                        }

                        // Reps Input
                        div {
                            class: "form-control w-full",
                            label {
                                class: "label justify-center mb-2",
                                span {
                                    class: "label-text font-black text-xl text-base-content/70 uppercase tracking-widest",
                                    "Reps"
                                }
                            }
                            TapeMeasure {
                                value: reps_input(),
                                min: 1.0,
                                max: 100.0,
                                step: 1.0,
                                on_change: move |val| reps_input.set(val)
                            }
                            div {
                                class: "text-center text-5xl font-black text-primary my-4",
                                "{reps_input} reps"
                            }
                            StepControls {
                                value: reps_input(),
                                steps: vec![-1.0, 5.0],
                                min: 1.0,
                                max: 100.0,
                                on_change: move |val| reps_input.set(val)
                            }
                        }

                        // RPE Input
                        div {
                            class: "form-control w-full",
                            label {
                                class: "label justify-center mb-2",
                                span {
                                    class: "label-text font-black text-xl text-base-content/70 uppercase tracking-widest",
                                    "Intensity (RPE)"
                                }
                            }
                            RPESlider {
                                value: rpe_input(),
                                on_change: move |val| rpe_input.set(val)
                            }
                        }
                    }

                    // Log Set Button
                    div {
                        class: "mt-12",
                        button {
                            class: "btn btn-primary btn-lg btn-block h-24 text-2xl font-black shadow-lg",
                            onclick: log_set,
                            "LOG SET"
                        }
                    }
                }
            }

            // Today's Sets Section
            if !session_for_display.completed_sets.is_empty() {
                div {
                    class: "collapse collapse-arrow bg-base-100 shadow-lg border border-base-300",
                    "data-testid": "todays-sets-section",
                    input { r#type: "checkbox", checked: true },
                    div {
                        class: "collapse-title text-xl font-bold",
                        {
                            let n = session_for_display.completed_sets.len();
                            let unit = if n == 1 { "set" } else { "sets" };
                            format!("Today's Sets ({n} {unit})")
                        }
                    }
                    div {
                        class: "collapse-content p-0",
                        div {
                            class: "overflow-x-auto",
                            table {
                                class: "table table-zebra w-full",
                                thead {
                                    tr {
                                        th { "Set" }
                                        if session_for_display.predicted.weight.is_some() {
                                            th { "Weight" }
                                        }
                                        th { "Reps" }
                                        th { "RPE" }
                                    }
                                }
                                tbody {
                                    for set in session_for_display.completed_sets.iter() {
                                        tr {
                                            td { class: "font-bold", "{set.set_number}" }
                                            if let SetType::Weighted { weight } = set.set_type {
                                                td { "{crate::format::fmt_weight(weight)} kg" }
                                            }
                                            td { "{set.reps}" }
                                            td { "{set.rpe}" }
                                        }
                                    }
                                }
                            }
                        }
                    }
                }
            }

            // Previous Sessions — collapsible history for this exercise (AC #1–#6)
            if let Some(eid) = session_for_display.exercise.id {
                PreviousSessions {
                    key: "{eid}",
                    state: state,
                    exercise_id: eid,
                    completed_sets_count: session_for_display.completed_sets.len(),
                }
            }

        }
    }
}
