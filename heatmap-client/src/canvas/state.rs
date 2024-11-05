// Contains the state struct which stores information needed for wgpu
//  to render a shader

use std::rc::Rc;

use wgpu::{BindGroup, Extent3d, Origin3d};
use winit::dpi::PhysicalSize;
use winit::event::WindowEvent;
use winit::event_loop::EventLoopProxy;
use winit::window::Window;

use super::app::UserMessage;
use super::camera::{Camera, CameraEvent};
use super::geometry::{generate_copy_buffer, Geometry};
use super::input::InputState;
use super::render_context::{CopyContext, MaxWeightState, RenderContext};
use super::texture::{generate_blend_texture, generate_copy_texture, generate_export_texture};

/// Tracks setup stage of state and png generations
#[derive(Default, PartialEq, Eq, Clone)]
pub enum InitStage {
    #[default]
    Incomplete,
    InProgress,
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
    pub size_storage: Option<PhysicalSize<u32>>,
    pub export_context: Option<ExportContext>,
}

impl<'a> State<'a> {
    // Process any user input on the heatmap
    pub fn handle_input_event(&mut self, event: WindowEvent) {
        self.input_state.eat_event(event);
    }

    // Configures the surface based on the passed physical size
    pub fn resize(&mut self, mut new_size: PhysicalSize<u32>) {
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

            // Textures must be the same size as the window to preserve resolution
            render_context.blend_texture_context =
                generate_blend_texture(&render_context.device, new_size);

            render_context.copy_context = CopyContext {
                texture: generate_copy_texture(&render_context.device, new_size),
                buffer: generate_copy_buffer(&render_context.device, new_size),
                buffer_mapped: false,
            };

            render_context.export_context =
                generate_export_texture(&render_context.device, new_size);

            web_sys::console::log_1(&format!("New Size: {:?}", new_size).into());
        }
    }

    /// Renders the contents of state to the canvas
    pub fn render(&mut self) -> Result<(), wgpu::SurfaceError> {
        // Check if we have generated a png for the current set of data,
        // if we have not then we resize the canvas to increase the resolution of
        // the generated image, resize() takes a mut& self so we must do this before
        // getting a mut& render_context which is why this check occurs here
        if self
            .export_context
            .as_ref()
            .expect("failed to get export context from state")
            .png_generated
            == InitStage::Incomplete
            && self.geometry.is_some()
        {
            let width = self
                .render_context
                .as_ref()
                .expect("Failed to get render context while getting max texture dims")
                .limits
                .max_texture_dimension_2d;
            let height = (width as f32 / (16.0 / 9.0)) as u32;

            // Resize our canvas to the highest resolution the GPU can support
            let new_size = PhysicalSize::<u32>::new(width, height);

            self.size_storage = Some(
                self.render_context
                    .as_ref()
                    .expect("Failed to get render context to store size")
                    .size,
            );

            self.resize(new_size);

            self.export_context
                .as_mut()
                .expect("Failed to get export context to update png_generated")
                .png_generated = InitStage::InProgress;
        }

        // Exit Render function if there is no geometry to render
        let Some(geometry) = self.geometry.as_ref() else {
            return Ok(());
        };

        let render_context = self
            .render_context
            .as_mut()
            .expect("ERROR: Failed to get render context in render");

        ///////////////////////
        // Blend Render Pass //
        ///////////////////////
        // This always runs, it renders all polygons in the vertex buffer
        //   onto the screen and then uses alpha blending to combine the
        //   weights of each polygon additivley with both alphas weighted
        //   at one

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

            // If we have not calculated the max weight or generated the png set the camera
            // to cover the entire screen, save the old camera
            if render_context.max_weight_context.state == MaxWeightState::Empty
                || self
                    .export_context
                    .as_ref()
                    .expect("failed to get export context from state")
                    .png_generated
                    == InitStage::InProgress
            {
                // Save values that will be changed
                if self.camera_storage.is_none() {
                    self.camera_storage = Some(render_context.camera_context.camera.clone());
                }

                render_context
                    .camera_context
                    .update_camera(CameraEvent::EntireView);
            }
            // Restore any saved values, if either is saved at this point then both are so we only
            // have to check a single one
            else if self.camera_storage.is_some() {
                render_context.camera_context.camera = self
                    .camera_storage
                    .as_ref()
                    .expect("Failed to get camera storage")
                    .clone();
                self.camera_storage = None;

                render_context.size = *self
                    .size_storage
                    .as_ref()
                    .expect("Failed to get stored size in render");

                self.camera_storage = None;
            }

            render_context
                .camera_context
                .write_camera_buffer(render_context);

            // Select the Level of Detail to use for the satellite granules based on the zoom
            let zoom = render_context.camera_context.camera.zoom;
            let mut active_blend_layer = &geometry.lod_layers[0];
            match zoom {
                15.0..30.0 => {
                    active_blend_layer = &geometry.lod_layers[1];
                }
                0.0..15.0 => {
                    active_blend_layer = &geometry.lod_layers[2];
                }
                _ => (),
            }

            // Configure render pass and set pipeline, bind groups, vertex buffer, and index buffer
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

        // Execute the configured render pass
        render_context
            .queue
            .submit(std::iter::once(blend_encoder.finish()));

        ////////////////////////////
        // Max Weight Render Pass //
        ////////////////////////////
        // This runs once any time we have new data including startup,
        //   It renders the texture generated by the blend render pass onto a
        //   rgba32Float texture that we can copy to the cpu to get the max weight

        // If we have not begun computing a max weight do so now
        if render_context.max_weight_context.state == MaxWeightState::Empty {
            let max_weight_output = &render_context.copy_context.texture;
            let max_weight_view =
                max_weight_output.create_view(&wgpu::TextureViewDescriptor::default());

            let mut max_weight_encoder =
                render_context
                    .device
                    .create_command_encoder(&wgpu::CommandEncoderDescriptor {
                        label: Some("Max Weight Render Encoder"),
                    });

            // Configure the render pass to render the blend texture to a rgba32Float texture
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

                max_weight_render_pass.set_pipeline(&render_context.export_render_pipeline);
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

            // Copy the rgba32Float texture into a buffer
            copy_encoder.copy_texture_to_buffer(
                wgpu::ImageCopyTexture {
                    texture: &render_context.copy_context.texture,
                    mip_level: 0,
                    origin: Origin3d { x: 0, y: 0, z: 0 },
                    aspect: wgpu::TextureAspect::All,
                },
                wgpu::ImageCopyBuffer {
                    buffer: &render_context.copy_context.buffer,
                    layout: wgpu::ImageDataLayout {
                        offset: 0,
                        bytes_per_row: Some(4 * 4 * render_context.copy_context.texture.width()),
                        rows_per_image: None,
                    },
                },
                Extent3d {
                    width: render_context.copy_context.texture.width(),
                    height: render_context.copy_context.texture.height(),
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

            // Begin mapping the buffer we just copied the texture into to the CPU,
            //    send a signal to the event loop upon completion
            render_context.copy_context.buffer.slice(..).map_async(
                wgpu::MapMode::Read,
                move |_| {
                    let _ = event_loop_proxy_clone.send_event(UserMessage::MaxWeightMapped);
                },
            );

            render_context.max_weight_context.state = MaxWeightState::InProgress;
        }
        //////////////////////////
        // Colormap Render Pass //
        //////////////////////////
        // This runs if the max weight pass does not, this takes the texture generated by the
        //   blend render pass and renders it to the winit window that is displayed. It also
        //   handles mapping the weights to the colormap it is given

        // If we have computed a max weight proceed with rendering the heatmap
        else if render_context.max_weight_context.state == MaxWeightState::Completed {
            let color_view: wgpu::TextureView;
            let mut colormap_output: Option<wgpu::SurfaceTexture> = None;
            let active_colormap: &BindGroup;
            let active_colormap_render_pipeline: &wgpu::RenderPipeline;

            // Draw to the export context texture if we have not generated a png yet
            if let Some(export) = &self.export_context
                && export.png_generated == InitStage::InProgress
            {
                color_view = render_context
                    .export_context
                    .texture
                    .create_view(&wgpu::TextureViewDescriptor::default());

                active_colormap = &render_context.colormap_texture_context.1.bind_group;
                active_colormap_render_pipeline = &render_context.export_colormap_render_pipeline;

                web_sys::console::log_1(&"Generating .png".into());
            } else {
                // We will draw to the surface of the window, this is displayed in the HtmlElement
                colormap_output = Some(render_context.surface.get_current_texture()?);
                color_view = colormap_output
                    .as_ref()
                    .expect("Failed to get colormap_output")
                    .texture
                    .create_view(&wgpu::TextureViewDescriptor::default());

                active_colormap = &render_context.colormap_texture_context.0.bind_group;
                active_colormap_render_pipeline = &render_context.display_colormap_render_pipeline;
            }

            let mut colormap_encoder =
                render_context
                    .device
                    .create_command_encoder(&wgpu::CommandEncoderDescriptor {
                        label: Some("Colormap Render Encoder"),
                    });

            // Select the level of detail for the world outline
            {
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

                // Render the outline of the world
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

                // Render the heatmap over the world outline, uses the blend texture we generated in the first render pass
                color_render_pass.set_pipeline(active_colormap_render_pipeline);
                color_render_pass.set_bind_group(0, active_colormap, &[]);
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
                color_render_pass.draw_indexed(0..geometry.rectangle_layer.num_indices, 0, 0..1);
            }

            render_context
                .queue
                .submit(std::iter::once(colormap_encoder.finish()));

            ////////////////////////////
            // Export PNG Render pass //
            ////////////////////////////
            // Runs if a png has not yet been generated and the copy_context buffer is
            //   not mapped to the cpu. This does essentially the same thing as the max weight render
            //   we should look into removing this render pass in favor of reusing the max weight pipeline
            //   to simplify the code, could break the pass out into a copy_pass() function

            if let Some(export) = self.export_context.as_mut()
                && export.png_generated == InitStage::InProgress
                && !render_context.copy_context.buffer_mapped
            {
                export.png_generated = InitStage::Complete;
                let export_output = &render_context.copy_context.texture;
                let export_view =
                    export_output.create_view(&wgpu::TextureViewDescriptor::default());

                let mut export_encoder =
                    render_context
                        .device
                        .create_command_encoder(&wgpu::CommandEncoderDescriptor {
                            label: Some("Export Render Encoder"),
                        });

                // Configure the render pass to render the export texture to a rgba32Float texture
                {
                    let mut export_render_pass =
                        export_encoder.begin_render_pass(&wgpu::RenderPassDescriptor {
                            label: Some("Export Render Pass"),
                            color_attachments: &[Some(wgpu::RenderPassColorAttachment {
                                view: &export_view,
                                resolve_target: None,
                                ops: wgpu::Operations {
                                    load: wgpu::LoadOp::Clear(wgpu::Color {
                                        r: 1.0,
                                        g: 1.0,
                                        b: 1.0,
                                        a: 1.0,
                                    }),
                                    store: wgpu::StoreOp::Store,
                                },
                            })],
                            depth_stencil_attachment: None,
                            occlusion_query_set: None,
                            timestamp_writes: None,
                        });

                    export_render_pass.set_pipeline(&render_context.export_render_pipeline);
                    export_render_pass.set_bind_group(
                        0,
                        &render_context.export_context.bind_group,
                        &[],
                    );
                    export_render_pass
                        .set_vertex_buffer(0, geometry.rectangle_layer.vertex_buffer.slice(..));
                    export_render_pass.set_index_buffer(
                        geometry.rectangle_layer.index_buffer.slice(..),
                        wgpu::IndexFormat::Uint16,
                    );

                    export_render_pass.draw_indexed(
                        0..geometry.rectangle_layer.num_indices,
                        0,
                        0..1,
                    );
                }

                render_context
                    .queue
                    .submit(std::iter::once(export_encoder.finish()));

                let mut copy_encoder =
                    render_context
                        .device
                        .create_command_encoder(&wgpu::CommandEncoderDescriptor {
                            label: Some("Export Copy Encoder"),
                        });

                // Copy the rgba32Float texture into a buffer
                copy_encoder.copy_texture_to_buffer(
                    wgpu::ImageCopyTexture {
                        texture: &render_context.copy_context.texture,
                        mip_level: 0,
                        origin: Origin3d { x: 0, y: 0, z: 0 },
                        aspect: wgpu::TextureAspect::All,
                    },
                    wgpu::ImageCopyBuffer {
                        buffer: &render_context.copy_context.buffer,
                        layout: wgpu::ImageDataLayout {
                            offset: 0,
                            bytes_per_row: Some(
                                4 * 4 * render_context.copy_context.texture.width(),
                            ),
                            rows_per_image: Some(render_context.copy_context.texture.height()),
                        },
                    },
                    Extent3d {
                        width: render_context.copy_context.texture.width(),
                        height: render_context.copy_context.texture.height(),
                        depth_or_array_layers: 1,
                    },
                );

                web_sys::console::log_1(
                    &format!(
                        "Copied {:?} bytes",
                        (render_context.copy_context.texture.width()
                            * render_context.copy_context.texture.height()
                            * 4
                            * 4)
                    )
                    .into(),
                );
                web_sys::console::log_1(
                    &format!(
                        "Copy Texture Dimensions: {:?} by {:?}",
                        render_context.copy_context.texture.width(),
                        render_context.copy_context.texture.height()
                    )
                    .into(),
                );

                render_context
                    .queue
                    .submit(std::iter::once(copy_encoder.finish()));

                let event_loop_proxy_clone = self
                    .event_loop_proxy
                    .as_ref()
                    .expect("Failed to get event loop proxy when mapping max weight buffer to cpu")
                    .clone();

                // Begin mapping the buffer we just copied the texture into to the CPU,
                //    send a signal to the event loop upon completion
                render_context.copy_context.buffer_mapped = true;

                render_context.copy_context.buffer.slice(..).map_async(
                    wgpu::MapMode::Read,
                    move |_| {
                        let _ = event_loop_proxy_clone.send_event(UserMessage::ExportMapped);
                    },
                );
            }

            // If we rendered onto the winit surface display the render
            if let Some(output) = colormap_output {
                output.present();
            }
        }
        Ok(())
    }
}

/// Contains a tracker for the state of png generation and the image that was generated
#[derive(Clone)]
pub struct ExportContext {
    pub png_generated: InitStage,
    pub img: leptos::WriteSignal<Option<image::Rgba32FImage>>,
}
