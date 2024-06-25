use winit::event_loop::EventLoopProxy;

use super::request::{request, HeatmapData};

pub struct BufferStorage {
    vertices: Vec<(f32, f32)>,
    indices: Vec<u32>,
    num_indices: u32,
}
pub struct DataLoader {
    event_loop_proxy: EventLoopProxy,
}

impl DataLoader {
    pub fn load_data(&self) {
        leptos::spawn_local(load_data_async(self.event_loop_proxy.clone()));
    }
}

async fn load_data_async(event_loop_proxy: EventLoopProxy) {
    let data = request().await;

    let filtered_data = filter(data);

    mesh_data(filtered_data);
}

fn filter(data: HeatmapData) -> HeatmapData {}

// I think this is working but it has not been tested, I couldnt figure out how to get the heatmap-service set up to actually recieve the post request
fn mesh_data(data: HeatmapData) -> BufferStorage {
    let (length, positions, weights) = data;

    // Loop over each polygon and create triangles, v[0] v[n + 1] v[n +2] where n is the # of triangles already created, until n+2 is the end of the vertexes
    // Algorithm found at https://www.scratchapixel.com/lessons/3d-basic-rendering/ray-tracing-polygon-mesh/polygon-to-triangle-mesh.html
    // Need to remove duplicate last element, was need

    //These store the values that will be placed inside the wgpu::Buffer
    let mut indices = Vec::new();
    let mut vertices = Vec::new();

    for polygon in data {
        let offset = vertices.len();
        let mut n = 0;
        while (n + 2) < polygon.len() {
            indices.push(0 + offset);
            indices.push(n + 1 + offset);
            indices.push(n + 2 + offset);
            n += 1;
        }

        for vertex in polygon {
            vertices.push(vertex);
        }
    }

    let num_indices = indices.len();

    BufferStorage {
        vertices,
        indices,
        num_indices,
    }
}
