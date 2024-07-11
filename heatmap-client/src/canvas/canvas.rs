// Canvas() is a leptos component which contains a HtmlCanvasElement containing
//  a surface that wgpu can render to

use std::cell::RefCell;
use std::rc::Rc;

use leptos::*;
use winit::event_loop::EventLoop;
use winit::platform::web::EventLoopExtWebSys;

use super::app::{App, ExternalState, UserMessage};
use super::state::State;
use crate::ingest::load::DataLoader;

stylance::import_crate_style!(style, "style/canvas.scss");

/// Component to display a wgsl shader
#[component]
pub fn Canvas() -> impl IntoView {
    // Create event loop that can handle UserMessage events
    let event_loop = EventLoop::<UserMessage>::with_user_event()
        .build()
        .expect("ERROR: Failed to create event loop");

    // The canvas element will be stored here once it has been created
    let external_state = Rc::new(RefCell::new(ExternalState::default()));

    let app = App {
        external_state: external_state.clone(),
        state: State::default(),
        event_loop_proxy: event_loop.create_proxy(),
    };

    let event_loop_proxy = app.event_loop_proxy.clone();

    // Start the event loop
    event_loop.spawn_app(app);

    let canvas = external_state
        .borrow()
        .canvas
        .clone()
        .expect("ERROR: Failed to get external state")
        .attr("class", style::wgpu_surface);

    let data_loader = DataLoader { event_loop_proxy };

    data_loader.load_data();

    mount_to_body(move || canvas)
}
