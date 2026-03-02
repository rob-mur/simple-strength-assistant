use cucumber::{World, given, then, when};

#[derive(Debug, Default, World)]
pub struct ExerciseListWorld {
    // Basic state for the world, will be expanded in next phases
}

#[given("the app is loaded")]
async fn app_is_loaded(_world: &mut ExerciseListWorld) {}

#[given("there are no exercises in the database")]
async fn no_exercises(_world: &mut ExerciseListWorld) {}

#[when("I navigate to the Library tab")]
async fn navigate_to_library(_world: &mut ExerciseListWorld) {}

#[then(regex = r#"^I should see the "([^"]*)" empty state message$"#)]
async fn should_see_empty_state_message(_world: &mut ExerciseListWorld, _message: String) {}

#[given("the following exercises exist:")]
async fn following_exercises_exist(_world: &mut ExerciseListWorld) {}

#[then(regex = r#"^I should see "([^"]*)" in the exercise list$"#)]
async fn should_see_exercise_in_list(_world: &mut ExerciseListWorld, _exercise: String) {}

#[then(regex = r#"^the "([^"]*)" exercise should have a "([^"]*)" badge$"#)]
async fn exercise_should_have_badge(
    _world: &mut ExerciseListWorld,
    _exercise: String,
    _badge: String,
) {
}

#[given("the user is on the Library tab")]
async fn user_is_on_library_tab(_world: &mut ExerciseListWorld) {}

#[given("the database contains standard exercises")]
async fn database_contains_standard_exercises(_world: &mut ExerciseListWorld) {}

#[then("the user should see a list of exercises")]
async fn user_should_see_list_of_exercises(_world: &mut ExerciseListWorld) {}

#[then("each exercise should display its name and type badge")]
async fn each_exercise_should_display_name_and_badge(_world: &mut ExerciseListWorld) {}

#[tokio::test]
async fn run_exercise_list_tests() {
    ExerciseListWorld::cucumber()
        .run("tests/features/exercise_list.feature")
        .await;
}
