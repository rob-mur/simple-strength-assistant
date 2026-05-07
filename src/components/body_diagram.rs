use crate::models::{ContributionTier, ExerciseMuscleGroup, MuscleGroup};
use dioxus::prelude::*;

fn fill_for(muscle: &MuscleGroup, selections: &[ExerciseMuscleGroup]) -> &'static str {
    match selections.iter().find(|s| &s.muscle_group == muscle) {
        Some(s) => match s.tier {
            ContributionTier::Primary => "#3b82f6",
            ContributionTier::Secondary => "#93c5fd",
            ContributionTier::Tertiary => "#dbeafe",
        },
        None => "#d1d5db",
    }
}

fn toggle(
    muscle: MuscleGroup,
    mut selections: Vec<ExerciseMuscleGroup>,
) -> Vec<ExerciseMuscleGroup> {
    if let Some(pos) = selections.iter().position(|s| s.muscle_group == muscle) {
        selections.remove(pos);
    } else {
        selections.push(ExerciseMuscleGroup {
            exercise_id: String::new(),
            muscle_group: muscle,
            tier: ContributionTier::Primary,
        });
    }
    selections
}

/// Interactive SVG body diagram for selecting muscle groups.
///
/// The parent owns the selection state; this component only emits change events.
#[component]
pub fn BodyDiagram(
    /// Current muscle group selections.
    selections: Vec<ExerciseMuscleGroup>,
    /// Called with the updated selection list when the user taps a region.
    on_selection_change: EventHandler<Vec<ExerciseMuscleGroup>>,
) -> Element {
    let mut show_back = use_signal(|| false);

    let sel = selections.clone();

    rsx! {
        div {
            "data-testid": "body-diagram",
            class: "flex flex-col items-center gap-3",

            div {
                class: "join",
                button {
                    "data-testid": "body-diagram-front-btn",
                    class: if !show_back() { "join-item btn btn-sm btn-primary" } else { "join-item btn btn-sm btn-ghost" },
                    onclick: move |_| show_back.set(false),
                    "Front"
                }
                button {
                    "data-testid": "body-diagram-back-btn",
                    class: if show_back() { "join-item btn btn-sm btn-primary" } else { "join-item btn btn-sm btn-ghost" },
                    onclick: move |_| show_back.set(true),
                    "Back"
                }
            }

            if !show_back() {
                FrontDiagram {
                    selections: sel.clone(),
                    on_tap: move |muscle| {
                        on_selection_change.call(toggle(muscle, selections.clone()));
                    },
                }
            } else {
                BackDiagram {
                    selections: sel.clone(),
                    on_tap: move |muscle| {
                        on_selection_change.call(toggle(muscle, selections.clone()));
                    },
                }
            }
        }
    }
}

