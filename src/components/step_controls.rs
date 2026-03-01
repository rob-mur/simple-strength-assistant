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

    neg_steps.sort_by(|a, b| a.total_cmp(b));
    pos_steps.sort_by(|a, b| a.total_cmp(b));

    rsx! {
        div {
            class: "flex flex-row justify-between items-center w-full mt-6 px-1 gap-4",

            // Left side (Decrements)
            div {
                class: "flex-1 flex justify-start gap-3",
                for step in neg_steps {
                    {
                        rsx! {
                            button {
                                key: "{step}",
                                class: "btn btn-circle btn-md shadow-lg glass border-2 border-error/30 hover:border-error text-error transition-all hover:scale-110 active:scale-95",
                                onclick: move |_| {
                                    let new_val = (props.value + step).clamp(props.min, props.max);
                                    if (new_val - props.value).abs() > 0.001 {
                                        props.on_change.call(new_val);
                                    }
                                },
                                div {
                                    class: "flex flex-col items-center justify-center gap-0",
                                    svg {
                                        xmlns: "http://www.w3.org/2000/svg",
                                        fill: "none",
                                        view_box: "0 0 24 24",
                                        stroke_width: "4",
                                        stroke: "currentColor",
                                        class: "w-4 h-4",
                                        path { stroke_linecap: "round", stroke_linejoin: "round", d: "M5 12h14" }
                                    }
                                    span { class: "text-xs font-black", "{step.abs()}" }
                                }
                            }
                        }
                    }
                }
            }

            // Right side (Increments)
            div {
                class: "flex-1 flex justify-end gap-3",
                for step in pos_steps {
                    {
                        rsx! {
                            button {
                                key: "{step}",
                                class: "btn btn-circle btn-md shadow-lg glass border-2 border-success/30 hover:border-success text-success transition-all hover:scale-110 active:scale-95",
                                onclick: move |_| {
                                    let new_val = (props.value + step).clamp(props.min, props.max);
                                    if (new_val - props.value).abs() > 0.001 {
                                        props.on_change.call(new_val);
                                    }
                                },
                                div {
                                    class: "flex flex-col items-center justify-center gap-0",
                                    svg {
                                        xmlns: "http://www.w3.org/2000/svg",
                                        fill: "none",
                                        view_box: "0 0 24 24",
                                        stroke_width: "4",
                                        stroke: "currentColor",
                                        class: "w-4 h-4",
                                        path { stroke_linecap: "round", stroke_linejoin: "round", d: "M12 4.5v15m7.5-7.5h-15" }
                                    }
                                    span { class: "text-xs font-black", "{step.abs()}" }
                                }
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
