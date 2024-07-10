// Contains the state struct which stores information needed for wgpu
//  to render a shader

use std::rc::Rc;

use winit::event::WindowEvent;
use winit::window::Window;

use super::camera::CameraEvent;
use super::geometry::Geometry;
use super::input::InputState;
use super::render_context::RenderContext;
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

        // run camera logic
        render_context
            .camera_context
            .run_camera_logic(&mut self.input_state);

        render_context
            .camera_context
            .write_camera_buffer(render_context);

        if let Some(geometry) = self.geometry.as_ref() {
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
            blend_render_pass.set_vertex_buffer(0, geometry.blend_vertex_buffer.slice(..));
            blend_render_pass.set_index_buffer(
                geometry.blend_index_buffer.slice(..),
                wgpu::IndexFormat::Uint32,
            );
            blend_render_pass.draw_indexed(0..geometry.blend_num_indices, 0, 0..1);
        }

        render_context
            .queue
            .submit(std::iter::once(blend_encoder.finish()));

        ////////////////////////////
        // Color Ramp Render Pass //
        ////////////////////////////

        let color_ramp_output = render_context.surface.get_current_texture()?;
        let color_view = color_ramp_output
            .texture
            .create_view(&wgpu::TextureViewDescriptor::default());

        let mut color_ramp_encoder =
            render_context
                .device
                .create_command_encoder(&wgpu::CommandEncoderDescriptor {
                    label: Some("Color Ramp Render Encoder"),
                });

        if let Some(geometry) = self.geometry.as_ref() {
            let mut color_render_pass =
                color_ramp_encoder.begin_render_pass(&wgpu::RenderPassDescriptor {
                    label: Some("Color Ramp Render Pass"),
                    color_attachments: &[Some(wgpu::RenderPassColorAttachment {
                        view: &color_view,
                        resolve_target: None,
                        ops: wgpu::Operations {
                            load: wgpu::LoadOp::Clear(wgpu::Color {
                                r: 0.0,
                                g: 0.0,
                                b: 0.0,
                                a: 1.0,
                            }),
                            store: wgpu::StoreOp::Store,
                        },
                    })],
                    depth_stencil_attachment: None,
                    occlusion_query_set: None,
                    timestamp_writes: None,
                });

            color_render_pass.set_pipeline(&render_context.color_ramp_render_pipeline);
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
            color_render_pass.set_vertex_buffer(0, geometry.color_vertex_buffer.slice(..));
            color_render_pass.set_index_buffer(
                geometry.color_index_buffer.slice(..),
                wgpu::IndexFormat::Uint16,
            );
            color_render_pass.draw_indexed(0..geometry.color_num_indices, 0, 0..1);
        }

        render_context
            .queue
            .submit(std::iter::once(color_ramp_encoder.finish()));
        color_ramp_output.present();

        Ok(())
    }
}
