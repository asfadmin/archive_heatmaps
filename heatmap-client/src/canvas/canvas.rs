// Canvas() is a leptos component which contains a HtmlCanvasElement containing
//  a surface that wgpu can render to

use std::cell::RefCell;
use std::rc::Rc;

use leptos::*;
use stylers::style_str;
use winit::event_loop::EventLoop;
use winit::platform::web::EventLoopExtWebSys;

use super::app::{App, ExternalState, UserMessage};
use super::state::State;
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
        .attr("class", "wgpu_surface");

    let data_loader = DataLoader { event_loop_proxy };

    create_effect(move |_| data_loader.load_data(filter()));

    let (class_name, style_val) = style_str! {"Canvas",
        .wgpu_surface {
            position: absolute;
            top: 0px;
            left: 0px;
            z-index: 0;
        }
    };

    // Compiler is dumb, these are necessary braces
    view! { class=class_name,
        <style>{style_val}</style>
        {canvas}
    }
}
