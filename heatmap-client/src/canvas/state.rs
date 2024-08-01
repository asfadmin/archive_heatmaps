// Contains the state struct which stores information needed for wgpu
//  to render a shader

use std::rc::Rc;

use wgpu::{Extent3d, Origin3d};
use winit::event::WindowEvent;
use winit::event_loop::EventLoopProxy;
use winit::window::Window;

use super::app::UserMessage;
use super::camera::{Camera, CameraEvent};
use super::geometry::Geometry;
use super::input::InputState;
use super::render_context::{MaxWeightState, RenderContext};
use super::texture::generate_blend_texture;

/// Tracks wether state is finished setting up
#[derive(Default, PartialEq, Eq)]
pub enum InitStage {
    #[default]
    Incomplete,
    Complete,
}

/// Stores the information needed to draw to a surface with a shader
#[derive(Default)]
pub struct State<'a> {
    pub render_context: Option<RenderContext<'a>>,
    pub geometry: Option<Geometry>,
    pub window: Option<Rc<Window>>,
    pub input_state: InputState,
    pub init_stage: InitStage,
    pub event_loop_proxy: Option<EventLoopProxy<UserMessage<'static>>>,
    pub camera_storage: Option<Camera>,
}

impl<'a> State<'a> {
    // handles input events
    pub fn handle_input_event(&mut self, event: WindowEvent) {
        self.input_state.eat_event(event);
    }

    // Configures the surface based on the passed physical size
    pub fn resize(&mut self, mut new_size: winit::dpi::PhysicalSize<u32>) {
        if new_size.width > 0 && new_size.height > 0 {
            let render_context = self
                .render_context
                .as_mut()
                .expect("ERROR: Failed to get render context in resize");

            render_context
                .camera_context
                .update_camera(CameraEvent::Resize(new_size.width, new_size.height));

            // Ensure new render surface size is within the maximum supported
            // texture size by the graphics card.
            if new_size.width > render_context.limits.max_texture_dimension_2d {
                new_size.width = render_context.limits.max_texture_dimension_2d;
            }

            if new_size.height > render_context.limits.max_texture_dimension_2d {
                new_size.height = render_context.limits.max_texture_dimension_2d;
            }

            render_context.size = new_size;

            let mut config = render_context.config.clone();
            config.width = new_size.width;
            config.height = new_size.height;
            render_context.config = config;

            render_context
                .surface
                .configure(&render_context.device, &render_context.config);

            render_context.blend_texture_context =
                generate_blend_texture(&render_context.device, new_size);
        }
    }

