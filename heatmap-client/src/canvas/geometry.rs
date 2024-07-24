use wgpu::util::DeviceExt;

use super::render_context::RenderContext;
use crate::ingest::load::BufferStorage;

// To-do: Make this based on the size of the surface
const COLOR_VERTICES: &[Vertex] = &[
    Vertex {
        position: [-180.0, -90.0, 0.0],
        weight: 0,
    },
    Vertex {
        position: [180.0, -90.0, 0.0],
        weight: 0,
    },
    Vertex {
        position: [180.0, 90.0, 0.0],
        weight: 0,
    },
    Vertex {
        position: [-180.0, 90.0, 0.0],
        weight: 0,
    },
];

const COLOR_INDICES: &[u16] = &[0, 2, 3, 0, 2, 1];
pub struct Geometry {
    pub lod_layers: Vec<BufferLayer>,
    pub color_layer: BufferLayer,
    pub outline_layers: Vec<BufferLayer>,
}

pub struct BufferLayer {
    pub vertex_buffer: wgpu::Buffer,
    pub index_buffer: wgpu::Buffer,
    pub num_indices: u32,
}

impl Geometry {
    pub fn generate_buffers(
        render_context: &RenderContext,
        buffer_data: Vec<BufferStorage>,
        outline_data: Vec<BufferStorage>,
    ) -> Self {
        //////////////////////////////
        // Set up buffers to render //
        //////////////////////////////

        let lod_layers = gen_lod_layers(render_context, buffer_data, "Heatmap");

        let outline_layers = gen_lod_layers(render_context, outline_data, "Outline");

        //Colormap Texture
        let color_vertex_buffer =
            render_context
                .device
                .create_buffer_init(&wgpu::util::BufferInitDescriptor {
                    label: Some("Color Map Vertex Buffer"),
                    contents: bytemuck::cast_slice(COLOR_VERTICES),
                    usage: wgpu::BufferUsages::VERTEX,
                });

        let color_index_buffer =
            render_context
                .device
                .create_buffer_init(&wgpu::util::BufferInitDescriptor {
                    label: Some("Color Map Index Buffer"),
                    contents: bytemuck::cast_slice(COLOR_INDICES),
                    usage: wgpu::BufferUsages::INDEX,
                });

        let color_num_indices = COLOR_INDICES.len() as u32;

        Geometry {
            lod_layers,
            outline_layers,
            color_layer: BufferLayer {
                vertex_buffer: color_vertex_buffer,
                index_buffer: color_index_buffer,
                num_indices: color_num_indices,
            },
        }
    }
}

fn gen_lod_layers(
    render_context: &RenderContext,
    buffer_data: Vec<BufferStorage>,
    label: &str,
) -> Vec<BufferLayer> {
    let mut lod_layers: Vec<BufferLayer> = Vec::new();

    for (i, layer) in buffer_data.iter().enumerate() {
        let lod_vertex_buffer =
            render_context
                .device
                .create_buffer_init(&wgpu::util::BufferInitDescriptor {
                    label: Some(&(format!("{:?} LOD {:?} Vertex Buffer", label, i))),
                    contents: bytemuck::cast_slice(layer.vertices.as_slice()),
                    usage: wgpu::BufferUsages::VERTEX,
                });

        let lod_index_buffer =
            render_context
                .device
                .create_buffer_init(&wgpu::util::BufferInitDescriptor {
                    label: Some(&(format!("{:?} LOD {:?} Index Buffer", label, i))),
                    contents: bytemuck::cast_slice(layer.indices.as_slice()),
                    usage: wgpu::BufferUsages::INDEX,
                });

        let lod_num_indices = buffer_data[i].num_indices;

        lod_layers.push(BufferLayer {
            vertex_buffer: lod_vertex_buffer,
            index_buffer: lod_index_buffer,
            num_indices: lod_num_indices,
        })
    }

    lod_layers
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
