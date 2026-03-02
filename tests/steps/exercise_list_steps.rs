use cucumber::{World, given, then, when};
use dioxus::prelude::*;
use simple_strength_assistant::components::library_view::LibraryView;
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

    rsx! {
        LibraryView {}
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
                    name: name.clone(),
                    set_type_config: config,
                });
            }
        }
    }
}

#[then(regex = r#"^I should see "([^"]*)" in the exercise list$"#)]
async fn should_see_exercise_in_list(world: &mut ExerciseListWorld, exercise: String) {
    assert!(
        world.rendered_html.contains(&exercise),
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
    assert!(
        world.rendered_html.contains(&exercise),
        "Expected HTML to contain exercise: {}",
        exercise
    );
    assert!(
        world.rendered_html.contains(&badge),
        "Expected HTML to contain badge: {}",
        badge
    );
}

#[given("the user is on the Library tab")]
async fn user_is_on_library_tab(world: &mut ExerciseListWorld) {
    world.active_tab = "Library".to_string();
}

#[given("the database contains standard exercises")]
async fn database_contains_standard_exercises(world: &mut ExerciseListWorld) {
    world.exercises.push(ExerciseMetadata {
        name: "Squat".to_string(),
        set_type_config: SetTypeConfig::Weighted {
            min_weight: 20.0,
            increment: 2.5,
        },
    });
    world.exercises.push(ExerciseMetadata {
        name: "Push-up".to_string(),
        set_type_config: SetTypeConfig::Bodyweight,
    });
}

#[then("the user should see a list of exercises")]
async fn user_should_see_list_of_exercises(world: &mut ExerciseListWorld) {
    world.render_component();
    assert!(world.rendered_html.contains("Squat"));
    assert!(world.rendered_html.contains("Push-up"));
}

#[then("each exercise should display its name and type badge")]
async fn each_exercise_should_display_name_and_badge(world: &mut ExerciseListWorld) {
    assert!(world.rendered_html.contains("Weighted"));
    assert!(world.rendered_html.contains("Bodyweight"));
}
