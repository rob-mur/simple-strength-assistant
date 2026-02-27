use dioxus::prelude::*;
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
const FRICTION: f64 = 0.92; // Slightly more glide than 0.88
const VELOCITY_THRESHOLD: f64 = 0.01; // Much lower to allow slow slides to finish
const SNAP_STIFFNESS: f64 = 0.25;
const VIEWPORT_WIDTH: f64 = 300.0;
const VIEWPORT_HEIGHT: f64 = 80.0;
const CENTER_X: f64 = VIEWPORT_WIDTH / 2.0;

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

    // Sync state if props change (prop-to-signal sync pattern)
    let mut last_value = use_signal(|| props.value);
    if props.value != *last_value.read() && !*is_dragging.peek() && !*is_snapping.peek() {
        last_value.set(props.value);
        offset.set((props.value - props.min) / props.step * -PIXELS_PER_STEP);
        velocity.set(0.0);
    }

    // Momentum and Snapping Loop
    use_future(move || async move {
        loop {
            gloo_timers::future::sleep(std::time::Duration::from_millis(16)).await;

            // NEVER snap while the user is actively dragging.
            if *is_dragging.peek() {
                continue;
            }

            let current_velocity_val: f64 = velocity();
            let current_offset_val: f64 = offset();

            if current_velocity_val.abs() > VELOCITY_THRESHOLD {
                // Momentum Phase (Only runs after release)
                let mut new_v = current_velocity_val * FRICTION;
                let mut new_o = current_offset_val + new_v;

                // Edge Resistance (Hard Wall)
                let total_steps = (props.max - props.min) / props.step;
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
                if new_v.abs() <= VELOCITY_THRESHOLD {
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
                    let final_value = props.min + steps_from_min * props.step;
                    if (final_value - props.value).abs() > 0.001 {
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
            style: "touch-action: none; overflow: hidden; width: 100%; height: {VIEWPORT_HEIGHT}px; position: relative; cursor: grab;",
            onmounted: move |el| {
                if let Some(raw) = el.data.downcast::<web_sys::Element>() {
                    container_element.set(Some(raw.clone()));
                }
            },
            onpointerdown: move |evt| {
                let e = evt.data.downcast::<PointerEvent>().unwrap();
                is_dragging.set(true);
                is_snapping.set(false);
                last_pointer_x.set(e.client_x() as f64);
                last_update_time.set(js_sys::Date::now());
                velocity.set(0.0);

                if let Some(el) = container_element.peek().as_ref() {
                    let _ = el.set_pointer_capture(e.pointer_id());
                }
            },
            onpointermove: move |evt| {
                if is_dragging() {
                    let e = evt.data.downcast::<PointerEvent>().unwrap();
                    let current_x = e.client_x() as f64;
                    let now = js_sys::Date::now();
                    let delta_x = current_x - last_pointer_x();
                    let delta_t = now - last_update_time();

                    let mut new_offset = offset() + delta_x;
                    let total_steps = (props.max - props.min) / props.step;
                    let min_offset = total_steps * -PIXELS_PER_STEP;
                    let max_offset = 0.0;

                    if new_offset > max_offset { new_offset = max_offset; }
                    else if new_offset < min_offset { new_offset = min_offset; }

                    offset.set(new_offset);

                    if delta_t > 0.0 {
                        let inst_velocity = delta_x / delta_t * 16.0;
                        velocity.with_mut(|v| *v = *v * 0.5 + inst_velocity * 0.5);
                    }

                    last_pointer_x.set(current_x);
                    last_update_time.set(now);
                }
            },
            onpointerup: move |evt| {
                if is_dragging() {
                    let e = evt.data.downcast::<PointerEvent>().unwrap();
                    is_dragging.set(false);
                    // If moving very slowly, jump straight to snapping
                    if velocity().abs() <= VELOCITY_THRESHOLD {
                        is_snapping.set(true);
                    }
                    if let Some(el) = container_element.peek().as_ref() {
                        let _ = el.release_pointer_capture(e.pointer_id());
                    }
                }
            },
            onpointercancel: move |evt| {
                if is_dragging() {
                    let e = evt.data.downcast::<PointerEvent>().unwrap();
                    is_dragging.set(false);
                    is_snapping.set(true); // Snap immediately on cancel
                    if let Some(el) = container_element.peek().as_ref() {
                        let _ = el.release_pointer_capture(e.pointer_id());
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
                            let is_major = (val % (props.step * 2.0)).abs() < 0.001 || props.step >= 1.0;

                            rsx! {
                                g {
                                    key: "{val}",
                                    style: "opacity: {opacity}",
                                    onclick: move |_| {
                                        let target_offset = (val - props.min) / props.step * -PIXELS_PER_STEP;
                                        velocity.set(0.0);
                                        offset.set(target_offset);
                                        props.on_change.call(val);
                                    },
                                    // Broad hitbox for clicks
                                    rect {
                                        x: "{x - PIXELS_PER_STEP / 2.0}",
                                        y: "0",
                                        width: "{PIXELS_PER_STEP}",
                                        height: "{VIEWPORT_HEIGHT}",
                                        fill: "transparent",
                                        style: "cursor: pointer;"
                                    }
                                    line {
                                        x1: "{x}", y1: if is_major { "50" } else { "55" },
                                        x2: "{x}", y2: "70",
                                        stroke: "currentColor", stroke_width: "2"
                                    }
                                    if is_major {
                                        text {
                                            x: "{x}", y: "40", text_anchor: "middle",
                                            font_size: "20", font_weight: "bold", fill: "currentColor",
                                            style: "pointer-events: none;",
                                            "{val}"
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
