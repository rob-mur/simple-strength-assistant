use dioxus::prelude::*;

#[derive(Props, PartialEq, Clone)]
pub struct StepControlsProps {
    pub value: f64,
    pub steps: Vec<f64>,
    pub min: f64,
    pub max: f64,
    pub on_change: EventHandler<f64>,
}

#[component]
pub fn StepControls(props: StepControlsProps) -> Element {
    let mut neg_steps = Vec::new();
    let mut pos_steps = Vec::new();

    for &step in &props.steps {
        if step < 0.0 {
            neg_steps.push(step);
        } else {
            pos_steps.push(step);
        }
    }

    neg_steps.sort_by(|a, b| a.partial_cmp(b).unwrap());
    pos_steps.sort_by(|a, b| a.partial_cmp(b).unwrap());

    rsx! {
        div {
            class: "flex flex-row justify-between items-center w-full mt-4 px-1 gap-4",
            style: "width: 100%;",

            // Left side (Decrements)
            div {
                class: "flex-1 flex justify-start gap-2",
                for step in neg_steps {
                    {
                        rsx! {
                            button {
                                key: "{step}",
                                class: "btn btn-circle btn-outline btn-error btn-sm",
                                onclick: move |_| {
                                    let new_val = (props.value + step).clamp(props.min, props.max);
                                    if (new_val - props.value).abs() > 0.001 {
                                        props.on_change.call(new_val);
                                    }
                                },
                                "{step}"
                            }
                        }
                    }
                }
            }

            // Right side (Increments)
            div {
                class: "flex-1 flex justify-end gap-2",
                for step in pos_steps {
                    {
                        rsx! {
                            button {
                                key: "{step}",
                                class: "btn btn-circle btn-outline btn-success btn-sm",
                                onclick: move |_| {
                                    let new_val = (props.value + step).clamp(props.min, props.max);
                                    if (new_val - props.value).abs() > 0.001 {
                                        props.on_change.call(new_val);
                                    }
                                },
                                "+{step}"
                            }
                        }
                    }
                }
            }
        }
    }
}

#[cfg(test)]
mod tests {

    // Since we can't easily test Dioxus component events in pure Rust without a lot of ceremony,
    // we'll at least verify that our logic for new_val calculation is sound.
    #[test]
    fn test_clamping_logic() {
        let value: f64 = 100.0;
        let min: f64 = 20.0;
        let max: f64 = 200.0;

        // Normal step
        let step: f64 = 10.0;
        let new_val = (value + step).clamp(min, max);
        assert_eq!(new_val, 110.0);

        // Clamping at max
        let step: f64 = 150.0;
        let new_val = (value + step).clamp(min, max);
        assert_eq!(new_val, 200.0);

        // Clamping at min
        let step: f64 = -100.0;
        let new_val = (value + step).clamp(min, max);
        assert_eq!(new_val, 20.0);

        // Negative step
        let step: f64 = -50.0;
        let new_val = (value + step).clamp(min, max);
        assert_eq!(new_val, 50.0);
    }
}
