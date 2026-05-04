use crate::models::PlanExercise;
use dioxus::prelude::*;

#[component]
pub fn ExerciseTabStrip(
    exercises: Vec<PlanExercise>,
    active_index: usize,
    completed_counts: Vec<u32>,
    on_select: EventHandler<usize>,
    #[props(default)] on_add: Option<EventHandler<()>>,
) -> Element {
    let show_add = on_add.is_some();

    rsx! {
        div {
            class: "relative w-full mb-2",
            "data-testid": "exercise-tab-strip",

            button {
                class: "hidden sm:flex absolute left-0 top-1/2 -translate-y-1/2 z-10 btn btn-circle btn-ghost btn-sm bg-base-100/80",
                "data-testid": "scroll-left",
                onclick: move |_| {
                    #[cfg(target_arch = "wasm32")]
                    {
                        use wasm_bindgen::JsCast;
                        if let Some(el) = web_sys::window()
                            .and_then(|w| w.document())
                            .and_then(|d| {
                                d.query_selector("[data-testid=\"tab-scroll-container\"]")
                                    .ok()
                            })
                            .flatten()
                        {
                            let container = el.dyn_ref::<web_sys::HtmlElement>().unwrap();
                            container.scroll_by_with_x_and_y(-150.0, 0.0);
                        }
                    }
                },
                "‹"
            }

            div {
                class: "overflow-x-auto scrollbar-none px-8 sm:px-10",
                "data-testid": "tab-scroll-container",
                div {
                    class: "flex gap-2 w-max",
                    for (idx, pe) in exercises.iter().enumerate() {
                        {
                            let is_active = idx == active_index;
                            let exercise_name = pe.exercise.name.clone();
                            let planned = pe.planned_sets;
                            let completed = completed_counts.get(idx).copied().unwrap_or(0);
                            let all_done = completed >= planned;
                            let over_plan = completed > planned;

                            rsx! {
                                button {
                                    key: "{pe.id}",
                                    class: if is_active {
                                        "flex flex-col items-center px-4 py-2 rounded-lg min-w-[80px] bg-primary text-primary-content shadow-md transition-all"
                                    } else {
                                        "flex flex-col items-center px-4 py-2 rounded-lg min-w-[80px] bg-base-200 hover:bg-base-300 transition-all"
                                    },
                                    "data-testid": "exercise-tab",
                                    onclick: move |_| on_select.call(idx),
                                    div {
                                        class: "flex items-center gap-1",
                                        span {
                                            class: "text-xs font-bold truncate max-w-[80px]",
                                            "{exercise_name.to_uppercase()}"
                                        }
                                        if all_done {
                                            span {
                                                class: "text-xs",
                                                "data-testid": "tab-checkmark",
                                                "✓"
                                            }
                                        }
                                    }
                                    span {
                                        class: if over_plan {
                                            "text-xs font-semibold mt-1 text-warning"
                                        } else {
                                            "text-xs font-semibold mt-1"
                                        },
                                        "data-testid": "set-count-badge",
                                        "{completed}/{planned}"
                                    }
                                    div {
                                        class: "flex gap-1 mt-1",
                                        for dot_idx in 0..planned {
                                            span {
                                                key: "{dot_idx}",
                                                class: if dot_idx < completed {
                                                    if is_active { "w-2 h-2 rounded-full bg-primary-content" } else { "w-2 h-2 rounded-full bg-primary" }
                                                } else {
                                                    if is_active { "w-2 h-2 rounded-full bg-primary-content/30" } else { "w-2 h-2 rounded-full bg-base-content/20" }
                                                },
                                                "data-testid": if dot_idx < completed { "dot-filled" } else { "dot-empty" },
                                            }
                                        }
                                    }
                                }
                            }
                        }
                    }
                    if show_add {
                        button {
                            class: "flex items-center justify-center px-3 py-2 rounded-lg min-w-[44px] bg-base-200 hover:bg-base-300 transition-all border-2 border-dashed border-base-content/20",
                            "data-testid": "add-exercise-tab",
                            onclick: move |_| {
                                if let Some(ref handler) = on_add {
                                    handler.call(());
                                }
                            },
                            span { class: "text-lg font-bold text-base-content/40", "+" }
                        }
                    }
                }
            }

            button {
                class: "hidden sm:flex absolute right-0 top-1/2 -translate-y-1/2 z-10 btn btn-circle btn-ghost btn-sm bg-base-100/80",
                "data-testid": "scroll-right",
                onclick: move |_| {
                    #[cfg(target_arch = "wasm32")]
                    {
                        use wasm_bindgen::JsCast;
                        if let Some(el) = web_sys::window()
                            .and_then(|w| w.document())
                            .and_then(|d| {
                                d.query_selector("[data-testid=\"tab-scroll-container\"]")
                                    .ok()
                            })
                            .flatten()
                        {
                            let container = el.dyn_ref::<web_sys::HtmlElement>().unwrap();
                            container.scroll_by_with_x_and_y(150.0, 0.0);
                        }
                    }
                },
                "›"
            }
        }
    }
}
