use crate::models::{ContributionTier, ExerciseMuscleGroup};
use dioxus::prelude::*;

fn cycle_tier(tier: &ContributionTier) -> Option<ContributionTier> {
    match tier {
        ContributionTier::Primary => Some(ContributionTier::Secondary),
        ContributionTier::Secondary => Some(ContributionTier::Tertiary),
        ContributionTier::Tertiary => None,
    }
}

fn tier_label(tier: &ContributionTier) -> &'static str {
    match tier {
        ContributionTier::Primary => "Primary",
        ContributionTier::Secondary => "Secondary",
        ContributionTier::Tertiary => "Tertiary",
    }
}

fn tier_badge_class(tier: &ContributionTier) -> &'static str {
    match tier {
        ContributionTier::Primary => "badge badge-primary",
        ContributionTier::Secondary => "badge badge-secondary",
        ContributionTier::Tertiary => "badge badge-ghost",
    }
}

/// Card list showing the contribution tier for each selected muscle group.
///
/// Tapping a card cycles through Primary → Secondary → Tertiary → removed.
/// The parent owns selection state; this component emits changes.
#[component]
pub fn MuscleGroupTierCards(
    /// Current muscle group selections.
    selections: Vec<ExerciseMuscleGroup>,
    /// Called with the updated selection list when the user modifies a card.
    on_selection_change: EventHandler<Vec<ExerciseMuscleGroup>>,
) -> Element {
    if selections.is_empty() {
        return rsx! {
            p {
                "data-testid": "tier-cards-empty",
                class: "text-sm text-base-content/50 text-center py-2",
                "Tap a muscle region to add it"
            }
        };
    }

    rsx! {
        div {
            "data-testid": "tier-cards-list",
            class: "flex flex-wrap gap-2",
            for emg in selections.iter().cloned() {
                {
                    let sels = selections.clone();
                    let name = emg.muscle_group.to_string();
                    let badge_class = tier_badge_class(&emg.tier);
                    let label = tier_label(&emg.tier);
                    let muscle = emg.muscle_group.clone();
                    let tier = emg.tier.clone();

                    rsx! {
                        button {
                            key: "{name}",
                            "data-testid": "tier-card-{name}",
                            class: "flex items-center gap-2 px-3 py-2 rounded-lg bg-base-200 hover:bg-base-300 transition-colors cursor-pointer text-left",
                            onclick: {
                                let sels = sels.clone();
                                let muscle = muscle.clone();
                                let tier = tier.clone();
                                move |_| {
                                    let mut updated = sels.clone();
                                    if let Some(pos) = updated.iter().position(|s| s.muscle_group == muscle) {
                                        match cycle_tier(&tier) {
                                            Some(next_tier) => {
                                                updated[pos].tier = next_tier;
                                            }
                                            None => {
                                                updated.remove(pos);
                                            }
                                        }
                                    }
                                    on_selection_change.call(updated);
                                }
                            },
                            span {
                                class: "text-sm font-medium",
                                "{name}"
                            }
                            span {
                                class: "{badge_class} badge-sm",
                                "{label}"
                            }
                        }
                    }
                }
            }
        }
    }
}
