// We allow expect for the whole module as winit makes it nearly impossible
// to properly manage error handling.
#![allow(clippy::expect_used)]

pub mod app;
mod camera;
pub mod geometry;
mod input;
mod render_context;
mod state;
mod texture;

// Canvas() is a leptos component which contains a HtmlCanvasElement containing
//  a surface that wgpu can render to

use std::cell::RefCell;
use std::rc::Rc;

use app::{App, ExternalState, UserMessage};
use leptos::*;
use state::State;
use winit::event_loop::EventLoop;
use winit::platform::web::EventLoopExtWebSys;

use crate::ingest::load::DataLoader;

/// Component to display a wgsl shader
#[component]
pub fn Canvas() -> impl IntoView {
    // Signal from the UI containing the filter
    let filter = use_context::<ReadSignal<heatmap_api::Filter>>()
        .expect("ERROR: Failed to get colormap read signal context in Canvas()");

    // Create event loop that can handle UserMessage events
    let event_loop = EventLoop::<UserMessage>::with_user_event()
        .build()
        .expect("ERROR: Failed to create event loop");

    let (ready, set_ready) = create_signal(false);

    // The canvas element will be stored here once it has been created
    let external_state = Rc::new(RefCell::new(ExternalState {
        set_ready,
        canvas: None,
    }));

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
        .attr("class", "wgpu_surface");

    let (active_requests, set_active_requests) = create_signal(0);
    let data_loader = DataLoader {
        event_loop_proxy,
        active_requests,
        set_active_requests,
    };

    create_effect(move |_| data_loader.load_data(filter()));

    view! {
        <Show when=move || { !ready() }>
            <div id="loader">
                <span class="loader"></span>
            </div>
        </Show>
        {canvas}
    }
}
