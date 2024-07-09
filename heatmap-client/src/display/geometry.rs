use wgpu::util::DeviceExt;

use super::render_context::RenderContext;
use crate::ingest::load::BufferStorage;


// To-do: Make this based on the size of the surface
const COLOR_VERTICES: &[Vertex] = &[
    Vertex { position: [-180.0, -90.0, 0.0], weight: 0},
    Vertex { position: [180.0, -90.0, 0.0], weight: 0},
    Vertex { position: [180.0, 90.0, 0.0], weight: 0},
    Vertex { position: [-180.0, 90.0, 0.0], weight: 0},
];

const COLOR_INDICES: &[u16] = &[
    0, 2, 3,
    0, 2, 1,
];
pub struct Geometry {
    pub blend_vertex_buffer: wgpu::Buffer,
    pub blend_index_buffer: wgpu::Buffer,
    pub blend_num_indices: u32,
    pub color_vertex_buffer: wgpu::Buffer,
    pub color_index_buffer: wgpu::Buffer,
    pub color_num_indices: u32,
}

impl Geometry {
    pub fn generate_buffers(render_context: &RenderContext, buffer_data: BufferStorage) -> Self {
        //////////////////////////////
        // Set up buffers to render //
        //////////////////////////////

        web_sys::console::log_3(
            &"Buffered Data: \n".into(),
            &format!("Vertices: {:?}", buffer_data.vertices.as_slice()).into(),
            &format!("Indices: {:?}", buffer_data.indices.as_slice()).into(),
        );

        let blend_vertex_buffer =
            render_context
                .device
                .create_buffer_init(&wgpu::util::BufferInitDescriptor {
                    label: Some("Blending Vertex Buffer"),
                    contents: bytemuck::cast_slice(buffer_data.vertices.as_slice()),
                    usage: wgpu::BufferUsages::VERTEX,
                });

        let blend_index_buffer =
            render_context
                .device
                .create_buffer_init(&wgpu::util::BufferInitDescriptor {
                    label: Some("Blending Index Buffer"),
                    contents: bytemuck::cast_slice(buffer_data.indices.as_slice()),
                    usage: wgpu::BufferUsages::INDEX,
                });

        let blend_num_indices = buffer_data.num_indices;

        let color_vertex_buffer = 
                render_context
                    .device
                    .create_buffer_init(&wgpu::util::BufferInitDescriptor {
                        label: Some("Color Ramp Vertex Buffer"),
                        contents: bytemuck::cast_slice(COLOR_VERTICES),
                        usage: wgpu::BufferUsages::VERTEX,
                    }
                );

        let color_index_buffer =
            render_context
                    .device
                    .create_buffer_init(&wgpu::util::BufferInitDescriptor {
                        label: Some("Color Ramp Index Buffer"),
                        contents: bytemuck::cast_slice(COLOR_INDICES),
                        usage: wgpu::BufferUsages::INDEX,
                    });

        let color_num_indices = COLOR_INDICES.len() as u32;

        Geometry {
            blend_vertex_buffer,
            blend_index_buffer,
            blend_num_indices,
            color_vertex_buffer,
            color_index_buffer,
            color_num_indices,
        }
    }
}

/// A vertex passed into the wgsl shader
#[repr(C)]
#[derive(Copy, Clone, Debug, bytemuck::Pod, bytemuck::Zeroable)]
pub struct Vertex {
    pub position: [f32; 3],
    pub weight: u32,
}

impl Vertex {
    ///Create a vertex descriptor
    pub fn desc() -> wgpu::VertexBufferLayout<'static> {
        wgpu::VertexBufferLayout {
            array_stride: std::mem::size_of::<Vertex>() as wgpu::BufferAddress,
            step_mode: wgpu::VertexStepMode::Vertex,
            attributes: &[
                wgpu::VertexAttribute {
                    offset: 0,
                    shader_location: 0,
                    format: wgpu::VertexFormat::Float32x3,
                },
                wgpu::VertexAttribute {
                    offset: std::mem::size_of::<[f32; 3]>() as wgpu::BufferAddress,
                    shader_location: 1,
                    format: wgpu::VertexFormat::Uint32,
                },
            ],
        }
    }
}
