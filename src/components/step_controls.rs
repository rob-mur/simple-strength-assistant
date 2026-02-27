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
    rsx! {
        div {
            class: "flex justify-center mt-2 overflow-x-auto",
            div {
                class: "join w-full flex-nowrap",
                for step in props.steps {
                    {
                        let is_negative = step < 0.0;
                        let abs_step = step.abs();
                        let label = format!("{}{}", if is_negative { "-" } else { "+" }, abs_step);
                        let btn_class = if is_negative { "btn-outline btn-error" } else { "btn-outline btn-success" };

                        rsx! {
                            button {
                                key: "{step}",
                                class: "join-item btn btn-xs sm:btn-sm flex-1 {btn_class}",
                                onclick: move |_| {
                                    let new_val = (props.value + step).clamp(props.min, props.max);
                                    if (new_val - props.value).abs() > 0.001 {
                                        props.on_change.call(new_val);
                                    }
                                },
                                {label}
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
