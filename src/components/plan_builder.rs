use crate::models::{ExerciseMetadata, SetTypeConfig};
use crate::state::{WorkoutState, WorkoutStateManager};
use dioxus::prelude::*;

#[component]
pub fn PlanBuilder(state: WorkoutState) -> Element {
    let mut show_exercise_picker = use_signal(|| false);
    let mut search_query = use_signal(String::new);
    let settings = state.settings();

    let plan = state.current_plan();
    let exercises = plan
        .as_ref()
        .map(|p| p.exercises.clone())
        .unwrap_or_default();
    let has_exercises = !exercises.is_empty();

    rsx! {
        div {
            class: "max-w-md mx-auto py-6",
            "data-testid": "plan-builder",

            h2 {
                class: "text-xl font-black uppercase tracking-tight mb-6",
                "Plan Your Workout"
            }

            // Exercise list
            if has_exercises {
                div {
                    class: "grid gap-3 mb-4",
                    for pe in exercises.iter() {
                        {
                            let pe_id = pe.id.clone();
                            let exercise_name = pe.exercise.name.clone();
                            let planned_sets = pe.planned_sets;
                            let pe_id_remove = pe_id.clone();
                            rsx! {
                                div {
                                    key: "{pe_id}",
                                    class: "card bg-base-100 shadow-md border border-base-200",
                                    "data-testid": "plan-exercise-row",
                                    div {
                                        class: "card-body p-4",
                                        div {
                                            class: "flex justify-between items-center",
                                            div {
                                                class: "flex-1 min-w-0",
                                                h3 {
                                                    class: "font-bold text-base truncate",
                                                    "{exercise_name.to_uppercase()}"
                                                }
                                            }
                                            div {
                                                class: "flex items-center gap-2 ml-3",
                                                // Set-count stepper
                                                button {
                                                    class: "btn btn-circle btn-sm btn-ghost",
                                                    "data-testid": "decrement-sets",
                                                    disabled: planned_sets <= 1,
                                                    onclick: {
                                                        let pe_id = pe_id.clone();
                                                        move |_| {
                                                            if planned_sets > 1 {
                                                                let pe_id = pe_id.clone();
                                                                let new_sets = planned_sets - 1;
                                                                spawn(async move {
                                                                    let db = state.database().unwrap();
                                                                    let now = js_sys::Date::now();
                                                                    let _ = db.execute(
                                                                        "UPDATE workout_plan_exercises SET planned_sets = ?, updated_at = ? WHERE id = ?",
                                                                        &[
                                                                            wasm_bindgen::JsValue::from_f64(new_sets as f64),
                                                                            wasm_bindgen::JsValue::from_f64(now),
                                                                            wasm_bindgen::JsValue::from_str(&pe_id),
                                                                        ],
                                                                    ).await;
                                                                    let _ = WorkoutStateManager::resume_active_plan(&state).await;
                                                                });
                                                            }
                                                        }
                                                    },
                                                    "−"
                                                }
                                                span {
                                                    class: "text-lg font-bold w-8 text-center",
                                                    "data-testid": "planned-sets-value",
                                                    "{planned_sets}"
                                                }
                                                button {
                                                    class: "btn btn-circle btn-sm btn-ghost",
                                                    "data-testid": "increment-sets",
                                                    onclick: {
                                                        let pe_id = pe_id.clone();
                                                        move |_| {
                                                            let pe_id = pe_id.clone();
                                                            let new_sets = planned_sets + 1;
                                                            spawn(async move {
                                                                let db = state.database().unwrap();
                                                                let now = js_sys::Date::now();
                                                                let _ = db.execute(
                                                                    "UPDATE workout_plan_exercises SET planned_sets = ?, updated_at = ? WHERE id = ?",
                                                                    &[
                                                                        wasm_bindgen::JsValue::from_f64(new_sets as f64),
                                                                        wasm_bindgen::JsValue::from_f64(now),
                                                                        wasm_bindgen::JsValue::from_str(&pe_id),
                                                                    ],
                                                                ).await;
                                                                let _ = WorkoutStateManager::resume_active_plan(&state).await;
                                                            });
                                                        }
                                                    },
                                                    "+"
                                                }
                                                // Remove button
                                                button {
                                                    class: "btn btn-circle btn-sm btn-ghost text-error ml-1",
                                                    "data-testid": "remove-exercise",
                                                    onclick: move |_| {
                                                        let pe_id = pe_id_remove.clone();
                                                        spawn(async move {
                                                            if let Err(e) = WorkoutStateManager::remove_exercise_from_plan(&state, &pe_id).await {
                                                                log::warn!("Failed to remove exercise: {}", e);
                                                            }
                                                        });
                                                    },
                                                    "✕"
                                                }
                                            }
                                        }
                                    }
                                }
                            }
                        }
                    }
                }
            } else {
                div {
                    class: "text-center py-8 text-base-content/50",
                    "data-testid": "plan-empty-hint",
                    p { "Add exercises to build your workout plan." }
                }
            }

            // Add exercise button
            button {
                class: "btn btn-outline btn-block mb-4",
                "data-testid": "add-exercise-button",
                onclick: move |_| {
                    search_query.set(String::new());
                    show_exercise_picker.set(true);
                },
                "+ Add Exercise"
            }

            // Start Workout button (only when exercises present)
            if has_exercises {
                button {
                    class: "btn btn-primary btn-block btn-lg shadow-lg font-bold",
                    "data-testid": "start-workout-button",
                    onclick: move |_| {
                        spawn(async move {
                            if let Err(e) = WorkoutStateManager::start_plan(&state).await {
                                log::warn!("Failed to start plan: {}", e);
                            }
                        });
                    },
                    "Start Workout"
                }
            }

            // Exercise picker modal
            if show_exercise_picker() {
                ExercisePickerModal {
                    state,
                    search_query,
                    default_planned_sets: settings.default_planned_sets,
                    on_close: move |_| show_exercise_picker.set(false),
                }
            }
        }
    }
}

