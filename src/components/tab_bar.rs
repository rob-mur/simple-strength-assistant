use dioxus::prelude::*;
use serde::{Deserialize, Serialize};

#[derive(Clone, Copy, PartialEq, Debug, Serialize, Deserialize, Default)]
pub enum Tab {
    #[default]
    Workout,
    Library,
}

/// Tab bar component. `active_tab` controls which tab appears selected.
/// `on_change` is called when the user taps a tab; the caller handles navigation.
#[component]
pub fn TabBar(active_tab: Tab, on_change: EventHandler<Tab>) -> Element {
    rsx! {
        div {
            role: "tablist",
            class: "tabs tabs-boxed bg-base-100 shadow-lg z-50 p-2 pb-safe-tabbar",

            button {
                role: "tab",
                aria_selected: if active_tab == Tab::Workout { "true" } else { "false" },
                class: if active_tab == Tab::Workout {
                    "tab tab-active h-12"
                } else {
                    "tab h-12"
                },
                "data-testid": "tab-workout",
                onclick: move |_| on_change.call(Tab::Workout),
                "Workout"
            }

            button {
                role: "tab",
                aria_selected: if active_tab == Tab::Library { "true" } else { "false" },
                class: if active_tab == Tab::Library {
                    "tab tab-active h-12"
                } else {
                    "tab h-12"
                },
                "data-testid": "tab-library",
                onclick: move |_| on_change.call(Tab::Library),
                "Library"
            }
        }
    }
}
