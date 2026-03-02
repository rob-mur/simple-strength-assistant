use crate::app::ActiveSession;
use crate::components::tab_bar::Tab;
use crate::state::WorkoutState;
use dioxus::prelude::*;

#[component]
pub fn WorkoutView(state: WorkoutState) -> Element {
    let current_session = state.current_session();
    let mut active_tab = consume_context::<Signal<Tab>>();

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
        rsx! {
            div {
                class: "max-w-md mx-auto py-12 text-center",
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
                            class: "card-actions mt-8",
                            button {
                                class: "btn btn-primary btn-lg px-8 shadow-lg font-bold",
                                onclick: move |_| {
                                    active_tab.set(Tab::Library);
                                },
                                "Go to Library"
                            }
                        }
                    }
                }
            }
        }
    }
}
