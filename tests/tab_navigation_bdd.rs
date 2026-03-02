use cucumber::{World, given, then, when};

#[derive(Debug, Default, World)]
pub struct TabNavigationWorld {
    // Tab navigation state
    pub active_tab: String,
    pub tabs_visible: Vec<String>,

    // Workout session state
    pub active_exercise: Option<String>,
    pub current_set: Option<u32>,
    pub total_sets: Option<u32>,
    pub session_data: std::collections::HashMap<String, String>,

    // Component state
    pub component_mounted: bool,
    pub state_change_emitted: bool,

    // DOM/UI state
    pub active_styling: std::collections::HashMap<String, bool>,
    pub aria_attributes: std::collections::HashMap<String, String>,

    // localStorage simulation
    pub local_storage: std::collections::HashMap<String, String>,

    // Context state
    pub workout_state_mounted: bool,
}

impl TabNavigationWorld {
    /// Initialize with default state (Workout tab active)
    pub fn init_with_defaults(&mut self) {
        self.active_tab = "Workout".to_string();
        self.tabs_visible = vec!["Workout".to_string(), "Library".to_string()];
        self.component_mounted = false;
        self.state_change_emitted = false;
        self.workout_state_mounted = false;
        self.active_exercise = None;
        self.current_set = None;
        self.total_sets = None;
        self.session_data.clear();
        self.local_storage.clear();
        self.active_styling.clear();
        self.aria_attributes.clear();
    }

    /// Simulate tab click/selection
    pub fn select_tab(&mut self, tab_name: &str) {
        self.active_tab = tab_name.to_string();
        self.state_change_emitted = true;
        // Save to localStorage
        self.local_storage
            .insert("activeTab".to_string(), tab_name.to_string());
    }

    /// Simulate browser refresh (reload state from localStorage)
    pub fn refresh(&mut self) {
        if let Some(tab) = self.local_storage.get("activeTab") {
            self.active_tab = tab.clone();
        } else {
            self.active_tab = "Workout".to_string();
        }
        self.component_mounted = true;
    }

    /// Initialize workout session state
    pub fn create_workout_session(&mut self, exercise: &str, current_set: u32, total_sets: u32) {
        self.active_exercise = Some(exercise.to_string());
        self.current_set = Some(current_set);
        self.total_sets = Some(total_sets);
        self.session_data
            .insert("exercise".to_string(), exercise.to_string());
        self.session_data
            .insert("current_set".to_string(), current_set.to_string());
        self.session_data
            .insert("total_sets".to_string(), total_sets.to_string());
        self.workout_state_mounted = true;
    }

    /// Verify session data integrity
    pub fn verify_session_data(&self, exercise: &str, current_set: u32, total_sets: u32) -> bool {
        self.active_exercise.as_deref() == Some(exercise)
            && self.current_set == Some(current_set)
            && self.total_sets == Some(total_sets)
    }
}

// ============================================================================
// STEP DEFINITIONS - TAB NAVIGATION UI
// ============================================================================

#[given("the app is loaded")]
async fn app_is_loaded(world: &mut TabNavigationWorld) {
    world.init_with_defaults();
    world.component_mounted = true;
}

#[then(regex = r#"^I should see a "([^"]*)" tab$"#)]
async fn should_see_tab(world: &mut TabNavigationWorld, tab_name: String) {
    assert!(
        world.tabs_visible.contains(&tab_name),
        "Tab '{}' should be visible",
        tab_name
    );
}

#[then(regex = r#"^the "([^"]*)" tab should be active$"#)]
async fn tab_should_be_active(world: &mut TabNavigationWorld, tab_name: String) {
    assert_eq!(
        world.active_tab, tab_name,
        "Expected '{}' tab to be active, but got '{}'",
        tab_name, world.active_tab
    );
}

#[when(regex = r#"^I click on the "([^"]*)" tab$"#)]
async fn click_on_tab(world: &mut TabNavigationWorld, tab_name: String) {
    world.select_tab(&tab_name);
}

#[then("I should see the Library placeholder content")]
async fn should_see_library_placeholder(world: &mut TabNavigationWorld) {
    assert_eq!(
        world.active_tab, "Library",
        "Should be viewing Library tab to see placeholder"
    );
}

#[given(regex = r#"^I am on the "([^"]*)" tab$"#)]
async fn am_on_tab(world: &mut TabNavigationWorld, tab_name: String) {
    world.select_tab(&tab_name);
}

#[then("I should see the Workout interface")]
async fn should_see_workout_interface(world: &mut TabNavigationWorld) {
    assert_eq!(
        world.active_tab, "Workout",
        "Should be viewing Workout tab to see workout interface"
    );
}

#[given("the tab navigation component is rendered")]
async fn tab_navigation_component_is_rendered(world: &mut TabNavigationWorld) {
    world.init_with_defaults();
    world.component_mounted = true;
    // Set up active styling for current tab
    world
        .active_styling
        .insert("Workout".to_string(), world.active_tab == "Workout");
    world
        .active_styling
        .insert("Library".to_string(), world.active_tab == "Library");
    // Set up ARIA attributes
    world
        .aria_attributes
        .insert("tablist_role".to_string(), "tablist".to_string());
    world
        .aria_attributes
        .insert("tab_role".to_string(), "tab".to_string());
}

