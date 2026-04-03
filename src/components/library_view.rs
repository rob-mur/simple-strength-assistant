use crate::app::Route;
use crate::components::exercise_form::ExerciseForm;
use crate::models::{ExerciseMetadata, SetTypeConfig};
use crate::state::{WorkoutState, WorkoutStateManager};
use dioxus::prelude::*;

#[derive(Clone, PartialEq)]
pub struct TestSearchQuery(pub String);

#[derive(Clone, PartialEq)]
pub enum FormState {
    Closed,
    New,
    Edit(ExerciseMetadata),
}

#[component]
pub fn LibraryView() -> Element {
    let workout_state = consume_context::<WorkoutState>();
    let navigator = use_navigator();
    // Allow injecting a search query context for easier unit testing
    let test_query = try_consume_context::<TestSearchQuery>();
    let mut search_query = use_signal(|| test_query.map(|t| t.0).unwrap_or_default());
    let mut show_form = use_signal(|| FormState::Closed);
    let filtered_exercises = use_memo(move || {
        let query = search_query().to_lowercase();
        workout_state
            .exercises()
            .iter()
            .filter(|e| e.name.to_lowercase().contains(&query))
            .cloned()
            .collect::<Vec<_>>()
    });

    match show_form() {
        FormState::New | FormState::Edit(_) => {
            let initial_exercise = if let FormState::Edit(e) = show_form() {
                Some(e)
            } else {
                None
            };
            return rsx! {
                div {
                    class: "max-w-2xl mx-auto p-4",
                    ExerciseForm {
                        initial_exercise,
                        on_cancel: move |_| show_form.set(FormState::Closed),
                        on_save: move |exercise| {
                            spawn(async move {
                                if let Err(e) = WorkoutStateManager::save_exercise(&workout_state, exercise).await {
                                    WorkoutStateManager::handle_error(&workout_state, e);
                                }
                                // Ordering dependency: show_form.set(FormState::Closed) only executes after
                                // sync_exercises completes inside save_exercise, avoiding stale ID async races.
                                show_form.set(FormState::Closed);
                            });
                        }
                    }
                }
            };
        }
        FormState::Closed => {}
    }

    rsx! {
        div {
            class: "max-w-2xl mx-auto p-4",
            "data-testid": "library-view",
            div {
                class: "flex justify-between items-center mb-6",
                h2 {
                    class: "text-3xl font-black text-base-content tracking-tighter min-h-8",
                    "LIBRARY"
                }
                button {
                    class: "btn btn-primary btn-circle shadow-lg",
                    onclick: move |_| show_form.set(FormState::New),
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

            if workout_state.exercises().is_empty() {
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
                            onclick: move |_| show_form.set(FormState::New),
                            "Add First Exercise"
                        }
                    }
                }
            } else if filtered_exercises().is_empty() {
                div {
                    class: "text-center py-12",
                    p { class: "text-base-content/50 italic", "No exercises match your search" }
                }
            } else {
                div {
                    class: "grid gap-4",
                    for exercise in filtered_exercises() {
                        div {
                            key: "{exercise.id.unwrap_or(0)}",
                            class: "card bg-base-100 shadow-md hover:shadow-lg transition-all border border-base-200 cursor-pointer",
                            onclick: {
                                let id = exercise.id.unwrap_or(0);
                                move |_| { navigator.push(Route::LibraryExercise { exercise_id: id }); }
                            },
                            div {
                                class: "card-body p-4",
                                div {
                                    class: "flex justify-between items-start",
                                    div {
                                        h3 {
                                    class: "font-black text-xl text-base-content tracking-tight min-h-6",
                                    "{exercise.name.to_uppercase()}"
                                }
                                        div {
                                            class: "flex gap-2 mt-1 items-center",
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
                                    div {
                                        class: "flex gap-2",
                                        button {
                                            class: "btn btn-ghost btn-sm btn-circle",
                                            onclick: {
                                                let e = exercise.clone();
                                                // Note: e must be cloned again here because the onclick handler is an FnMut
                                                // and FormState::Edit takes ownership of the value.
                                                move |evt| {
                                                    evt.stop_propagation();
                                                    show_form.set(FormState::Edit(e.clone()));
                                                }
                                            },
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
                                                    d: "m16.862 4.487 1.687-1.688a1.875 1.875 0 1 1 2.652 2.652L10.582 16.07a4.5 4.5 0 0 1-1.897 1.13L6 18l.8-2.685a4.5 4.5 0 0 1 1.13-1.897l8.932-8.931Zm0 0L19.5 7.125M18 14v4.75A2.25 2.25 0 0 1 15.75 21H5.25A2.25 2.25 0 0 1 3 18.75V8.25A2.25 2.25 0 0 1 5.25 6H10"
                                                }
                                            }
                                        }
                                        button {
                                            class: "btn btn-primary btn-sm px-4 font-bold shadow-sm",
                                            onclick: {
                                                let e = exercise.clone();
                                                move |evt| {
                                                    evt.stop_propagation();
                                                    let e_clone = e.clone();
                                                    spawn(async move {
                                                        if let Err(err) = WorkoutStateManager::start_session(&workout_state, e_clone).await {
                                                            WorkoutStateManager::handle_error(&workout_state, err);
                                                        } else {
                                                            navigator.push(Route::WorkoutTab);
                                                        }
                                                    });
                                                }
                                            },
                                            "START"
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
