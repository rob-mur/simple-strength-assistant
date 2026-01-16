use dioxus::prelude::*;
use futures_util::stream::StreamExt;
use gloo_timers::future::IntervalStream;
use wasm_bindgen_futures::spawn_local;
use web_sys::console;

use super::{check_for_updates, register_service_worker, setup_controller_change_listener, setup_update_listener};

/// Dioxus hook to manage service worker registration and updates
///
/// This hook:
/// - Registers the service worker on component mount
/// - Sets up update and controller change listeners
/// - Checks for updates every hour
///
/// Usage:
/// ```rust
/// #[component]
/// pub fn App() -> Element {
///     use_service_worker_manager();
///     // ... rest of component
/// }
/// ```
pub fn use_service_worker_manager() {
    use_effect(move || {
        // Spawn async task for service worker management
        spawn_local(async move {
            // Register service worker
            match register_service_worker().await {
                Ok(registration) => {
                    console::log_1(&"Service worker registered successfully".into());

                    // Set up update listener
                    if let Err(e) = setup_update_listener(&registration) {
                        console::error_1(
                            &format!("Failed to setup update listener: {:?}", e).into(),
                        );
                    }

                    // Set up controller change listener
                    if let Err(e) = setup_controller_change_listener() {
                        console::error_1(
                            &format!("Failed to setup controller change listener: {:?}", e)
                                .into(),
                        );
                    }

                    // Start interval for periodic update checks (every hour)
                    spawn_local(async move {
                        let mut interval = IntervalStream::new(60 * 60 * 1000); // 1 hour in milliseconds

                        loop {
                            interval.next().await;
                            console::log_1(&"Checking for service worker updates...".into());

                            if let Err(e) = check_for_updates(&registration).await {
                                console::error_1(
                                    &format!("Failed to check for updates: {:?}", e).into(),
                                );
                            }
                        }
                    });
                }
                Err(e) => {
                    console::error_1(&format!("ServiceWorker registration failed: {:?}", e).into());
                }
            }
        });
    });
}
