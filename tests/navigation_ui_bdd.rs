use cucumber::World;

#[path = "steps/navigation_steps.rs"]
mod navigation_steps;

#[tokio::test]
async fn run_navigation_ui_tests() {
    navigation_steps::NavigationWorld::cucumber()
        .run("tests/features/navigation_ui.feature")
        .await;
}
