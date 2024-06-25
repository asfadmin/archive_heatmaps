use super::render_context::RenderContext;
use super::state::{InitStage, State};

use std::rc::Rc;
use std::sync::{Arc, Mutex};
use winit::{
    application::ApplicationHandler,
    event::WindowEvent,
    event_loop::{ActiveEventLoop, EventLoopProxy},
    window::{Window, WindowId},
};

pub struct App<'a> {
    pub state: State<'a>,
    pub external_state: Arc<Mutex<ExternalState>>,
    pub event_loop_proxy: EventLoopProxy<UserMessage<'static>>,
}

impl<'a> ApplicationHandler<UserMessage<'static>> for App<'a> {
    fn resumed(&mut self, event_loop: &ActiveEventLoop) {
        // Create the window and store the canvas to external_state
        self.state.window = Some(Rc::new(
            event_loop
                .create_window(
                    Window::default_attributes()
                        .with_inner_size(winit::dpi::PhysicalSize::new(400, 450)),
                )
                .unwrap(),
        ));

        #[cfg(target_arch = "wasm32")]
        {
            use leptos::html::ToHtmlElement;
            use winit::platform::web::WindowExtWebSys;

            self.external_state.lock().as_mut().unwrap().canvas = self
                .state
                .window
                .clone()
                .unwrap()
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
                // Continously re-render the surface
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

                self.state.window.as_ref().unwrap().request_redraw();
            }

            WindowEvent::Resized(physical_size) => {
                // Reconfigure the surface if the window is resized
                if self.state.init_stage == InitStage::Incomplete {
                    web_sys::console::log_1(&"Generating state...".into());
                    leptos::spawn_local(super::render_context::generate_render_context(
                        self.state.window.as_ref().unwrap().clone(),
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
                };

                self.state
                    .resize(self.state.window.as_ref().unwrap().inner_size());
            }
        }
    }
}

// All user events that can be sent to the event loop
pub enum UserMessage<'a> {
    StateMessage(RenderContext<'a>),
}

/// Stores the canvas as an html element
#[derive(Default)]
pub struct ExternalState {
    pub canvas: Option<leptos::HtmlElement<leptos::html::AnyElement>>,
}
