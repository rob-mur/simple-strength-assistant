use cucumber::{World, given, then, when};
use dioxus::prelude::*;
use simple_strength_assistant::components::sync_status_indicator::SyncStatusIndicator;
use simple_strength_assistant::state::SyncStatus;

#[derive(Debug, Default, World)]
pub struct SyncStatusWorld {
    pub sync_status: SyncStatus,
    pub rendered_html: String,
}

#[derive(Props, Clone, PartialEq)]
struct WrapperProps {
    status: SyncStatus,
}

#[component]
fn TestWrapper(props: WrapperProps) -> Element {
    rsx! {
        div {
            "data-testid": "header",
            SyncStatusIndicator { status: props.status }
        }
        div {
            "data-testid": "main-content",
            p { "Workout UI" }
        }
    }
}

impl SyncStatusWorld {
    pub fn render_component(&mut self) {
        let mut vdom = VirtualDom::new_with_props(
            TestWrapper,
            WrapperProps {
                status: self.sync_status,
            },
        );
        vdom.rebuild_in_place();
        self.rendered_html = dioxus_ssr::render(&vdom);
    }
}

// ── Given steps ───────────────────────────────────────────────────────────────

#[given("the app is initialized")]
async fn step_app_initialized(world: &mut SyncStatusWorld) {
    world.sync_status = SyncStatus::Idle;
    world.render_component();
}

#[given("no sync is configured")]
async fn step_no_sync_configured(world: &mut SyncStatusWorld) {
    world.sync_status = SyncStatus::Idle;
    world.render_component();
}

#[given(expr = "the sync status is {string}")]
async fn step_sync_status_set(world: &mut SyncStatusWorld, status: String) {
    world.sync_status = match status.as_str() {
        "never synced" => SyncStatus::NeverSynced,
        "syncing" => SyncStatus::Syncing,
        "up to date" => SyncStatus::UpToDate,
        "error" => SyncStatus::Error,
        other => panic!("Unknown sync status: {other}"),
    };
    world.render_component();
}

// ── When steps ────────────────────────────────────────────────────────────────

#[when("I view the app")]
async fn step_view_app(world: &mut SyncStatusWorld) {
    if world.rendered_html.is_empty() {
        world.render_component();
    }
}

// ── Then steps ────────────────────────────────────────────────────────────────

#[then("I should see the sync status indicator")]
async fn step_indicator_visible(world: &mut SyncStatusWorld) {
    assert!(
        world
            .rendered_html
            .contains("data-testid=\"sync-status-indicator\""),
        "Expected sync-status-indicator in rendered HTML.\nHTML: {}",
        world.rendered_html
    );
}

#[then(expr = "the sync status indicator should show {string}")]
async fn step_indicator_shows(world: &mut SyncStatusWorld, label: String) {
    assert!(
        world.rendered_html.contains(label.as_str()),
        "Expected indicator to show '{label}' but it was not found.\nHTML: {}",
        world.rendered_html
    );
}

#[then(expr = "the sync status data attribute should be {string}")]
async fn step_data_attribute(world: &mut SyncStatusWorld, attr_value: String) {
    let expected = format!("data-sync-status=\"{attr_value}\"");
    assert!(
        world.rendered_html.contains(expected.as_str()),
        "Expected data-sync-status=\"{attr_value}\" in HTML.\nHTML: {}",
        world.rendered_html
    );
}

#[then("the sync status indicator should be inside the header")]
async fn step_indicator_in_header(world: &mut SyncStatusWorld) {
    // The indicator should appear before (or within) the header test div,
    // and before the main content div.
    let header_pos = world
        .rendered_html
        .find("data-testid=\"header\"")
        .expect("header not found in HTML");
    let indicator_pos = world
        .rendered_html
        .find("data-testid=\"sync-status-indicator\"")
        .expect("sync-status-indicator not found in HTML");
    let main_pos = world
        .rendered_html
        .find("data-testid=\"main-content\"")
        .expect("main-content not found in HTML");

    assert!(
        indicator_pos > header_pos,
        "Indicator should appear after the header opening tag"
    );
    assert!(
        indicator_pos < main_pos,
        "Indicator should appear before the main content div"
    );
}

#[then("the main content area should be present")]
async fn step_main_content_present(world: &mut SyncStatusWorld) {
    assert!(
        world.rendered_html.contains("data-testid=\"main-content\""),
        "Expected main-content area in rendered HTML.\nHTML: {}",
        world.rendered_html
    );
}
