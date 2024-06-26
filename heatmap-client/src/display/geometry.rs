use wgpu::util::DeviceExt;

use super::render_context::RenderContext;
use crate::ingest::load::BufferStorage;

pub struct Geometry {
    pub vertex_buffer: wgpu::Buffer,
    pub index_buffer: wgpu::Buffer,
    pub num_indices: u32,
}

impl Geometry {
    pub fn generate_buffers(render_context: &RenderContext, buffer_data: BufferStorage) -> Self {
        //////////////////////////////
        // Set up buffers to render //
        //////////////////////////////

        let vertex_buffer = render_context.device.create_buffer_init(&wgpu::util::BufferInitDescriptor {
            label: Some("Vertex Buffer"),
            contents: bytemuck::cast_slice(buffer_data.vertices.as_slice()),
            usage: wgpu::BufferUsages::VERTEX,
        });

        let index_buffer = render_context.device.create_buffer_init(&wgpu::util::BufferInitDescriptor {
            label: Some("Index Buffer"),
            contents: bytemuck::cast_slice(buffer_data.indices.as_slice()),
            usage: wgpu::BufferUsages::INDEX,
        });

        let num_indices = buffer_data.num_indices;

        Geometry {
            vertex_buffer,
            index_buffer,
            num_indices
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
