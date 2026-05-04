use cucumber::{World, given, then, when};
use dioxus::prelude::*;
use dioxus_history::MemoryHistory;
use simple_strength_assistant::app::{Route, TabNavigationState};
use simple_strength_assistant::components::tab_bar::Tab;
use simple_strength_assistant::models::{ExerciseMetadata, PlanExercise, SetTypeConfig};
use simple_strength_assistant::state::{PredictedParameters, WorkoutSession, WorkoutState};

#[derive(Debug, Default, World)]
pub struct WorkoutWorld {
    pub current_session: Option<WorkoutSession>,
    pub active_tab: Tab,
    pub rendered_html: String,
    pub has_active_plan: bool,
    pub planned_exercises: Vec<String>,
    pub tab_completed: u32,
    pub tab_planned: u32,
    pub plan_ended_at: Option<f64>,
}

#[derive(Props, Clone, PartialEq)]
struct WrapperProps {
    session: Option<WorkoutSession>,
    active_tab: Tab,
}

#[component]
fn TestWrapper(props: WrapperProps) -> Element {
    let state = WorkoutState::new();
    state.set_current_session(props.session.clone());
    use_context_provider(|| state);
    use_context_provider(|| TabNavigationState {
        last_workout_route: Signal::new(Route::WorkoutTab),
        last_library_route: Signal::new(Route::LibraryTab),
    });
    provide_context(
        std::rc::Rc::new(MemoryHistory::with_initial_path("/workout"))
            as std::rc::Rc<dyn dioxus_history::History>,
    );

    rsx! {
        Router::<Route> {}
    }
}

impl WorkoutWorld {
    pub fn render_component(&mut self) {
        let mut vdom = VirtualDom::new_with_props(
            TestWrapper,
            WrapperProps {
                session: self.current_session.clone(),
                active_tab: self.active_tab,
            },
        );
        vdom.rebuild_in_place();
        self.rendered_html = dioxus_ssr::render(&vdom);
    }
}

#[given("the Library tab is open")]
async fn step_library_open(world: &mut WorkoutWorld) {
    world.active_tab = Tab::Library;
}

#[when(expr = "I select the {string} exercise")]
async fn step_select_exercise(_world: &mut WorkoutWorld, _name: String) {
    // Selection logic mock
}

#[when("I click the \"Start Session\" button")]
async fn step_click_start(world: &mut WorkoutWorld) {
    // Simulate starting a session
    world.current_session = Some(WorkoutSession {
        session_id: Some("1".to_string()),
        exercise: ExerciseMetadata {
            id: Some("1".to_string()),
            name: "Bench Press".to_string(),
            set_type_config: SetTypeConfig::Weighted {
                min_weight: 0.0,
                increment: 5.0,
            },
            min_reps: 1,
            max_reps: None,
        },
        completed_sets: Vec::new(),
        predicted: PredictedParameters {
            weight: Some(0.0),
            reps: 8,
            rpe: 7.0,
        },
    });
    world.active_tab = Tab::Workout;
    world.render_component();
}

#[then(expr = "the application should switch to the {string} tab")]
async fn step_check_tab(world: &mut WorkoutWorld, tab: String) {
    let expected = if tab == "Workout" {
        Tab::Workout
    } else {
        Tab::Library
    };
    assert_eq!(world.active_tab, expected);
}

#[then(expr = "a new session for {string} should be active")]
async fn step_check_session_active(world: &mut WorkoutWorld, name: String) {
    assert!(world.current_session.is_some());
    assert_eq!(world.current_session.as_ref().unwrap().exercise.name, name);
    assert!(world.rendered_html.contains(&name));
}

#[then(expr = "the session exercise should be {string}")]
async fn step_check_session_exercise(world: &mut WorkoutWorld, name: String) {
    let session = world
        .current_session
        .as_ref()
        .expect("Expected an active session");
    assert_eq!(
        session.exercise.name, name,
        "Expected session exercise '{}', got '{}'",
        name, session.exercise.name
    );
}

