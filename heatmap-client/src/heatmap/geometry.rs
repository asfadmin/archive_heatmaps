use std::rc::Rc;
use wgpu::util::DeviceExt;
/// A vertex passed into the wgsl shader
#[repr(C)]
#[derive(Copy, Clone, Debug, bytemuck::Pod, bytemuck::Zeroable)]
pub struct Vertex {
    position: [f32; 3],
    weight: u32,
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

// The vertices of the shape stored in the vertex buffer
pub const VERTICES: &[Vertex] = &[
    Vertex {
        position: [-0.0868241, 0.49240386, 0.0],
        weight: 1,
    }, // A
    Vertex {
        position: [-0.49513406, 0.06958647, 0.0],
        weight: 1,
    }, // B
    Vertex {
        position: [-0.21918549, -0.44939706, 0.0],
        weight: 5,
    }, // C
    Vertex {
        position: [0.35966998, -0.3473291, 0.0],
        weight: 5,
    }, // D
    Vertex {
        position: [0.44147372, 0.2347359, 0.0],
        weight: 5,
    }, // E
    Vertex {
        position: [0.44147372, 0.2347359, 0.0],
        weight: 1,
    }, // E2
];

// The order to draw these vertices, each set of 3 represent a triangle
pub const INDICES: &[u16] = &[0, 1, 5, 1, 2, 4, 2, 3, 4];

pub fn generate_buffers(device: Rc<wgpu::Device>) -> (wgpu::Buffer, u32, wgpu::Buffer, u32) {
    //////////////////////////////
    // Set up buffers to render //
    //////////////////////////////

    let vertex_buffer = device.create_buffer_init(&wgpu::util::BufferInitDescriptor {
        label: Some("Vertex Buffer"),
        contents: bytemuck::cast_slice(VERTICES),
        usage: wgpu::BufferUsages::VERTEX,
    });

    let num_vertices = VERTICES.len() as u32;

    let index_buffer = device.create_buffer_init(&wgpu::util::BufferInitDescriptor {
        label: Some("Index Buffer"),
        contents: bytemuck::cast_slice(INDICES),
        usage: wgpu::BufferUsages::INDEX,
    });

    let num_indices = INDICES.len() as u32;

    (vertex_buffer, num_vertices, index_buffer, num_indices)
}
