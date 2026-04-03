use crate::components::edit_set_modal::EditSetModal;
use crate::models::{ExerciseMetadata, HistorySet};
use crate::state::{Database, WorkoutState, WorkoutStateManager};
use dioxus::prelude::*;
use wasm_bindgen::prelude::*;
use web_sys::{IntersectionObserver, IntersectionObserverEntry, IntersectionObserverInit};

// ── Day-grouping types ────────────────────────────────────────────────────────

#[derive(Clone, Debug, PartialEq)]
pub struct ExerciseGroup {
    pub exercise_id: i64,
    pub exercise_name: String,
    pub sets: Vec<HistorySet>,
}

/// A single calendar day with its exercise sub-groups.
#[derive(Clone, Debug, PartialEq)]
pub struct DayGroup {
    /// ISO date string, e.g. "2026-03-30"
    pub date_label: String,
    pub exercises: Vec<ExerciseGroup>,
}

/// Group a reverse-chronological slice of `HistorySet`s into calendar day groups.
///
/// `utc_offset_minutes` is the device's UTC offset in **minutes** (e.g. −300 for UTC−5).
/// Positive values are east of UTC.
///
/// The function preserves the reverse-chronological ordering within each day.
pub fn group_sets_by_day(sets: &[HistorySet], utc_offset_minutes: i32) -> Vec<DayGroup> {
    let mut days: Vec<DayGroup> = Vec::new();

    for set in sets {
        let date_label = ms_to_date_label(set.recorded_at, utc_offset_minutes);

        // Find or create the day group
        let day = match days.iter_mut().find(|d| d.date_label == date_label) {
            Some(d) => d,
            None => {
                days.push(DayGroup {
                    date_label: date_label.clone(),
                    exercises: Vec::new(),
                });
                days.last_mut().unwrap()
            }
        };

        // Find or create the exercise sub-group within the day
        match day
            .exercises
            .iter_mut()
            .find(|eg| eg.exercise_id == set.exercise_id)
        {
            Some(eg) => eg.sets.push(set.clone()),
            None => day.exercises.push(ExerciseGroup {
                exercise_id: set.exercise_id,
                exercise_name: set.exercise_name.clone(),
                sets: vec![set.clone()],
            }),
        }
    }

    days
}

/// Convert Unix milliseconds to a local-date string "YYYY-MM-DD".
fn ms_to_date_label(ms: f64, utc_offset_minutes: i32) -> String {
    // Shift the timestamp by the UTC offset so that integer-dividing by one day
    // gives the correct local date.
    let offset_ms = (utc_offset_minutes as f64) * 60_000.0;
    let local_ms = ms + offset_ms;

    // Total days since Unix epoch (1970-01-01)
    let days_since_epoch = (local_ms / 86_400_000.0).floor() as i64;

    // Convert days since epoch to a Gregorian (year, month, day) triple.
    // Algorithm from https://www.researchgate.net/publication/316558298
    let z = days_since_epoch + 719_468;
    let era = if z >= 0 { z } else { z - 146_096 } / 146_097;
    let doe = z - era * 146_097;
    let yoe = (doe - doe / 1460 + doe / 36524 - doe / 146_096) / 365;
    let y = yoe + era * 400;
    let doy = doe - (365 * yoe + yoe / 4 - yoe / 100);
    let mp = (5 * doy + 2) / 153;
    let d = doy - (153 * mp + 2) / 5 + 1;
    let m = if mp < 10 { mp + 3 } else { mp - 9 };
    let y = if m <= 2 { y + 1 } else { y };

    format!("{:04}-{:02}-{:02}", y, m, d)
}

// ── History scope toggle ──────────────────────────────────────────────────────

#[derive(Clone, Copy, PartialEq, Debug)]
pub enum HistoryScope {
    /// Show sets for a single exercise
    Exercise,
    /// Show sets across all exercises
    All,
}

// ── Component ─────────────────────────────────────────────────────────────────

const PAGE_SIZE: i64 = 20;