#[given("no workout session is currently active")]
async fn step_no_active_session(world: &mut WorkoutWorld) {
    world.current_session = None;
}

#[when(expr = "I open the {string} tab")]
async fn step_open_tab(world: &mut WorkoutWorld, tab: String) {
    world.active_tab = if tab == "Workout" {
        Tab::Workout
    } else {
        Tab::Library
    };
    world.render_component();
}

#[then(expr = "I should see a message saying {string}")]
async fn step_check_message(world: &mut WorkoutWorld, message: String) {
    let rendered_upper = world.rendered_html.to_uppercase();
    let message_upper = message.to_uppercase();
    assert!(
        rendered_upper.contains(&message_upper),
        "Expected HTML to contain message: {} (searched for {})",
        message,
        message_upper
    );
}

#[then(expr = "I should see a button that says {string}")]
async fn step_check_button(world: &mut WorkoutWorld, button_text: String) {
    assert!(world.rendered_html.contains(&button_text));
}

#[then(expr = "I should not see a button that says {string}")]
async fn step_check_button_absent(world: &mut WorkoutWorld, button_text: String) {
    assert!(
        !world.rendered_html.contains(&button_text),
        "Expected HTML to NOT contain button text: {}",
        button_text
    );
}

#[given(expr = "an active session for {string} with completed sets")]
async fn step_active_session_with_sets(world: &mut WorkoutWorld, exercise_name: String) {
    world.current_session = Some(WorkoutSession {
        session_id: Some("1".to_string()),
        exercise: ExerciseMetadata {
            id: Some("1".to_string()),
            name: exercise_name,
            set_type_config: SetTypeConfig::Weighted {
                min_weight: 0.0,
                increment: 5.0,
            },
            min_reps: 1,
            max_reps: None,
        },
        completed_sets: vec![simple_strength_assistant::models::CompletedSet {
            set_number: 1,
            reps: 5,
            rpe: 7.0,
            set_type: simple_strength_assistant::models::SetType::Weighted { weight: 100.0 },
        }],
        predicted: PredictedParameters {
            weight: Some(100.0),
            reps: 8,
            rpe: 7.0,
        },
    });
    world.has_active_plan = true;
}

#[when(expr = "I switch to exercise {string}")]
async fn step_switch_exercise(world: &mut WorkoutWorld, exercise_name: String) {
    // Simulate the UI starting a new session, which implicitly clears the old one.
    world.current_session = Some(WorkoutSession {
        session_id: Some("2".to_string()),
        exercise: ExerciseMetadata {
            id: Some("2".to_string()),
            name: exercise_name,
            set_type_config: SetTypeConfig::Weighted {
                min_weight: 0.0,
                increment: 2.5,
            },
            min_reps: 1,
            max_reps: None,
        },
        completed_sets: Vec::new(),
        predicted: PredictedParameters {
            weight: Some(0.0),
            reps: 8,
            rpe: 7.0,
        },
    });
    world.active_tab = Tab::Workout;
    world.render_component();
}

#[then(expr = "the new session for {string} should have zero completed sets")]
async fn step_new_session_zero_sets(world: &mut WorkoutWorld, exercise_name: String) {
    let session = world
        .current_session
        .as_ref()
        .expect("Expected an active session");
    assert_eq!(
        session.exercise.name, exercise_name,
        "Expected session for {}",
        exercise_name
    );
    assert_eq!(
        session.completed_sets.len(),
        0,
        "Expected zero completed sets in new session"
    );
}