#[component]
fn FrontDiagram(
    selections: Vec<ExerciseMuscleGroup>,
    on_tap: EventHandler<MuscleGroup>,
) -> Element {
    let chest_fill = fill_for(&MuscleGroup::Chest, &selections);
    let shoulders_fill = fill_for(&MuscleGroup::Shoulders, &selections);
    let biceps_fill = fill_for(&MuscleGroup::Biceps, &selections);
    let forearms_fill = fill_for(&MuscleGroup::Forearms, &selections);
    let core_fill = fill_for(&MuscleGroup::Core, &selections);
    let quads_fill = fill_for(&MuscleGroup::Quads, &selections);

    rsx! {
        svg {
            xmlns: "http://www.w3.org/2000/svg",
            view_box: "0 0 200 400",
            width: "160",
            height: "320",
            "aria-label": "Front body diagram",

            // ── Head ───────────────────────────────────────────────────────
            circle {
                cx: "100",
                cy: "30",
                r: "22",
                fill: "#f3f4f6",
                stroke: "#9ca3af",
                stroke_width: "1.5",
            }

            // ── Neck ───────────────────────────────────────────────────────
            rect {
                x: "88",
                y: "52",
                width: "24",
                height: "16",
                fill: "#f3f4f6",
                stroke: "#9ca3af",
                stroke_width: "1",
            }

            // ── Left Shoulder ──────────────────────────────────────────────
            polygon {
                "data-testid": "region-Shoulders",
                points: "28,68 68,68 68,100 24,100",
                fill: "{shoulders_fill}",
                stroke: "#6b7280",
                stroke_width: "1.5",
                style: "cursor: pointer",
                onclick: {
                    let on_tap = on_tap;
                    move |_| on_tap.call(MuscleGroup::Shoulders)
                },
            }

            // ── Right Shoulder ─────────────────────────────────────────────
            polygon {
                points: "132,68 172,68 176,100 132,100",
                fill: "{shoulders_fill}",
                stroke: "#6b7280",
                stroke_width: "1.5",
                style: "cursor: pointer",
                onclick: move |_| on_tap.call(MuscleGroup::Shoulders),
            }

            // ── Chest ──────────────────────────────────────────────────────
            rect {
                "data-testid": "region-Chest",
                x: "68",
                y: "68",
                width: "64",
                height: "50",
                fill: "{chest_fill}",
                stroke: "#6b7280",
                stroke_width: "1.5",
                style: "cursor: pointer",
                onclick: {
                    let on_tap = on_tap;
                    move |_| on_tap.call(MuscleGroup::Chest)
                },
            }

            // ── Core ───────────────────────────────────────────────────────
            rect {
                "data-testid": "region-Core",
                x: "68",
                y: "118",
                width: "64",
                height: "65",
                fill: "{core_fill}",
                stroke: "#6b7280",
                stroke_width: "1.5",
                style: "cursor: pointer",
                onclick: {
                    let on_tap = on_tap;
                    move |_| on_tap.call(MuscleGroup::Core)
                },
            }

            // ── Left Bicep ─────────────────────────────────────────────────
            polygon {
                "data-testid": "region-Biceps",
                points: "20,100 68,100 65,155 15,155",
                fill: "{biceps_fill}",
                stroke: "#6b7280",
                stroke_width: "1.5",
                style: "cursor: pointer",
                onclick: {
                    let on_tap = on_tap;
                    move |_| on_tap.call(MuscleGroup::Biceps)
                },
            }

            // ── Right Bicep ────────────────────────────────────────────────
            polygon {
                points: "132,100 180,100 185,155 135,155",
                fill: "{biceps_fill}",
                stroke: "#6b7280",
                stroke_width: "1.5",
                style: "cursor: pointer",
                onclick: {
                    let on_tap = on_tap;
                    move |_| on_tap.call(MuscleGroup::Biceps)
                },
            }

            // ── Left Forearm ───────────────────────────────────────────────
            polygon {
                "data-testid": "region-Forearms",
                points: "15,155 65,155 60,205 10,205",
                fill: "{forearms_fill}",
                stroke: "#6b7280",
                stroke_width: "1.5",
                style: "cursor: pointer",
                onclick: {
                    let on_tap = on_tap;
                    move |_| on_tap.call(MuscleGroup::Forearms)
                },
            }

            // ── Right Forearm ──────────────────────────────────────────────
            polygon {
                points: "135,155 185,155 190,205 140,205",
                fill: "{forearms_fill}",
                stroke: "#6b7280",
                stroke_width: "1.5",
                style: "cursor: pointer",
                onclick: {
                    let on_tap = on_tap;
                    move |_| on_tap.call(MuscleGroup::Forearms)
                },
            }

            // ── Left Quad ──────────────────────────────────────────────────
            polygon {
                "data-testid": "region-Quads",
                points: "68,183 97,183 93,305 62,305",
                fill: "{quads_fill}",
                stroke: "#6b7280",
                stroke_width: "1.5",
                style: "cursor: pointer",
                onclick: {
                    let on_tap = on_tap;
                    move |_| on_tap.call(MuscleGroup::Quads)
                },
            }

            // ── Right Quad ─────────────────────────────────────────────────
            polygon {
                points: "103,183 132,183 138,305 107,305",
                fill: "{quads_fill}",
                stroke: "#6b7280",
                stroke_width: "1.5",
                style: "cursor: pointer",
                onclick: move |_| on_tap.call(MuscleGroup::Quads),
            }

            // ── Region Labels ──────────────────────────────────────────────
            text { x: "100", y: "98",  text_anchor: "middle", font_size: "9", fill: "#374151", pointer_events: "none", "Chest" }
            text { x: "100", y: "155", text_anchor: "middle", font_size: "9", fill: "#374151", pointer_events: "none", "Core" }
            text { x: "44",  y: "84",  text_anchor: "middle", font_size: "8", fill: "#374151", pointer_events: "none", "Shoulder" }
            text { x: "156", y: "84",  text_anchor: "middle", font_size: "8", fill: "#374151", pointer_events: "none", "Shoulder" }
            text { x: "42",  y: "132", text_anchor: "middle", font_size: "8", fill: "#374151", pointer_events: "none", "Bicep" }
            text { x: "158", y: "132", text_anchor: "middle", font_size: "8", fill: "#374151", pointer_events: "none", "Bicep" }
            text { x: "38",  y: "183", text_anchor: "middle", font_size: "7", fill: "#374151", pointer_events: "none", "Forearm" }
            text { x: "162", y: "183", text_anchor: "middle", font_size: "7", fill: "#374151", pointer_events: "none", "Forearm" }
            text { x: "78",  y: "248", text_anchor: "middle", font_size: "8", fill: "#374151", pointer_events: "none", "Quad" }
            text { x: "122", y: "248", text_anchor: "middle", font_size: "8", fill: "#374151", pointer_events: "none", "Quad" }
        }
    }
}

