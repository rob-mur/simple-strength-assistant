use cucumber::World;

#[path = "steps/workout_steps.rs"]
mod workout_steps;

#[tokio::test]
async fn run_workout_flow_new_tests() {
    workout_steps::WorkoutWorld::cucumber()
        .run("tests/features/workout_flow_new.feature")
        .await;
}