#[component]
pub fn ExercisePickerModal(
    state: WorkoutState,
    search_query: Signal<String>,
    default_planned_sets: u32,
    on_close: EventHandler<()>,
) -> Element {
    let all_exercises = state.exercises();
    let query = search_query().to_lowercase();
    let filtered: Vec<&ExerciseMetadata> = all_exercises
        .iter()
        .filter(|e| query.is_empty() || e.name.to_lowercase().contains(&query))
        .collect();

    rsx! {
        div {
            class: "fixed inset-0 z-[200] flex items-end sm:items-center justify-center bg-black/60 backdrop-blur-sm",
            "data-testid": "exercise-picker-modal",
            onclick: move |_| on_close.call(()),
            div {
                class: "bg-base-100 rounded-2xl shadow-2xl w-full max-w-md flex flex-col max-h-[90dvh] animate-in fade-in zoom-in duration-200",
                onclick: move |evt| evt.stop_propagation(),

                // Header
                div {
                    class: "p-4 border-b border-base-200 flex justify-between items-center",
                    h3 { class: "text-lg font-bold", "Add Exercise" }
                    button {
                        class: "btn btn-circle btn-ghost btn-sm",
                        "data-testid": "close-picker",
                        onclick: move |_| on_close.call(()),
                        "✕"
                    }
                }

                // Search
                div {
                    class: "p-4 border-b border-base-200",
                    input {
                        r#type: "text",
                        placeholder: "Search exercises...",
                        class: "input input-bordered w-full",
                        "data-testid": "exercise-search",
                        value: "{search_query}",
                        oninput: move |evt| search_query.set(evt.value()),
                    }
                }

                // Exercise list
                div {
                    class: "flex-1 overflow-y-auto p-4",
                    if filtered.is_empty() {
                        p {
                            class: "text-center text-base-content/50 py-4",
                            "No exercises found"
                        }
                    } else {
                        div {
                            class: "grid gap-2",
                            for exercise in filtered {
                                {
                                    let exercise_id = exercise.id.clone().unwrap_or_default();
                                    let exercise_name = exercise.name.clone();
                                    let badge = match &exercise.set_type_config {
                                        SetTypeConfig::Weighted { .. } => ("WEIGHTED", "badge-primary"),
                                        SetTypeConfig::Bodyweight => ("BODYWEIGHT", "badge-secondary"),
                                    };
                                    rsx! {
                                        button {
                                            key: "{exercise_id}",
                                            class: "w-full text-left p-3 rounded-lg hover:bg-base-200 transition-colors flex justify-between items-center",
                                            "data-testid": "exercise-picker-item",
                                            onclick: {
                                                let eid = exercise_id.clone();
                                                move |_| {
                                                    let eid = eid.clone();
                                                    let sets = default_planned_sets;
                                                    let on_close = on_close;
                                                    spawn(async move {
                                                        // Lazily create plan if none exists
                                                        if state.current_plan().is_none()
                                                            && let Err(e) = WorkoutStateManager::create_plan(&state).await
                                                        {
                                                            log::warn!("Failed to create plan: {}", e);
                                                            on_close.call(());
                                                            return;
                                                        }
                                                        if let Err(e) = WorkoutStateManager::add_exercise_to_plan(&state, &eid, sets).await {
                                                            log::warn!("Failed to add exercise to plan: {}", e);
                                                        }
                                                        on_close.call(());
                                                    });
                                                }
                                            },
                                            div {
                                                span { class: "font-bold", "{exercise_name.to_uppercase()}" }
                                                span {
                                                    class: "badge {badge.1} badge-sm font-bold ml-2",
                                                    "{badge.0}"
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