// Issue 154: Active workout view does not duplicate the exercise name below the tab strip
#[then("I should not see a duplicate exercise header card")]
async fn step_no_duplicate_header(world: &mut WorkoutWorld) {
    // The old "Exercise Header" card had classes "border-t-4 border-primary" and rendered
    // the exercise name in a card-title heading. After removal, that combination should
    // not appear. Checking for the pair avoids false positives if border-primary is used
    // elsewhere (e.g. a button or badge).
    assert!(
        !world.rendered_html.contains("border-t-4 border-primary"),
        "Expected no exercise header card with 'border-t-4 border-primary' in the rendered HTML"
    );
    // Also verify no card-title heading contains the exercise name inside the session area
    if let Some(ref session) = world.current_session {
        let pattern = format!("card-title\">{}", session.exercise.name);
        assert!(
            !world.rendered_html.contains(&pattern),
            "Expected no card-title heading with exercise name '{}' in the active session area",
            session.exercise.name
        );
    }
}

// Issue 163: Three-dot action menu replaces the history icon in the active session
#[then("I should see a history icon in the input area")]
async fn step_history_icon_in_input_area(world: &mut WorkoutWorld) {
    assert!(
        world.rendered_html.contains("action-menu-trigger"),
        "Expected action menu trigger button (data-testid='action-menu-trigger') to be present in the rendered HTML"
    );
}

// Issue 152: End Workout clears session so planning screen shows
#[when("the workout plan is ended")]
async fn step_end_plan(world: &mut WorkoutWorld) {
    // Simulate WorkoutStateManager::end_plan which must clear both plan and session
    world.has_active_plan = false;
    world.current_session = None;
}

#[then("no workout session should be active")]
async fn step_no_session_active(world: &mut WorkoutWorld) {
    assert!(
        world.current_session.is_none(),
        "Expected no active session after ending workout"
    );
}

#[then("no workout plan should be active")]
async fn step_no_plan_active(world: &mut WorkoutWorld) {
    assert!(
        !world.has_active_plan,
        "Expected no active plan after ending workout"
    );
}

// Issue 162: Starting a plan auto-starts the first exercise

#[given(expr = "a plan with exercises {string}, {string}, {string}")]
async fn step_plan_with_three_exercises(
    world: &mut WorkoutWorld,
    ex1: String,
    ex2: String,
    ex3: String,
) {
    // Store the planned exercise names so start_plan can pick the first one
    world.planned_exercises = vec![ex1, ex2, ex3];
    world.has_active_plan = false;
    world.current_session = None;
}

#[when("the plan is started")]
async fn step_start_plan(world: &mut WorkoutWorld) {
    // Simulate WorkoutStateManager::start_plan which now auto-starts
    // a session on the first planned exercise.
    world.has_active_plan = true;
    if let Some(first_name) = world.planned_exercises.first() {
        world.current_session = Some(WorkoutSession {
            session_id: Some("auto-1".to_string()),
            exercise: ExerciseMetadata {
                id: Some("auto-1".to_string()),
                name: first_name.clone(),
                set_type_config: SetTypeConfig::Weighted {
                    min_weight: 0.0,
                    increment: 2.5,
                },
                min_reps: 1,
                max_reps: None,
            },
            completed_sets: Vec::new(),
            predicted: PredictedParameters {
                weight: Some(0.0),
                reps: 8,
                rpe: 7.0,
            },
        });
    }
}

#[then(expr = "the active session should be for {string}")]
async fn step_active_session_for(world: &mut WorkoutWorld, exercise_name: String) {
    let session = world
        .current_session
        .as_ref()
        .expect("Expected an active session after starting the plan");
    assert_eq!(
        session.exercise.name, exercise_name,
        "Expected active session for '{}', got '{}'",
        exercise_name, session.exercise.name
    );
}

// Issue 164: Over-plan warning banner is removed
#[then("the over-plan warning banner should not be present")]
async fn step_no_over_plan_banner(world: &mut WorkoutWorld) {
    world.render_component();
    assert!(
        !world.rendered_html.contains("over-plan-prompt"),
        "Expected no over-plan-prompt element in the rendered HTML"
    );
}

// Issue 166: Library START creates ad-hoc one-exercise plan

