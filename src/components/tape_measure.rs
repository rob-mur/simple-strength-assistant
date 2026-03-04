use dioxus::prelude::*;
use wasm_bindgen::JsCast;
use web_sys::PointerEvent;

#[derive(Props, PartialEq, Clone)]
pub struct TapeMeasureProps {
    pub value: f64,
    pub min: f64,
    pub max: f64,
    pub step: f64,
    pub on_change: EventHandler<f64>,
}

const PIXELS_PER_STEP: f64 = 60.0;
const FRICTION: f64 = 0.85; // Faster deceleration
const VELOCITY_THRESHOLD: f64 = 0.5; // Trigger snap much sooner
const SNAP_STIFFNESS: f64 = 0.25;
const VIEWPORT_WIDTH: f64 = 300.0;
const VIEWPORT_HEIGHT: f64 = 80.0;
const CENTER_X: f64 = VIEWPORT_WIDTH / 2.0;
const EPSILON_TOLERANCE: f64 = 1e-9;
const CLICK_DRAG_THRESHOLD: f64 = 5.0;

#[component]
pub fn TapeMeasure(props: TapeMeasureProps) -> Element {
    // Offset is pixels relative to the center.
    let mut offset = use_signal(|| (props.value - props.min) / props.step * -PIXELS_PER_STEP);
    let mut velocity = use_signal(|| 0.0);
    let mut is_dragging = use_signal(|| false);
    let mut last_pointer_x = use_signal(|| 0.0);
    let mut last_update_time = use_signal(|| 0.0);
    let mut is_snapping = use_signal(|| false);
    let mut container_element = use_signal(|| None::<web_sys::Element>);
    let mut drag_start_offset = use_signal(|| 0.0);

    // Sync state if props change (prop-to-signal sync pattern)
    let mut last_value = use_signal(|| props.value);
    let mut last_step = use_signal(|| props.step);
    let mut last_min = use_signal(|| props.min);
    let mut last_max = use_signal(|| props.max);

    if (props.value != *last_value.read()
        // Note: float equality on step/min is currently low risk as they are literal constants.
        // If they ever become computed values, consider an epsilon comparison instead.
        || props.step != *last_step.read()
        || props.min != *last_min.read()
        || props.max != *last_max.read())
        && !*is_dragging.peek()
        && !*is_snapping.peek()
    {
        last_value.set(props.value);
        last_step.set(props.step);
        last_min.set(props.min);
        last_max.set(props.max);
        offset.set((props.value - props.min) / props.step * -PIXELS_PER_STEP);
        // Force velocity to 0.0 to ensure sync triggers immediately
        velocity.set(0.0);
    }

    // Momentum and Snapping Loop
    // TODO: Consider RAF-based animation loop instead of 16ms timer for battery efficiency
    use_future(move || async move {
        loop {
            gloo_timers::future::sleep(std::time::Duration::from_millis(16)).await;

            // NEVER snap while the user is actively dragging.
            if *is_dragging.peek() {
                continue;
            }

            let current_velocity_val: f64 = velocity();
            let current_offset_val: f64 = offset();

            // Idle animation guard: skip iteration if component is idle
            if current_velocity_val.abs() < f64::EPSILON && !*is_snapping.peek() {
                continue;
            }

            if current_velocity_val.abs() >= VELOCITY_THRESHOLD + f64::EPSILON {
                // Momentum Phase (Only runs after release)
                let mut new_v = current_velocity_val * FRICTION;
                let mut new_o = current_offset_val + new_v;

                // Edge Resistance (Hard Wall)
                let total_steps = (*last_max.peek() - *last_min.peek()) / *last_step.peek();
                let min_offset = total_steps * -PIXELS_PER_STEP;
                let max_offset = 0.0;

                if new_o > max_offset {
                    new_o = max_offset;
                    new_v = 0.0;
                } else if new_o < min_offset {
                    new_o = min_offset;
                    new_v = 0.0;
                }

                velocity.set(new_v);
                offset.set(new_o);

                // When velocity stops, trigger snap
                if new_v.abs() < VELOCITY_THRESHOLD {
                    velocity.set(0.0);
                    is_snapping.set(true);
                }
            } else if is_snapping() {
                // Snapping Phase
                let target_offset =
                    (current_offset_val / PIXELS_PER_STEP).round() * PIXELS_PER_STEP;
                let diff = target_offset - current_offset_val;

                if diff.abs() > 0.01 {
                    // Tighter tolerance
                    offset.set(current_offset_val + diff * SNAP_STIFFNESS);
                } else {
                    offset.set(target_offset);
                    velocity.set(0.0);
                    is_snapping.set(false);

                    // Notify parent of the final value
                    let steps_from_min = (target_offset / -PIXELS_PER_STEP).round();
                    let final_value = *last_min.peek() + steps_from_min * *last_step.peek();
                    if (final_value - *last_value.peek()).abs() > 0.001 {
                        web_sys::console::log_1(
                            &format!("TapeMeasure Snapping to new value: {}", final_value).into(),
                        );
                        props.on_change.call(final_value);
                    }
                }
            }
        }
    });

    // Calculate range of values to render
    let current_value_raw = props.min + (offset() / -PIXELS_PER_STEP) * props.step;
    let start_val = (current_value_raw - 4.0 * props.step).max(props.min);
    let end_val = (current_value_raw + 4.0 * props.step).min(props.max);

    // We need to iterate by steps
    let mut values = Vec::new();
    let mut v_iter = (start_val / props.step).floor() * props.step;
    // Align v_iter to steps from min
    v_iter = props.min + ((v_iter - props.min) / props.step).floor() * props.step;

    while v_iter <= end_val {
        if v_iter >= props.min {
            values.push(v_iter);
        }
        v_iter += props.step;
    }

    rsx! {
        div {
            class: "tape-measure-container bg-base-100 rounded-lg shadow-inner",
            "data-value": "{props.value}",
            style: "touch-action: none; overflow: hidden; width: 100%; height: {VIEWPORT_HEIGHT}px; position: relative; cursor: grab;",
            onmounted: move |el| {
                // Try HtmlElement first, then fallback to Element
                if let Some(raw) = el.data.downcast::<web_sys::HtmlElement>() {
                    if let Ok(element) = raw.clone().dyn_into::<web_sys::Element>() {
                        container_element.set(Some(element));
                    }
                } else if let Some(raw) = el.data.downcast::<web_sys::Element>() {
                    container_element.set(Some(raw.clone()));
                } else {
                    log::warn!("TapeMeasure: Failed to downcast mounted element to HtmlElement or Element");
                }
            },
            onpointerdown: move |evt| {
                if let Some(e) = evt.data.downcast::<PointerEvent>() {
                    is_dragging.set(true);
                    is_snapping.set(false);
                    last_pointer_x.set(e.client_x() as f64);
                    last_update_time.set(js_sys::Date::now());
                    velocity.set(0.0);
                    drag_start_offset.set(offset());

                    if let Some(el) = container_element.peek().as_ref() {
                        let _ = el.set_pointer_capture(e.pointer_id());
                    }
                } else {
                    log::warn!("Unexpected event type in TapeMeasure onpointerdown");
                }
            },
            onpointermove: move |evt| {
                if is_dragging() {
                    if let Some(e) = evt.data.downcast::<PointerEvent>() {
                        let current_x = e.client_x() as f64;
                        let now = js_sys::Date::now();

                        // Calculate scale factor: SVG units per screen pixel
                        let scale_factor = if let Some(el) = container_element.peek().as_ref() {
                            let rect = el.get_bounding_client_rect();
                            if rect.width() > 0.0 {
                                VIEWPORT_WIDTH / rect.width()
                            } else {
                                1.0
                            }
                        } else {
                            1.0
                        };

                        let delta_x_screen = current_x - last_pointer_x();
                        let delta_x_svg = delta_x_screen * scale_factor;
                        let delta_t = now - last_update_time();

                        let mut new_offset = offset() + delta_x_svg;
                        let total_steps = (props.max - props.min) / props.step;
                        let min_offset = total_steps * -PIXELS_PER_STEP;
                        let max_offset = 0.0;

                        if new_offset > max_offset { new_offset = max_offset; }
                        else if new_offset < min_offset { new_offset = min_offset; }

                        offset.set(new_offset);

                        if delta_t > 0.0 {
                            let inst_velocity = delta_x_svg / delta_t * 16.0;
                            velocity.with_mut(|v| *v = *v * 0.5 + inst_velocity * 0.5);
                        }

                        last_pointer_x.set(current_x);
                        last_update_time.set(now);
                    } else {
                        log::warn!("Unexpected event type in TapeMeasure onpointermove");
                    }
                }
            },
            onpointerup: move |evt| {
                if is_dragging() {
                    if let Some(e) = evt.data.downcast::<PointerEvent>() {
                        is_dragging.set(false);

                        // Calculate drag distance
                        let drag_distance = (offset() - drag_start_offset()).abs();

                        // If moving very slowly, jump straight to snapping
                        if velocity().abs() < VELOCITY_THRESHOLD {
                            is_snapping.set(true);
                        }

                        // If it was a small drag, treat it as a click
                        if drag_distance < CLICK_DRAG_THRESHOLD {
                            #[allow(clippy::collapsible_if)]
                            if let Some(el) = container_element.peek().as_ref() {
                                let rect = el.get_bounding_client_rect();
                                let relative_x = e.client_x() as f64 - rect.left();
                                let scale_factor = if rect.width() > 0.0 { VIEWPORT_WIDTH / rect.width() } else { 1.0 };
                                let svg_x = relative_x * scale_factor;

                                // Calculate which value was clicked based on SVG coordinates
                                let clicked_steps = ((svg_x - CENTER_X - offset()) / PIXELS_PER_STEP).round();
                                let mut final_val = props.min + clicked_steps * props.step;

                                // Clamp final_val
                                if final_val < props.min { final_val = props.min; }
                                if final_val > props.max { final_val = props.max; }

                                let target_offset = (final_val - props.min) / props.step * -PIXELS_PER_STEP;
                                velocity.set(0.0);
                                offset.set(target_offset);

                                log::debug!("TapeMeasure Calculated Click: updating to {}", final_val);
                                props.on_change.call(final_val);
                            }
                        }

                        if let Some(el) = container_element.peek().as_ref() {
                            let _ = el.release_pointer_capture(e.pointer_id());
                        }
                    } else {
                        log::warn!("Unexpected event type in TapeMeasure onpointerup");
                    }
                }
            },
            onpointercancel: move |evt| {
                if is_dragging() {
                    if let Some(e) = evt.data.downcast::<PointerEvent>() {
                        is_dragging.set(false);
                        is_snapping.set(true); // Snap immediately on cancel
                        if let Some(el) = container_element.peek().as_ref() {
                            let _ = el.release_pointer_capture(e.pointer_id());
                        }
                    } else {
                        log::warn!("Unexpected event type in TapeMeasure onpointercancel");
                    }
                }
            },
            onlostpointercapture: move |_| {
                if is_dragging() {
                    is_dragging.set(false);
                    is_snapping.set(true);
                }
            },
            // trust pointer capture for out-of-bounds movement.

            svg {
                view_box: "0 0 {VIEWPORT_WIDTH} {VIEWPORT_HEIGHT}",
                width: "100%",
                height: "100%",

                line {
                    x1: "{CENTER_X}", y1: "10", x2: "{CENTER_X}", y2: "70",
                    stroke: "currentColor", stroke_width: "3", class: "text-primary"
                }

                g {
                    transform: "translate({offset() + CENTER_X}, 0)",

                    for val in values {
                        {
                            let x = (val - props.min) / props.step * PIXELS_PER_STEP;
                            let dist = (x + offset()).abs();
                            let opacity = (1.0 - dist / (VIEWPORT_WIDTH / 1.5)).max(0.0).powf(2.0);
                            let is_major = (val % (props.step * 2.0)).abs() < EPSILON_TOLERANCE || props.step >= 1.0;

                            rsx! {
                                                                    g {
                                                                        key: "{val}",
                                                                        style: "opacity: {opacity}",
                                                                        line {
                                                                            x1: "{x}", y1: if is_major { "50" } else { "55" },                                        x2: "{x}", y2: "70",
                                        stroke: "currentColor", stroke_width: "2"
                                    }
                                    if is_major {
                                        {
                                            let decimals = if props.step.fract() == 0.0 { 0 }
                                                           else if (props.step * 10.0).fract() == 0.0 { 1 }
                                                           else { 2 };
                                            let rounded_val = (val * 100.0).round() / 100.0;
                                            let formatted_val = format!("{:.prec$}", rounded_val, prec = decimals);
                                            rsx! {
                                                text {
                                                    x: "{x}", y: "40", text_anchor: "middle",
                                                    font_size: "20", font_weight: "bold", fill: "currentColor",
                                                    style: "pointer-events: none;",
                                                    "{formatted_val}"
                                                }
                                            }
                                        }
                                    }
                                }
                            }
                        }
                    }
                }
            }
        }
    }
}