    // Renders the contents of state to the canvas
    pub fn render(&mut self) -> Result<(), wgpu::SurfaceError> {
        let render_context = self
            .render_context
            .as_mut()
            .expect("ERROR: Failed to get render context in render");

        if let Some(geometry) = self.geometry.as_ref() {
            ///////////////////////
            // Blend Render Pass //
            ///////////////////////

            let output = &render_context.blend_texture_context.texture;
            let view = output.create_view(&wgpu::TextureViewDescriptor::default());

            let mut blend_encoder =
                render_context
                    .device
                    .create_command_encoder(&wgpu::CommandEncoderDescriptor {
                        label: Some("Blend Render Encoder"),
                    });
            {
                // run camera logic
                render_context
                    .camera_context
                    .run_camera_logic(&mut self.input_state);

                if render_context.max_weight_context.state == MaxWeightState::Empty {
                    self.camera_storage = Some(render_context.camera_context.camera.clone());

                    render_context
                        .camera_context
                        .update_camera(CameraEvent::EntireView);
                } else if self.camera_storage.is_some() {
                    render_context.camera_context.camera = self
                        .camera_storage
                        .as_ref()
                        .expect("Failed to get camera storage")
                        .clone();
                    self.camera_storage = None;
                }

                render_context
                    .camera_context
                    .write_camera_buffer(render_context);

                let zoom = render_context.camera_context.camera.zoom;
                let mut active_blend_layer = &geometry.lod_layers[0];
                match zoom {
                    6.0..7.5 => {
                        active_blend_layer = &geometry.lod_layers[1];
                    }
                    0.0..6.0 => {
                        active_blend_layer = &geometry.lod_layers[2];
                    }
                    _ => (),
                }

                let mut blend_render_pass =
                    blend_encoder.begin_render_pass(&wgpu::RenderPassDescriptor {
                        label: Some("Blend Render Pass"),
                        color_attachments: &[Some(wgpu::RenderPassColorAttachment {
                            view: &view,
                            resolve_target: None,
                            ops: wgpu::Operations {
                                load: wgpu::LoadOp::Clear(wgpu::Color {
                                    r: 0.0,
                                    g: 0.0,
                                    b: 0.0,
                                    a: 0.0,
                                }),
                                store: wgpu::StoreOp::Store,
                            },
                        })],
                        depth_stencil_attachment: None,
                        occlusion_query_set: None,
                        timestamp_writes: None,
                    });

                blend_render_pass.set_pipeline(&render_context.blend_render_pipeline);
                blend_render_pass.set_bind_group(
                    0,
                    &render_context.camera_context.camera_bind_group,
                    &[],
                );
                blend_render_pass.set_vertex_buffer(0, active_blend_layer.vertex_buffer.slice(..));
                blend_render_pass.set_index_buffer(
                    active_blend_layer.index_buffer.slice(..),
                    wgpu::IndexFormat::Uint32,
                );

                blend_render_pass.draw_indexed(0..active_blend_layer.num_indices, 0, 0..1);
            }

            render_context
                .queue
                .submit(std::iter::once(blend_encoder.finish()));

            ////////////////////////////
            // Max Weight Render Pass //
            ////////////////////////////

            if render_context.max_weight_context.state == MaxWeightState::Empty {
                let max_weight_output = &render_context.max_weight_context.texture;
                let max_weight_view =
                    max_weight_output.create_view(&wgpu::TextureViewDescriptor::default());

                let mut max_weight_encoder =
                    render_context
                        .device
                        .create_command_encoder(&wgpu::CommandEncoderDescriptor {
                            label: Some("Max Weight Render Encoder"),
                        });
                {
                    let mut max_weight_render_pass =
                        max_weight_encoder.begin_render_pass(&wgpu::RenderPassDescriptor {
                            label: Some("Max Weight Render Pass"),
                            color_attachments: &[Some(wgpu::RenderPassColorAttachment {
                                view: &max_weight_view,
                                resolve_target: None,
                                ops: wgpu::Operations {
                                    load: wgpu::LoadOp::Clear(wgpu::Color {
                                        r: 0.0,
                                        g: 0.0,
                                        b: 0.0,
                                        a: 0.0,
                                    }),
                                    store: wgpu::StoreOp::Store,
                                },
                            })],
                            depth_stencil_attachment: None,
                            occlusion_query_set: None,
                            timestamp_writes: None,
                        });

                    max_weight_render_pass.set_pipeline(&render_context.max_weight_render_pipeline);
                    max_weight_render_pass.set_bind_group(
                        0,
                        &render_context.blend_texture_context.bind_group,
                        &[],
                    );
                    max_weight_render_pass
                        .set_vertex_buffer(0, geometry.rectangle_layer.vertex_buffer.slice(..));
                    max_weight_render_pass.set_index_buffer(
                        geometry.rectangle_layer.index_buffer.slice(..),
                        wgpu::IndexFormat::Uint16,
                    );

                    max_weight_render_pass.draw_indexed(
                        0..geometry.rectangle_layer.num_indices,
                        0,
                        0..1,
                    );
                }

                render_context
                    .queue
                    .submit(std::iter::once(max_weight_encoder.finish()));

                let mut copy_encoder =
                    render_context
                        .device
                        .create_command_encoder(&wgpu::CommandEncoderDescriptor {
                            label: Some("Copy Encoder"),
                        });

                copy_encoder.copy_texture_to_buffer(
                    wgpu::ImageCopyTexture {
                        texture: &render_context.max_weight_context.texture,
                        mip_level: 0,
                        origin: Origin3d { x: 0, y: 0, z: 0 },
                        aspect: wgpu::TextureAspect::All,
                    },
                    wgpu::ImageCopyBuffer {
                        buffer: &render_context.max_weight_context.buffer,
                        layout: wgpu::ImageDataLayout {
                            offset: 0,
                            bytes_per_row: Some(
                                4 * 4 * render_context.max_weight_context.texture.width(),
                            ),
                            rows_per_image: None,
                        },
                    },
                    Extent3d {
                        width: render_context.max_weight_context.texture.width(),
                        height: render_context.max_weight_context.texture.height(),
                        depth_or_array_layers: 1,
                    },
                );

                render_context
                    .queue
                    .submit(std::iter::once(copy_encoder.finish()));

                let event_loop_proxy_clone = self
                    .event_loop_proxy
                    .as_ref()
                    .expect("Failed to get event loop proxy when mapping max weight buffer to cpu")
                    .clone();

                render_context
                    .max_weight_context
                    .buffer
                    .slice(..)
                    .map_async(wgpu::MapMode::Read, move |_| {
                        let _ = event_loop_proxy_clone.send_event(UserMessage::BufferMapped);
                    });

                render_context.max_weight_context.state = MaxWeightState::InProgress;
            }
            ////////////////////////////
            // Colormap Render Pass //
            ////////////////////////////
            else if render_context.max_weight_context.state == MaxWeightState::Completed {
                let colormap_output = render_context.surface.get_current_texture()?;
                let color_view = colormap_output
                    .texture
                    .create_view(&wgpu::TextureViewDescriptor::default());

                let mut colormap_encoder =
                    render_context
                        .device
                        .create_command_encoder(&wgpu::CommandEncoderDescriptor {
                            label: Some("Colormap Render Encoder"),
                        });

                if let Some(geometry) = self.geometry.as_ref() {
                    let zoom = render_context.camera_context.camera.zoom;
                    let mut active_outline_layer = &geometry.outline_layers[0];
                    match zoom {
                        15.0..30.0 => {
                            active_outline_layer = &geometry.outline_layers[1];
                        }
                        0.0..15.0 => {
                            active_outline_layer = &geometry.outline_layers[2];
                        }
                        _ => (),
                    }

                    let mut color_render_pass =
                        colormap_encoder.begin_render_pass(&wgpu::RenderPassDescriptor {
                            label: Some("Colormap Render Pass"),
                            color_attachments: &[Some(wgpu::RenderPassColorAttachment {
                                view: &color_view,
                                resolve_target: None,
                                ops: wgpu::Operations {
                                    load: wgpu::LoadOp::Clear(wgpu::Color {
                                        r: 0.02,
                                        g: 0.02,
                                        b: 0.02,
                                        a: 1.0,
                                    }),
                                    store: wgpu::StoreOp::Store,
                                },
                            })],
                            depth_stencil_attachment: None,
                            occlusion_query_set: None,
                            timestamp_writes: None,
                        });

                    color_render_pass.set_pipeline(&render_context.outline_render_pipeline);

                    color_render_pass.set_bind_group(
                        0,
                        &render_context.camera_context.camera_bind_group,
                        &[],
                    );

                    color_render_pass
                        .set_vertex_buffer(0, active_outline_layer.vertex_buffer.slice(..));
                    color_render_pass.set_index_buffer(
                        active_outline_layer.index_buffer.slice(..),
                        wgpu::IndexFormat::Uint32,
                    );

                    color_render_pass.draw_indexed(0..active_outline_layer.num_indices, 0, 0..1);

                    color_render_pass.set_pipeline(&render_context.colormap_render_pipeline);
                    color_render_pass.set_bind_group(
                        0,
                        &render_context.colormap_texture_context.bind_group,
                        &[],
                    );
                    color_render_pass.set_bind_group(
                        1,
                        &render_context.blend_texture_context.bind_group,
                        &[],
                    );
                    color_render_pass.set_bind_group(
                        2,
                        &render_context.max_weight_context.uniform_buffer.bind_group,
                        &[],
                    );
                    color_render_pass
                        .set_vertex_buffer(0, geometry.rectangle_layer.vertex_buffer.slice(..));
                    color_render_pass.set_index_buffer(
                        geometry.rectangle_layer.index_buffer.slice(..),
                        wgpu::IndexFormat::Uint16,
                    );
                    color_render_pass.draw_indexed(
                        0..geometry.rectangle_layer.num_indices,
                        0,
                        0..1,
                    );
                }

                render_context
                    .queue
                    .submit(std::iter::once(colormap_encoder.finish()));
                colormap_output.present();
            }
        }

        Ok(())
    }
}
