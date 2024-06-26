use winit::event_loop::EventLoopProxy;

use super::request::{request, HeatmapData};
use crate::display::app::UserMessage;
use crate::display::geometry::Vertex;

pub struct BufferStorage {
    pub vertices: Vec<Vertex>,
    pub indices: Vec<u32>,
    pub num_indices: u32,
}
pub struct DataLoader {
    pub event_loop_proxy: EventLoopProxy<UserMessage<'static>>,
}

impl DataLoader {
    pub fn load_data(&self) {
        leptos::spawn_local(load_data_async(self.event_loop_proxy.clone()));
    }
}

async fn load_data_async(event_loop_proxy: EventLoopProxy<UserMessage<'static>>) {
    let data = request().await;

    let filtered_data = filter(data);

    let meshed_data = mesh_data(filtered_data);

    let _ = event_loop_proxy.send_event(UserMessage::IncomingData(meshed_data));
}

fn filter(data: HeatmapData) -> HeatmapData {
    return data;
}

// I think this is working but it has not been tested, I couldnt figure out how to get the heatmap-service set up to actually recieve the post request
fn mesh_data(data: HeatmapData) -> BufferStorage {

    // Loop over each polygon and create triangles, v[0] v[n + 1] v[n +2] where n is the # of triangles already created, until n+2 is the end of the vertexes
    // Algorithm found at https://www.scratchapixel.com/lessons/3d-basic-rendering/ray-tracing-polygon-mesh/polygon-to-triangle-mesh.html
    // Need to remove duplicate last element, was need

    //These store the values that will be placed inside the wgpu::Buffer
    let mut indices = Vec::new();
    let mut vertices = Vec::new();

    let j: usize = 0;
    while j < data.length.try_into().unwrap() {

        let offset: u32 = vertices.len().try_into().expect("ERROR: Failed to convert usize into u32");
        let mut n:u32 = 0;

        while (n + 2) < data.positions[j].len().try_into().unwrap() {
            indices.push(offset);
            indices.push(n + 1 + offset);
            indices.push(n + 2 + offset);
            n += 1;
        }

        for vertex in data.positions[j].clone() {
            let vert = Vertex {
                position: [ vertex.0 as f32, vertex.1 as f32, 0.0 ],
                weight: data.weights[j] as u32,
            };
            vertices.push(vert);
        }
    }

    let num_indices = indices.len().try_into().expect("ERROR: Failed to convert usize into u32");

    BufferStorage {
        vertices,
        indices,
        num_indices,
    }
}
