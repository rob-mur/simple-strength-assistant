use dioxus::prelude::*;
use wasm_bindgen::JsCast;
use web_sys::PointerEvent;

#[derive(Props, PartialEq, Clone)]
pub struct RPESliderProps {
    pub value: f64,
    pub on_change: EventHandler<f64>,
}

const MIN_RPE: f64 = 1.0;
const MAX_RPE: f64 = 10.0;
const STEP_RPE: f64 = 0.5;
const SLIDER_HEIGHT: f64 = 60.0;
const TRACK_HEIGHT: f64 = 12.0;
const THUMB_SIZE: f64 = 32.0;

fn get_rpe_value_from_x(x: f64, width: f64) -> f64 {
    let percent = (x / width).clamp(0.0, 1.0);
    let raw_val = MIN_RPE + percent * (MAX_RPE - MIN_RPE);
    (raw_val / STEP_RPE).round() * STEP_RPE
}

#[component]
pub fn RPESlider(props: RPESliderProps) -> Element {
    let mut is_dragging = use_signal(|| false);
    // Use a signal to store the track element once mounted
    let mut track_element = use_signal(|| None::<web_sys::Element>);

    let handle_pointer = move |evt: &PointerEvent, element: &web_sys::Element| {
        let rect = element.get_bounding_client_rect();
        let x = (evt.client_x() as f64) - rect.left();
        let val = get_rpe_value_from_x(x, rect.width());
        if (val - props.value).abs() > 0.001 {
            props.on_change.call(val);
        }
    };

    let color_class = match props.value {
        v if v >= 9.0 => "bg-error",
        v if v >= 7.5 => "bg-warning",
        v if v >= 6.0 => "bg-accent",
        _ => "bg-success",
    };

    let text_color_class = match props.value {
        v if v >= 9.0 => "text-error",
        v if v >= 7.5 => "text-warning",
        v if v >= 6.0 => "text-accent",
        _ => "text-success",
    };

    let percent = (props.value - MIN_RPE) / (MAX_RPE - MIN_RPE);

    rsx! {
        div {
            class: "rpe-slider-container w-full py-4 px-2 select-none",
            style: "touch-action: none;",

            // Value Display
            div {
                class: "flex justify-center mb-6",
                div {
                    class: "text-5xl font-black {text_color_class} transition-colors duration-300",
                    "{props.value:.1}"
                }
            }

            // Slider Track Area
            div {
                class: "relative w-full px-4 rpe-slider-track",
                style: "height: {SLIDER_HEIGHT}px; display: flex; align-items: center; cursor: pointer;",
                onmounted: move |_evt| {
                    if let Some(window) = web_sys::window() {
                        if let Some(document) = window.document() {
                            if let Ok(elements) = document.query_selector_all(".rpe-slider-track") {
                                for i in 0..elements.length() {
                                    if let Some(el) = elements.get(i) {
                                        if let Ok(element) = el.dyn_into::<web_sys::Element>() {
                                            track_element.set(Some(element));
                                        }
                                    }
                                }
                            }
                        }
                    }
                },
                onpointerdown: move |evt| {
                    let e = evt.data.downcast::<PointerEvent>().unwrap();
                    is_dragging.set(true);

                    if let Some(el) = track_element() {
                        handle_pointer(&e, &el);
                        let _ = el.set_pointer_capture(e.pointer_id());
                    }
                },
                onpointermove: move |evt| {
                    if is_dragging() {
                        let e = evt.data.downcast::<PointerEvent>().unwrap();
                        if let Some(el) = track_element() {
                            handle_pointer(&e, &el);
                        }
                    }
                },
                onpointerup: move |evt| {
                    if is_dragging() {
                        let e = evt.data.downcast::<PointerEvent>().unwrap();
                        is_dragging.set(false);
                        if let Some(el) = track_element() {
                            let _ = el.release_pointer_capture(e.pointer_id());
                        }
                    }
                },

                // Background Track
                div {
                    class: "absolute left-4 right-4 bg-base-300 rounded-full",
                    style: "height: {TRACK_HEIGHT}px;"
                }

                // Active Track (Colored)
                div {
                    class: "absolute left-4 {color_class} rounded-full transition-colors duration-300",
                    style: "height: {TRACK_HEIGHT}px; width: calc({percent * 100.0}%);"
                }

                // Ticks
                div {
                    class: "absolute left-4 right-4 flex justify-between px-0",
                    style: "pointer-events: none;",
                    for i in 0..=18 {
                        {
                            let is_major = i % 2 == 0;
                            let val = MIN_RPE + (i as f64) * STEP_RPE;
                            rsx! {
                                div {
                                    key: "{val}",
                                    class: "flex flex-col items-center",
                                    div {
                                        class: if is_major { "h-4 w-0.5 bg-base-content opacity-40" } else { "h-2 w-0.5 bg-base-content opacity-20" }
                                    }
                                    if is_major {
                                        span {
                                            class: "text-xs mt-6 font-bold opacity-50",
                                            "{val}"
                                        }
                                    }
                                }
                            }
                        }
                    }
                }

                // Thumb
                div {
                    class: "absolute {color_class} border-4 border-base-100 rounded-full shadow-lg transition-colors duration-300",
                    style: "width: {THUMB_SIZE}px; height: {THUMB_SIZE}px; left: calc(1rem + {percent * 100.0}% - {THUMB_SIZE / 2.0}px); pointer-events: none;"
                }
            }
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_rpe_snapping() {
        assert_eq!(get_rpe_value_from_x(0.0, 100.0), 1.0);
        assert_eq!(get_rpe_value_from_x(100.0, 100.0), 10.0);
        assert_eq!(get_rpe_value_from_x(50.0, 100.0), 5.5);
        assert_eq!(get_rpe_value_from_x(5.0, 100.0), 1.5);
        assert_eq!(get_rpe_value_from_x(95.0, 100.0), 9.5);
    }
}
