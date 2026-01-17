pub mod hooks;

use wasm_bindgen::JsCast;
use wasm_bindgen::prelude::*;
use wasm_bindgen_futures::JsFuture;
use web_sys::{ServiceWorkerRegistration, console};

pub use hooks::use_service_worker_manager;

/// Register the service worker
/// Returns the ServiceWorkerRegistration if successful
pub async fn register_service_worker() -> Result<ServiceWorkerRegistration, JsValue> {
    // Get window and navigator
    let window = web_sys::window().ok_or("No window object")?;
    let navigator = window.navigator();

    // Get service worker container
    let service_worker_container = navigator.service_worker();

    console::log_1(&"Registering service worker...".into());

    // Register the service worker
    let registration_promise = service_worker_container.register("/service-worker.js");

    // Convert promise to future and await
    let registration = JsFuture::from(registration_promise).await?;

    // Cast to ServiceWorkerRegistration
    let registration = registration
        .dyn_into::<ServiceWorkerRegistration>()
        .map_err(|_| "Failed to cast to ServiceWorkerRegistration")?;

    console::log_1(
        &format!(
            "ServiceWorker registration successful: {:?}",
            registration.scope()
        )
        .into(),
    );

    Ok(registration)
}

/// Check for service worker updates
pub async fn check_for_updates(registration: &ServiceWorkerRegistration) -> Result<(), JsValue> {
    let update_promise = registration.update()?;
    JsFuture::from(update_promise).await?;
    Ok(())
}

/// Set up listener for service worker updates
/// This will reload the page when a new service worker is installed
pub fn setup_update_listener(registration: &ServiceWorkerRegistration) -> Result<(), JsValue> {
    let registration_clone = registration.clone();

    // Create closure for updatefound event
    let updatefound_callback = Closure::wrap(Box::new(move || {
        console::log_1(&"Service worker update found".into());

        if let Some(installing) = registration_clone.installing() {
            let installing_clone = installing.clone();

            // Create closure for statechange event on the installing worker
            #[allow(clippy::collapsible_if)]
            let statechange_callback = Closure::wrap(Box::new(move || {
                let state = installing_clone.state();
                console::log_1(&format!("Service worker state: {:?}", state).into());

                // Check if the new worker is installed and there's a controller
                // (meaning there's an old version running)
                if state == web_sys::ServiceWorkerState::Installed {
                    if let Some(window) = web_sys::window() {
                        let navigator = window.navigator().service_worker();
                        if navigator.controller().is_some() {
                            console::log_1(&"New version available! Reloading...".into());
                            let _ = window.location().reload();
                        }
                    }
                }
            }) as Box<dyn Fn()>);

            // Add event listener
            let _ = installing.add_event_listener_with_callback(
                "statechange",
                statechange_callback.as_ref().unchecked_ref(),
            );

            // Forget the closure so it persists
            statechange_callback.forget();
        }
    }) as Box<dyn Fn()>);

    // Add event listener
    registration.add_event_listener_with_callback(
        "updatefound",
        updatefound_callback.as_ref().unchecked_ref(),
    )?;

    // Forget the closure so it persists
    updatefound_callback.forget();

    Ok(())
}

/// Set up listener for controller changes
/// This will reload the page when the service worker controller changes
pub fn setup_controller_change_listener() -> Result<(), JsValue> {
    let window = web_sys::window().ok_or("No window object")?;
    let navigator = window.navigator();
    let service_worker_container = navigator.service_worker();

    let window_clone = window.clone();

    // Create closure for controllerchange event
    let controllerchange_callback = Closure::wrap(Box::new(move || {
        console::log_1(&"Service worker updated, reloading page".into());
        let _ = window_clone.location().reload();
    }) as Box<dyn Fn()>);

    // Add event listener
    service_worker_container.add_event_listener_with_callback(
        "controllerchange",
        controllerchange_callback.as_ref().unchecked_ref(),
    )?;

    // Forget the closure so it persists
    controllerchange_callback.forget();

    Ok(())
}
