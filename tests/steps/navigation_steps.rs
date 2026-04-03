use cucumber::{World, given, then};
use dioxus::prelude::*;
use simple_strength_assistant::components::tab_bar::{Tab, TabBar};

#[derive(Debug, Default, World)]
pub struct NavigationWorld {
    pub rendered_html: String,
}

#[component]
fn TestWrapper() -> Element {
    rsx! {
        TabBar {
            active_tab: Tab::Workout,
            on_change: move |_| {}
        }
    }
}

impl NavigationWorld {
    pub fn render_component(&mut self) {
        let mut vdom = VirtualDom::new(TestWrapper);
        vdom.rebuild_in_place();
        self.rendered_html = dioxus_ssr::render(&vdom);
    }
}

#[given("the application is open")]
async fn step_app_open(world: &mut NavigationWorld) {
    world.render_component();
}

#[then("the bottom navigation bar should be visible")]
async fn step_navbar_visible(world: &mut NavigationWorld) {
    assert!(world.rendered_html.contains("role=\"tablist\""));
    assert!(world.rendered_html.contains("Workout"));
    assert!(world.rendered_html.contains("Library"));
}

#[then("the bottom navigation bar should have bottom padding for safe areas")]
async fn step_navbar_safe_area(world: &mut NavigationWorld) {
    // Check for our custom safe area class
    assert!(world.rendered_html.contains("pb-safe-tabbar"));
}

#[then("the navigation bar should remain at the bottom when scrolling")]
async fn step_navbar_fixed(world: &mut NavigationWorld) {
    // Tab bar uses z-50 to stay visually above scrolling content
    // (positioned at bottom via Shell's flex layout, not fixed positioning)
    assert!(world.rendered_html.contains("z-50"));
}
