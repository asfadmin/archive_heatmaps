use std::rc::Rc;

use winit::event_loop::EventLoopProxy;
use winit::window::Window;

use super::app::UserMessage;
use super::camera::{Camera, CameraContext};
use super::geometry::Vertex;
use crate::display::texture::{generate_texture, TextureContext};

pub struct RenderContext<'a> {
    pub surface: wgpu::Surface<'a>,
    pub device: wgpu::Device,
    pub queue: wgpu::Queue,
    pub config: wgpu::SurfaceConfiguration,
    pub size: winit::dpi::PhysicalSize<u32>,
    pub blend_render_pipeline: wgpu::RenderPipeline,
    pub color_ramp_render_pipeline: wgpu::RenderPipeline,
    pub camera_context: CameraContext,
    pub texture_context: TextureContext,
}

/// Create a new state
pub async fn generate_render_context<'a>(
    window: Rc<Window>,
    event_loop_proxy: EventLoopProxy<UserMessage<'_>>,
) {
    let size = window.inner_size();

    ////////////////////
    // Set up surface //
    ////////////////////

    let instance = wgpu::Instance::new(wgpu::InstanceDescriptor {
        #[cfg(not(target_arch = "wasm32"))]
        backends: wgpu::Backends::PRIMARY,
        #[cfg(target_arch = "wasm32")]
        backends: wgpu::Backends::GL,
        ..Default::default()
    });

    let surface = instance
        .create_surface(window.clone())
        .expect("Failed to create surface");

    //////////////////////////
    // Get device and queue //
    //////////////////////////

    let adapter = instance
        .request_adapter(&wgpu::RequestAdapterOptions {
            power_preference: wgpu::PowerPreference::default(),
            compatible_surface: Some(&surface),
            force_fallback_adapter: false,
        })
        .await
        .expect("ERROR: Failed to get adapter");

    let (device, queue) = adapter
        .request_device(
            &wgpu::DeviceDescriptor {
                label: None,
                required_features: wgpu::Features::empty(),
                // WebGL doesn't support all of wgpu's features, so if
                // we're building for the web we'll have to disable some.
                required_limits: if cfg!(target_arch = "wasm32") {
                    wgpu::Limits::downlevel_webgl2_defaults()
                } else {
                    wgpu::Limits::default()
                },
            },
            None,
        )
        .await
        .expect("ERROR: Failed to get device and queue");

    ///////////////////////////
    // Set up surface config //
    ///////////////////////////

    let surface_caps = surface.get_capabilities(&adapter);

    let surface_format = surface_caps
        .formats
        .iter()
        .copied()
        .find(|f| f.is_srgb())
        .unwrap_or(surface_caps.formats[0]);
    let config = wgpu::SurfaceConfiguration {
        usage: wgpu::TextureUsages::RENDER_ATTACHMENT,
        format: surface_format,
        width: size.width,
        height: size.height,
        present_mode: surface_caps.present_modes[0],
        alpha_mode: surface_caps.alpha_modes[0],
        desired_maximum_frame_latency: 2,
        view_formats: vec![],
    };

    ////////////////////////
    // Set up Bind Groups //
    ////////////////////////

    let camera_context = Camera::generate_camera_data(&device, &config);

    let texture_context = generate_texture(&device, size);

    ////////////////////////////
    // Set up render pipeline //
    ////////////////////////////
    let blend_shader = device.create_shader_module(wgpu::include_wgsl!("shaders/blend.wgsl"));

    let blend_render_pipeline_layout =
        device.create_pipeline_layout(&wgpu::PipelineLayoutDescriptor {
            label: Some("Blend Render Pipeline Layout"),
            bind_group_layouts: &[&camera_context.camera_bind_group_layout],
            push_constant_ranges: &[],
        });

    let blend_render_pipeline = device.create_render_pipeline(&wgpu::RenderPipelineDescriptor {
        label: Some("Blend Render Pipeline"),
        layout: Some(&blend_render_pipeline_layout),
        vertex: wgpu::VertexState {
            module: &blend_shader,
            entry_point: "vs_main",
            buffers: &[Vertex::desc()],
            compilation_options: Default::default(),
        },
        fragment: Some(wgpu::FragmentState {
            module: &blend_shader,
            entry_point: "fs_main",
            compilation_options: Default::default(),
            targets: &[Some(wgpu::ColorTargetState {
                format: wgpu::TextureFormat::R16Float,
                blend: Some(wgpu::BlendState {
                    color: wgpu::BlendComponent {
                        src_factor: wgpu::BlendFactor::One,
                        dst_factor: wgpu::BlendFactor::One,
                        operation: wgpu::BlendOperation::Add,
                    },
                    alpha: wgpu::BlendComponent {
                        src_factor: wgpu::BlendFactor::One,
                        dst_factor: wgpu::BlendFactor::Zero,
                        operation: wgpu::BlendOperation::Add,
                    },
                }),
                write_mask: wgpu::ColorWrites::ALL,
            })],
        }),
        primitive: wgpu::PrimitiveState {
            topology: wgpu::PrimitiveTopology::TriangleList,
            strip_index_format: None,
            front_face: wgpu::FrontFace::Cw,
            cull_mode: None,
            polygon_mode: wgpu::PolygonMode::Fill,
            unclipped_depth: false,
            conservative: false,
        },
        depth_stencil: None,
        multisample: wgpu::MultisampleState {
            count: 1,
            mask: !0,
            alpha_to_coverage_enabled: false,
        },
        multiview: None,
    });

    let color_ramp_shader =
        device.create_shader_module(wgpu::include_wgsl!("shaders/color_ramp.wgsl"));

    let color_ramp_render_pipeline_layout =
        device.create_pipeline_layout(&wgpu::PipelineLayoutDescriptor {
            label: Some("Color Ramp Render Pipeline Layout"),
            bind_group_layouts: &[
                &camera_context.camera_bind_group_layout,
                &texture_context.bind_group_layout,
            ],
            push_constant_ranges: &[],
        });

    let color_ramp_render_pipeline =
        device.create_render_pipeline(&wgpu::RenderPipelineDescriptor {
            label: Some("Color Ramp Render Pipeline"),
            layout: Some(&color_ramp_render_pipeline_layout),
            vertex: wgpu::VertexState {
                module: &color_ramp_shader,
                entry_point: "vs_main",
                buffers: &[Vertex::desc()],
                compilation_options: Default::default(),
            },
            fragment: Some(wgpu::FragmentState {
                module: &color_ramp_shader,
                entry_point: "fs_main",
                compilation_options: Default::default(),
                targets: &[Some(wgpu::ColorTargetState {
                    format: config.format,
                    blend: None,
                    write_mask: wgpu::ColorWrites::ALL,
                })],
            }),
            primitive: wgpu::PrimitiveState {
                topology: wgpu::PrimitiveTopology::TriangleList,
                strip_index_format: None,
                front_face: wgpu::FrontFace::Cw,
                cull_mode: None,
                polygon_mode: wgpu::PolygonMode::Fill,
                unclipped_depth: false,
                conservative: false,
            },
            depth_stencil: None,
            multisample: wgpu::MultisampleState {
                count: 1,
                mask: !0,
                alpha_to_coverage_enabled: false,
            },
            multiview: None,
        });

    // StateMessage is sent to the event loop with the contained variables
    let message = RenderContext {
        surface,
        device,
        queue,
        config,
        size,
        blend_render_pipeline,
        color_ramp_render_pipeline,
        camera_context,
        texture_context,
    };

    web_sys::console::log_1(&"Done Generating State".into());
    let _ = event_loop_proxy.send_event(UserMessage::StateMessage(message));
}