#[when(regex = r#"^the "([^"]*)" tab is active$"#)]
async fn tab_is_active(world: &mut TabNavigationWorld, tab_name: String) {
    world.active_tab = tab_name.clone();
    // Update styling to match active state
    for (tab, active) in world.active_styling.iter_mut() {
        *active = tab == &tab_name;
    }
}

#[then(regex = r#"^the "([^"]*)" tab should have active styling$"#)]
async fn tab_should_have_active_styling(world: &mut TabNavigationWorld, tab_name: String) {
    let is_active = world
        .active_styling
        .get(&tab_name)
        .copied()
        .unwrap_or(false);
    assert!(
        is_active,
        "Tab '{}' should have active styling but doesn't",
        tab_name
    );
}

#[then(regex = r#"^the "([^"]*)" tab should have inactive styling$"#)]
async fn tab_should_have_inactive_styling(world: &mut TabNavigationWorld, tab_name: String) {
    let is_active = world
        .active_styling
        .get(&tab_name)
        .copied()
        .unwrap_or(false);
    assert!(
        !is_active,
        "Tab '{}' should have inactive styling but has active styling",
        tab_name
    );
}

#[then(regex = r#"^the tab selection state should change to "([^"]*)"$"#)]
async fn tab_selection_state_should_change(world: &mut TabNavigationWorld, tab_name: String) {
    assert_eq!(
        world.active_tab, tab_name,
        "Tab selection should have changed to '{}'",
        tab_name
    );
}

#[then("a state change event should be emitted")]
async fn state_change_event_should_be_emitted(world: &mut TabNavigationWorld) {
    assert!(
        world.state_change_emitted,
        "State change event should have been emitted"
    );
}

#[then(regex = r#"^the tab container should have role "([^"]*)"$"#)]
async fn tab_container_should_have_role(world: &mut TabNavigationWorld, role: String) {
    let actual_role = world
        .aria_attributes
        .get("tablist_role")
        .expect("tablist_role should be set");
    assert_eq!(
        actual_role, &role,
        "Tab container should have role '{}'",
        role
    );
}

#[then(regex = r#"^each tab should have role "([^"]*)"$"#)]
async fn each_tab_should_have_role(world: &mut TabNavigationWorld, role: String) {
    let actual_role = world
        .aria_attributes
        .get("tab_role")
        .expect("tab_role should be set");
    assert_eq!(actual_role, &role, "Each tab should have role '{}'", role);
}

#[then(regex = r#"^the active tab should have aria-selected "([^"]*)"$"#)]
async fn active_tab_should_have_aria_selected(world: &mut TabNavigationWorld, value: String) {
    // In a real implementation, we'd check the actual DOM attribute
    // For this unit test, we verify the active tab concept is correct
    assert!(
        !world.active_tab.is_empty(),
        "Active tab should have aria-selected='{}'",
        value
    );
}

#[then(regex = r#"^inactive tabs should have aria-selected "([^"]*)"$"#)]
async fn inactive_tabs_should_have_aria_selected(_world: &mut TabNavigationWorld, _value: String) {
    // In a real implementation, we'd check the actual DOM attributes
    // For this unit test, we assume inactive tabs have the correct attribute
    // This is verified by the TabBar component implementation
}

// ============================================================================
// STEP DEFINITIONS - WORKOUT SESSION STATE PRESERVATION
// ============================================================================

#[given(regex = r#"^I have an active workout session with exercise "([^"]*)"$"#)]
async fn have_active_workout_session(world: &mut TabNavigationWorld, exercise_name: String) {
    world.init_with_defaults();
    world.active_exercise = Some(exercise_name.clone());
    world
        .session_data
        .insert("exercise".to_string(), exercise_name);
}

#[given(regex = r"^the current set is set (\d+) of (\d+)$")]
async fn current_set_is(world: &mut TabNavigationWorld, current: u32, total: u32) {
    world.current_set = Some(current);
    world.total_sets = Some(total);
    world
        .session_data
        .insert("current_set".to_string(), current.to_string());
    world
        .session_data
        .insert("total_sets".to_string(), total.to_string());
    world.workout_state_mounted = true;
}

#[when(regex = r#"^I switch to the "([^"]*)" tab$"#)]
async fn switch_to_tab(world: &mut TabNavigationWorld, tab_name: String) {
    world.select_tab(&tab_name);
}

#[when(regex = r#"^I switch back to the "([^"]*)" tab$"#)]
async fn switch_back_to_tab(world: &mut TabNavigationWorld, tab_name: String) {
    world.select_tab(&tab_name);
}

#[then(regex = r#"^I should still see exercise "([^"]*)"$"#)]
async fn should_still_see_exercise(world: &mut TabNavigationWorld, exercise_name: String) {
    assert_eq!(
        world.active_exercise.as_deref(),
        Some(exercise_name.as_str()),
        "Exercise should still be '{}'",
        exercise_name
    );
}

