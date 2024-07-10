extern crate earcutr;
use winit::event_loop::EventLoopProxy;

use super::request::{request, HeatmapData};
use crate::display::app::UserMessage;
use crate::display::geometry::Vertex;

pub struct BufferStorage {
    pub vertices: Vec<Vertex>,
    pub indices: Vec<u32>,
    pub num_indices: u32,
    pub _max_weight: u64,
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
    web_sys::console::log_3(
        &"Meshed Data: \n".into(),
        &format!("Vertices: {:?}", meshed_data.vertices).into(),
        &format!("Indices: {:?}", meshed_data.indices).into(),
    );

    // Send the triangular mesh to the event loop
    web_sys::console::log_1(&"Sending Mesh to event loop".into());
    let _ = event_loop_proxy.send_event(UserMessage::IncomingData(meshed_data));
}

fn filter(data: HeatmapData) -> HeatmapData {
    data
}

// I think this is working but it has not been tested, I couldnt figure out how to get the heatmap-service set up to actually recieve the post request
fn mesh_data(data_exterior: HeatmapData) -> BufferStorage {
    let data = data_exterior.data;
    let mut total_vertices: Vec<Vertex> = Vec::new();
    let mut indices: Vec<u32> = Vec::new();

    let mut i: usize = 0;
    while i < data.positions.len() {
        // Format the polygon to conform to earcutr crate
        let mut original_polygon = data.positions[i].clone();
        let _ = original_polygon.pop();
        let mut new_polygon = Vec::<Vec<f64>>::new();
        for vertex in original_polygon {
            new_polygon.push(vec![vertex.0, vertex.1]);
        }

        // Run the ear cutting algorithm, triangles contains a list of indices after
        let (vertices, holes, dimensions) = earcutr::flatten(&vec![new_polygon.clone()]);
        let triangles = earcutr::earcut(&vertices, &holes, dimensions)
            .expect("ERROR: Faile to earcut in mesh_data()");

        // Append current indices to the end of prior indices with offset
        let mut j = 0;
        let offset = total_vertices.len();
        while j < triangles.len() {
            indices.push(
                (triangles[j] + offset)
                    .try_into()
                    .expect("ERROR: Failed to convert usize to u32"),
            );
            j += 1;
        }

        // Place data for each vertex into a vertex struct
        let mut j = 0;
        while j < new_polygon.len() {
            total_vertices.push(Vertex {
                position: [new_polygon[j][0] as f32, new_polygon[j][1] as f32, 0.0],
                weight: data.weights[i] as u32,
            });

            j += 1;
        }

        i += 1;
    }

    let num_indices = indices
        .len()
        .try_into()
        .expect("ERROR: Failed to convert usize into u32");

    // Value currently unused
    let max_weight = *data.weights.iter().max().expect("ERROR: Weights was empty");

    web_sys::console::log_1(&format!("Max Weight: {:?}", max_weight).into());

    BufferStorage {
        vertices: total_vertices,
        indices,
        num_indices,
        _max_weight: max_weight,
    }
}
