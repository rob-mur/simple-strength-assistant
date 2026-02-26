use crate::models::{CompletedSet, ExerciseMetadata, SetType, SetTypeConfig};
use crate::state::{InitializationState, WorkoutState, WorkoutStateManager};
use dioxus::prelude::*;

#[component]
pub fn App() -> Element {
    let workout_state = use_context_provider(WorkoutState::new);
    let _init_state = use_signal(|| workout_state.initialization_state());

    let state_for_init = workout_state.clone();
    use_effect(move || {
        let state = state_for_init.clone();
        spawn(async move {
            if let Err(e) = WorkoutStateManager::setup_database(&state).await {
                WorkoutStateManager::handle_error(&state, e);
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
                        let state_for_select = workout_state.clone();
                        rsx! {
                            div {
                                class: "flex items-center justify-center h-full",
                                div {
                                    class: "card bg-base-100 shadow-xl max-w-md",
                                    div {
                                        class: "card-body",
                                        h2 {
                                            class: "card-title",
                                            "Select Database File"
                                        }
                                        p {
                                            "Please select or create a database file to store your workout data."
                                        }
                                        p {
                                            class: "text-sm text-gray-600 mt-2",
                                            "Your data will be stored locally on your device and remain completely private."
                                        }
                                        div {
                                            class: "card-actions justify-end mt-4",
                                            button {
                                                class: "btn btn-primary",
                                                onclick: move |_| {
                                                    let state_clone = state_for_select.clone();
                                                    spawn(async move {
                                                        web_sys::console::log_1(&"[UI] User clicked file selection button - has user gesture".into());
                                                        let mut file_manager = crate::state::FileSystemManager::new();

                                                        match file_manager.prompt_for_file().await {
                                                            Ok(_) => {
                                                                web_sys::console::log_1(&"[UI] File selected successfully".into());

                                                                // Continue initialization inline - file_manager already has handle from prompt_for_file
                                                                state_clone.set_initialization_state(InitializationState::Initializing);

                                                                // Read file data if handle exists
                                                                let file_data = if file_manager.has_handle() {
                                                                    web_sys::console::log_1(&"[UI] Reading existing file...".into());
                                                                    match file_manager.read_file().await {
                                                                        Ok(data) if !data.is_empty() => {
                                                                            web_sys::console::log_1(&format!("[UI] Read {} bytes from file", data.len()).into());
                                                                            Some(data)
                                                                        }
                                                                        Ok(_) => {
                                                                            web_sys::console::log_1(&"[UI] File is empty, creating new database".into());
                                                                            None
                                                                        }
                                                                        Err(e) => {
                                                                            web_sys::console::warn_1(&format!("[UI] Failed to read file: {}", e).into());
                                                                            None
                                                                        }
                                                                    }
                                                                } else {
                                                                    None
                                                                };

                                                                // Initialize database
                                                                web_sys::console::log_1(&"[UI] Initializing database...".into());
                                                                let mut database = crate::state::Database::new();
                                                                match database.init(file_data).await {
                                                                    Ok(_) => {
                                                                        web_sys::console::log_1(&"[UI] Database initialized successfully".into());

                                                                        // Store database and file manager in state
                                                                        state_clone.set_database(database);
                                                                        state_clone.set_file_manager(file_manager);
                                                                        state_clone.set_initialization_state(InitializationState::Ready);

                                                                        web_sys::console::log_1(&"[UI] Setup complete! State is now Ready".into());
                                                                    }
                                                                    Err(e) => {
                                                                        let error_msg = format!("Database initialization failed: {}", e);
                                                                        web_sys::console::error_1(&error_msg.clone().into());
                                                                        crate::state::WorkoutStateManager::handle_error(&state_clone, error_msg);
                                                                    }
                                                                }
                                                            }
                                                            Err(e) => {
                                                                let error_msg = format!("File selection failed: {}", e);
                                                                web_sys::console::error_1(&error_msg.clone().into());
                                                                crate::state::WorkoutStateManager::handle_error(&state_clone, error_msg);
                                                            }
                                                        }
                                                    });
                                                },
                                                "Select Database Location"
                                            }
                                        }
                                    }
                                }
                            }
                        }
                    }
                    InitializationState::Ready => {
                        rsx! {
                            WorkoutInterface { state: workout_state.clone() }
                        }
                    }
                    InitializationState::Error => {
                        let state_for_retry = workout_state.clone();
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
                                                    "Initialization Error"
                                                }
                                                p {
                                                    class: "text-sm mt-2",
                                                    {workout_state.error_message().unwrap_or_else(|| "Unknown error occurred".to_string())}
                                                }
                                            }
                                        }
                                        div {
                                            class: "card-actions justify-end",
                                            button {
                                                class: "btn btn-primary",
                                                onclick: move |_| {
                                                    let state = state_for_retry.clone();
                                                    spawn(async move {
                                                        // Reset error state
                                                        state.set_error_message(None);
                                                        state.set_initialization_state(InitializationState::NotInitialized);
                                                        // Retry initialization
                                                        if let Err(e) = WorkoutStateManager::setup_database(&state).await {
                                                            WorkoutStateManager::handle_error(&state, e);
                                                        }
                                                    });
                                                },
                                                "Retry"
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
