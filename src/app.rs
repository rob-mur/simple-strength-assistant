use crate::components::library_view::LibraryView;
use crate::components::rpe_slider::RPESlider;
use crate::components::step_controls::StepControls;
use crate::components::tab_bar::{Tab, TabBar};
use crate::components::tape_measure::TapeMeasure;
use crate::components::workout_view::WorkoutView;
use crate::models::{CompletedSet, SetType, SetTypeConfig};
#[cfg(feature = "test-mode")]
use crate::state::StorageBackend;
use crate::state::{InitializationState, WorkoutError, WorkoutState, WorkoutStateManager};
use dioxus::prelude::*;
use gloo_storage::{LocalStorage, Storage};
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

pub(crate) const ACTIVE_TAB_KEY: &str = "active_tab";

#[component]
pub fn App() -> Element {
    let workout_state = use_context_provider(WorkoutState::new);
    let mut active_tab = use_context_provider(|| {
        Signal::new(LocalStorage::get(ACTIVE_TAB_KEY).unwrap_or(Tab::Workout))
    });

    use_effect(move || {
        let _ = LocalStorage::set(ACTIVE_TAB_KEY, active_tab());
    });

    use_effect(move || {
        spawn(async move {
            if let Err(e) = WorkoutStateManager::setup_database(&workout_state).await {
                WorkoutStateManager::handle_error(&workout_state, e);
            }
        });
    });

    // Set data-hydrated attribute after WASM initialization
    use_effect(move || {
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
    });

    rsx! {
        div {
            class: "flex flex-col min-h-screen bg-base-200",
            header {
                class: "navbar bg-primary text-primary-content",
                div {
                    class: "flex-1",
                    h1 {
                        class: "text-2xl font-bold px-4",
                        "Simple Strength Assistant"
                    }
                }
            }
            main {
                class: "flex-1 container mx-auto p-4",
                match workout_state.initialization_state() {
                    InitializationState::NotInitialized | InitializationState::Initializing => {
                        rsx! {
                            div {
                                class: "flex items-center justify-center h-full",
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
                            div {
                                class: "flex items-center justify-center h-full",
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

                                                                        // Store database and file manager in state
                                                                        workout_state.set_database(database);
                                                                        workout_state.set_file_manager(file_manager);

                                                                        workout_state.set_initialization_state(InitializationState::Ready);

                                                                        log::debug!("[UI] Setup complete! State is now Ready");
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

                                                                        // Store database and file manager in state
                                                                        workout_state.set_database(database);
                                                                        workout_state.set_file_manager(file_manager);

                                                                        workout_state.set_initialization_state(InitializationState::Ready);

                                                                        log::debug!("[UI] Setup complete! State is now Ready");
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
                                            h4 {
                                                class: "font-bold",
                                                "Browser Storage Mode"
                                            }
                                            p {
                                                class: "text-sm",
                                                "Your data is stored in browser LocalStorage. This works offline but won't sync across devices or browsers."
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
                                    h4 {
                                        class: "font-bold",
                                        "Sync Warning"
                                    }
                                    p {
                                        class: "text-sm",
                                        "{err_msg}"
                                    }
                                }
                            }
                        });

                        rsx! {
                            div {
                                class: "pb-safe-nav",
                                {storage_mode_banner}
                                {save_error_banner}
                                match active_tab() {
                                    Tab::Workout => rsx! { WorkoutView { state: workout_state } },
                                    Tab::Library => rsx! { LibraryView {} },
                                }
                            }
                            TabBar {
                                active_tab: active_tab(),
                                on_change: move |tab| {
                                    active_tab.set(tab);
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
                            div {
                                class: "flex items-center justify-center h-full",
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
            }
        }
    }
}

#[component]
pub fn ActiveSession(state: WorkoutState, session: crate::state::WorkoutSession) -> Element {
    let session_clone = session.clone();
    let session_for_display = session_clone.clone();
    let mut reps_input = use_signal(|| session.predicted.reps as f64);
    let mut rpe_input = use_signal(|| session.predicted.rpe as f64);
    let mut weight_input = use_signal(|| session.predicted.weight.map(|w| w as f64).unwrap_or(0.0));

    // Sync inputs when session or predicted changes (e.g., after logging a set or starting a new session)
    let mut last_exercise_id = use_signal(|| session.exercise.id);
    let mut last_predicted = use_signal(|| session.predicted);

    if *last_exercise_id.peek() != session.exercise.id
        || *last_predicted.peek() != session.predicted
    {
        last_exercise_id.set(session.exercise.id);
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

    let state_for_complete = state;
    let complete_session = move |_| {
        let state_clone = state_for_complete;
        spawn(async move {
            if let Err(e) = WorkoutStateManager::complete_session(&state_clone).await {
                WorkoutStateManager::handle_error(&state_clone, e);
            }
        });
    };

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
                            class: "badge badge-primary badge-lg font-bold",
                            "Set {session_for_display.completed_sets.len() + 1}"
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

            // History Section
            if !session_for_display.completed_sets.is_empty() {
                div {
                    class: "collapse collapse-arrow bg-base-100 shadow-lg border border-base-300",
                    input { r#type: "checkbox", checked: true },
                    div {
                        class: "collapse-title text-xl font-bold",
                        "History ({session_for_display.completed_sets.len()} sets)"
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
                                    for set in session_for_display.completed_sets.iter().rev() {
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

            // Finish Session Button
            div {
                class: "flex justify-center pt-4",
                button {
                    class: "btn btn-ghost btn-sm opacity-50 hover:opacity-100",
                    onclick: complete_session,
                    "Finish Workout Session"
                }
            }
        }
    }
}
