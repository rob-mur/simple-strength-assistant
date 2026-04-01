use crate::components::rpe_slider::RPESlider;
use crate::components::step_controls::StepControls;
use crate::components::tape_measure::TapeMeasure;
use crate::models::{ExerciseMetadata, HistorySet, SetType, SetTypeConfig};
use dioxus::prelude::*;

#[component]
pub fn EditSetModal(
    set: HistorySet,
    exercise: ExerciseMetadata,
    on_save: EventHandler<(u32, f32, Option<f32>)>,
    on_delete: EventHandler<i64>,
    on_cancel: EventHandler<()>,
) -> Element {
    let mut reps = use_signal(|| set.reps);
    let mut rpe = use_signal(|| set.rpe as f64);
    let mut weight = use_signal(|| {
        if let SetType::Weighted { weight } = set.set_type {
            Some(weight as f64)
        } else {
            None
        }
    });

    let is_weighted = matches!(exercise.set_type_config, SetTypeConfig::Weighted { .. });
    let (min_weight, increment) = match exercise.set_type_config {
        SetTypeConfig::Weighted {
            min_weight,
            increment,
        } => (min_weight as f64, increment as f64),
        SetTypeConfig::Bodyweight => (0.0, 1.0),
    };

    rsx! {
        div {
            class: "fixed inset-0 z-50 overflow-y-auto bg-black/60 backdrop-blur-sm",
            "data-testid": "edit-set-modal",
            div {
                class: "flex min-h-full items-center justify-center p-4",
                div {
                class: "bg-base-100 rounded-2xl shadow-2xl w-full max-w-md overflow-hidden animate-in fade-in zoom-in duration-200",
                div {
                    class: "p-6",
                    div {
                        class: "flex justify-between items-start mb-1",
                        h3 { class: "text-xl font-bold", "{set.exercise_name}" }
                        button {
                            class: "btn btn-ghost btn-sm btn-circle",
                            onclick: move |_| on_cancel.call(()),
                            svg {
                                xmlns: "http://www.w3.org/2000/svg",
                                class: "h-6 w-6",
                                fill: "none",
                                view_box: "0 0 24 24",
                                stroke: "currentColor",
                                path {
                                    stroke_linecap: "round",
                                    stroke_linejoin: "round",
                                    stroke_width: "2",
                                    d: "M6 18L18 6M6 6l12 12"
                                }
                            }
                        }
                    }
                    p { class: "text-sm text-base-content/60 mb-6", "Set {set.set_number}" }

                    if is_weighted {
                        div {
                            class: "mb-8",
                            label { class: "label font-bold", "Weight (kg)" }
                            TapeMeasure {
                                value: weight().unwrap_or(min_weight),
                                min: min_weight,
                                max: 500.0,
                                step: increment,
                                on_change: move |val| weight.set(Some(val))
                            }
                            div {
                                class: "text-center text-4xl font-black text-primary mt-2",
                                "{crate::format::fmt_weight(weight().unwrap_or(min_weight) as f32)} kg"
                            }
                        }
                    }

                    div {
                        class: "mb-8",
                        "data-testid": "reps-section",
                        label { class: "label font-bold", "Reps" }
                        div {
                            class: "text-center text-5xl font-black text-primary mb-2",
                            "{reps}"
                        }
                        StepControls {
                            value: reps() as f64,
                            steps: vec![-5.0, -1.0, 1.0, 5.0],
                            min: 1.0,
                            max: 100.0,
                            on_change: move |val| reps.set(val as u32)
                        }
                    }

                    div {
                        class: "mb-8",
                        label { class: "label font-bold", "RPE" }
                        RPESlider {
                            value: rpe(),
                            on_change: move |val| rpe.set(val)
                        }
                    }

                    div {
                        class: "flex flex-col gap-3 mt-4",
                        button {
                            class: "btn btn-primary btn-block shadow-lg",
                            "data-testid": "save-set-button",
                            onclick: move |_| {
                                on_save.call((reps(), rpe() as f32, weight().map(|w| w as f32)));
                            },
                            "Save Changes"
                        }
                        div {
                            class: "flex gap-2",
                            button {
                                class: "btn btn-error btn-outline flex-1",
                                "data-testid": "delete-set-button",
                                onclick: move |_| on_delete.call(set.id),
                                "Delete Set"
                            }
                            button {
                                class: "btn btn-ghost flex-1",
                                onclick: move |_| on_cancel.call(()),
                                "Cancel"
                            }
                        }
                    }
                }
            }
            }
        }
    }
}
