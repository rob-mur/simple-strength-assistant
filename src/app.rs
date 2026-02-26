use crate::models::{CompletedSet, ExerciseMetadata, SetType, SetTypeConfig};
use crate::state::{InitializationState, WorkoutState, WorkoutStateManager};
use dioxus::prelude::*;
use wasm_bindgen::JsCast;

struct ErrorInfo {
    title: String,
    message: String,
    recovery_tip: Option<String>,
    retry_label: String,
}

fn parse_error_for_ui(error_msg: &str) -> ErrorInfo {
    let error_lower = error_msg.to_lowercase();

    if error_lower.contains("not a valid sqlite database") || error_lower.contains("invalid format")
    {
        ErrorInfo {
            title: "Invalid File Format".to_string(),
            message: "The selected file is not a valid SQLite database.".to_string(),
            recovery_tip: Some(
                "Please select a .sqlite or .db file, or create a new database file.".to_string(),
            ),
            retry_label: "Select Different File".to_string(),
        }
    } else if error_lower.contains("permission denied") || error_lower.contains("notallowederror") {
        ErrorInfo {
            title: "Permission Denied".to_string(),
            message: "File access permission was not granted.".to_string(),
            recovery_tip: Some(
                "Grant permission to access the file, or use browser storage instead.".to_string(),
            ),
            retry_label: "Grant Permission".to_string(),
        }
    } else if error_lower.contains("user cancelled") {
        ErrorInfo {
            title: "File Selection Cancelled".to_string(),
            message: "No database file was selected.".to_string(),
            recovery_tip: Some(
                "Click below to select where to store your workout data.".to_string(),
            ),
            retry_label: "Select File".to_string(),
        }
    } else if error_lower.contains("file is too large") || error_lower.contains("filetoolarge") {
        ErrorInfo {
            title: "File Too Large".to_string(),
            message: "The selected database file exceeds the 100 MB limit.".to_string(),
            recovery_tip: Some(
                "Try selecting a smaller file or export your data to start fresh.".to_string(),
            ),
            retry_label: "Select Different File".to_string(),
        }
    } else if error_lower.contains("failed to initialize database") {
        ErrorInfo {
            title: "Database Initialization Failed".to_string(),
            message: "Could not set up the database. The file may be corrupted.".to_string(),
            recovery_tip: Some(
                "Try selecting a different file or creating a new database.".to_string(),
            ),
            retry_label: "Try Again".to_string(),
        }
    } else {
        ErrorInfo {
            title: "Initialization Error".to_string(),
            message: error_msg.to_string(),
            recovery_tip: Some("Check your browser console for details and try again.".to_string()),
            retry_label: "Retry".to_string(),
        }
    }
}