#[component]
fn BackDiagram(selections: Vec<ExerciseMuscleGroup>, on_tap: EventHandler<MuscleGroup>) -> Element {
    let back_fill = fill_for(&MuscleGroup::Back, &selections);
    let traps_fill = fill_for(&MuscleGroup::Traps, &selections);
    let shoulders_fill = fill_for(&MuscleGroup::Shoulders, &selections);
    let triceps_fill = fill_for(&MuscleGroup::Triceps, &selections);
    let glutes_fill = fill_for(&MuscleGroup::Glutes, &selections);
    let hamstrings_fill = fill_for(&MuscleGroup::Hamstrings, &selections);
    let calves_fill = fill_for(&MuscleGroup::Calves, &selections);

    rsx! {
        svg {
            xmlns: "http://www.w3.org/2000/svg",
            view_box: "0 0 200 400",
            width: "160",
            height: "320",
            "aria-label": "Back body diagram",

            // ── Head ───────────────────────────────────────────────────────
            circle {
                cx: "100",
                cy: "30",
                r: "22",
                fill: "#f3f4f6",
                stroke: "#9ca3af",
                stroke_width: "1.5",
            }

            // ── Neck ───────────────────────────────────────────────────────
            rect {
                x: "88",
                y: "52",
                width: "24",
                height: "16",
                fill: "#f3f4f6",
                stroke: "#9ca3af",
                stroke_width: "1",
            }

            // ── Traps ──────────────────────────────────────────────────────
            polygon {
                "data-testid": "region-Traps",
                points: "68,52 132,52 136,95 64,95",
                fill: "{traps_fill}",
                stroke: "#6b7280",
                stroke_width: "1.5",
                style: "cursor: pointer",
                onclick: {
                    let on_tap = on_tap;
                    move |_| on_tap.call(MuscleGroup::Traps)
                },
            }

            // ── Left Shoulder (back) ───────────────────────────────────────
            polygon {
                "data-testid": "region-Shoulders",
                points: "28,68 68,68 64,100 24,100",
                fill: "{shoulders_fill}",
                stroke: "#6b7280",
                stroke_width: "1.5",
                style: "cursor: pointer",
                onclick: {
                    let on_tap = on_tap;
                    move |_| on_tap.call(MuscleGroup::Shoulders)
                },
            }

            // ── Right Shoulder (back) ──────────────────────────────────────
            polygon {
                points: "132,68 172,68 176,100 136,100",
                fill: "{shoulders_fill}",
                stroke: "#6b7280",
                stroke_width: "1.5",
                style: "cursor: pointer",
                onclick: {
                    let on_tap = on_tap;
                    move |_| on_tap.call(MuscleGroup::Shoulders)
                },
            }

            // ── Back (lats) ────────────────────────────────────────────────
            polygon {
                "data-testid": "region-Back",
                points: "64,95 136,95 132,183 68,183",
                fill: "{back_fill}",
                stroke: "#6b7280",
                stroke_width: "1.5",
                style: "cursor: pointer",
                onclick: {
                    let on_tap = on_tap;
                    move |_| on_tap.call(MuscleGroup::Back)
                },
            }

            // ── Left Tricep ────────────────────────────────────────────────
            polygon {
                "data-testid": "region-Triceps",
                points: "20,100 64,100 62,155 16,155",
                fill: "{triceps_fill}",
                stroke: "#6b7280",
                stroke_width: "1.5",
                style: "cursor: pointer",
                onclick: {
                    let on_tap = on_tap;
                    move |_| on_tap.call(MuscleGroup::Triceps)
                },
            }

            // ── Right Tricep ───────────────────────────────────────────────
            polygon {
                points: "136,100 180,100 184,155 138,155",
                fill: "{triceps_fill}",
                stroke: "#6b7280",
                stroke_width: "1.5",
                style: "cursor: pointer",
                onclick: {
                    let on_tap = on_tap;
                    move |_| on_tap.call(MuscleGroup::Triceps)
                },
            }

            // ── Glutes ─────────────────────────────────────────────────────
            polygon {
                "data-testid": "region-Glutes",
                points: "68,183 132,183 136,225 64,225",
                fill: "{glutes_fill}",
                stroke: "#6b7280",
                stroke_width: "1.5",
                style: "cursor: pointer",
                onclick: {
                    let on_tap = on_tap;
                    move |_| on_tap.call(MuscleGroup::Glutes)
                },
            }

            // ── Left Hamstring ─────────────────────────────────────────────
            polygon {
                "data-testid": "region-Hamstrings",
                points: "64,225 97,225 93,305 60,305",
                fill: "{hamstrings_fill}",
                stroke: "#6b7280",
                stroke_width: "1.5",
                style: "cursor: pointer",
                onclick: {
                    let on_tap = on_tap;
                    move |_| on_tap.call(MuscleGroup::Hamstrings)
                },
            }

            // ── Right Hamstring ────────────────────────────────────────────
            polygon {
                points: "103,225 136,225 140,305 107,305",
                fill: "{hamstrings_fill}",
                stroke: "#6b7280",
                stroke_width: "1.5",
                style: "cursor: pointer",
                onclick: {
                    let on_tap = on_tap;
                    move |_| on_tap.call(MuscleGroup::Hamstrings)
                },
            }

            // ── Left Calf ──────────────────────────────────────────────────
            polygon {
                "data-testid": "region-Calves",
                points: "60,305 93,305 89,375 56,375",
                fill: "{calves_fill}",
                stroke: "#6b7280",
                stroke_width: "1.5",
                style: "cursor: pointer",
                onclick: {
                    let on_tap = on_tap;
                    move |_| on_tap.call(MuscleGroup::Calves)
                },
            }

            // ── Right Calf ─────────────────────────────────────────────────
            polygon {
                points: "107,305 140,305 144,375 111,375",
                fill: "{calves_fill}",
                stroke: "#6b7280",
                stroke_width: "1.5",
                style: "cursor: pointer",
                onclick: move |_| on_tap.call(MuscleGroup::Calves),
            }

            // ── Region Labels ──────────────────────────────────────────────
            text { x: "100", y: "78",  text_anchor: "middle", font_size: "8",  fill: "#374151", pointer_events: "none", "Traps" }
            text { x: "44",  y: "84",  text_anchor: "middle", font_size: "8",  fill: "#374151", pointer_events: "none", "Shoulder" }
            text { x: "156", y: "84",  text_anchor: "middle", font_size: "8",  fill: "#374151", pointer_events: "none", "Shoulder" }
            text { x: "100", y: "143", text_anchor: "middle", font_size: "9",  fill: "#374151", pointer_events: "none", "Back" }
            text { x: "42",  y: "132", text_anchor: "middle", font_size: "8",  fill: "#374151", pointer_events: "none", "Tricep" }
            text { x: "158", y: "132", text_anchor: "middle", font_size: "8",  fill: "#374151", pointer_events: "none", "Tricep" }
            text { x: "100", y: "207", text_anchor: "middle", font_size: "8",  fill: "#374151", pointer_events: "none", "Glutes" }
            text { x: "78",  y: "268", text_anchor: "middle", font_size: "8",  fill: "#374151", pointer_events: "none", "Hamstring" }
            text { x: "122", y: "268", text_anchor: "middle", font_size: "8",  fill: "#374151", pointer_events: "none", "Hamstring" }
            text { x: "75",  y: "345", text_anchor: "middle", font_size: "8",  fill: "#374151", pointer_events: "none", "Calf" }
            text { x: "125", y: "345", text_anchor: "middle", font_size: "8",  fill: "#374151", pointer_events: "none", "Calf" }
        }
    }
}
