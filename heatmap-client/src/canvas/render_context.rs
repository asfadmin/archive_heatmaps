use std::rc::Rc;

use winit::window::Window;
use winit::{dpi::PhysicalSize, event_loop::EventLoopProxy};

use super::app::UserMessage;
use super::camera::CameraContext;
use super::geometry::{generate_copy_buffer, generate_uniform_buffer, BufferContext};
use super::pipeline::{
    generate_blend_pipeline, generate_display_colormap_pipeline, generate_export_colormap_pipeline,
    generate_export_pipeline, generate_outline_pipeline,
};
use super::texture::{
    generate_blend_texture, generate_colormaps, generate_copy_texture, generate_export_texture,
    TextureContext,
};

// Stores all the things we need to set up wgpu and run render passes,
pub struct RenderContext<'a> {
    pub surface: wgpu::Surface<'a>,
    pub device: wgpu::Device,
    pub queue: wgpu::Queue,
    pub config: wgpu::SurfaceConfiguration,
    pub size: winit::dpi::PhysicalSize<u32>,
    pub limits: wgpu::Limits,
    pub blend_render_pipeline: wgpu::RenderPipeline,
    pub display_colormap_render_pipeline: wgpu::RenderPipeline,
    pub export_colormap_render_pipeline: wgpu::RenderPipeline,
    pub outline_render_pipeline: wgpu::RenderPipeline,
    pub export_render_pipeline: wgpu::RenderPipeline,
    pub camera_context: CameraContext,
    pub blend_texture_context: TextureContext,
    pub colormap_texture_context: (TextureContext, TextureContext),
    pub export_texture_context: TextureContext,
    pub copy_context: CopyContext,
    pub max_weight_context: MaxWeightContext,
}

/// Create a new RenderContext
pub async fn generate_render_context(
    window: Rc<Window>,
    event_loop_proxy: EventLoopProxy<UserMessage<'_>>,
) {
    // Default starting size, gets changed as soon as WindowEvent::Resize is fired
    let size = PhysicalSize::new(800, 800);

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

    let limits = device.limits();

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

    ////////////////////////////////////////////
    // Set up resources used in Render Passes //
    ////////////////////////////////////////////

    // Used to modify the displayed viewport, ie zoom and pan
    let camera_context = CameraContext::generate_camera_context(&device, &config);

    // Used to convert polygons into heatmap
    let blend_texture_context = generate_blend_texture(&device, size);
    let colormap_texture_context = generate_colormaps(&device, &queue);
    let export_texture_context = generate_export_texture(&device, size);

    // Used to get data from GPU to CPU
    let copy_texture = generate_copy_texture(&device, size);
    let copy_buffer = generate_copy_buffer(&device, size);

    let copy_context = CopyContext {
        texture: copy_texture,
        buffer: copy_buffer,
        buffer_mapped: false,
    };

    // Used to pass calculated max weight into Render Pass
    let max_weight_context = MaxWeightContext {
        state: MaxWeightState::Empty,
        uniform_buffer: generate_uniform_buffer(&device),
        value: None,
    };

    /////////////////////////////
    // Set up render pipelines //
    /////////////////////////////

    let blend_render_pipeline = generate_blend_pipeline(&device, &camera_context);
    let display_colormap_render_pipeline = generate_display_colormap_pipeline(
        &device,
        (
            &colormap_texture_context.0.bind_group_layout,
            &blend_texture_context.bind_group_layout,
            &max_weight_context.uniform_buffer.bind_group_layout,
        ),
        &config,
    );
    let export_colormap_render_pipeline = generate_export_colormap_pipeline(
        &device,
        (
            &colormap_texture_context.0.bind_group_layout,
            &blend_texture_context.bind_group_layout,
            &max_weight_context.uniform_buffer.bind_group_layout,
        ),
        &config,
    );
    let outline_render_pipeline = generate_outline_pipeline(&device, &camera_context);
    let export_render_pipeline =
        generate_export_pipeline(&device, &export_texture_context.bind_group_layout);

    // StateMessage is sent to the event loop with the contained variables
    let message = RenderContext {
        surface,
        device,
        queue,
        config,
        size,
        limits,
        blend_render_pipeline,
        display_colormap_render_pipeline,
        export_colormap_render_pipeline,
        outline_render_pipeline,
        export_render_pipeline,
        camera_context,
        blend_texture_context,
        colormap_texture_context,
        export_texture_context,
        copy_context,
        max_weight_context,
    };

    web_sys::console::log_1(&"Done Generating State".into());
    // Because this is a wasm application we cannot block on async calls so we instead send a message
    //    back to the application handler when this function completes
    let _ = event_loop_proxy.send_event(UserMessage::StateMessage(Box::new(message)));
}

/// Contains a texture and buffer used to map a texture onto the CPU
pub struct CopyContext {
    pub texture: wgpu::Texture,
    pub buffer: wgpu::Buffer,
    pub buffer_mapped: bool,
}
/// Contains resources neccessary to calculate the maximum weight of a set of data
pub struct MaxWeightContext {
    pub state: MaxWeightState,
    pub uniform_buffer: BufferContext,
    pub value: Option<f32>,
}

#[derive(PartialEq)]
pub enum MaxWeightState {
    Empty,
    InProgress,
    Completed,
}
