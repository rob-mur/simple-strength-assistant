use dioxus::prelude::*;
use serde::{Deserialize, Serialize};

#[derive(Clone, Copy, PartialEq, Debug, Serialize, Deserialize)]
pub enum Tab {
    Workout,
    Library,
}

#[component]
pub fn TabBar(active_tab: Tab, on_change: EventHandler<Tab>) -> Element {
    rsx! {
        div {
            role: "tablist",
            class: "tabs tabs-boxed fixed bottom-0 left-0 right-0 bg-base-100 shadow-lg z-50 p-2",

            button {
                role: "tab",
                class: if active_tab == Tab::Workout {
                    "tab tab-active"
                } else {
                    "tab"
                },
                onclick: move |_| on_change.call(Tab::Workout),
                "Workout"
            }

            button {
                role: "tab",
                class: if active_tab == Tab::Library {
                    "tab tab-active"
                } else {
                    "tab"
                },
                onclick: move |_| on_change.call(Tab::Library),
                "Library"
            }
        }
    }
}
