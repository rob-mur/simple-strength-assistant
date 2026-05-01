use crate::app::ActiveSession;
use crate::components::plan_builder::PlanBuilder;
use crate::state::WorkoutState;
use dioxus::prelude::*;

#[component]
pub fn WorkoutView(state: WorkoutState) -> Element {
    let current_session = state.current_session();
    let current_plan = state.current_plan();

    if let Some(session) = current_session {
        log::debug!(
            "[WorkoutView] Rendering ActiveSession for exercise: {}",
            session.exercise.name
        );
        rsx! {
            ActiveSession { state: state, session }
        }
    } else if let Some(ref plan) = current_plan {
        if plan.started_at.is_some() && plan.ended_at.is_none() {
            // Plan is started but no current session — show plan overview
            // (The recorder/tab strip will be added in issue #8/#9)
            log::debug!("[WorkoutView] Plan started, awaiting session selection");
            rsx! {
                div {
                    class: "max-w-md mx-auto py-6 text-center",
                    "data-testid": "plan-active-state",
                    h2 {
                        class: "text-xl font-black uppercase tracking-tight mb-4",
                        "Workout In Progress"
                    }
                    p {
                        class: "text-base-content/60 mb-6",
                        "Select an exercise from the tabs above to start recording sets."
                    }
                }
            }
        } else {
            // Plan exists but not started — show plan builder
            log::debug!("[WorkoutView] Rendering PlanBuilder");
            rsx! {
                PlanBuilder { state }
            }
        }
    } else {
        // No plan at all — show plan builder (will create one)
        log::debug!("[WorkoutView] No plan, rendering PlanBuilder");
        rsx! {
            PlanBuilder { state }
        }
    }
}
