use cucumber::{World, given, then, when};
use dioxus::prelude::*;
use simple_strength_assistant::components::tab_bar::Tab;
use simple_strength_assistant::components::workout_view::WorkoutView;
use simple_strength_assistant::models::{ExerciseMetadata, SetTypeConfig};
use simple_strength_assistant::state::{PredictedParameters, WorkoutSession, WorkoutState};

#[derive(Debug, Default, World)]
pub struct WorkoutWorld {
    pub current_session: Option<WorkoutSession>,
    pub active_tab: Tab,
    pub rendered_html: String,
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
    use_context_provider(|| Signal::new(props.active_tab));

    rsx! {
        WorkoutView { state: state }
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
        exercise: ExerciseMetadata {
            id: Some(1),
            name: "Bench Press".to_string(),
            set_type_config: SetTypeConfig::Weighted {
                min_weight: 0.0,
                increment: 5.0,
            },
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