#[component]
pub fn HistoryView(
    state: WorkoutState,
    /// When `Some`, the view defaults to the per-exercise scope for this exercise.
    exercise_id: Option<i64>,
) -> Element {
    // Track exercise_id prop in a signal for reactivity in effects
    let mut eid_signal = use_signal(|| exercise_id);
    if *eid_signal.peek() != exercise_id {
        eid_signal.set(exercise_id);
    }

    let initial_scope = if exercise_id.is_some() {
        HistoryScope::Exercise
    } else {
        HistoryScope::All
    };

    let mut scope = use_signal(|| initial_scope);
    // Reset scope if exercise_id changes
    use_effect(move || {
        let eid = eid_signal();
        let current_scope = *scope.peek();

        // If we just navigated to a specific exercise, default to that exercise view
        if eid.is_some() && current_scope == HistoryScope::All {
            scope.set(HistoryScope::Exercise);
        }
        // If we navigated to all exercises (None), we MUST use All scope
        if eid.is_none() && current_scope == HistoryScope::Exercise {
            scope.set(HistoryScope::All);
        }
    });
    let mut sets = use_signal(Vec::<HistorySet>::new);
    let mut has_more = use_signal(|| true);
    let mut loading = use_signal(|| false);
    let mut exercise_name = use_signal(String::new);

    // Edit modal state
    let mut editing_set = use_signal(|| None::<HistorySet>);
    let mut editing_exercise = use_signal(|| None::<ExerciseMetadata>);

    // Fetch the exercise name for the toggle label
    {
        let state_ref = state;
        use_effect(move || {
            let eid = eid_signal();
            if let Some(id) = eid {
                let state_ref = state_ref;
                spawn(async move {
                    if let Some(db) = state_ref.database()
                        && let Ok(exercises) = db.get_exercises().await
                        && let Some(ex) = exercises.iter().find(|e| e.id == Some(id))
                    {
                        exercise_name.set(ex.name.clone());
                    }
                });
            }
        });
    }

    // Load the first page whenever scope, exercise_id, or active session sets change (AC #9)
    {
        let state_ref = state;
        use_effect(move || {
            let current_scope = scope();
            let eid = eid_signal();

            // Subscribe to current session's completed sets count to refresh on log (AC #9)
            let _log_trigger = state_ref
                .current_session()
                .map(|s| s.completed_sets.len())
                .unwrap_or(0);

            sets.set(Vec::new());
            has_more.set(true);

            spawn(async move {
                loading.set(true);
                if let Some(db) = state_ref.database() {
                    let page = fetch_page(&db, current_scope, eid, PAGE_SIZE, 0).await;
                    match page {
                        Ok(new_sets) => {
                            has_more.set(new_sets.len() as i64 == PAGE_SIZE);
                            sets.set(new_sets);
                        }
                        Err(e) => log::error!("Failed to load history: {}", e),
                    }
                }
                loading.set(false);
            });
        });
    }

    let load_more = {
        let state_ref = state;
        move || {
            if loading() || !has_more() {
                return;
            }
            let current_scope = scope();
            let eid = *eid_signal.peek();
            let offset = sets.read().len() as i64;
            spawn(async move {
                loading.set(true);
                if let Some(db) = state_ref.database() {
                    match fetch_page(&db, current_scope, eid, PAGE_SIZE, offset).await {
                        Ok(mut new_sets) => {
                            has_more.set(new_sets.len() as i64 == PAGE_SIZE);
                            sets.write().append(&mut new_sets);
                        }
                        Err(e) => log::error!("Failed to load more history: {}", e),
                    }
                }
                loading.set(false);
            });
        }
    };

    // Trigger signal for IntersectionObserver callback (AC #6)
    let mut load_trigger = use_signal(|| 0u32);
    use_effect(move || {
        if load_trigger() > 0 {
            load_more();
        }
    });

    // Get local UTC offset from the browser
    let utc_offset = get_utc_offset_minutes();
    let grouped = group_sets_by_day(&sets.read(), utc_offset);

    rsx! {
        div {
            class: "max-w-md mx-auto pb-10",
            "data-testid": "history-view",

            // Toggle bar
            div {
                class: "flex rounded-lg overflow-hidden border border-base-300 mb-6 sticky top-0 bg-base-100 z-10",
                "data-testid": "history-scope-toggle",

                if exercise_id.is_some() {
                    button {
                        class: if scope() == HistoryScope::Exercise {
                            "flex-1 py-2 text-sm font-semibold bg-primary text-primary-content"
                        } else {
                            "flex-1 py-2 text-sm font-semibold"
                        },
                        "data-testid": "toggle-exercise",
                        onclick: move |_| scope.set(HistoryScope::Exercise),
                        if exercise_name().is_empty() { "Exercise" } else { "{exercise_name}" }
                    }
                }

                button {
                    class: if scope() == HistoryScope::All {
                        "flex-1 py-2 text-sm font-semibold bg-primary text-primary-content"
                    } else {
                        "flex-1 py-2 text-sm font-semibold"
                    },
                    "data-testid": "toggle-all",
                    onclick: move |_| scope.set(HistoryScope::All),
                    "All Exercises"
                }
            }

            // Feed
            if grouped.is_empty() && !loading() {
                div {
                    class: "text-center text-base-content/50 py-16",
                    "data-testid": "history-empty",
                    "No workout history yet."
                }
            } else {
                div {
                    "data-testid": "history-feed",
                    for day in grouped.iter() {
                        div {
                            key: "{day.date_label}",
                            class: "mb-6",
                            "data-testid": "history-day-group",
                            h3 {
                                class: "text-sm font-bold text-base-content/50 uppercase tracking-widest mb-2",
                                "data-testid": "history-day-label",
                                "{day.date_label}"
                            }
                            for eg in day.exercises.iter() {
                                {
                                    let has_weighted = eg.sets.iter().any(|s| matches!(s.set_type, crate::models::SetType::Weighted { .. }));
                                    rsx! {
                                        div {
                                            key: "{eg.exercise_id}",
                                            class: "card bg-base-100 shadow mb-3",
                                            "data-testid": "history-exercise-group",
                                            div {
                                                class: "card-body p-4",
                                                h4 {
                                                    class: "font-bold text-base mb-2",
                                                    "{eg.exercise_name}"
                                                }
                                                table {
                                                    class: "table table-xs w-full",
                                                    "data-testid": "history-table",
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
                                                                "data-testid": "history-set-row",
                                                                class: "hover:bg-base-200 cursor-pointer transition-colors active:bg-base-300",
                                                                onclick: {
                                                                    let set = set.clone();
                                                                    let state_ref = state;
                                                                    move |_| {
                                                                        let set = set.clone();
                                                                        let state_ref = state_ref;
                                                                        spawn(async move {
                                                                            if let Some(db) = state_ref.database()
                                                                                && let Ok(exercises) = db.get_exercises().await
                                                                                && let Some(ex) = exercises.iter().find(|e| e.id == Some(set.exercise_id))
                                                                            {
                                                                                editing_exercise.set(Some(ex.clone()));
                                                                                editing_set.set(Some(set));
                                                                            }
                                                                        });
                                                                    }
                                                                },
                                                                td { "{set.set_number}" }
                                                                if has_weighted {
                                                                    if let crate::models::SetType::Weighted { weight } = set.set_type {
                                                                        td { "{crate::format::fmt_weight(weight)} kg" }
                                                                    } else {
                                                                        td { "—" }
                                                                    }
                                                                }
                                                                td { "{set.reps}" }
                                                                td { "{set.rpe:.1}" }
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

            // Loading spinner (initial load)
            if loading() && grouped.is_empty() {
                div {
                    class: "flex justify-center py-8",
                    div { class: "loading loading-spinner loading-md" }
                }
            }

            // Infinite-scroll sentinel (AC #6)
            if has_more() && !grouped.is_empty() {
                div {
                    id: "history-view-sentinel",
                    key: "sentinel-{sets.read().len()}",
                    class: "flex justify-center mt-4",
                    onmounted: move |el| {
                        // Skip the first observer callback, which fires synchronously on
                        // `observe()` whenever the sentinel is already in the viewport.
                        // Only react to subsequent callbacks triggered by actual scrolling.
                        let fire_count = std::rc::Rc::new(std::cell::Cell::new(0u32));
                        let fire_count_cb = fire_count.clone();
                        let callback = Closure::wrap(Box::new(move |entries: js_sys::Array, _observer: IntersectionObserver| {
                            fire_count_cb.set(fire_count_cb.get() + 1);
                            if fire_count_cb.get() == 1 {
                                return; // skip initial mount callback
                            }
                            if let Ok(entry) = entries.get(0).dyn_into::<IntersectionObserverEntry>()
                                && entry.is_intersecting()
                            {
                                load_trigger.with_mut(|v| *v += 1);
                            }
                        }) as Box<dyn FnMut(js_sys::Array, IntersectionObserver)>);

                        let options = IntersectionObserverInit::new();
                        options.set_threshold(&JsValue::from_f64(0.1));

                        if let Ok(observer) = IntersectionObserver::new_with_options(
                            callback.as_ref().unchecked_ref(),
                            &options,
                        )
                            && let Some(raw) = el.data().downcast::<web_sys::Element>()
                        {
                            observer.observe(raw);
                            callback.forget();
                        }
                    },
                    if loading() {
                        div { class: "loading loading-spinner loading-sm" }
                    } else {
                        // Fallback button for manual trigger or if observer fails
                        button {
                            class: "btn btn-ghost btn-sm",
                            "data-testid": "history-load-more",
                            onclick: move |_| load_trigger.with_mut(|v| *v += 1),
                            "Load more"
                        }
                    }
                }
            }
        }

        // Edit Modal
        if let (Some(set), Some(ex)) = (editing_set(), editing_exercise()) {
            EditSetModal {
                set: set.clone(),
                exercise: ex.clone(),
                on_cancel: move |_| {
                    editing_set.set(None);
                    editing_exercise.set(None);
                },
                on_save: {
                    let state_ref = state;
                    move |(reps, rpe, weight, recorded_at)| {
                        let state_ref = state_ref;
                        let set_id = set.id;
                        spawn(async move {
                            if let Some(db) = state_ref.database()
                                && db.update_set(set_id, reps, rpe, weight, recorded_at).await.is_ok()
                            {
                                // Update the set in the local signal to refresh the UI in place
                                sets.with_mut(|s| {
                                    if let Some(item) = s.iter_mut().find(|item| item.id == set_id) {
                                        item.reps = reps;
                                        item.rpe = rpe;
                                        item.recorded_at = recorded_at;
                                        if let Some(w) = weight {
                                            item.set_type = crate::models::SetType::Weighted { weight: w };
                                        } else {
                                            item.set_type = crate::models::SetType::Bodyweight;
                                        }
                                    }
                                });

                                // Persist to file
                                let _ = WorkoutStateManager::save_database(&state_ref).await;
                            }
                            editing_set.set(None);
                            editing_exercise.set(None);
                        });
                    }
                },
                on_delete: {
                    let state_ref = state;
                    move |set_id| {
                        let state_ref = state_ref;
                        spawn(async move {
                            if let Some(db) = state_ref.database()
                                && db.delete_set(set_id).await.is_ok()
                            {
                                // Remove the set from the local signal
                                sets.with_mut(|s| {
                                    s.retain(|item| item.id != set_id);
                                });

                                // Persist to file
                                let _ = WorkoutStateManager::save_database(&state_ref).await;
                            }
                            editing_set.set(None);
                            editing_exercise.set(None);
                        });
                    }
                }
            }
        }
    }
}

async fn fetch_page(
    db: &Database,
    scope: HistoryScope,
    exercise_id: Option<i64>,
    limit: i64,
    offset: i64,
) -> Result<Vec<HistorySet>, crate::state::DatabaseError> {
    match scope {
        HistoryScope::Exercise => {
            if let Some(id) = exercise_id {
                db.get_sets_for_exercise(id, limit, offset).await
            } else {
                db.get_all_sets_paginated(limit, offset).await
            }
        }
        HistoryScope::All => db.get_all_sets_paginated(limit, offset).await,
    }
}

fn get_utc_offset_minutes() -> i32 {
    let offset = js_sys::Date::new_0().get_timezone_offset();
    -(offset as i32)
}

// ── Unit tests for pure Rust day-grouping logic ───────────────────────────────

#[cfg(test)]
mod tests {
    use super::*;
    use crate::models::SetType;

    fn make_set(
        id: i64,
        exercise_id: i64,
        name: &str,
        set_number: u32,
        recorded_at: f64,
    ) -> HistorySet {
        HistorySet {
            id,
            exercise_id,
            exercise_name: name.to_string(),
            set_number,
            reps: 8,
            rpe: 7.0,
            set_type: SetType::Bodyweight,
            recorded_at,
        }
    }

    // 2026-01-01 00:00:00 UTC in ms
    const DAY1_START: f64 = 1735689600000.0;
    // 2026-01-02 00:00:00 UTC in ms
    const DAY2_START: f64 = DAY1_START + 86_400_000.0;

    // DAY1_START = 1735689600000 ms = 2025-01-01 00:00:00 UTC (verified)
    // DAY2_START = DAY1_START + 86_400_000 ms = 2025-01-02 00:00:00 UTC

    #[test]
    fn test_single_set_single_day() {
        let sets = vec![make_set(1, 1, "Bench Press", 1, DAY1_START + 3_600_000.0)];
        let groups = group_sets_by_day(&sets, 0);
        assert_eq!(groups.len(), 1);
        assert_eq!(groups[0].date_label, "2025-01-01");
        assert_eq!(groups[0].exercises.len(), 1);
        assert_eq!(groups[0].exercises[0].sets.len(), 1);
    }

    #[test]
    fn test_two_sets_same_day_same_exercise() {
        let sets = vec![
            make_set(2, 1, "Squat", 2, DAY1_START + 7_200_000.0),
            make_set(1, 1, "Squat", 1, DAY1_START + 3_600_000.0),
        ];
        let groups = group_sets_by_day(&sets, 0);
        assert_eq!(groups.len(), 1, "Should be one day group");
        assert_eq!(groups[0].exercises.len(), 1, "Should be one exercise group");
        assert_eq!(groups[0].exercises[0].sets.len(), 2);
    }

    #[test]
    fn test_two_exercises_same_day_share_one_date_header() {
        // AC #5: Multiple exercises on the same day appear under one date header
        let sets = vec![
            make_set(2, 2, "Deadlift", 1, DAY1_START + 7_200_000.0),
            make_set(1, 1, "Bench Press", 1, DAY1_START + 3_600_000.0),
        ];
        let groups = group_sets_by_day(&sets, 0);
        assert_eq!(groups.len(), 1, "One date header for same day");
        assert_eq!(
            groups[0].exercises.len(),
            2,
            "Two separate exercise sub-groups"
        );
    }

    #[test]
    fn test_two_sets_different_days() {
        // AC #4: reverse-chronological, grouped by day
        let sets = vec![
            make_set(2, 1, "Squat", 1, DAY2_START + 3_600_000.0),
            make_set(1, 1, "Squat", 1, DAY1_START + 3_600_000.0),
        ];
        let groups = group_sets_by_day(&sets, 0);
        assert_eq!(groups.len(), 2);
        assert_eq!(groups[0].date_label, "2025-01-02");
        assert_eq!(groups[1].date_label, "2025-01-01");
    }

    #[test]
    fn test_timezone_offset_shifts_day_boundary() {
        // A set at 2025-01-01 23:00 UTC is 2025-01-02 01:00 in UTC+2
        let set_ms = DAY1_START + 23.0 * 3_600_000.0;
        let sets = vec![make_set(1, 1, "Press", 1, set_ms)];

        let utc_groups = group_sets_by_day(&sets, 0);
        assert_eq!(utc_groups[0].date_label, "2025-01-01");

        let east2_groups = group_sets_by_day(&sets, 120); // UTC+2
        assert_eq!(east2_groups[0].date_label, "2025-01-02");
    }

    #[test]
    fn test_empty_input() {
        let groups = group_sets_by_day(&[], 0);
        assert!(groups.is_empty());
    }

    #[test]
    fn test_ms_to_date_label_known_date() {
        // DAY1_START = 2025-01-01 00:00:00 UTC
        // DAY2_START = 2025-01-02 00:00:00 UTC
        assert_eq!(ms_to_date_label(DAY1_START, 0), "2025-01-01");
        assert_eq!(ms_to_date_label(DAY2_START, 0), "2025-01-02");
    }
}
