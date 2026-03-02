use crate::models::SetTypeConfig;
use crate::state::WorkoutState;
use dioxus::prelude::*;

#[component]
pub fn LibraryView() -> Element {
    let workout_state = consume_context::<WorkoutState>();
    // Allow injecting a search query context for easier unit testing
    let test_query = try_consume_context::<String>();
    let mut search_query = use_signal(|| test_query.unwrap_or_default());
    let exercises = workout_state.exercises();

    let filtered_exercises: Vec<_> = exercises
        .iter()
        .filter(|e| {
            e.name
                .to_lowercase()
                .contains(&search_query().to_lowercase())
        })
        .collect();

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
                    if !exercises.is_empty() {
                        div {
                            class: "form-control w-full mb-4",
                            input {
                                r#type: "text",
                                placeholder: "Search exercises...",
                                class: "input input-bordered w-full",
                                value: "{search_query}",
                                oninput: move |evt| search_query.set(evt.value())
                            }
                        }
                    }
                    if exercises.is_empty() {
                        div {
                            class: "text-center py-8",
                            p {
                                class: "text-base-content/70",
                                "No exercises yet. Add exercises during your first workout."
                            }
                        }
                    } else if filtered_exercises.is_empty() {
                        div {
                            class: "text-center py-8",
                            p {
                                class: "text-base-content/70",
                                "No matching exercises"
                            }
                        }
                    } else {
                        ul {
                            class: "menu bg-base-200 w-full rounded-box",
                            for exercise in filtered_exercises {
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