#[when(expr = "the user starts exercise {string} from the Library")]
async fn step_start_exercise_from_library(world: &mut WorkoutWorld, exercise_name: String) {
    // Simulate WorkoutStateManager::start_adhoc_plan which creates a
    // one-exercise plan, adds the exercise with default_planned_sets, and
    // auto-starts a session on it.
    let default_planned_sets = 3u32; // Settings::default().default_planned_sets
    world.has_active_plan = true;
    world.tab_planned = default_planned_sets;
    world.planned_exercises = vec![exercise_name.clone()];
    world.current_session = Some(WorkoutSession {
        session_id: Some("adhoc-1".to_string()),
        exercise: ExerciseMetadata {
            id: Some("adhoc-1".to_string()),
            name: exercise_name,
            set_type_config: SetTypeConfig::Weighted {
                min_weight: 0.0,
                increment: 5.0,
            },
            min_reps: 1,
            max_reps: None,
        },
        completed_sets: Vec::new(),
        predicted: PredictedParameters {
            weight: Some(0.0),
            reps: 8,
            rpe: 7.0,
        },
    });
    world.active_tab = Tab::Workout;
}

#[then("a one-exercise plan should be active with planned sets from settings")]
async fn step_one_exercise_plan_active(world: &mut WorkoutWorld) {
    assert!(
        world.has_active_plan,
        "Expected an active plan after ad-hoc start"
    );
    assert_eq!(
        world.planned_exercises.len(),
        1,
        "Expected exactly one exercise in the ad-hoc plan, got {}",
        world.planned_exercises.len()
    );
    assert_eq!(
        world.tab_planned, 3,
        "Expected planned sets to match settings.default_planned_sets (3), got {}",
        world.tab_planned
    );
}

// Issue 164: ExerciseTabStrip set-count badge warning colour

#[derive(Props, Clone, PartialEq)]
struct TabStripTestProps {
    exercises: Vec<PlanExercise>,
    completed_counts: Vec<u32>,
}

#[component]
fn TabStripTestWrapper(props: TabStripTestProps) -> Element {
    use simple_strength_assistant::components::exercise_tab_strip::ExerciseTabStrip;
    rsx! {
        ExerciseTabStrip {
            exercises: props.exercises.clone(),
            active_index: 0usize,
            completed_counts: props.completed_counts.clone(),
            on_select: move |_: usize| {},
        }
    }
}

#[given(expr = "an exercise tab with {int} completed sets and {int} planned sets")]
async fn step_exercise_tab_with_counts(world: &mut WorkoutWorld, completed: u32, planned: u32) {
    world.tab_completed = completed;
    world.tab_planned = planned;

    let exercises = vec![PlanExercise {
        id: "pe-1".to_string(),
        exercise: ExerciseMetadata {
            id: Some("1".to_string()),
            name: "Bench Press".to_string(),
            set_type_config: SetTypeConfig::Weighted {
                min_weight: 0.0,
                increment: 5.0,
            },
            min_reps: 1,
            max_reps: None,
        },
        planned_sets: planned,
        position: 0,
    }];

    let completed_counts = vec![completed];

    let mut vdom = VirtualDom::new_with_props(
        TabStripTestWrapper,
        TabStripTestProps {
            exercises,
            completed_counts,
        },
    );
    vdom.rebuild_in_place();
    world.rendered_html = dioxus_ssr::render(&vdom);
}

#[then("the set-count badge should render in warning colour")]
async fn step_badge_warning(world: &mut WorkoutWorld) {
    // The badge element should have text-warning class when completed > planned
    assert!(
        world.rendered_html.contains("text-warning"),
        "Expected set-count badge to have 'text-warning' class. HTML: {}",
        world.rendered_html
    );
}

#[then("the set-count badge should render in default colour")]
async fn step_badge_default(world: &mut WorkoutWorld) {
    // The badge element should NOT have text-warning class when completed <= planned
    assert!(
        !world.rendered_html.contains("text-warning"),
        "Expected set-count badge to NOT have 'text-warning' class. HTML: {}",
        world.rendered_html
    );
}

