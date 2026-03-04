use crate::components::step_controls::StepControls;
use crate::components::tape_measure::TapeMeasure;
use crate::models::{ExerciseMetadata, SetTypeConfig};
use dioxus::prelude::*;

const MAX_EXERCISE_NAME_LENGTH: usize = 100;

fn validate_exercise_name(name: &str) -> Result<(), String> {
    if name.trim().is_empty() {
        return Err("Exercise name cannot be empty".to_string());
    }
    if name.chars().count() > MAX_EXERCISE_NAME_LENGTH {
        return Err(format!(
            "Exercise name must be {} characters or less",
            MAX_EXERCISE_NAME_LENGTH
        ));
    }
    // Basic security validation: We only check for '<' and '>' to prevent
    // basic XSS injection. Note: Dioxus automatically HTML-escapes text in
    // RSX interpolations (the primary XSS defense), making this supplementary.
    // A full sanitization library would be overkill for this simple check.
    if name.contains('<') || name.contains('>') {
        return Err("Exercise name cannot contain HTML characters".to_string());
    }
    Ok(())
}

#[component]
pub fn ExerciseForm(
    initial_exercise: Option<ExerciseMetadata>,
    on_cancel: EventHandler<()>,
    on_save: EventHandler<ExerciseMetadata>,
) -> Element {
    let mut exercise_name = use_signal(|| {
        initial_exercise
            .as_ref()
            .map(|e| e.name.clone())
            .unwrap_or_else(|| "Bench Press".to_string())
    });
    let mut is_weighted = use_signal(|| {
        initial_exercise
            .as_ref()
            .map(|e| matches!(e.set_type_config, SetTypeConfig::Weighted { .. }))
            .unwrap_or(true)
    });
    let mut min_weight = use_signal(|| {
        if let Some(ExerciseMetadata {
            set_type_config: SetTypeConfig::Weighted { min_weight, .. },
            ..
        }) = &initial_exercise
        {
            *min_weight
        } else {
            0.0
        }
    });
    let mut increment = use_signal(|| {
        if let Some(ExerciseMetadata {
            set_type_config: SetTypeConfig::Weighted { increment, .. },
            ..
        }) = &initial_exercise
        {
            *increment
        } else {
            // 2.5 kg is a common barbell increment (1.25kg plates on each side)
            2.5
        }
    });
    let mut validation_error = use_signal(|| None::<String>);

    let initial_id = initial_exercise.as_ref().and_then(|e| e.id);
    let is_edit = initial_exercise.is_some();

    let handle_save = move |_| {
        let name = exercise_name().trim().to_string();

        if let Err(e) = validate_exercise_name(&name) {
            validation_error.set(Some(e));
            return;
        }

        validation_error.set(None);

        let exercise = ExerciseMetadata {
            id: initial_id,
            name,
            set_type_config: if is_weighted() {
                SetTypeConfig::Weighted {
                    min_weight: min_weight(),
                    increment: increment(),
                }
            } else {
                SetTypeConfig::Bodyweight
            },
        };

        on_save.call(exercise);
    };

    rsx! {
        div {
            class: "card bg-base-100 shadow-xl",
            div {
                class: "card-body",
                h2 {
                    class: "card-title text-2xl mb-4",
                    if is_edit { "Edit Exercise" } else { "Add New Exercise" }
                }
                div {
                    class: "form-control",
                    label {
                        class: "label",
                        r#for: "exercise-name-input",
                        span {
                            class: "label-text",
                            "Exercise Name"
                        }
                    }
                    input {
                        id: "exercise-name-input",
                        class: if validation_error().is_some() {
                            "input input-bordered input-error w-full"
                        } else {
                            "input input-bordered w-full"
                        },
                        r#type: "text",
                        value: "{exercise_name}",
                        maxlength: MAX_EXERCISE_NAME_LENGTH,
                        oninput: move |e| {
                            exercise_name.set(e.value());
                            validation_error.set(None);
                        }
                    }
                    if let Some(error) = validation_error() {
                        label {
                            class: "label",
                            span {
                                class: "label-text-alt text-error",
                                "{error}"
                            }
                        }
                    }
                }
                div {
                    class: "form-control mt-4",
                    label {
                        class: "label cursor-pointer",
                        span {
                            class: "label-text",
                            "Weighted Exercise"
                        }
                        input {
                            class: "checkbox",
                            r#type: "checkbox",
                            checked: is_weighted(),
                            oninput: move |e| is_weighted.set(e.checked())
                        }
                    }
                }
                if is_weighted() {
                    div {
                        class: "flex flex-col gap-8 mt-6",
                        div {
                            class: "form-control w-full",
                            label {
                                class: "label",
                                span {
                                    class: "label-text font-bold text-lg",
                                    "Minimum Weight (kg)"
                                }
                            }
                            TapeMeasure {
                                value: min_weight() as f64,
                                min: 0.0,
                                max: 500.0,
                                step: increment() as f64,
                                on_change: move |val| min_weight.set(val as f32)
                            }
                            div {
                                class: "text-center text-3xl font-black text-primary mt-2",
                                "{ (min_weight() * 100.0).round() / 100.0 } kg"
                            }
                            StepControls {
                                value: min_weight() as f64,
                                steps: vec![-10.0, 10.0],
                                min: 0.0,
                                max: 500.0,
                                on_change: move |val| min_weight.set(val as f32)
                            }
                        }
                        div {
                            class: "form-control w-full",
                            label {
                                class: "label",
                                span {
                                    class: "label-text font-bold text-lg",
                                    "Weight Increment (kg)"
                                }
                            }
                            div {
                                class: "flex flex-wrap gap-3 justify-center mt-2",
                                for &inc in &[1.25, 2.5, 5.0, 10.0] {
                                    button {
                                        key: "{inc}",
                                        class: if (increment() - inc as f32).abs() < 0.001 {
                                            "btn btn-primary btn-md flex-1 min-w-[70px] shadow-lg"
                                        } else {
                                            "btn btn-outline btn-md flex-1 min-w-[70px]"
                                        },
                                        onclick: move |_| increment.set(inc as f32),
                                        "{inc}"
                                    }
                                }
                            }
                        }
                    }
                }
                div {
                    class: "card-actions justify-end mt-8 gap-2",
                    button {
                        class: "btn btn-ghost",
                        onclick: move |_| on_cancel.call(()),
                        "Cancel"
                    }
                    button {
                        class: "btn btn-primary",
                        onclick: handle_save,
                        "Save Exercise"
                    }
                }
            }
        }
    }
}
