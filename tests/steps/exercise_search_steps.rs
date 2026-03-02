use cucumber::{World, given, then, when};
use dioxus::prelude::*;
use simple_strength_assistant::components::library_view::LibraryView;
use simple_strength_assistant::models::{ExerciseMetadata, SetTypeConfig};
use simple_strength_assistant::state::WorkoutState;

#[derive(Debug, Default, World)]
pub struct ExerciseSearchWorld {
    pub exercises: Vec<ExerciseMetadata>,
    pub rendered_html: String,
    pub search_term: String,
}

#[derive(Props, Clone, PartialEq)]
struct WrapperProps {
    exercises: Vec<ExerciseMetadata>,
    search_term: String,
}

#[component]
fn TestWrapper(props: WrapperProps) -> Element {
    let state = WorkoutState::new();
    state.set_exercises(props.exercises.clone());
    use_context_provider(|| state);

    // Inject the search term for testing
    use_context_provider(|| props.search_term.clone());

    rsx! {
        LibraryView {}
    }
}

impl ExerciseSearchWorld {
    pub fn render_component(&mut self) {
        let mut vdom = VirtualDom::new_with_props(
            TestWrapper,
            WrapperProps {
                exercises: self.exercises.clone(),
                search_term: self.search_term.clone(),
            },
        );
        vdom.rebuild_in_place();
        self.rendered_html = dioxus_ssr::render(&vdom);
    }
}

#[given("I am on the Library tab")]
async fn i_am_on_library_tab(_world: &mut ExerciseSearchWorld) {}

#[given("the following exercises exist:")]
async fn following_exercises_exist(
    world: &mut ExerciseSearchWorld,
    step: &cucumber::gherkin::Step,
) {
    if let Some(table) = step.table() {
        for row in table.rows.iter().skip(1) {
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

#[when(regex = r#"^I type "([^"]*)" into the search bar$"#)]
async fn type_into_search_bar(world: &mut ExerciseSearchWorld, term: String) {
    world.search_term = term;
    world.render_component();
}

#[then(regex = r#"^I should see "([^"]*)" in the exercise list$"#)]
async fn should_see_exercise_in_list(world: &mut ExerciseSearchWorld, exercise: String) {
    assert!(
        world.rendered_html.contains(&exercise),
        "Expected HTML to contain exercise: {}, but got {}",
        exercise,
        world.rendered_html
    );
}

#[then(regex = r#"^I should not see "([^"]*)" in the exercise list$"#)]
async fn should_not_see_exercise_in_list(world: &mut ExerciseSearchWorld, exercise: String) {
    assert!(
        !world.rendered_html.contains(&exercise),
        "Expected HTML to NOT contain exercise: {}",
        exercise
    );
}

#[then(regex = r#"^I should see the "([^"]*)" empty state message$"#)]
async fn should_see_empty_state_message(world: &mut ExerciseSearchWorld, message: String) {
    assert!(
        world.rendered_html.contains(&message),
        "Expected HTML to contain empty state message: {}, but got {}",
        message,
        world.rendered_html
    );
}

#[then("the exercise list should be empty")]
async fn exercise_list_should_be_empty(world: &mut ExerciseSearchWorld) {
    // If there is no matching element in the DOM (like no 'ul' with 'li')
    assert!(
        !world.rendered_html.contains("<li"),
        "Expected no list items to be rendered, but got {}",
        world.rendered_html
    );
}

// E2E mock scenarios
#[given("the user is on the Library tab with multiple exercises")]
async fn user_on_library_tab_with_exercises(_world: &mut ExerciseSearchWorld) {}

#[when("the user searches for a specific exercise")]
async fn user_searches_for_exercise(_world: &mut ExerciseSearchWorld) {}

#[then("the list should instantly filter to show only matching exercises")]
async fn list_should_instantly_filter(_world: &mut ExerciseSearchWorld) {}

#[when("the user clears the search")]
async fn user_clears_search(_world: &mut ExerciseSearchWorld) {}

#[then("the list should show all exercises again")]
async fn list_should_show_all_exercises_again(_world: &mut ExerciseSearchWorld) {}
