use super::app::{App, ExternalState, UserMessage};
use super::state::State;

use leptos::*;
use std::sync::{Arc, Mutex};
use winit::event_loop::EventLoop;

/// Component to display a wgsl shader
#[component]
pub fn Canvas() -> impl IntoView {
    // Create event loop that can handle UserMessage events
    let event_loop = EventLoop::<UserMessage>::with_user_event().build().unwrap();

    #[cfg(target_arch = "wasm32")]
    {
        use winit::platform::web::EventLoopExtWebSys;
        // The canvas element will be stored here once it has been created
        let external_state = Arc::new(Mutex::new(ExternalState::default()));

        let app = App {
            external_state: external_state.clone(),
            state: State::default(),
            event_loop_proxy: event_loop.create_proxy(),
        };

        // Start the event loop
        event_loop.spawn_app(app);

        // Wait until canvas has a value
        let canvas = external_state.lock().unwrap().canvas.clone().unwrap();

        mount_to_body(move || canvas)
    }
}
