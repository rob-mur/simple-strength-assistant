use crate::components::history_view::group_sets_by_day;
use crate::models::{HistorySet, SetType};
use crate::state::WorkoutState;
use dioxus::prelude::*;
use wasm_bindgen::prelude::*;
use web_sys::{IntersectionObserver, IntersectionObserverEntry, IntersectionObserverInit};

const PAGE_SIZE: i64 = 20;

/// Collapsible "Previous Sessions" panel shown inside the active workout view.
///
/// Displays paginated sets for `exercise_id` grouped by local calendar date.
/// Reactively reloads the first page whenever `completed_sets_count` changes so
/// sets logged in the current session appear immediately (AC #6).
#[component]
pub fn PreviousSessions(
    state: WorkoutState,
    exercise_id: i64,
    /// Pass `session.completed_sets.len()` so the panel refreshes on each new set.
    completed_sets_count: usize,
) -> Element {
    let mut expanded = use_signal(|| false);
    let mut sets = use_signal(Vec::<HistorySet>::new);
    let mut has_more = use_signal(|| true);
    let mut loading = use_signal(|| false);

    // Track props in signals so use_effect can subscribe to them
    let mut eid_signal = use_signal(|| exercise_id);
    if *eid_signal.peek() != exercise_id {
        eid_signal.set(exercise_id);
    }
    let mut count_signal = use_signal(|| completed_sets_count);
    if *count_signal.peek() != completed_sets_count {
        count_signal.set(completed_sets_count);
    }

    let load_more = move || {
        if loading() || !has_more() {
            return;
        }
        let offset = sets.read().len() as i64;
        let eid = *eid_signal.peek();
        spawn(async move {
            loading.set(true);
            if let Some(db) = state.database() {
                match db.get_sets_for_exercise(eid, PAGE_SIZE, offset).await {
                    Ok(mut new_sets) => {
                        has_more.set(new_sets.len() as i64 == PAGE_SIZE);
                        sets.write().append(&mut new_sets);
                    }
                    Err(e) => {
                        log::error!("PreviousSessions: failed to load more: {}", e)
                    }
                }
            }
            loading.set(false);
        });
    };

    // Trigger signal for infinite scroll callback (AC #5)
    let mut load_trigger = use_signal(|| 0);
    use_effect(move || {
        if load_trigger() > 0 {
            load_more();
        }
    });

    // Reload first page whenever exercise or completed-set count changes (AC #6).
    use_effect(move || {
        // Subscribe to these signals
        let eid = eid_signal();
        let _count = count_signal();

        // Only load if expanded to save resources (AC #2 optimization)
        if !expanded() {
            return;
        }

        spawn(async move {
            loading.set(true);
            if let Some(db) = state.database() {
                match db.get_sets_for_exercise(eid, PAGE_SIZE, 0).await {
                    Ok(new_sets) => {
                        has_more.set(new_sets.len() as i64 == PAGE_SIZE);
                        sets.set(new_sets);
                    }
                    Err(e) => log::error!("PreviousSessions: failed to load history: {}", e),
                }
            }
            loading.set(false);
        });
    });

    let utc_offset = get_utc_offset_minutes();
    let grouped = group_sets_by_day(&sets.read(), utc_offset);
    let total = sets.read().len();

    rsx! {
        div {
            class: "collapse collapse-arrow bg-base-100 shadow-lg border border-base-300",
            "data-testid": "previous-sessions",

            // Checkbox controls the DaisyUI collapse; unchecked = collapsed (default).
            input {
                r#type: "checkbox",
                checked: expanded(),
                onchange: move |_| expanded.toggle(),
            }

            div {
                class: "collapse-title text-xl font-bold",
                "data-testid": "previous-sessions-header",
                if total > 0 {
                    "Previous Sessions ({total} sets)"
                } else {
                    "Previous Sessions"
                }
            }

            div {
                class: "collapse-content p-0",
                "data-testid": "previous-sessions-content",

                if loading() && grouped.is_empty() {
                    div {
                        class: "flex justify-center py-8",
                        div { class: "loading loading-spinner loading-md" }
                    }
                } else if grouped.is_empty() {
                    div {
                        class: "text-center text-base-content/50 py-6 text-sm",
                        "No previous sessions for this exercise."
                    }
                } else {
                    div {
                        class: "px-4 pt-2 pb-4",
                        for day in grouped.iter() {
                            div {
                                key: "{day.date_label}",
                                class: "mb-4",
                                h4 {
                                    class: "text-xs font-bold text-base-content/50 uppercase tracking-widest mb-1",
                                    "{day.date_label}"
                                }
                                for eg in day.exercises.iter() {
                                    {
                                        let has_weighted = eg.sets.iter().any(|s| matches!(s.set_type, SetType::Weighted { .. }));
                                        rsx! {
                                            div {
                                                key: "{eg.exercise_id}",
                                                class: "overflow-x-auto",
                                                table {
                                                    class: "table table-xs w-full",
                                                    thead {
                                                        tr {
                                                            th { "Set" }
                                                            if has_weighted {
                                                                th { "Weight" }
                                                            }
                                                            th { "Reps" }
                                                            th { "RPE" }
                                                        }
                                                    }
                                                    tbody {
                                                        for set in eg.sets.iter() {
                                                            tr {
                                                                key: "{set.id}",
                                                                td { "{set.set_number}" }
                                                                if has_weighted {
                                                                    if let SetType::Weighted { weight } = set.set_type {
                                                                        td { "{crate::format::fmt_weight(weight)} kg" }
                                                                    } else {
                                                                        td { "—" }
                                                                    }
                                                                }
                                                                td { "{set.reps}" }
                                                                td { "{set.rpe}" }
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

                        // Sentinel for infinite scroll (AC #5)
                        if has_more() {
                            div {
                                id: "history-sentinel",
                                // Use key to force re-mount and re-observe if sets change
                                key: "sentinel-{sets.read().len()}",
                                class: "flex justify-center mt-2 py-4",
                                onmounted: move |el| {
                                    // Use a closure to handle the intersection event.
                                    // We leak it because we need it to stay alive.
                                    let callback = Closure::wrap(Box::new(move |entries: js_sys::Array, _observer: IntersectionObserver| {
                                        if let Ok(entry) = entries.get(0).dyn_into::<IntersectionObserverEntry>()
                                            && entry.is_intersecting() {
                                                // Trigger load_more via signal to stay within Dioxus reactivity system
                                                // and avoid runtime panics from calling spawn() in JS callback context.
                                                load_trigger.with_mut(|v| *v += 1);
                                        }
                                    }) as Box<dyn FnMut(js_sys::Array, IntersectionObserver)>);

                                    let options = IntersectionObserverInit::new();
                                    options.set_threshold(&JsValue::from_f64(0.1));

                                    if let Ok(observer) = IntersectionObserver::new_with_options(
                                        callback.as_ref().unchecked_ref(),
                                        &options,
                                    )
                                        && let Some(raw) = el.data().downcast::<web_sys::Element>() {
                                            observer.observe(raw);
                                            callback.forget(); // Leak for simplicity in this exercise
                                    }
                                },
                                if loading() {
                                    div { class: "loading loading-spinner loading-sm" }
                                } else {
                                    // Fallback button for manual trigger or if observer fails
                                    button {
                                        class: "btn btn-ghost btn-sm",
                                        "data-testid": "previous-sessions-load-more",
                                        onclick: move |_| load_more(),
                                        "Load more"
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

fn get_utc_offset_minutes() -> i32 {
    let offset = js_sys::Date::new_0().get_timezone_offset();
    -(offset as i32)
}
