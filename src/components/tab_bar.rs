use dioxus::prelude::*;
use serde::{Deserialize, Serialize};

#[derive(Clone, Copy, PartialEq, Debug, Serialize, Deserialize, Default)]
pub enum Tab {
    #[default]
    Workout,
    Library,
    Analysis,
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
                    "tab tab-active h-12 gap-1"
                } else {
                    "tab h-12 gap-1"
                },
                "data-testid": "tab-workout",
                onclick: move |_| on_change.call(Tab::Workout),
                // Dumbbell icon
                svg {
                    xmlns: "http://www.w3.org/2000/svg",
                    fill: "none",
                    view_box: "0 0 24 24",
                    stroke_width: "2",
                    stroke: "currentColor",
                    class: "w-5 h-5",
                    path {
                        stroke_linecap: "round",
                        stroke_linejoin: "round",
                        d: "M6.5 6.5h-1a1 1 0 00-1 1v9a1 1 0 001 1h1a1 1 0 001-1v-9a1 1 0 00-1-1zM4.5 9.5h-1a1 1 0 00-1 1v3a1 1 0 001 1h1M17.5 6.5h1a1 1 0 011 1v9a1 1 0 01-1 1h-1a1 1 0 01-1-1v-9a1 1 0 011-1zM19.5 9.5h1a1 1 0 011 1v3a1 1 0 01-1 1h-1M7.5 12h9"
                    }
                }
                span { "Workout" }
            }

            button {
                role: "tab",
                aria_selected: if active_tab == Tab::Library { "true" } else { "false" },
                class: if active_tab == Tab::Library {
                    "tab tab-active h-12 gap-1"
                } else {
                    "tab h-12 gap-1"
                },
                "data-testid": "tab-library",
                onclick: move |_| on_change.call(Tab::Library),
                // Book/library icon
                svg {
                    xmlns: "http://www.w3.org/2000/svg",
                    fill: "none",
                    view_box: "0 0 24 24",
                    stroke_width: "2",
                    stroke: "currentColor",
                    class: "w-5 h-5",
                    path {
                        stroke_linecap: "round",
                        stroke_linejoin: "round",
                        d: "M12 6.042A8.967 8.967 0 006 3.75c-1.052 0-2.062.18-3 .512v14.25A8.987 8.987 0 016 18c2.305 0 4.408.867 6 2.292m0-14.25a8.966 8.966 0 016-2.292c1.052 0 2.062.18 3 .512v14.25A8.987 8.987 0 0018 18a8.967 8.967 0 00-6 2.292m0-14.25v14.25"
                    }
                }
                span { "Library" }
            }

            button {
                role: "tab",
                aria_selected: if active_tab == Tab::Analysis { "true" } else { "false" },
                class: if active_tab == Tab::Analysis {
                    "tab tab-active h-12 gap-1"
                } else {
                    "tab h-12 gap-1"
                },
                "data-testid": "tab-analysis",
                onclick: move |_| on_change.call(Tab::Analysis),
                // Chart/analysis icon
                svg {
                    xmlns: "http://www.w3.org/2000/svg",
                    fill: "none",
                    view_box: "0 0 24 24",
                    stroke_width: "2",
                    stroke: "currentColor",
                    class: "w-5 h-5",
                    path {
                        stroke_linecap: "round",
                        stroke_linejoin: "round",
                        d: "M3 13.125C3 12.504 3.504 12 4.125 12h2.25c.621 0 1.125.504 1.125 1.125v6.75C7.5 20.496 6.996 21 6.375 21h-2.25A1.125 1.125 0 013 19.875v-6.75zM9.75 8.625c0-.621.504-1.125 1.125-1.125h2.25c.621 0 1.125.504 1.125 1.125v11.25c0 .621-.504 1.125-1.125 1.125h-2.25a1.125 1.125 0 01-1.125-1.125V8.625zM16.5 4.125c0-.621.504-1.125 1.125-1.125h2.25C20.496 3 21 3.504 21 4.125v15.75c0 .621-.504 1.125-1.125 1.125h-2.25a1.125 1.125 0 01-1.125-1.125V4.125z"
                    }
                }
                span { "Analysis" }
            }
        }
    }
}
