use crate::models::SetTypeConfig;
use crate::state::WorkoutState;
use dioxus::prelude::*;

#[component]
pub fn LibraryView() -> Element {
    let workout_state = consume_context::<WorkoutState>();
    let exercises = workout_state.exercises();

    rsx! {
        div {
            class: "max-w-2xl mx-auto",
            div {
                class: "card bg-base-100 shadow-xl",
                div {
                    class: "card-body",
                    h2 {
                        class: "card-title text-2xl mb-4 justify-center",
                        "Exercise Library"
                    }
                    if exercises.is_empty() {
                        div {
                            class: "text-center py-8",
                            p {
                                class: "text-base-content/70",
                                "No exercises yet. Add exercises during your first workout."
                            }
                        }
                    } else {
                        ul {
                            class: "menu bg-base-200 w-full rounded-box",
                            for exercise in exercises.iter() {
                                li {
                                    class: "mb-2",
                                    a {
                                        class: "flex justify-between items-center p-4",
                                        span {
                                            class: "font-semibold text-lg",
                                            "{exercise.name}"
                                        }
                                        {
                                            match exercise.set_type_config {
                                                SetTypeConfig::Weighted { .. } => rsx! { span { class: "badge badge-primary", "Weighted" } },
                                                SetTypeConfig::Bodyweight => rsx! { span { class: "badge badge-secondary", "Bodyweight" } },
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
