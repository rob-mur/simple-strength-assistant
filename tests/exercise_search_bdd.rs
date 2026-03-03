use cucumber::World;

#[path = "steps/exercise_search_steps.rs"]
mod exercise_search_steps;

#[tokio::test]
async fn run_exercise_search_tests() {
    exercise_search_steps::ExerciseSearchWorld::cucumber()
        .run("tests/features/exercise_search.feature")
        .await;
}
