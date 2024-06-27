// Contains event loop logic for the window containing the wgpu surface

use std::cell::RefCell;
use std::rc::Rc;

use winit::{
    application::ApplicationHandler,
    event::WindowEvent,
    event_loop::{ActiveEventLoop, EventLoopProxy},
    window::{Window, WindowId},
};

use super::geometry::Geometry;
use super::render_context::RenderContext;
use super::state::{InitStage, State};
use crate::ingest::load::BufferStorage;

pub struct App<'a> {
    pub state: State<'a>,
    pub external_state: Rc<RefCell<ExternalState>>,
    pub event_loop_proxy: EventLoopProxy<UserMessage<'static>>,
}

// The application handler instance doesnt allow for error handling

impl<'a> ApplicationHandler<UserMessage<'static>> for App<'a> {
    fn resumed(&mut self, event_loop: &ActiveEventLoop) {
        // Create the window and add it to state
        self.state.window = Some(Rc::new(
            event_loop
                .create_window(
                    Window::default_attributes()
                        .with_inner_size(winit::dpi::PhysicalSize::new(400, 450)),
                )
                .expect("ERROR: Failed to create window"),
        ));

        // Store the window canvas to external state
        #[cfg(target_arch = "wasm32")]
        {
            use leptos::html::ToHtmlElement;
            use winit::platform::web::WindowExtWebSys;

            self.external_state.borrow_mut().canvas = self
                .state
                .window
                .clone()
                .expect("ERROR: Failed to get window when creating HtmlCanvasElement")
                .as_ref()
                .canvas()
                .map(|canvas_element| canvas_element.to_leptos_element());
        }
    }

    fn window_event(
        &mut self,
        event_loop: &ActiveEventLoop,
        _window_id: WindowId,
        event: WindowEvent,
    ) {
        match event {
            WindowEvent::CloseRequested => self.exiting(event_loop),

            WindowEvent::RedrawRequested => {
                // Continously re-render the surface if setup is complete
                if self.state.init_stage == InitStage::Complete {
                    match self.state.render() {
                        Ok(_) => {}

                        Err(wgpu::SurfaceError::OutOfMemory) => {
                            log::error!("OutOfMemory");
                            self.exiting(event_loop);
                        }

                        Err(e) => eprintln!("{:?}", e),
                    }
                }

                self.state
                    .window
                    .as_ref()
                    .expect("ERROR: Failed to get window when requesting redraw")
                    .request_redraw();
            }

            WindowEvent::Resized(physical_size) => {
                // Initialize setup of state when resized is first called
                if self.state.init_stage == InitStage::Incomplete {
                    web_sys::console::log_1(&"Generating state...".into());
                    leptos::spawn_local(super::render_context::generate_render_context(
                        self.state
                            .window
                            .as_ref()
                            .expect("ERROR: Failed to get window while generating render context")
                            .clone(),
                        self.event_loop_proxy.clone(),
                    ));
                } else {
                    web_sys::console::log_1(&"Resizing".into());
                    self.state.resize(physical_size);
                }
            }

            _ => {}
        }
    }

    fn user_event(&mut self, _event_loop: &ActiveEventLoop, event: UserMessage<'static>) {
        match event {
            UserMessage::StateMessage(render_context) => {
                // Fill out the rest of the state class with the contents of StateMessage
                web_sys::console::log_1(&"Assign state values in application handler...".into());
                self.state = State {
                    render_context: Some(render_context),
                    window: self.state.window.clone(),
                    init_stage: InitStage::Complete,
                    geometry: None,
                };

                // Resize configures the surface based on current canvas size
                self.state.resize(
                    self.state
                        .window
                        .as_ref()
                        .expect(
                            "ERROR: Failed to get window in user_event UserMessage::StateMessage",
                        )
                        .inner_size(),
                );
            }

            UserMessage::IncomingData(data) => {
                web_sys::console::log_1(&"Generating Buffers...".into());
                web_sys::console::log_3(
                    &"Incoming Data: \n".into(),
                    &format!("Vertices: {:?}", data.vertices).into(),
                    &format!("Indices: {:?}", data.indices).into(),
                );
                self.state.geometry = Some(Geometry::generate_buffers(
                    self.state.render_context.as_ref().unwrap(),
                    data,
                ));
                web_sys::console::log_1(&"Done Generating Buffers".into());
            }
        }
    }
}

// All user events that can be sent to the event loop
pub enum UserMessage<'a> {
    StateMessage(RenderContext<'a>),
    IncomingData(BufferStorage),
}

/// Stores the canvas as an html element
#[derive(Default)]
pub struct ExternalState {
    pub canvas: Option<leptos::HtmlElement<leptos::html::AnyElement>>,
}
