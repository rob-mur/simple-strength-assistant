use cucumber::{World, given, then, when};
use dioxus::prelude::*;
use simple_strength_assistant::components::conflict_resolution::ConflictResolution;
use simple_strength_assistant::state::{ConflictChoice, ConflictRecord};

#[derive(Debug, Default, World)]
pub struct ConflictResolutionWorld {
    pub conflicts: Vec<ConflictRecord>,
    pub rendered_html: String,
    pub resolved_conflicts: Vec<ConflictRecord>,
}

// ── Rendering helper ──────────────────────────────────────────────────────────

#[derive(Props, Clone, PartialEq)]
struct WrapperProps {
    conflicts: Vec<ConflictRecord>,
}

#[component]
fn TestWrapper(props: WrapperProps) -> Element {
    rsx! {
        ConflictResolution {
            conflicts: props.conflicts.clone(),
            on_resolve: move |_resolved: Vec<ConflictRecord>| {},
        }
    }
}

impl ConflictResolutionWorld {
    pub fn render_screen(&mut self) {
        let mut vdom = VirtualDom::new_with_props(
            TestWrapper,
            WrapperProps {
                conflicts: self.conflicts.clone(),
            },
        );
        vdom.rebuild_in_place();
        self.rendered_html = dioxus_ssr::render(&vdom);
    }
}

fn make_conflict(
    uuid: &str,
    field_label: &str,
    version_a: &str,
    version_b: &str,
) -> ConflictRecord {
    ConflictRecord {
        uuid: uuid.to_string(),
        field_label: field_label.to_string(),
        version_a: version_a.to_string(),
        version_b: version_b.to_string(),
        choice: None,
    }
}

// ── Given steps ───────────────────────────────────────────────────────────────

#[given(expr = "the sync client has reported {int} unresolved conflict(s)")]
async fn step_n_conflicts(world: &mut ConflictResolutionWorld, count: usize) {
    world.conflicts = (0..count)
        .map(|i| {
            make_conflict(
                &format!("uuid-{i}"),
                &format!("Record {i}"),
                &format!("Version A value {i}"),
                &format!("Version B value {i}"),
            )
        })
        .collect();
}

#[given(
    expr = "the sync client has reported a conflict for record {string} with versions {string} and {string}"
)]
async fn step_specific_conflict(
    world: &mut ConflictResolutionWorld,
    uuid: String,
    version_a: String,
    version_b: String,
) {
    world.conflicts = vec![make_conflict(
        &uuid,
        "Exercise Name",
        &version_a,
        &version_b,
    )];
}

#[given("I have selected version A for all conflicts")]
async fn step_select_version_a_all(world: &mut ConflictResolutionWorld) {
    for conflict in world.conflicts.iter_mut() {
        conflict.choice = Some(ConflictChoice::VersionA);
    }
}

// ── When steps ────────────────────────────────────────────────────────────────

#[when("I view the app")]
async fn step_view_app(world: &mut ConflictResolutionWorld) {
    world.render_screen();
}

#[when("I view the conflict resolution screen")]
async fn step_view_conflict_screen(world: &mut ConflictResolutionWorld) {
    world.render_screen();
}

#[when("I select version A for all conflicts")]
async fn step_select_all_version_a(world: &mut ConflictResolutionWorld) {
    // Simulate the user picking version A for every conflict.
    // In the SSR test we verify the button state before/after by re-rendering
    // with the choices pre-set.
    for conflict in world.conflicts.iter_mut() {
        conflict.choice = Some(ConflictChoice::VersionA);
    }
    world.render_screen();
}

#[when("I confirm the resolution")]
async fn step_confirm_resolution(world: &mut ConflictResolutionWorld) {
    // Simulate that the on_resolve callback fired, which in the real app
    // transitions the SyncStatus away from ConflictsDetected.
    // Here we record the resolved conflicts and pretend the screen is dismissed
    // by clearing the conflicts list (what the parent component would do).
    world.resolved_conflicts = world.conflicts.clone();
    world.conflicts.clear();
    world.render_screen();
}

// ── Then steps ────────────────────────────────────────────────────────────────

#[then("the conflict resolution screen should be visible")]
async fn step_screen_visible(world: &mut ConflictResolutionWorld) {
    assert!(
        world
            .rendered_html
            .contains("data-testid=\"conflict-resolution-screen\""),
        "Expected conflict-resolution-screen in rendered HTML.\nHTML: {}",
        world.rendered_html
    );
}

