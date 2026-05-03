use simple_strength_assistant::App;

fn main() {
    // Install the in-memory ring-buffer logger FIRST so we own the `log` facade.
    // This captures log::info!/warn!/error! calls for the on-device Debug Logs
    // viewer (Settings page) and also forwards them to the browser console.
    #[cfg(target_arch = "wasm32")]
    simple_strength_assistant::log_buffer::install_buffer_logger();

    // Initialize tracing subscriber for development.  This uses the `tracing`
    // global (separate from `log::set_logger`) so it does not conflict.
    dioxus::logger::init(tracing::Level::DEBUG).expect("failed to init logger");

    dioxus::launch(App);
}
