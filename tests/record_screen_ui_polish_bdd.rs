use cucumber::World;

#[path = "steps/workout_steps.rs"]
mod workout_steps;

#[tokio::test]
async fn run_record_screen_ui_polish_tests() {
    workout_steps::WorkoutWorld::cucumber()
        .run("tests/features/record_screen_ui_polish.feature")
        .await;
}
