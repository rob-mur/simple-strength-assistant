use cucumber::World;

#[path = "steps/sync_status_steps.rs"]
mod sync_status_steps;

#[tokio::test]
async fn run_sync_status_indicator_tests() {
    sync_status_steps::SyncStatusWorld::cucumber()
        .run("tests/features/sync_status_indicator.feature")
        .await;
}
