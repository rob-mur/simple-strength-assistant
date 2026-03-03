use cucumber::World;

#[path = "steps/library_steps.rs"]
mod library_steps;

#[tokio::test]
async fn run_library_management_tests() {
    library_steps::LibraryWorld::cucumber()
        .run("tests/features/library_management.feature")
        .await;
}
