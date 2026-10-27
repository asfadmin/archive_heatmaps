use wgpu::util::DeviceExt;

use super::render_context::RenderContext;
use crate::ingest::load::BufferStorage;

// Used to render the blended texture onto
const RECTANGLE_VERTICES: &[Vertex] = &[
    Vertex {
        position: [-180.0, -90.0, 0.0],
    },
    Vertex {
        position: [180.0, -90.0, 0.0],
    },
    Vertex {
        position: [180.0, 90.0, 0.0],
    },
    Vertex {
        position: [-180.0, 90.0, 0.0],
    },
];

const RECTANGLE_INDICES: &[u16] = &[0, 2, 3, 0, 2, 1];

// The Golden Ratio
const PHI: f32 = 1.61803398875;

// 12 Vertices of a regular icosahedron
// Ensure your Vertex struct has #[derive(Copy, Clone, Debug, bytemuck::Pod, bytemuck::Zeroable)]
pub const ICOSAHEDRON_VERTICES: [[f32; 3]; 12] = [
    [-1.0, PHI, 0.0],
    [1.0, PHI, 0.0],
    [-1.0, -PHI, 0.0],
    [1.0, -PHI, 0.0],
    [0.0, -1.0, PHI],
    [0.0, 1.0, PHI],
    [0.0, -1.0, -PHI],
    [0.0, 1.0, -PHI],
    [PHI, 0.0, -1.0],
    [PHI, 0.0, 1.0],
    [-PHI, 0.0, -1.0],
    [-PHI, 0.0, 1.0],
];

// 20 Triangular faces (counter-clockwise winding order)
pub const ICOSAHEDRON_INDICES: [u16; 60] = [
    0, 11, 5, 0, 5, 1, 0, 1, 7, 0, 7, 10, 0, 10, 11, 1, 5, 9, 5, 11, 4, 11, 10, 2, 10, 7, 6, 7, 1,
    8, 3, 9, 4, 3, 4, 2, 3, 2, 6, 3, 6, 8, 3, 8, 9, 4, 9, 5, 2, 4, 11, 6, 2, 10, 8, 6, 7, 9, 8, 1,
];

// All the things needed to bind a buffer to a render pass
pub struct BufferContext {
    pub buffer: wgpu::Buffer,
    pub bind_group_layout: wgpu::BindGroupLayout,
    pub bind_group: wgpu::BindGroup,
}

// All the things needed to draw a vertex buffer using indices
pub struct BufferLayer {
    pub vertex_buffer: wgpu::Buffer,
    pub index_buffer: wgpu::Buffer,
    pub num_indices: u32,
}

// All the geometry that is used in the blend render pass to create a colormap texture
pub struct Geometry {
    pub lod_layers: Vec<BufferLayer>,
    pub outline_layers: Vec<BufferLayer>,
    pub geometry_layer: BufferLayer,
}

impl Geometry {
    pub fn generate_cartesian_buffers(
        render_context: &RenderContext,
        buffer_data: &[BufferStorage],
        outline_data: &[BufferStorage],
    ) -> Self {
        //////////////////////////////
        // Set up buffers to render //
        //////////////////////////////

        let lod_layers = gen_lod_layers(render_context, buffer_data, "Heatmap");

        let outline_layers = gen_lod_layers(render_context, outline_data, "Outline");

        // Rectangle that is used in the colormap and max_weight render passes
        let rectangle_vertex_buffer =
            render_context
                .device
                .create_buffer_init(&wgpu::util::BufferInitDescriptor {
                    label: Some("Color Map Vertex Buffer"),
                    contents: bytemuck::cast_slice(RECTANGLE_VERTICES),
                    usage: wgpu::BufferUsages::VERTEX,
                });

        let rectangle_index_buffer =
            render_context
                .device
                .create_buffer_init(&wgpu::util::BufferInitDescriptor {
                    label: Some("Color Map Index Buffer"),
                    contents: bytemuck::cast_slice(RECTANGLE_INDICES),
                    usage: wgpu::BufferUsages::INDEX,
                });

        let rectangle_num_indices = RECTANGLE_INDICES.len() as u32;

        Self {
            lod_layers,
            outline_layers,
            geometry_layer: BufferLayer {
                vertex_buffer: rectangle_vertex_buffer,
                index_buffer: rectangle_index_buffer,
                num_indices: rectangle_num_indices,
            },
        }
    }

    pub fn generate_spherical_buffers(
        render_context: &RenderContext,
        buffer_data: &[BufferStorage],
        outline_data: &[BufferStorage],
    ) -> Self {
        let lod_layers = gen_lod_layers(render_context, buffer_data, "Heatmap");

        let outline_layers = gen_lod_layers(render_context, outline_data, "Outline");

        let icosahedron_vertex_slice: Vec<Vertex> = ICOSAHEDRON_VERTICES
            .iter()
            .map(|x| Vertex { position: *x })
            .collect();

        // Rectangle that is used in the colormap and max_weight render passes
        let icosahedron_vertex_buffer =
            render_context
                .device
                .create_buffer_init(&wgpu::util::BufferInitDescriptor {
                    label: Some("Icosahedron Vertex Buffer"),
                    contents: bytemuck::cast_slice(&icosahedron_vertex_slice),
                    usage: wgpu::BufferUsages::VERTEX,
                });

        let icosahedron_index_buffer =
            render_context
                .device
                .create_buffer_init(&wgpu::util::BufferInitDescriptor {
                    label: Some("Icosahedron Index Buffer"),
                    contents: bytemuck::cast_slice(&ICOSAHEDRON_INDICES),
                    usage: wgpu::BufferUsages::INDEX,
                });

        let icosahedron_num_indices = ICOSAHEDRON_INDICES.len() as u32;

        Self {
            lod_layers,
            outline_layers,
            geometry_layer: BufferLayer {
                vertex_buffer: icosahedron_vertex_buffer,
                index_buffer: icosahedron_index_buffer,
                num_indices: icosahedron_num_indices,
            },
        }
    }
}

