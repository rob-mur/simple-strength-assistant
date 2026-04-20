use cucumber::World;

#[path = "steps/sync_edge_case_steps.rs"]
mod sync_edge_case_steps;

#[tokio::test]
async fn run_sync_edge_case_tests() {
    sync_edge_case_steps::SyncEdgeCaseWorld::cucumber()
        .run("tests/features/sync_edge_cases.feature")
        .await;
}