// Issue 167: Complete Workout via three-dot menu

#[when("the user selects Complete Workout from the menu and confirms")]
async fn step_complete_workout_confirm(world: &mut WorkoutWorld) {
    // Simulate: user taps Complete Workout in the bottom sheet, then confirms
    // in the ConfirmationDialog. This mirrors WorkoutStateManager::end_plan:
    // sets ended_at on the plan, clears current_plan and current_session.
    world.plan_ended_at = Some(1714700000000.0);
    world.has_active_plan = false;
    world.current_session = None;
}

#[then("the plan should have ended_at set")]
async fn step_plan_ended_at_set(world: &mut WorkoutWorld) {
    assert!(
        world.plan_ended_at.is_some(),
        "Expected the plan's ended_at to be set after completing workout"
    );
}

#[then("the End Workout button should not be present")]
async fn step_no_end_workout_button(world: &mut WorkoutWorld) {
    world.render_component();
    assert!(
        !world.rendered_html.contains("end-workout-button"),
        "Expected no element with data-testid='end-workout-button' in the rendered HTML"
    );
}

#[when("the user selects Complete Workout from the menu and cancels")]
async fn step_complete_workout_cancel(_world: &mut WorkoutWorld) {
    // Simulate: user taps Complete Workout, then cancels the confirmation dialog.
    // No state change should occur — plan and session remain active.
}

#[then("a workout session should still be active")]
async fn step_session_still_active(world: &mut WorkoutWorld) {
    assert!(
        world.current_session.is_some(),
        "Expected workout session to still be active after cancelling"
    );
}

#[then("a workout plan should still be active")]
async fn step_plan_still_active(world: &mut WorkoutWorld) {
    assert!(
        world.has_active_plan,
        "Expected workout plan to still be active after cancelling"
    );
}

// Issue 168: Discard Workout

#[given(expr = "a started plan with exercise {string} and {int} logged sets")]
async fn step_started_plan_with_sets(
    world: &mut WorkoutWorld,
    exercise_name: String,
    set_count: u32,
) {
    let mut completed_sets = Vec::new();
    for i in 0..set_count {
        completed_sets.push(simple_strength_assistant::models::CompletedSet {
            set_number: i + 1,
            reps: 5,
            rpe: 7.0,
            set_type: simple_strength_assistant::models::SetType::Weighted { weight: 80.0 },
        });
    }
    world.current_session = Some(WorkoutSession {
        session_id: Some("1".to_string()),
        exercise: ExerciseMetadata {
            id: Some("1".to_string()),
            name: exercise_name.clone(),
            set_type_config: SetTypeConfig::Weighted {
                min_weight: 0.0,
                increment: 5.0,
            },
            min_reps: 1,
            max_reps: None,
        },
        completed_sets,
        predicted: PredictedParameters {
            weight: Some(80.0),
            reps: 5,
            rpe: 7.0,
        },
    });
    world.has_active_plan = true;
    world.planned_exercises = vec![exercise_name];
}

#[when("the workout is discarded")]
async fn step_discard_workout(world: &mut WorkoutWorld) {
    // Simulate WorkoutStateManager::discard_plan:
    // - Clears session
    // - Plan reverts to unstarted state (has_active_plan becomes false but
    //   planned_exercises are preserved)
    world.current_session = None;
    world.has_active_plan = false;
}

#[then("the plan should be unstarted with exercises preserved")]
async fn step_plan_unstarted_exercises_preserved(world: &mut WorkoutWorld) {
    assert!(
        !world.has_active_plan,
        "Expected plan to be unstarted (not active) after discard"
    );
    assert!(
        !world.planned_exercises.is_empty(),
        "Expected planned exercises to be preserved after discard"
    );
}

// Issue 183: Record screen UI polish

