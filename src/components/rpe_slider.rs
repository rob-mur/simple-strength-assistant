use dioxus::prelude::*;

#[derive(Props, PartialEq, Clone)]
pub struct RPESliderProps {
    pub value: f64,
    pub on_change: EventHandler<f64>,
}

const MIN_RPE: f64 = 1.0;
const MAX_RPE: f64 = 10.0;
const STEP_RPE: f64 = 0.5;

#[component]
pub fn RPESlider(props: RPESliderProps) -> Element {
    let text_color_class = match props.value {
        v if v >= 9.0 => "text-error",
        v if v >= 7.5 => "text-warning",
        v if v >= 6.0 => "text-accent",
        _ => "text-success",
    };

    // Use range slider color for different intensities
    let range_class = match props.value {
        v if v >= 9.0 => "range-error",
        v if v >= 7.5 => "range-warning",
        v if v >= 6.0 => "range-accent",
        _ => "range-success",
    };

    rsx! {
        div {
            class: "rpe-slider-container w-full py-4 px-2",

            // Value Display
            div {
                class: "flex justify-center mb-4",
                div {
                    class: "text-6xl font-black {text_color_class} transition-colors duration-300",
                    "{props.value:.1}"
                }
            }

            // Standard HTML Range Slider (Enhanced with DaisyUI)
            div {
                class: "px-4",
                input {
                    r#type: "range",
                    min: "{MIN_RPE}",
                    max: "{MAX_RPE}",
                    step: "{STEP_RPE}",
                    value: "{props.value}",
                    class: "range range-lg {range_class}",
                    oninput: move |evt| {
                        if let Ok(val) = evt.value().parse::<f64>() {
                            props.on_change.call(val);
                        }
                    }
                }
            }

            // Legend/Context
            div {
                class: "flex justify-center mt-6",
                div {
                    class: "badge badge-outline opacity-70",
                    match props.value {
                        v if v >= 9.0 => "Extremely Hard",
                        v if v >= 7.5 => "Heavy / Challenging",
                        v if v >= 6.0 => "Moderate / Solid",
                        _ => "Warmup / Recovery",
                    }
                }
            }
        }
    }
}