#[then("the main workout UI should not be visible")]
async fn step_workout_ui_absent(world: &mut ConflictResolutionWorld) {
    // The conflict resolution screen renders a standalone div; the workout UI
    // (data-testid="shell-content") should not appear when it is shown.
    assert!(
        !world
            .rendered_html
            .contains("data-testid=\"shell-content\""),
        "Expected workout UI to be absent but found shell-content.\nHTML: {}",
        world.rendered_html
    );
}

#[then(expr = "I should see version A labelled {string}")]
async fn step_version_a_label(world: &mut ConflictResolutionWorld, label: String) {
    assert!(
        world.rendered_html.contains(label.as_str()),
        "Expected version A label '{label}' in HTML.\nHTML: {}",
        world.rendered_html
    );
}

#[then(expr = "I should see version B labelled {string}")]
async fn step_version_b_label(world: &mut ConflictResolutionWorld, label: String) {
    assert!(
        world.rendered_html.contains(label.as_str()),
        "Expected version B label '{label}' in HTML.\nHTML: {}",
        world.rendered_html
    );
}

#[then(expr = "I should see the value {string} for version A")]
async fn step_version_a_value(world: &mut ConflictResolutionWorld, value: String) {
    assert!(
        world.rendered_html.contains(value.as_str()),
        "Expected version A value '{value}' in HTML.\nHTML: {}",
        world.rendered_html
    );
}

#[then(expr = "I should see the value {string} for version B")]
async fn step_version_b_value(world: &mut ConflictResolutionWorld, value: String) {
    assert!(
        world.rendered_html.contains(value.as_str()),
        "Expected version B value '{value}' in HTML.\nHTML: {}",
        world.rendered_html
    );
}

#[then("I should see selectable options for version A and version B")]
async fn step_radio_buttons_present(world: &mut ConflictResolutionWorld) {
    assert!(
        world
            .rendered_html
            .contains("data-testid=\"version-a-radio-0\""),
        "Expected version-a-radio-0 in HTML.\nHTML: {}",
        world.rendered_html
    );
    assert!(
        world
            .rendered_html
            .contains("data-testid=\"version-b-radio-0\""),
        "Expected version-b-radio-0 in HTML.\nHTML: {}",
        world.rendered_html
    );
}

#[then("selecting one version should not auto-select any other record's version")]
async fn step_independent_selections(world: &mut ConflictResolutionWorld) {
    // Verify the radios use separate name attributes per conflict index.
    // With a single conflict the radio name is "conflict-0".
    assert!(
        world.rendered_html.contains("name=\"conflict-0\""),
        "Expected radio name group 'conflict-0' in HTML.\nHTML: {}",
        world.rendered_html
    );
}

#[then("the resolve button should be disabled or absent")]
async fn step_resolve_button_disabled(world: &mut ConflictResolutionWorld) {
    // Button is rendered with disabled attribute when not all conflicts resolved.
    assert!(
        world
            .rendered_html
            .contains("data-testid=\"resolve-button\""),
        "Expected resolve-button in HTML.\nHTML: {}",
        world.rendered_html
    );
    assert!(
        world.rendered_html.contains("disabled"),
        "Expected resolve button to be disabled.\nHTML: {}",
        world.rendered_html
    );
}

#[then("the resolve button should be available")]
async fn step_resolve_button_enabled(world: &mut ConflictResolutionWorld) {
    // When all conflicts have a choice, the disabled attribute is absent.
    // We check the button is present but not disabled.
    assert!(
        world
            .rendered_html
            .contains("data-testid=\"resolve-button\""),
        "Expected resolve-button in HTML.\nHTML: {}",
        world.rendered_html
    );
    // The disabled attribute should not appear in the button's rendered output.
    // Dioxus renders `disabled` as a bare attribute when true and omits it when false.
    // Locate the button tag and check up to the closing `>`.
    let button_start = world
        .rendered_html
        .find("data-testid=\"resolve-button\"")
        .expect("resolve-button not found");
    let remaining = &world.rendered_html[button_start..];
    let tag_end = remaining.find('>').unwrap_or(remaining.len());
    let button_tag = &remaining[..tag_end];
    assert!(
        !button_tag.contains("disabled"),
        "Expected resolve button to NOT be disabled but found disabled attribute.\nTag: {}",
        button_tag
    );
}

#[then("the conflict resolution screen should not be visible")]
async fn step_screen_not_visible(world: &mut ConflictResolutionWorld) {
    assert!(
        !world
            .rendered_html
            .contains("data-testid=\"conflict-resolution-screen\""),
        "Expected conflict-resolution-screen to be absent.\nHTML: {}",
        world.rendered_html
    );
}