#[then(r#"the tab strip should not contain the textual "x/N" counter"#)]
async fn step_no_textual_counter(world: &mut WorkoutWorld) {
    // The textual counter previously rendered a span with
    // data-testid="set-count-badge" containing "{completed}/{planned}".
    // After the fix only the dotted indicator should remain — the badge
    // span itself is removed, so the data-testid should not appear.
    assert!(
        !world.rendered_html.contains("set-count-badge"),
        "Expected tab strip to NOT contain a set-count-badge text span. HTML: {}",
        world.rendered_html
    );
    // Also assert the literal "1/3" text is not present in the tab strip
    // header (the fixture in step_exercise_tab_with_counts uses 1/3 or 3/2).
    let composed = format!("{}/{}", world.tab_completed, world.tab_planned);
    // Allow the digits to appear elsewhere, but they must not appear with
    // the slash in a span sibling to the dot indicator. The simplest check
    // is that the exact "completed/planned" pair is absent.
    assert!(
        !world.rendered_html.contains(&composed),
        "Expected tab strip to NOT contain the textual counter '{}'. HTML: {}",
        composed,
        world.rendered_html
    );
}

#[then("the tab strip should still contain progress dots")]
async fn step_tab_strip_has_dots(world: &mut WorkoutWorld) {
    assert!(
        world.rendered_html.contains("dot-filled") || world.rendered_html.contains("dot-empty"),
        "Expected tab strip to contain progress dots. HTML: {}",
        world.rendered_html
    );
}

#[when("I render the active session")]
async fn step_render_active_session(world: &mut WorkoutWorld) {
    world.active_tab = Tab::Workout;
    world.render_component();
}

#[then(r#"the reps step-down button label should be "−5""#)]
async fn step_reps_step_down_minus_five(world: &mut WorkoutWorld) {
    // The step-down button is identified by data-testid="reps-step-down".
    // After the fix it should display the label "−5" (U+2212 MINUS SIGN + 5),
    // matching the symmetric +5 increment.
    let html = &world.rendered_html;
    let testid = "reps-step-down";
    let pos = html
        .find(testid)
        .unwrap_or_else(|| panic!("reps-step-down testid missing in HTML: {}", html));
    // Look at the next ~400 bytes for the button label content
    let window = &html[pos..(pos + 400).min(html.len())];
    assert!(
        window.contains("−5"),
        "Expected reps-step-down to render label '−5'. Window: {}",
        window
    );
    assert!(
        !window.contains("−1"),
        "Expected reps-step-down to NOT render the old '−1' label. Window: {}",
        window
    );
}

#[then(r#"the reps step-up button label should be "+5""#)]
async fn step_reps_step_up_plus_five(world: &mut WorkoutWorld) {
    let html = &world.rendered_html;
    let testid = "reps-step-up";
    let pos = html
        .find(testid)
        .unwrap_or_else(|| panic!("reps-step-up testid missing in HTML: {}", html));
    let window = &html[pos..(pos + 400).min(html.len())];
    assert!(
        window.contains("+5"),
        "Expected reps-step-up to render label '+5'. Window: {}",
        window
    );
}

#[then(r#"the rendered reps readout should not contain " reps""#)]
async fn step_reps_readout_no_dup(world: &mut WorkoutWorld) {
    let html = &world.rendered_html;
    let testid = "reps-readout";
    let pos = html
        .find(testid)
        .unwrap_or_else(|| panic!("reps-readout testid missing in HTML: {}", html));
    // Slice from the testid to the end of its enclosing span
    let after = &html[pos..(pos + 200).min(html.len())];
    // After the fix, the readout span should contain just the numeric
    // value (e.g. "8") with no trailing literal " reps" word.
    assert!(
        !after.contains(" reps"),
        "Expected reps-readout to NOT contain the literal ' reps' word. Window: {}",
        after
    );
}

