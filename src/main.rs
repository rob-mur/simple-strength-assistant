mod app;
mod models;
mod state;

use app::App;

fn main() {
    // Initialize logger with Debug level for development
    // This routes all log::* calls throughout the codebase to browser console via tracing-wasm
    dioxus::logger::init(tracing::Level::DEBUG).expect("failed to init logger");

    dioxus::launch(App);
}
