use crate::app::{ActiveSession, StartSessionView};
use crate::state::WorkoutState;
use dioxus::prelude::*;

#[component]
pub fn WorkoutView(state: WorkoutState) -> Element {
    let current_session = state.current_session();

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
        log::debug!("[WorkoutView] No current_session, rendering StartSessionView");
        rsx! {
            StartSessionView { state: state }
        }
    }
}
