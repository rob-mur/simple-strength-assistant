use crate::app::Route;
use crate::components::exercise_form::ExerciseForm;
use crate::models::{ExerciseMetadata, SetTypeConfig};
use crate::state::{WorkoutState, WorkoutStateManager};
use dioxus::prelude::*;

#[derive(Clone, PartialEq)]
pub struct TestSearchQuery(pub String);

#[component]
pub fn LibraryView() -> Element {
    let workout_state = consume_context::<WorkoutState>();
    let navigator = use_navigator();
    // Allow injecting a search query context for easier unit testing
    let test_query = try_consume_context::<TestSearchQuery>();
    let mut search_query = use_signal(|| test_query.map(|t| t.0).unwrap_or_default());
    let mut show_new_form = use_signal(|| false);
    let mut show_archived = use_signal(|| false);
    // Holds archived exercises fetched on demand when the toggle is ON.
    let mut archived_exercises: Signal<Vec<ExerciseMetadata>> = use_signal(Vec::new);

    // When show_archived flips to true, fetch archived exercises from DB.
    use_effect(move || {
        if show_archived() {
            spawn(async move {
                match WorkoutStateManager::fetch_archived_exercises(&workout_state).await {
                    Ok(exercises) => archived_exercises.set(exercises),
                    Err(e) => log::warn!("Failed to fetch archived exercises: {}", e),
                }
            });
        } else {
            // Clear stale archived list when toggling back to active view.
            archived_exercises.set(Vec::new());
        }
    });

    let filtered_exercises = use_memo(move || {
        let query = search_query().to_lowercase();
        let source: Vec<ExerciseMetadata> = if show_archived() {
            archived_exercises()
        } else {
            workout_state.exercises()
        };
        source
            .into_iter()
            .filter(|e| e.name.to_lowercase().contains(&query))
            .collect::<Vec<_>>()
    });

    if show_new_form() {
        return rsx! {
            div {
                class: "max-w-2xl mx-auto p-4",
                ExerciseForm {
                    initial_exercise: None,
                    on_cancel: move |_| show_new_form.set(false),
                    on_save: move |exercise| {
                        spawn(async move {
                            if let Err(e) = WorkoutStateManager::save_exercise(&workout_state, exercise).await {
                                WorkoutStateManager::handle_error(&workout_state, e);
                            }
                            // Ordering dependency: show_new_form.set(false) only executes after
                            // sync_exercises completes inside save_exercise, avoiding stale ID async races.
                            show_new_form.set(false);
                        });
                    }
                }
            }
        };
    }

    // Determine empty-state copy based on toggle
    let is_truly_empty = workout_state.exercises().is_empty() && !show_archived();

    rsx! {
        div {
            class: "relative max-w-2xl mx-auto p-4",
            "data-testid": "library-view",

            // Header: title + "Show archived" toggle
            div {
                class: "flex justify-between items-center mb-6",
                h2 {
                    class: "text-3xl font-black text-base-content tracking-tighter min-h-8",
                    "LIBRARY"
                }
                label {
                    class: "flex items-center gap-2 cursor-pointer",
                    "data-testid": "show-archived-label",
                    span {
                        class: "text-sm font-semibold text-base-content/70",
                        "Show archived"
                    }
                    input {
                        r#type: "checkbox",
                        class: "toggle toggle-primary toggle-sm",
                        "data-testid": "show-archived-toggle",
                        checked: show_archived(),
                        onchange: move |evt| {
                            show_archived.set(evt.checked());
                        }
                    }
                }
            }

            // Search Bar
            div {
                class: "form-control w-full mb-6",
                div {
                    class: "relative w-full",
                    div {
                        class: "absolute inset-y-0 left-0 pl-3 flex items-center pointer-events-none",
                        svg {
                            class: "h-5 w-5 text-base-content/50",
                            fill: "none",
                            view_box: "0 0 24 24",
                            stroke: "currentColor",
                            path {
                                stroke_linecap: "round",
                                stroke_linejoin: "round",
                                stroke_width: "2",
                                d: "M21 21l-6-6m2-5a7 7 0 11-14 0 7 7 0 0114 0z"
                            }
                        }
                    }
                    input {
                        r#type: "text",
                        placeholder: "Search exercises...",
                        class: "input input-bordered w-full pl-10 bg-base-100 shadow-sm focus:shadow-md transition-shadow",
                        value: "{search_query}",
                        oninput: move |evt| search_query.set(evt.value())
                    }
                }
            }

            if is_truly_empty {
                div {
                    class: "card bg-base-100 shadow-xl py-12 text-center",
                    div {
                        class: "card-body items-center",
                        div {
                            class: "bg-base-200 p-6 rounded-full mb-4",
                            svg {
                                xmlns: "http://www.w3.org/2000/svg",
                                fill: "none",
                                view_box: "0 0 24 24",
                                stroke_width: "1.5",
                                stroke: "currentColor",
                                class: "w-12 h-12 text-base-content/30",
                                path {
                                    stroke_linecap: "round",
                                    stroke_linejoin: "round",
                                    d: "M12 6.042A8.967 8.967 0 006 3.75c-1.052 0-2.062.18-3 .512v14.25A8.987 8.987 0 016 18c2.305 0 4.408.867 6 2.292m0-14.25a8.966 8.966 0 016-2.292c1.052 0 2.062.18 3 .512v14.25A8.987 8.987 0 0018 18c-2.305 0-4.408.867-6 2.292m0-14.25v14.25"
                                }
                            }
                        }
                        h3 { class: "text-xl font-bold", "Your library is empty" }
                        p {
                            class: "text-base-content/60 max-w-xs mx-auto mt-2",
                            "Add your first exercise to start tracking your strength journey."
                        }
                        button {
                            class: "btn btn-primary mt-6",
                            onclick: move |_| show_new_form.set(true),
                            "Add First Exercise"
                        }
                    }
                }
            } else if filtered_exercises().is_empty() {
                div {
                    class: "text-center py-12",
                    p {
                        class: "text-base-content/50 italic",
                        "data-testid": "empty-archived-state",
                        if show_archived() {
                            "No archived exercises"
                        } else {
                            "No exercises match your search"
                        }
                    }
                }
            } else {
                div {
                    class: "grid gap-4",
                    for exercise in filtered_exercises() {
                        div {
                            key: "{exercise.id.clone().unwrap_or_default()}",
                            class: "card bg-base-100 shadow-md hover:shadow-lg transition-all border border-base-200 cursor-pointer",
                            role: "link",
                            tabindex: "0",
                            "aria-label": "View {exercise.name} details",
                            onclick: {
                                let id = exercise.id.clone().unwrap_or_default();
                                move |_| { navigator.push(Route::LibraryExercise { exercise_id: id.clone() }); }
                            },
                            onkeydown: {
                                let id = exercise.id.clone().unwrap_or_default();
                                move |evt: KeyboardEvent| {
                                    if evt.key() == Key::Enter {
                                        navigator.push(Route::LibraryExercise { exercise_id: id.clone() });
                                    }
                                }
                            },
                            div {
                                class: "card-body p-4",
                                div {
                                    class: "flex justify-between items-start",
                                    div {
                                        h3 {
                                    class: if show_archived() { "font-black text-xl text-base-content/40 tracking-tight min-h-6" } else { "font-black text-xl text-base-content tracking-tight min-h-6" },
                                    "data-testid": if show_archived() { "archived-exercise-name" } else { "exercise-name" },
                                    "{exercise.name.to_uppercase()}"
                                }
                                        div {
                                            class: "flex gap-2 mt-1 items-center",
                                            if show_archived() {
                                                span {
                                                    class: "badge badge-ghost badge-sm font-bold",
                                                    "data-testid": "archived-badge",
                                                    "ARCHIVED"
                                                }
                                            } else {
                                                match exercise.set_type_config {
                                                    SetTypeConfig::Weighted { min_weight, increment } => rsx! {
                                                        span { class: "badge badge-primary badge-sm font-bold", "WEIGHTED" }
                                                        span { class: "text-xs font-bold text-base-content/50", "START: {crate::format::fmt_weight(min_weight)}kg (+{crate::format::fmt_weight(increment)}kg)" }
                                                    },
                                                    SetTypeConfig::Bodyweight => rsx! {
                                                        span { class: "badge badge-secondary badge-sm font-bold", "BODYWEIGHT" }
                                                    },
                                                }
                                            }
                                        }
                                    }
                                    div {
                                        class: "flex gap-2 items-center",
                                        // Action buttons: only for active (non-archived) exercises.
                                        // Edit is accessible exclusively from the detail view.
                                        if !show_archived() {
                                            if workout_state.current_plan().is_some() {
                                                {
                                                    let eid = exercise.id.clone().unwrap_or_default();
                                                    let ex_for_session = exercise.clone();
                                                    let default_sets = workout_state.settings().default_planned_sets;
                                                    rsx! {
                                                        button {
                                                            class: "btn btn-secondary btn-sm px-4 font-bold shadow-sm",
                                                            "data-testid": "add-to-workout-btn",
                                                            onclick: move |evt| {
                                                                evt.stop_propagation();
                                                                let eid = eid.clone();
                                                                let ex = ex_for_session.clone();
                                                                spawn(async move {
                                                                    if let Err(e) = WorkoutStateManager::add_exercise_to_plan(&workout_state, &eid, default_sets).await {
                                                                        log::warn!("Failed to add exercise to plan: {}", e);
                                                                    } else {
                                                                        // Start a session on the newly added exercise so
                                                                        // the user lands on the recording UI for it.
                                                                        if let Err(e) = WorkoutStateManager::start_session(&workout_state, ex).await {
                                                                            log::warn!("Failed to start session: {}", e);
                                                                        }
                                                                        navigator.push(Route::WorkoutTab);
                                                                    }
                                                                });
                                                            },
                                                            "Add to workout"
                                                        }
                                                    }
                                                }
                                            } else {
                                                {
                                                    let e = exercise.clone();
                                                    rsx! {
                                                        button {
                                                            class: "btn btn-primary btn-sm px-4 font-bold shadow-sm",
                                                            onclick: move |evt| {
                                                                evt.stop_propagation();
                                                                let e_clone = e.clone();
                                                                spawn(async move {
                                                                    if let Err(err) = WorkoutStateManager::start_adhoc_plan(&workout_state, &e_clone).await {
                                                                        WorkoutStateManager::handle_error(&workout_state, err);
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
                                        }
                                        span {
                                            "data-testid": "card-nav-chevron",
                                            "aria-hidden": "true",
                                            class: "text-base-content/30",
                                            svg {
                                                xmlns: "http://www.w3.org/2000/svg",
                                                fill: "none",
                                                view_box: "0 0 24 24",
                                                stroke_width: "2",
                                                stroke: "currentColor",
                                                class: "w-4 h-4",
                                                path {
                                                    stroke_linecap: "round",
                                                    stroke_linejoin: "round",
                                                    d: "m8.25 4.5 7.5 7.5-7.5 7.5"
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

            // FAB: only shown on Library list (not on archived view — archived exercises
            // cannot be started). Hidden when form is open (handled by early return above).
            if !show_archived() {
                button {
                    class: "btn btn-primary btn-circle shadow-lg fixed bottom-20 right-4 z-[60]",
                    "data-testid": "add-exercise-fab",
                    "aria-label": "Add Exercise",
                    onclick: move |_| show_new_form.set(true),
                    svg {
                        xmlns: "http://www.w3.org/2000/svg",
                        fill: "none",
                        view_box: "0 0 24 24",
                        stroke_width: "3",
                        stroke: "currentColor",
                        class: "w-6 h-6",
                        path {
                            stroke_linecap: "round",
                            stroke_linejoin: "round",
                            d: "M12 4.5v15m7.5-7.5h-15"
                        }
                    }
                }
            }
        }
    }
}