// Stores each Level of Detail into its own BufferLayer to be used in the blend render pass
fn gen_lod_layers(
    render_context: &RenderContext,
    buffer_data: &[BufferStorage],
    label: &str,
) -> Vec<BufferLayer> {
    let mut lod_layers: Vec<BufferLayer> = Vec::new();

    for (i, layer) in buffer_data.iter().enumerate() {
        let lod_vertex_buffer =
            render_context
                .device
                .create_buffer_init(&wgpu::util::BufferInitDescriptor {
                    label: Some(&(format!("{label:?} LOD {i:?} Vertex Buffer"))),
                    contents: bytemuck::cast_slice(layer.vertices.as_slice()),
                    usage: wgpu::BufferUsages::VERTEX,
                });

        let lod_index_buffer =
            render_context
                .device
                .create_buffer_init(&wgpu::util::BufferInitDescriptor {
                    label: Some(&(format!("{label:?} LOD {i:?} Index Buffer"))),
                    contents: bytemuck::cast_slice(layer.indices.as_slice()),
                    usage: wgpu::BufferUsages::INDEX,
                });

        let lod_num_indices = buffer_data[i].num_indices;

        lod_layers.push(BufferLayer {
            vertex_buffer: lod_vertex_buffer,
            index_buffer: lod_index_buffer,
            num_indices: lod_num_indices,
        });
    }

    lod_layers
}

// A uniform buffer that a texture can be copied to and can be mapped to the cpu
//    used to calculate the max weight
pub fn generate_copy_buffer(
    device: &wgpu::Device,
    size: winit::dpi::PhysicalSize<u32>,
) -> wgpu::Buffer {
    let temp_contents = vec![0u8; (4 * 4 * size.width * size.height) as usize];

    device.create_buffer_init(&wgpu::util::BufferInitDescriptor {
        label: Some("Max Weight Buffer"),
        contents: temp_contents.as_slice(),
        usage: wgpu::BufferUsages::COPY_DST | wgpu::BufferUsages::MAP_READ,
    })
}

// Used to store the camera_uniform
pub fn generate_uniform_buffer(device: &wgpu::Device) -> BufferContext {
    let uniform_buffer = device.create_buffer(&wgpu::BufferDescriptor {
        label: Some("Uniform Buffer"),
        size: 16,
        usage: wgpu::BufferUsages::UNIFORM | wgpu::BufferUsages::COPY_DST,
        mapped_at_creation: false,
    });

    let uniform_bind_group_layout =
        device.create_bind_group_layout(&wgpu::BindGroupLayoutDescriptor {
            label: Some("Uniform Bind Group Layout"),
            entries: &[wgpu::BindGroupLayoutEntry {
                binding: 0,
                visibility: wgpu::ShaderStages::FRAGMENT,
                ty: wgpu::BindingType::Buffer {
                    ty: wgpu::BufferBindingType::Uniform,
                    has_dynamic_offset: false,
                    min_binding_size: None,
                },
                count: None,
            }],
        });

    let uniform_bind_group = device.create_bind_group(&wgpu::BindGroupDescriptor {
        label: Some("Uniform Bind Group"),
        layout: &uniform_bind_group_layout,
        entries: &[wgpu::BindGroupEntry {
            binding: 0,
            resource: uniform_buffer.as_entire_binding(),
        }],
    });

    BufferContext {
        buffer: uniform_buffer,
        bind_group_layout: uniform_bind_group_layout,
        bind_group: uniform_bind_group,
    }
}
/// A vertex passed into blend.wgsl
#[repr(C)]
#[derive(Copy, Clone, Debug, bytemuck::Pod, bytemuck::Zeroable)]
pub struct BlendVertex {
    pub position: [f32; 3],
    pub weight: u32,
}

impl BlendVertex {
    ///Create a vertex descriptor
    pub const fn desc() -> wgpu::VertexBufferLayout<'static> {
        wgpu::VertexBufferLayout {
            array_stride: std::mem::size_of::<Self>() as wgpu::BufferAddress,
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

// A vertex used in colormap.wgsl and max_weight.wgsl
#[repr(C)]
#[derive(Copy, Clone, Debug, bytemuck::Pod, bytemuck::Zeroable)]
pub struct Vertex {
    pub position: [f32; 3],
}

impl Vertex {
    pub const fn desc() -> wgpu::VertexBufferLayout<'static> {
        wgpu::VertexBufferLayout {
            array_stride: std::mem::size_of::<Self>() as wgpu::BufferAddress,
            step_mode: wgpu::VertexStepMode::Vertex,
            attributes: &[wgpu::VertexAttribute {
                offset: 0,
                shader_location: 0,
                format: wgpu::VertexFormat::Float32x3,
            }],
        }
    }
}
