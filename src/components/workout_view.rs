use crate::app::{ActiveSession, Route};
use crate::components::data_management::DataManagementPanel;
use crate::state::WorkoutState;
use dioxus::prelude::*;

#[component]
pub fn WorkoutView(state: WorkoutState) -> Element {
    let current_session = state.current_session();

    if let Some(session) = current_session {
        log::debug!(
            "[WorkoutView] Rendering ActiveSession for exercise: {}",
            session.exercise.name
        );
        rsx! {
            ActiveSession { state: state, session }
        }
    } else {
        log::debug!("[WorkoutView] No current_session, rendering empty state");
        let navigator = use_navigator();
        rsx! {
            div {
                class: "max-w-md mx-auto py-12 text-center",
                "data-testid": "workout-empty-state",
                div {
                    class: "card bg-base-100 shadow-xl py-8 items-center",
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
                                class: "w-12 h-12 text-primary/40",
                                path {
                                    stroke_linecap: "round",
                                    stroke_linejoin: "round",
                                    d: "M15.362 5.214A8.252 8.252 0 0112 21 8.25 8.25 0 016.038 7.048 8.287 8.287 0 009 9.6a8.983 8.983 0 013.361-6.867 8.21 8.21 0 003 2.48z"
                                }
                            }
                        }
                        h2 { class: "card-title text-2xl font-black uppercase tracking-tight", "No Active Session" }
                        p {
                            class: "text-base-content/60 mt-2",
                            "Go to the library to choose an exercise and start your workout."
                        }
                        div {
                            class: "card-actions mt-6 flex-col items-center gap-3",
                            button {
                                class: "btn btn-primary btn-lg px-8 shadow-lg font-bold",
                                onclick: move |_| { navigator.push(Route::LibraryTab); },
                                "Go to Library"
                            }
                            button {
                                class: "btn btn-ghost btn-sm",
                                "data-testid": "view-history-btn",
                                onclick: move |_| { navigator.push(Route::WorkoutHistory); },
                                "View workout history"
                            }
                        }
                        DataManagementPanel { state }
                    }
                }
            }
        }
    }
}
