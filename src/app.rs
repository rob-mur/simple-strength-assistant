use dioxus::prelude::*;
use crate::service_worker::use_service_worker_manager;

#[component]
pub fn App() -> Element {
    // Initialize service worker management
    use_service_worker_manager();

    rsx! {
        div {
            "Hello World"
        }
    }
}
