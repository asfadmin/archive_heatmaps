// We allow expect for the whole module as winit makes it nearly impossible
// to properly manage error handling.
#![allow(clippy::expect_used)]

pub mod app;
mod camera;
pub mod geometry;
mod input;
mod pipeline;
mod png;
mod render_context;
mod state;
mod texture;

// Canvas() is a leptos component which contains a HtmlCanvasElement containing
//  a surface that wgpu can render to

use std::cell::RefCell;
use std::rc::Rc;

use app::{App, ExternalState, UserMessage};
use leptos::html::Div;
use leptos::logging::log;
use leptos::prelude::*;
use state::State;
use winit::event_loop::EventLoop;
use winit::platform::web::EventLoopExtWebSys;

use crate::canvas::png::{ExportContext, InitStage};
use crate::ingest::load::DataLoader;
use crate::types;

/// Component to display a heatmap generated using wgpu and wgsl shaders
#[component]
pub fn Canvas(set_generate_img: leptos::prelude::WriteSignal<bool>) -> impl IntoView {
    // Signal from the UI containing the filter
    let filter = use_context::<ReadSignal<types::Filter>>()
        .expect("ERROR: Failed to get filter read signal context in Canvas()");

    let generate_img = use_context::<ReadSignal<bool>>()
        .expect("ERROR: Failed to get generate_png read signal in Canvas()");

    // Create event loop that can handle UserMessage events
    let event_loop = EventLoop::<UserMessage>::with_user_event()
        .build()
        .expect("ERROR: Failed to create event loop");

    // Determines if the loading bar is displayed or not, false is displayed, true is hidden
    let (ready, set_ready) = signal(false);

    // The canvas element will be stored here once it has been created
    let external_state = Rc::new(RefCell::new(ExternalState {
        set_ready,
        canvas: None,
    }));

    let app = App {
        external_state: external_state.clone(),
        state: State {
            export_context: Some(ExportContext {
                generate_img,
                set_generate_img,
                stage: InitStage::Incomplete,
                base64_png: None,
            }),
            filter: Some(filter),
            ..Default::default()
        },
        event_loop_proxy: event_loop.create_proxy(),
    };

    // Get an event loop proxy before app is borrowed by event_loop.spawn_app
    let event_loop_proxy = app.event_loop_proxy.clone();

    // Start the event loop
    event_loop.spawn_app(app);

    let canvas_ref = NodeRef::<Div>::new();
    Effect::new(move |_| {
        log!("Adding canvas to DOM");
        if let Some(div) = canvas_ref.get() {
            let es = external_state.borrow();
            let canvas = es
                .canvas
                .as_ref()
                .expect("Failed to get canvas from external state");
            div.append_child(canvas)
                .expect("Failed to append canvas to div");
            log!("Canvas added to DOM");
        } else {
            log!("Failed to get ref to div");
        }
    });

    // Struct responsible for making requests to the service for new data

    leptos::task::spawn_local(async move {
        let data_loader = DataLoader::new(event_loop_proxy, set_ready, &filter()).await;
        // Anytime the filter signal changes the data loader now calls load data with the new signal
        // data_loader.load_data(filter());
        Effect::new(move |_| data_loader.load_data(filter()));
    });

    log!("Creating view!");
    view! {
        <div>
            <Show when=move || { !ready() }>
                <div id="loader">
                    <span class="loader"></span>
                </div>
            </Show>
            <div node_ref=canvas_ref></div>
        </div>
    }
}
