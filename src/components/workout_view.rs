use crate::app::ActiveSession;
use crate::components::exercise_tab_strip::ExerciseTabStrip;
use crate::components::plan_builder::{ExercisePickerModal, PlanBuilder};
use crate::state::{WorkoutState, WorkoutStateManager};
use dioxus::prelude::*;

#[component]
pub fn WorkoutView(state: WorkoutState) -> Element {
    let current_session = state.current_session();
    let current_plan = state.current_plan();
    let mut active_tab_index = use_signal(|| 0usize);
    let mut set_counts = use_signal(Vec::<u32>::new);
    let mut prompt_dismissed = use_signal(|| false);
    let mut show_add_exercise = use_signal(|| false);
    let mut add_search_query = use_signal(String::new);

    match (&current_plan, &current_session) {
        // Plan started — show tab strip + recorder + end workout
        (Some(plan), _) if plan.started_at.is_some() && plan.ended_at.is_none() => {
            let exercises = plan.exercises.clone();
            let plan_started_at = plan.started_at.unwrap_or(0.0);
            let active_idx = active_tab_index();

            // Fetch completed set counts per exercise
            {
                let exercise_ids: Vec<String> = exercises
                    .iter()
                    .filter_map(|pe| pe.exercise.id.clone())
                    .collect();
                let _current_counts = set_counts();

                use_effect(move || {
                    let eids = exercise_ids.clone();
                    let since = plan_started_at;
                    spawn(async move {
                        if let Some(db) = state.database() {
                            match db.count_sets_since(&eids, since).await {
                                Ok(counts) => {
                                    let mut result: Vec<u32> = vec![0; eids.len()];
                                    for (eid, cnt) in &counts {
                                        if let Some(pos) = eids.iter().position(|id| id == eid) {
                                            result[pos] = *cnt;
                                        }
                                    }
                                    if let Some(session) = state.current_session()
                                        && let Some(sid) = &session.exercise.id
                                        && let Some(pos) = eids.iter().position(|id| id == sid)
                                    {
                                        result[pos] =
                                            result[pos].max(result[pos].saturating_add(
                                                session.completed_sets.len() as u32,
                                            ));
                                    }
                                    set_counts.set(result);
                                }
                                Err(e) => log::warn!("Failed to count sets: {}", e),
                            }
                        }
                    });
                });
            }

            let counts = {
                let c = set_counts();
                if c.len() == exercises.len() {
                    c
                } else {
                    vec![0; exercises.len()]
                }
            };

            let mut display_counts = counts.clone();
            if let Some(ref session) = current_session
                && let Some(ref sid) = session.exercise.id
            {
                for (idx, pe) in exercises.iter().enumerate() {
                    if pe.exercise.id.as_ref() == Some(sid) {
                        display_counts[idx] =
                            display_counts[idx].max(session.completed_sets.len() as u32);
                    }
                }
            }

            // Check if current exercise exceeds planned sets
            let over_plan = if let Some(ref session) = current_session {
                let current_sets = session.completed_sets.len() as u32;
                exercises
                    .get(active_idx)
                    .map(|pe| current_sets > pe.planned_sets)
                    .unwrap_or(false)
            } else {
                false
            };

            rsx! {
                div {
                    class: "max-w-2xl mx-auto",

                    // End Workout button
                    div {
                        class: "flex justify-end px-4 py-2",
                        button {
                            class: "btn btn-ghost btn-sm text-error",
                            "data-testid": "end-workout-button",
                            onclick: move |_| {
                                spawn(async move {
                                    if let Err(e) = WorkoutStateManager::end_plan(&state).await {
                                        log::warn!("Failed to end plan: {}", e);
                                    }
                                });
                            },
                            "End Workout"
                        }
                    }

                    ExerciseTabStrip {
                        exercises: exercises.clone(),
                        active_index: active_idx,
                        completed_counts: display_counts,
                        on_add: move |_| {
                            add_search_query.set(String::new());
                            show_add_exercise.set(true);
                        },
                        on_select: move |idx: usize| {
                            active_tab_index.set(idx);
                            prompt_dismissed.set(false);
                            if let Some(pe) = exercises.get(idx) {
                                let exercise = pe.exercise.clone();
                                spawn(async move {
                                    if let Err(e) = WorkoutStateManager::start_session(&state, exercise).await {
                                        log::warn!("Failed to start session: {}", e);
                                    }
                                });
                            }
                        },
                    }

                    // Over-plan soft prompt
                    if over_plan && !prompt_dismissed() {
                        div {
                            class: "alert alert-warning mx-4 mb-4 flex justify-between items-center",
                            "data-testid": "over-plan-prompt",
                            span {
                                class: "text-sm",
                                "You've completed all planned sets for this exercise. Consider moving to the next one."
                            }
                            button {
                                class: "btn btn-ghost btn-xs",
                                "data-testid": "dismiss-prompt",
                                onclick: move |_| prompt_dismissed.set(true),
                                "✕"
                            }
                        }
                    }

                    if let Some(session) = current_session {
                        ActiveSession { state, session }
                    } else {
                        div {
                            class: "text-center py-8 text-base-content/50",
                            "data-testid": "plan-active-state",
                            p { "Tap an exercise tab above to start recording." }
                        }
                    }

                    // Exercise picker modal for adding mid-workout
                    if show_add_exercise() {
                        ExercisePickerModal {
                            state,
                            search_query: add_search_query,
                            default_planned_sets: state.settings().default_planned_sets,
                            on_close: move |_| show_add_exercise.set(false),
                        }
                    }
                }
            }
        }

        // Active session without a plan (legacy single-exercise flow)
        (_, Some(session)) => {
            rsx! {
                ActiveSession { state, session: session.clone() }
            }
        }

        // Plan exists but not started — show plan builder
        (Some(_plan), None) => {
            rsx! {
                PlanBuilder { state }
            }
        }

        // No plan — show plan builder
        (None, None) => {
            rsx! {
                PlanBuilder { state }
            }
        }
    }
}
