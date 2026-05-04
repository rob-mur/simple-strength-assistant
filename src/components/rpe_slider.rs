use dioxus::prelude::*;

#[derive(Props, PartialEq, Clone)]
pub struct RPESliderProps {
    pub value: f64,
    pub on_change: EventHandler<f64>,
    /// When true, hide the value display above the slider (caller shows it elsewhere).
    #[props(default = false)]
    pub hide_value: bool,
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
            class: "rpe-slider-container w-full px-2",

            // Value Display (hidden when caller provides its own)
            if !props.hide_value {
                div {
                    class: "flex justify-center mb-1",
                    div {
                        class: "text-3xl font-black {text_color_class} transition-colors duration-300",
                        "{props.value:.1}"
                    }
                }
            }

            // Standard HTML Range Slider (Enhanced with DaisyUI)
            // touch-action: pan-x keeps the drag alive when the finger drifts
            // vertically — the browser will not reclassify the gesture as a
            // page scroll and therefore will not fire pointercancel mid-drag.
            div {
                class: "px-4",
                style: "touch-action: pan-x;",
                input {
                    r#type: "range",
                    min: "{MIN_RPE}",
                    max: "{MAX_RPE}",
                    step: "{STEP_RPE}",
                    value: "{props.value}",
                    class: "range range-lg {range_class}",
                    style: "touch-action: pan-x;",
                    oninput: move |evt| {
                        if let Ok(val) = evt.value().parse::<f64>() {
                            props.on_change.call(val);
                        }
                    }
                }
            }
        }
    }
}
