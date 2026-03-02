use cucumber::{World, given, then, when};

#[derive(Debug, Default, World)]
pub struct ExerciseSearchWorld {
    // Basic state for the world, will be expanded in next phases
}

#[given("I am on the Library tab")]
async fn i_am_on_library_tab(_world: &mut ExerciseSearchWorld) {}

#[given("the following exercises exist:")]
async fn following_exercises_exist(_world: &mut ExerciseSearchWorld) {}

#[when(regex = r#"^I type "([^"]*)" into the search bar$"#)]
async fn type_into_search_bar(_world: &mut ExerciseSearchWorld, _term: String) {}

#[then(regex = r#"^I should see "([^"]*)" in the exercise list$"#)]
async fn should_see_exercise_in_list(_world: &mut ExerciseSearchWorld, _exercise: String) {}

#[then(regex = r#"^I should not see "([^"]*)" in the exercise list$"#)]
async fn should_not_see_exercise_in_list(_world: &mut ExerciseSearchWorld, _exercise: String) {}

#[then(regex = r#"^I should see the "([^"]*)" empty state message$"#)]
async fn should_see_empty_state_message(_world: &mut ExerciseSearchWorld, _message: String) {}

#[then("the exercise list should be empty")]
async fn exercise_list_should_be_empty(_world: &mut ExerciseSearchWorld) {}

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

#[tokio::test]
async fn run_exercise_search_tests() {
    ExerciseSearchWorld::cucumber()
        .run("tests/features/exercise_search.feature")
        .await;
}
