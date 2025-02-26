use super::camera::CameraContext;
use crate::canvas::geometry::BlendVertex;
use crate::canvas::geometry::Vertex;

pub fn generate_blend_pipeline(
    device: &wgpu::Device,
    camera_context: &CameraContext,
) -> wgpu::RenderPipeline {
    let blend_shader = device.create_shader_module(wgpu::include_wgsl!("shaders/blend.wgsl"));

    let blend_render_pipeline_layout =
        device.create_pipeline_layout(&wgpu::PipelineLayoutDescriptor {
            label: Some("Blend Render Pipeline Layout"),
            bind_group_layouts: &[&camera_context.camera_bind_group_layout],
            push_constant_ranges: &[],
        });

    device.create_render_pipeline(&wgpu::RenderPipelineDescriptor {
        label: Some("Blend Render Pipeline"),
        layout: Some(&blend_render_pipeline_layout),
        vertex: wgpu::VertexState {
            module: &blend_shader,
            entry_point: "vs_main",
            buffers: &[BlendVertex::desc()],
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
                write_mask: wgpu::ColorWrites::RED,
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
    })
}

pub fn generate_display_colormap_pipeline(
    device: &wgpu::Device,
    bind_group_layouts: (
        &wgpu::BindGroupLayout,
        &wgpu::BindGroupLayout,
        &wgpu::BindGroupLayout,
    ),
    config: &wgpu::SurfaceConfiguration,
) -> wgpu::RenderPipeline {
    let colormap_shader = device.create_shader_module(wgpu::include_wgsl!("shaders/colormap.wgsl"));

    // Both colormaps have the same bind group layout but need different bind groups, bind group layout was duplicated as I thought it would be more confusing to add
    // another context struct instead of duplicating a single field
    let colormap_render_pipeline_layout =
        device.create_pipeline_layout(&wgpu::PipelineLayoutDescriptor {
            label: Some("Color Ramp Render Pipeline Layout"),
            bind_group_layouts: &[
                bind_group_layouts.0,
                bind_group_layouts.1,
                bind_group_layouts.2,
            ],
            push_constant_ranges: &[],
        });

    device.create_render_pipeline(&wgpu::RenderPipelineDescriptor {
        label: Some("Color Ramp Render Pipeline"),
        layout: Some(&colormap_render_pipeline_layout),
        vertex: wgpu::VertexState {
            module: &colormap_shader,
            entry_point: "vs_main",
            buffers: &[Vertex::desc()],
            compilation_options: Default::default(),
        },
        fragment: Some(wgpu::FragmentState {
            module: &colormap_shader,
            entry_point: "fs_main",
            compilation_options: Default::default(),
            targets: &[Some(wgpu::ColorTargetState {
                format: config.format,
                blend: Some(wgpu::BlendState::ALPHA_BLENDING),
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
    })
}

pub fn generate_export_colormap_pipeline(
    device: &wgpu::Device,
    bind_group_layouts: (
        &wgpu::BindGroupLayout,
        &wgpu::BindGroupLayout,
        &wgpu::BindGroupLayout,
    ),
    config: &wgpu::SurfaceConfiguration,
) -> wgpu::RenderPipeline {
    let colormap_shader =
        device.create_shader_module(wgpu::include_wgsl!("shaders/export_colormap.wgsl"));

    // Both colormaps have the same bind group layout but need different bind groups, bind group layout was duplicated as I thought it would be more confusing to add
    // another context struct instead of duplicating a single field
    let colormap_render_pipeline_layout =
        device.create_pipeline_layout(&wgpu::PipelineLayoutDescriptor {
            label: Some("Color Ramp Render Pipeline Layout"),
            bind_group_layouts: &[
                bind_group_layouts.0,
                bind_group_layouts.1,
                bind_group_layouts.2,
            ],
            push_constant_ranges: &[],
        });

    device.create_render_pipeline(&wgpu::RenderPipelineDescriptor {
        label: Some("Color Ramp Render Pipeline"),
        layout: Some(&colormap_render_pipeline_layout),
        vertex: wgpu::VertexState {
            module: &colormap_shader,
            entry_point: "vs_main",
            buffers: &[Vertex::desc()],
            compilation_options: Default::default(),
        },
        fragment: Some(wgpu::FragmentState {
            module: &colormap_shader,
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
    })
}

pub fn generate_outline_pipeline(
    device: &wgpu::Device,
    camera_context: &CameraContext,
) -> wgpu::RenderPipeline {
    let outline_shader = device.create_shader_module(wgpu::include_wgsl!("shaders/outline.wgsl"));

    let outline_render_pipeline_layout =
        device.create_pipeline_layout(&wgpu::PipelineLayoutDescriptor {
            label: Some("Outline Render Pipeline Layout"),
            bind_group_layouts: &[&camera_context.camera_bind_group_layout],
            push_constant_ranges: &[],
        });

    device.create_render_pipeline(&wgpu::RenderPipelineDescriptor {
        label: Some("Outline Render Pipeline"),
        layout: Some(&outline_render_pipeline_layout),
        vertex: wgpu::VertexState {
            module: &outline_shader,
            entry_point: "vs_main",
            buffers: &[BlendVertex::desc()],
            compilation_options: Default::default(),
        },
        fragment: Some(wgpu::FragmentState {
            module: &outline_shader,
            entry_point: "fs_main",
            compilation_options: Default::default(),
            targets: &[Some(wgpu::ColorTargetState {
                format: wgpu::TextureFormat::Rgba8UnormSrgb,
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
    })
}

/// Generates a render pipeline that is used to get data from the GPU onto the CPU
pub fn generate_export_pipeline(
    device: &wgpu::Device,
    bind_group_layout: &wgpu::BindGroupLayout,
) -> wgpu::RenderPipeline {
    let export_shader = device.create_shader_module(wgpu::include_wgsl!("shaders/export.wgsl"));

    let export_render_pipeline_layout =
        device.create_pipeline_layout(&wgpu::PipelineLayoutDescriptor {
            label: Some("Export Render Pipeline Layout"),
            bind_group_layouts: &[bind_group_layout],
            push_constant_ranges: &[],
        });

    device.create_render_pipeline(&wgpu::RenderPipelineDescriptor {
        label: Some("Export Render Pipeline"),
        layout: Some(&export_render_pipeline_layout),
        vertex: wgpu::VertexState {
            module: &export_shader,
            entry_point: "vs_main",
            buffers: &[Vertex::desc()],
            compilation_options: Default::default(),
        },
        fragment: Some(wgpu::FragmentState {
            module: &export_shader,
            entry_point: "fs_main",
            compilation_options: Default::default(),
            targets: &[Some(wgpu::ColorTargetState {
                format: wgpu::TextureFormat::Rgba32Float,
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
    })
}
