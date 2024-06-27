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
    // Request data from the server
    let data = request().await;

    // Filter the recived data
    web_sys::console::log_1(&"Filtering data...".into());
    let filtered_data = filter(data);

    // Convert the filtered data into a triangular mesh
    web_sys::console::log_1(&"Meshing data...".into());
    let meshed_data = mesh_data(filtered_data);
    web_sys::console::log_3(&"Meshed Data: \n".into(), &format!("Vertices: {:?}", meshed_data.vertices).into(), &format!("Indices: {:?}", meshed_data.indices).into());

    // Send the triangular mesh to the event loop
    web_sys::console::log_1(&"Sending Mesh to event loop".into());
    let _ = event_loop_proxy.send_event(UserMessage::IncomingData(meshed_data));
}

fn filter(data: HeatmapData) -> HeatmapData {
    return data;
}

// I think this is working but it has not been tested, I couldnt figure out how to get the heatmap-service set up to actually recieve the post request
fn mesh_data(data_exterior: HeatmapData) -> BufferStorage {

    let mut data = data_exterior.data;
    // Loop over each polygon and create triangles, v[0] v[n + 1] v[n +2] where n is the # of triangles already created, until n+2 is the end of the vertexes
    // Algorithm found at https://www.scratchapixel.com/lessons/3d-basic-rendering/ray-tracing-polygon-mesh/polygon-to-triangle-mesh.html
    // Need to remove duplicate last element, was need

    //These store the values that will be placed inside the wgpu::Buffer
    let mut indices = Vec::new();
    let mut vertices = Vec::new();

    let mut j: usize = 0;
    web_sys::console::log_1(&"  Starting meshing process...".into());

    //REPLACE 1 WITH `data.length.try_into().unwrap()` AFTER TESTING
    while j < data.length.try_into().unwrap() {

        let offset: u32 = vertices.len().try_into().expect("ERROR: Failed to convert usize into u32");
        let mut n:u32 = 0;

        // Last element in each position is a duplicate of the first
        data.positions[j].pop();

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

        j += 1;
    }
    web_sys::console::log_1(&"  Done meshing".into());
    let num_indices = indices.len().try_into().expect("ERROR: Failed to convert usize into u32");

    BufferStorage {
        vertices,
        indices,
        num_indices,
    }
}