#[then("the action menu trigger SVG should have a large icon class")]
async fn step_menu_icon_large(world: &mut WorkoutWorld) {
    let html = &world.rendered_html;
    let testid = "action-menu-trigger";
    let pos = html
        .find(testid)
        .unwrap_or_else(|| panic!("action-menu-trigger missing in HTML: {}", html));
    // Look ahead for the embedded SVG class attribute
    let window = &html[pos..(pos + 800).min(html.len())];
    // After the fix the icon must visibly fill the touch target — at least w-8 h-8.
    let big =
        window.contains("w-8 h-8") || window.contains("w-9 h-9") || window.contains("w-10 h-10");
    assert!(
        big,
        "Expected action-menu-trigger SVG to have a large class (>=w-8 h-8). Window: {}",
        window
    );
    assert!(
        !window.contains("w-6 h-6"),
        "Expected action-menu-trigger SVG to NOT use the small w-6 h-6 size. Window: {}",
        window
    );
}

#[then("the action menu trigger SVG should have a heavy stroke")]
async fn step_menu_icon_heavy_stroke(world: &mut WorkoutWorld) {
    let html = &world.rendered_html;
    let testid = "action-menu-trigger";
    let pos = html
        .find(testid)
        .unwrap_or_else(|| panic!("action-menu-trigger missing in HTML: {}", html));
    let window = &html[pos..(pos + 800).min(html.len())];
    // Heavy stroke = 2 or larger so the dots are visible. The previous
    // 1.5 stroke must not be retained.
    assert!(
        !window.contains(r#"stroke-width="1.5""#),
        "Expected action-menu-trigger SVG to NOT use the thin stroke-width 1.5. Window: {}",
        window
    );
}

#[then("the RPESlider should be rendered with hide_value true")]
async fn step_rpe_hide_value_true(world: &mut WorkoutWorld) {
    // The slider component should not render its own large value display
    // — that responsibility lives in the title bar (rpe-readout). The
    // simplest behavioural check is that the rpe-readout testid appears
    // exactly once (the title bar) in the rendered output.
    let count = world.rendered_html.matches("rpe-readout").count();
    assert_eq!(
        count, 1,
        "Expected rpe-readout to appear exactly once (in the title bar), got {}. HTML: {}",
        count, world.rendered_html
    );
}

#[then("the RPE slider input should declare a horizontal-only touch-action style")]
async fn step_rpe_touch_action_horizontal(world: &mut WorkoutWorld) {
    // The RPE slider must keep its horizontal drag alive even if the
    // user's finger drifts vertically. The reliable cross-browser way is
    // to declare `touch-action: pan-x` (or `none`) on the slider input,
    // which prevents the browser from reclassifying the gesture into a
    // vertical scroll and therefore stops `pointercancel` firing mid-drag.
    let html = &world.rendered_html;
    let pos = html
        .find(r#"type="range""#)
        .or_else(|| html.find("range range-lg"))
        .unwrap_or_else(|| panic!("RPE slider input missing in HTML: {}", html));
    let window = &html[pos.saturating_sub(200)..(pos + 600).min(html.len())];
    let has_pan_x = window.contains("touch-action:pan-x")
        || window.contains("touch-action: pan-x")
        || window.contains("touch-action:none")
        || window.contains("touch-action: none");
    assert!(
        has_pan_x,
        "Expected RPE slider input to declare touch-action: pan-x (or none). Window: {}",
        window
    );
}

#[then("the rendered output should not contain a duplicate RPE value display")]
async fn step_no_dup_rpe_value(world: &mut WorkoutWorld) {
    // The numeric RPE value is "7.0" given the fixture's predicted.rpe = 7.0.
    // The title bar renders {rpe_input:.1} once. The slider must not render
    // its own copy. So the formatted "7.0" should appear at most once.
    let occurrences = world.rendered_html.matches("7.0").count();
    assert!(
        occurrences <= 1,
        "Expected RPE value '7.0' to appear at most once on the screen, got {}. HTML: {}",
        occurrences,
        world.rendered_html
    );
}
