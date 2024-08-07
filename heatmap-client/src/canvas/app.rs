// Contains event loop logic for the window containing the wgpu surface

use std::cell::RefCell;
use std::rc::Rc;

use leptos::html::ToHtmlElement;
use leptos::SignalSet;
use winit::platform::web::WindowExtWebSys;
use winit::{
    application::ApplicationHandler,
    dpi::{LogicalSize, PhysicalSize},
    event::WindowEvent,
    event_loop::{ActiveEventLoop, EventLoopProxy},
    window::{Window, WindowId},
};

use super::geometry::{generate_copy_buffer, Geometry};
use super::render_context::{MaxWeightState, RenderContext};
use super::state::{InitStage, State};
use super::texture::generate_copy_texture;
use crate::ingest::load::BufferStorage;

pub struct App<'a> {
    pub state: State<'a>,
    pub external_state: Rc<RefCell<ExternalState>>,
    pub event_loop_proxy: EventLoopProxy<UserMessage<'static>>,
}

// The application handler instance doesnt allow for error handling
// The application handler responds to changes in the event loop, we send custom events here using
//    an event_loop_proxy

impl<'a> ApplicationHandler<UserMessage<'static>> for App<'a> {
    // This is run on initial startup, creates a window and stores it in the state, also stores
    //     the windows canvas in external state
    fn resumed(&mut self, event_loop: &ActiveEventLoop) {
        self.state.window = Some(Rc::new(
            event_loop
                .create_window(
                    Window::default_attributes()
                        .with_inner_size(winit::dpi::PhysicalSize::new(400, 450)),
                )
                .expect("ERROR: Failed to create window"),
        ));

        // Convert web_sys HtmlCanvasElement into a leptos HtmlElement<AnyElement>
        self.external_state.borrow_mut().canvas = self
            .state
            .window
            .clone()
            .expect("ERROR: Failed to get window when creating HtmlCanvasElement")
            .as_ref()
            .canvas()
            .map(|canvas_element| canvas_element.to_leptos_element());
    }

    fn window_event(
        &mut self,
        event_loop: &ActiveEventLoop,
        _window_id: WindowId,
        event: WindowEvent,
    ) {
        // regardless of the event, we must poll the current browser window size
        // and resize the winit window (our canvas) to match
        {
            let window = self
                .state
                .window
                .clone()
                .expect("failed to get window in window event");
            let browser_window =
                web_sys::window().expect("failed to get browser window in window event");

            let width = browser_window
                .inner_width()
                .expect("failed to get window inner width in window event")
                .as_f64()
                .expect("failed type conversion in window event") as u32;
            let height = browser_window
                .inner_height()
                .expect("failed to get window inner height in window event")
                .as_f64()
                .expect("failed type conversion in window even") as u32;
            let factor = window.scale_factor();
            let logical = LogicalSize { width, height };
            let PhysicalSize { width, height }: PhysicalSize<u32> = logical.to_physical(factor);

            let _ = window.request_inner_size(PhysicalSize::new(width, height));
        }

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
                // Initialize setup of state when resized is first called, otherwise call state.resize
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

            // Any other event will be handled by the user input handler
            _ => {
                self.state.handle_input_event(event);
            }
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
                    input_state: self.state.input_state.clone(),
                    event_loop_proxy: Some(self.event_loop_proxy.clone()),
                    camera_storage: None,
                    export_signal: self.state.export_signal.clone(),
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

            // There is incoming data from the service, we need to place this new data into buffers to render
            UserMessage::IncomingData(data, outline_data) => {
                web_sys::console::log_1(&"Generating Buffers...".into());
                let render_context = self
                    .state
                    .render_context
                    .as_mut()
                    .expect("Failed to get render context in Incoming Data event");

                self.state.geometry = Some(Geometry::generate_buffers(
                    render_context,
                    data,
                    outline_data,
                ));

                render_context.copy_context.texture =
                    generate_copy_texture(&render_context.device, render_context.size);

                render_context.copy_context.buffer = generate_copy_buffer(
                    &render_context.device,
                    winit::dpi::PhysicalSize::<u32> {
                        width: render_context.copy_context.texture.width(),
                        height: render_context.copy_context.texture.height(),
                    },
                );

                render_context.max_weight_context.state = MaxWeightState::Empty;

                web_sys::console::log_1(&"Done Generating Buffers".into());

                // Turn off the loading wheel
                self.external_state.borrow_mut().set_ready.set(true);
            }

            // This is part of getting the max weight of a set of data, to get data from the GPU
            //    you have to map a buffer to the CPU, this is done asynchronously so we fire off
            //    a custom event on mapping completion
            UserMessage::MaxWeightMapped => {
                let render_context = self
                    .state
                    .render_context
                    .as_mut()
                    .expect("Failed to get render context in UserMessage::BufferMapped");

                // We read the data contained in the buffer and convert it from &[u8] to Vec<u8>
                let raw_bytes: Vec<u8> = (&*render_context
                    .copy_context
                    .buffer
                    .slice(..)
                    .get_mapped_range())
                    .into();

                // The buffer is formated for f32 but we pulled a Vec<u8>, we must reform the Vec<f32> from the bytes
                let mut red_data: Vec<f32> = Vec::new();
                let mut raw_iter = raw_bytes.iter();
                while let Ok(raw) = raw_iter.next_chunk::<4>() {
                    // Read one channel into a f32
                    red_data.push(f32::from_le_bytes([*raw[0], *raw[1], *raw[2], *raw[3]]));

                    // The texture we stored in the buffer was rgba32Float but only had red data so we skip the g, b, a channels
                    match raw_iter.advance_by(4 * 3) {
                        Ok(_) => {}
                        Err(_) => {
                            panic!("Rgba32Float texture was malformed, size not a multiple of 16")
                        }
                    }
                }

                // Find the max value in the Vec<f32> we just created
                let mut max = 0.0;

                for value in red_data.iter() {
                    if value > &max {
                        max = *value;
                    }
                }

                web_sys::console::log_1(&format!("Max: {:?}", max).into());

                // We now update the uniform buffer with our max weight
                //    so that we can read the max value in the colormap render pass
                let mut uniform_data: Vec<u8> = max.to_le_bytes().into();

                // Uniform Buffer must be 16 byte aligned so we pad it with 0's
                while uniform_data.len() % 16 != 0 {
                    uniform_data.push(0);
                }

                render_context.queue.write_buffer(
                    &render_context.max_weight_context.uniform_buffer.buffer,
                    0,
                    uniform_data.as_slice(),
                );

                render_context.queue.submit([]);

                render_context.max_weight_context.state = MaxWeightState::Completed;

                render_context.copy_context.buffer.unmap();
            }

            UserMessage::ExportMapped => {
                web_sys::console::log_1(&"In the export mapped event".into());

                let render_context = self
                    .state
                    .render_context
                    .as_mut()
                    .expect("Failed to get render context in UserMessage::BufferMapped");

                // We read the data contained in the buffer and convert it from &[u8] to Vec<u8>
                let raw_bytes: Vec<u8> = (&*render_context
                    .copy_context
                    .buffer
                    .slice(..)
                    .get_mapped_range())
                    .into();

                // The buffer is formated for f32 but we pulled a Vec<u8>, we must reform the Vec<f32> from the bytes
                let mut color_data: Vec<f32> = Vec::new();
                let mut raw_iter = raw_bytes.iter();
                while let Ok(raw) = raw_iter.next_chunk::<4>() {
                    // Read one channel into a f32
                    color_data.push(f32::from_le_bytes([*raw[0], *raw[1], *raw[2], *raw[3]]));
                }

                // Convert the raw image data into an ImageBuffer that can be saved
                let img = image::Rgba32FImage::from_vec(
                    render_context.size.width,
                    render_context.size.height,
                    color_data,
                )
                .expect("Failed to convert parsed floats into an Rgba<f32> ImageBuffer");

                // Save the image
                let _ = img.save("./heatmap_output.png");

                render_context.copy_context.buffer.unmap();
                render_context.copy_context.buffer_mapped = false;
            }
        }
    }
}

// All user events that can be sent to the event loop
pub enum UserMessage<'a> {
    StateMessage(RenderContext<'a>),
    IncomingData(Vec<BufferStorage>, Vec<BufferStorage>),
    MaxWeightMapped,
    ExportMapped,
}

/// Stores the canvas as an html element
pub struct ExternalState {
    pub canvas: Option<leptos::HtmlElement<leptos::html::AnyElement>>,
    pub set_ready: leptos::WriteSignal<bool>,
}