#[component]
pub fn App() -> Element {
    let workout_state = use_context_provider(WorkoutState::new);

    use_effect(move || {
        spawn(async move {
            if let Err(e) = WorkoutStateManager::setup_database(&workout_state).await {
                WorkoutStateManager::handle_error(&workout_state, e);
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
                                                        web_sys::console::log_1(&"[UI] User clicked create new database - has user gesture".into());
                                                        let mut file_manager = crate::state::FileSystemManager::new();

                                                        match file_manager.create_new_file().await {
                                                            Ok(_) => {
                                                                web_sys::console::log_1(&"[UI] New database file created successfully".into());

                                                                // Continue initialization inline
                                                                workout_state.set_initialization_state(InitializationState::Initializing);

                                                                // New file is always empty
                                                                web_sys::console::log_1(&"[UI] Initializing new database...".into());
                                                                let mut database = crate::state::Database::new();
                                                                match database.init(None).await {
                                                                    Ok(_) => {
                                                                        web_sys::console::log_1(&"[UI] New database initialized successfully".into());

                                                                        // Store database and file manager in state
                                                                        workout_state.set_database(database);
                                                                        workout_state.set_file_manager(file_manager);
                                                                        workout_state.set_initialization_state(InitializationState::Ready);

                                                                        web_sys::console::log_1(&"[UI] Setup complete! State is now Ready".into());
                                                                    }
                                                                    Err(e) => {
                                                                        let error_msg = format!("Database initialization failed: {}", e);
                                                                        web_sys::console::error_1(&error_msg.clone().into());
                                                                        WorkoutStateManager::handle_error(&workout_state, error_msg);
                                                                    }
                                                                }
                                                            }
                                                            Err(e) => {
                                                                let error_msg = format!("Failed to create new database: {}", e);
                                                                web_sys::console::error_1(&error_msg.clone().into());
                                                                WorkoutStateManager::handle_error(&workout_state, error_msg);
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
                                                        web_sys::console::log_1(&"[UI] User clicked open existing database - has user gesture".into());
                                                        let mut file_manager = crate::state::FileSystemManager::new();

                                                        match file_manager.prompt_for_file().await {
                                                            Ok(_) => {
                                                                web_sys::console::log_1(&"[UI] File selected successfully".into());

                                                                // Continue initialization inline
                                                                workout_state.set_initialization_state(InitializationState::Initializing);

                                                                // Read file data if handle exists
                                                                let file_data = if file_manager.has_handle() {
                                                                    web_sys::console::log_1(&"[UI] Reading file contents...".into());
                                                                    match file_manager.read_file().await {
                                                                        Ok(data) if data.is_empty() => {
                                                                            web_sys::console::log_1(&"[UI] File is empty (0 bytes), will create new database".into());
                                                                            None
                                                                        }
                                                                        Ok(data) => {
                                                                            web_sys::console::log_1(&format!("[UI] Read {} bytes from file, loading existing database", data.len()).into());
                                                                            Some(data)
                                                                        }
                                                                        Err(e) => {
                                                                            let error_msg = format!("Failed to read selected file: {}", e);
                                                                            web_sys::console::error_1(&error_msg.clone().into());
                                                                            WorkoutStateManager::handle_error(&workout_state, error_msg);
                                                                            return;
                                                                        }
                                                                    }
                                                                } else {
                                                                    web_sys::console::log_1(&"[UI] No file handle, will create new database in memory".into());
                                                                    None
                                                                };

                                                                // Initialize database
                                                                web_sys::console::log_1(&"[UI] Initializing database...".into());
                                                                let mut database = crate::state::Database::new();
                                                                match database.init(file_data).await {
                                                                    Ok(_) => {
                                                                        web_sys::console::log_1(&"[UI] Database initialized successfully".into());

                                                                        // Store database and file manager in state
                                                                        workout_state.set_database(database);
                                                                        workout_state.set_file_manager(file_manager);
                                                                        workout_state.set_initialization_state(InitializationState::Ready);

                                                                        web_sys::console::log_1(&"[UI] Setup complete! State is now Ready".into());
                                                                    }
                                                                    Err(e) => {
                                                                        let error_msg = format!("Database initialization failed: {}", e);
                                                                        web_sys::console::error_1(&error_msg.clone().into());
                                                                        WorkoutStateManager::handle_error(&workout_state, error_msg);
                                                                    }
                                                                }
                                                            }
                                                            Err(e) => {
                                                                let error_msg = format!("File selection failed: {}", e);
                                                                web_sys::console::error_1(&error_msg.clone().into());
                                                                WorkoutStateManager::handle_error(&workout_state, error_msg);
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

                        rsx! {
                            div {
                                {storage_mode_banner}
                                WorkoutInterface { state: workout_state.clone() }
                            }
                        }
                    }
                    InitializationState::Error => {
                        let error_msg = workout_state.error_message().unwrap_or_else(|| "Unknown error occurred".to_string());
                        let error_info = parse_error_for_ui(&error_msg);

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
                                                        // Reset error state
                                                        workout_state.set_error_message(None);
                                                        workout_state.set_initialization_state(InitializationState::NotInitialized);
                                                        // Retry initialization
                                                        if let Err(e) = WorkoutStateManager::setup_database(&workout_state).await {
                                                            WorkoutStateManager::handle_error(&workout_state, e);
                                                        }
                                                    });
                                                },
                                                {error_info.retry_label}
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
fn WorkoutInterface(state: WorkoutState) -> Element {
    let current_session = state.current_session();

    if let Some(session) = current_session {
        rsx! {
            ActiveSession { state: state.clone(), session }
        }
    } else {
        rsx! {
            StartSessionView { state: state.clone() }
        }
    }
}

const MAX_EXERCISE_NAME_LENGTH: usize = 100;

fn validate_exercise_name(name: &str) -> Result<(), String> {
    if name.trim().is_empty() {
        return Err("Exercise name cannot be empty".to_string());
    }
    if name.len() > MAX_EXERCISE_NAME_LENGTH {
        return Err(format!(
            "Exercise name must be {} characters or less",
            MAX_EXERCISE_NAME_LENGTH
        ));
    }
    Ok(())
}

#[component]
fn StartSessionView(state: WorkoutState) -> Element {
    let mut exercise_name = use_signal(|| "Bench Press".to_string());
    let mut is_weighted = use_signal(|| true);
    let mut min_weight = use_signal(|| 45.0);
    let mut increment = use_signal(|| 5.0);
    let mut validation_error = use_signal(|| None::<String>);

    let start_session = move |_| {
        let name = exercise_name().trim().to_string();

        if let Err(e) = validate_exercise_name(&name) {
            validation_error.set(Some(e));
            return;
        }

        validation_error.set(None);

        let exercise = ExerciseMetadata {
            name,
            set_type_config: if is_weighted() {
                SetTypeConfig::Weighted {
                    min_weight: min_weight(),
                    increment: increment(),
                }
            } else {
                SetTypeConfig::Bodyweight
            },
        };

        let state_clone = state.clone();
        spawn(async move {
            if let Err(e) = WorkoutStateManager::start_session(&state_clone, exercise).await {
                WorkoutStateManager::handle_error(&state_clone, e);
            }
        });
    };

    rsx! {
        div {
            class: "max-w-2xl mx-auto",
            div {
                class: "card bg-base-100 shadow-xl",
                div {
                    class: "card-body",
                    h2 {
                        class: "card-title text-2xl mb-4",
                        "Start New Workout Session"
                    }
                    div {
                        class: "form-control",
                        label {
                            class: "label",
                            span {
                                class: "label-text",
                                "Exercise Name"
                            }
                        }
                        input {
                            class: if validation_error().is_some() {
                                "input input-bordered input-error"
                            } else {
                                "input input-bordered"
                            },
                            r#type: "text",
                            value: "{exercise_name}",
                            maxlength: MAX_EXERCISE_NAME_LENGTH,
                            oninput: move |e| {
                                exercise_name.set(e.value());
                                validation_error.set(None);
                            }
                        }
                        if let Some(error) = validation_error() {
                            label {
                                class: "label",
                                span {
                                    class: "label-text-alt text-error",
                                    "{error}"
                                }
                            }
                        }
                    }
                    div {
                        class: "form-control mt-4",
                        label {
                            class: "label cursor-pointer",
                            span {
                                class: "label-text",
                                "Weighted Exercise"
                            }
                            input {
                                class: "checkbox",
                                r#type: "checkbox",
                                checked: is_weighted(),
                                oninput: move |e| is_weighted.set(e.checked())
                            }
                        }
                    }
                    if is_weighted() {
                        div {
                            class: "grid grid-cols-2 gap-4 mt-4",
                            div {
                                class: "form-control",
                                label {
                                    class: "label",
                                    span {
                                        class: "label-text",
                                        "Starting Weight"
                                    }
                                }
                                input {
                                    class: "input input-bordered",
                                    r#type: "number",
                                    value: "{min_weight}",
                                    oninput: move |e| {
                                        if let Ok(val) = e.value().parse::<f32>() {
                                            min_weight.set(val);
                                        }
                                    }
                                }
                            }
                            div {
                                class: "form-control",
                                label {
                                    class: "label",
                                    span {
                                        class: "label-text",
                                        "Weight Increment"
                                    }
                                }
                                input {
                                    class: "input input-bordered",
                                    r#type: "number",
                                    value: "{increment}",
                                    oninput: move |e| {
                                        if let Ok(val) = e.value().parse::<f32>() {
                                            increment.set(val);
                                        }
                                    }
                                }
                            }
                        }
                    }
                    div {
                        class: "card-actions justify-end mt-6",
                        button {
                            class: "btn btn-primary",
                            onclick: start_session,
                            "Start Session"
                        }
                    }
                }
            }
        }
    }
}

#[component]
fn ActiveSession(state: WorkoutState, session: crate::state::WorkoutSession) -> Element {
    let session_clone = session.clone();
    let session_for_display = session_clone.clone();
    let mut reps_input = use_signal(|| session.predicted.reps.to_string());
    let mut rpe_input = use_signal(|| session.predicted.rpe.to_string());
    let mut weight_input = use_signal(|| {
        session
            .predicted
            .weight
            .map(|w| w.to_string())
            .unwrap_or_default()
    });

    let state_for_log = state.clone();
    let session_for_log = session_clone.clone();
    let log_set = move |_| {
        let session = &session_for_log;
        let reps = reps_input().parse::<u32>().unwrap_or(0);
        let rpe = rpe_input().parse::<f32>().unwrap_or(0.0);
        let weight = if session.predicted.weight.is_some() {
            weight_input().parse::<f32>().ok()
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

        let state_clone = state_for_log.clone();
        spawn(async move {
            if let Err(e) = WorkoutStateManager::log_set(&state_clone, set).await {
                WorkoutStateManager::handle_error(&state_clone, e);
            }
        });
    };

    let state_for_complete = state.clone();
    let complete_session = move |_| {
        let state_clone = state_for_complete.clone();
        spawn(async move {
            if let Err(e) = WorkoutStateManager::complete_session(&state_clone).await {
                WorkoutStateManager::handle_error(&state_clone, e);
            }
        });
    };

    rsx! {
        div {
            class: "max-w-4xl mx-auto space-y-6",
            div {
                class: "card bg-base-100 shadow-xl",
                div {
                    class: "card-body",
                    h2 {
                        class: "card-title text-2xl",
                        {session_for_display.exercise.name.clone()}
                    }
                    p {
                        class: "text-gray-600",
                        "Sets completed: {session_for_display.completed_sets.len()}"
                    }
                }
            }
            div {
                class: "card bg-base-100 shadow-xl",
                div {
                    class: "card-body",
                    h3 {
                        class: "card-title",
                        "Log New Set"
                    }
                    div {
                        class: "grid grid-cols-3 gap-4 mt-4",
                        if session_for_display.predicted.weight.is_some() {
                            div {
                                class: "form-control",
                                label {
                                    class: "label",
                                    span {
                                        class: "label-text",
                                        "Weight"
                                    }
                                }
                                input {
                                    class: "input input-bordered",
                                    r#type: "number",
                                    value: "{weight_input}",
                                    oninput: move |e| weight_input.set(e.value())
                                }
                            }
                        }
                        div {
                            class: "form-control",
                            label {
                                class: "label",
                                span {
                                    class: "label-text",
                                    "Reps"
                                }
                            }
                            input {
                                class: "input input-bordered",
                                r#type: "number",
                                value: "{reps_input}",
                                oninput: move |e| reps_input.set(e.value())
                            }
                        }
                        div {
                            class: "form-control",
                            label {
                                class: "label",
                                span {
                                    class: "label-text",
                                    "RPE"
                                }
                            }
                            input {
                                class: "input input-bordered",
                                r#type: "number",
                                step: "0.5",
                                value: "{rpe_input}",
                                oninput: move |e| rpe_input.set(e.value())
                            }
                        }
                    }
                    div {
                        class: "card-actions justify-end mt-6",
                        button {
                            class: "btn btn-primary",
                            onclick: log_set,
                            "Log Set"
                        }
                    }
                }
            }
            if !session_for_display.completed_sets.is_empty() {
                div {
                    class: "card bg-base-100 shadow-xl",
                    div {
                        class: "card-body",
                        h3 {
                            class: "card-title",
                            "Completed Sets"
                        }
                        div {
                            class: "overflow-x-auto",
                            table {
                                class: "table table-zebra",
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
                                            td { "{set.set_number}" }
                                            if let SetType::Weighted { weight } = set.set_type {
                                                td { "{weight}" }
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
            div {
                class: "flex justify-end",
                button {
                    class: "btn btn-success",
                    onclick: complete_session,
                    "Complete Session"
                }
            }
        }
    }
}
