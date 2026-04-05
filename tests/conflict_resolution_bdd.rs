use cucumber::World;

#[path = "steps/conflict_resolution_steps.rs"]
mod conflict_resolution_steps;

#[tokio::test]
async fn run_conflict_resolution_tests() {
    conflict_resolution_steps::ConflictResolutionWorld::cucumber()
        .run("tests/features/conflict_resolution.feature")
        .await;
}