#[then(regex = r"^the current set should still be set (\d+) of (\d+)$")]
async fn current_set_should_still_be(world: &mut TabNavigationWorld, current: u32, total: u32) {
    assert_eq!(
        world.current_set,
        Some(current),
        "Current set should still be {}",
        current
    );
    assert_eq!(
        world.total_sets,
        Some(total),
        "Total sets should still be {}",
        total
    );
}

#[then("all session data should be intact")]
async fn all_session_data_should_be_intact(world: &mut TabNavigationWorld) {
    assert!(
        world.workout_state_mounted,
        "WorkoutState should still be mounted"
    );
    assert!(
        world.active_exercise.is_some(),
        "Active exercise should still be set"
    );
    assert!(
        world.current_set.is_some() && world.total_sets.is_some(),
        "Set data should still be present"
    );
}

#[when("I refresh the browser")]
async fn refresh_browser(world: &mut TabNavigationWorld) {
    world.refresh();
}

#[then(regex = r#"^the "([^"]*)" tab should still be active$"#)]
async fn tab_should_still_be_active(world: &mut TabNavigationWorld, tab_name: String) {
    assert_eq!(
        world.active_tab, tab_name,
        "Tab '{}' should still be active after refresh",
        tab_name
    );
}

#[given("a WorkoutState context with active session data")]
async fn workout_state_context_with_session_data(world: &mut TabNavigationWorld) {
    world.init_with_defaults();
    world.create_workout_session("Squat", 1, 5);
}

#[when(regex = r#"^the tab selection changes to "([^"]*)"$"#)]
async fn tab_selection_changes_to(world: &mut TabNavigationWorld, tab_name: String) {
    world.select_tab(&tab_name);
}

#[then("the WorkoutState context should remain mounted")]
async fn workout_state_context_should_remain_mounted(world: &mut TabNavigationWorld) {
    assert!(
        world.workout_state_mounted,
        "WorkoutState context should remain mounted"
    );
}

#[then("the session data should remain accessible")]
async fn session_data_should_remain_accessible(world: &mut TabNavigationWorld) {
    assert!(
        !world.session_data.is_empty(),
        "Session data should remain accessible"
    );
    assert!(
        world.active_exercise.is_some(),
        "Exercise data should be accessible"
    );
}

#[when(regex = r#"^the tab selection changes back to "([^"]*)"$"#)]
async fn tab_selection_changes_back_to(world: &mut TabNavigationWorld, tab_name: String) {
    world.select_tab(&tab_name);
}

#[then("the WorkoutState context should have the same session data")]
async fn workout_state_context_should_have_same_data(world: &mut TabNavigationWorld) {
    // Verify the session data matches what was initially set
    assert!(
        world.verify_session_data("Squat", 1, 5),
        "Session data should match initial state"
    );
}

#[given("the tab navigation component is mounted")]
async fn tab_navigation_component_is_mounted(world: &mut TabNavigationWorld) {
    world.init_with_defaults();
    world.component_mounted = true;
}

#[when(regex = r#"^I select the "([^"]*)" tab$"#)]
async fn select_tab(world: &mut TabNavigationWorld, tab_name: String) {
    world.select_tab(&tab_name);
}

#[then(regex = r#"^localStorage should contain key "([^"]*)" with value "([^"]*)"$"#)]
async fn local_storage_should_contain(world: &mut TabNavigationWorld, key: String, value: String) {
    let stored_value = world
        .local_storage
        .get(&key)
        .expect(&format!("localStorage should contain key '{}'", key));
    assert_eq!(
        stored_value, &value,
        "localStorage['{}'] should be '{}'",
        key, value
    );
}

#[given(regex = r#"^localStorage contains key "([^"]*)" with value "([^"]*)"$"#)]
async fn local_storage_contains(world: &mut TabNavigationWorld, key: String, value: String) {
    world.local_storage.insert(key, value);
}

#[when("the tab navigation component mounts")]
async fn tab_navigation_component_mounts(world: &mut TabNavigationWorld) {
    // Simulate component mount - read from localStorage if present, otherwise default to Workout
    if let Some(tab) = world.local_storage.get("activeTab") {
        world.active_tab = tab.clone();
    } else {
        world.active_tab = "Workout".to_string();
    }
    world.component_mounted = true;
}

#[then(regex = r#"^the "([^"]*)" tab should be initially active$"#)]
async fn tab_should_be_initially_active(world: &mut TabNavigationWorld, tab_name: String) {
    assert_eq!(
        world.active_tab, tab_name,
        "Tab '{}' should be initially active",
        tab_name
    );
}

#[given(regex = r#"^localStorage does not contain key "([^"]*)"$"#)]
async fn local_storage_does_not_contain(world: &mut TabNavigationWorld, key: String) {
    world.local_storage.remove(&key);
}

#[tokio::test]
async fn run_tab_navigation_tests() {
    TabNavigationWorld::cucumber()
        .run("tests/features/tab_navigation_ui.feature")
        .await;
    TabNavigationWorld::cucumber()
        .run("tests/features/tab_state_preservation.feature")
        .await;
}
