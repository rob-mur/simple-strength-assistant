use cucumber::World;

#[path = "steps/exercise_list_steps.rs"]
mod exercise_list_steps;

#[tokio::test]
async fn run_exercise_list_tests() {
    exercise_list_steps::ExerciseListWorld::cucumber()
        .run("tests/features/exercise_list.feature")
        .await;
}
