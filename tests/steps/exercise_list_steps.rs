use cucumber::{World, given, then, when};
use dioxus::prelude::*;
use dioxus_history::MemoryHistory;
use simple_strength_assistant::app::{Route, TabNavigationState};
use simple_strength_assistant::models::{ExerciseMetadata, SetTypeConfig};
use simple_strength_assistant::state::WorkoutState;

#[derive(Debug, Default, World)]
pub struct ExerciseListWorld {
    pub exercises: Vec<ExerciseMetadata>,
    pub active_tab: String,
    pub rendered_html: String,
}

#[derive(Props, Clone, PartialEq)]
struct WrapperProps {
    exercises: Vec<ExerciseMetadata>,
}

#[component]
fn TestWrapper(props: WrapperProps) -> Element {
    let state = WorkoutState::new();
    state.set_exercises(props.exercises.clone());
    use_context_provider(|| state);
    use_context_provider(|| TabNavigationState {
        last_workout_route: Signal::new(Route::WorkoutTab),
        last_library_route: Signal::new(Route::LibraryTab),
    });
    provide_context(
        std::rc::Rc::new(MemoryHistory::with_initial_path("/library"))
            as std::rc::Rc<dyn dioxus_history::History>,
    );

    rsx! {
        Router::<Route> {}
    }
}

impl ExerciseListWorld {
    pub fn render_component(&mut self) {
        let mut vdom = VirtualDom::new_with_props(
            TestWrapper,
            WrapperProps {
                exercises: self.exercises.clone(),
            },
        );
        vdom.rebuild_in_place();
        self.rendered_html = dioxus_ssr::render(&vdom);
    }
}

#[given("the app is loaded")]
async fn app_is_loaded(world: &mut ExerciseListWorld) {
    world.exercises.clear();
}

#[given("there are no exercises in the database")]
async fn no_exercises(world: &mut ExerciseListWorld) {
    world.exercises.clear();
}

#[when("I navigate to the Library tab")]
async fn navigate_to_library(world: &mut ExerciseListWorld) {
    world.active_tab = "Library".to_string();
    world.render_component();
}

#[then(regex = r#"^I should see the "([^"]*)" empty state message$"#)]
async fn should_see_empty_state_message(world: &mut ExerciseListWorld, message: String) {
    assert!(
        world.rendered_html.contains(&message),
        "Expected HTML to contain empty state message: {}, but got {}",
        message,
        world.rendered_html
    );
}

#[given("the following exercises exist:")]
async fn following_exercises_exist(world: &mut ExerciseListWorld, step: &cucumber::gherkin::Step) {
    if let Some(table) = step.table() {
        for row in table.rows.iter().skip(1) {
            // Skip header
            let r: &Vec<String> = row;
            if r.len() >= 2 {
                let name = &r[0];
                let ex_type = &r[1];
                let is_weighted = ex_type.to_lowercase() == "weighted";

                let config = if is_weighted {
                    SetTypeConfig::Weighted {
                        min_weight: 0.0,
                        increment: 2.5,
                    }
                } else {
                    SetTypeConfig::Bodyweight
                };

                world.exercises.push(ExerciseMetadata {
                    id: None,
                    name: name.clone(),
                    set_type_config: config,
                    min_reps: 1,
                    max_reps: None,
                });
            }
        }
    }
}

#[then(regex = r#"^I should see "([^"]*)" in the exercise list$"#)]
async fn should_see_exercise_in_list(world: &mut ExerciseListWorld, exercise: String) {
    let html_upper = world.rendered_html.to_uppercase();
    let exercise_upper = exercise.to_uppercase();
    assert!(
        html_upper.contains(&exercise_upper),
        "Expected HTML to contain exercise: {}, but got {}",
        exercise,
        world.rendered_html
    );
}

#[then(regex = r#"^the "([^"]*)" exercise should have a "([^"]*)" badge$"#)]
async fn exercise_should_have_badge(
    world: &mut ExerciseListWorld,
    exercise: String,
    badge: String,
) {
    let rendered_upper = world.rendered_html.to_uppercase();
    let exercise_upper = exercise.to_uppercase();
    assert!(
        rendered_upper.contains(&exercise_upper),
        "Expected HTML to contain exercise: {}",
        exercise
    );
    let badge_upper = badge.to_uppercase();
    assert!(
        rendered_upper.contains(&badge_upper),
        "Expected HTML to contain badge: {} (searched for {})",
        badge,
        badge_upper
    );
}

#[given("the user is on the Library tab")]
async fn user_is_on_library_tab(world: &mut ExerciseListWorld) {
    world.active_tab = "Library".to_string();
}

#[given("the database contains standard exercises")]
async fn database_contains_standard_exercises(world: &mut ExerciseListWorld) {
    world.exercises.push(ExerciseMetadata {
        id: None,
        name: "Squat".to_string(),
        set_type_config: SetTypeConfig::Weighted {
            min_weight: 20.0,
            increment: 2.5,
        },
        min_reps: 1,
        max_reps: None,
    });
    world.exercises.push(ExerciseMetadata {
        id: None,
        name: "Push-up".to_string(),
        set_type_config: SetTypeConfig::Bodyweight,
        min_reps: 1,
        max_reps: None,
    });
}

#[then("the user should see a list of exercises")]
async fn user_should_see_list_of_exercises(world: &mut ExerciseListWorld) {
    world.render_component();
    let html_upper = world.rendered_html.to_uppercase();
    assert!(html_upper.contains("SQUAT"));
    assert!(html_upper.contains("PUSH-UP"));
}

#[then("each exercise should display its name and type badge")]
async fn each_exercise_should_display_name_and_badge(world: &mut ExerciseListWorld) {
    assert!(world.rendered_html.contains("WEIGHTED"));
    assert!(world.rendered_html.contains("BODYWEIGHT"));
}
