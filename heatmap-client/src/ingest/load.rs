extern crate earcutr;
use std::collections::VecDeque;

use geo::geometry::Polygon;
use geo::{coord, TriangulateEarcut};
use winit::event_loop::EventLoopProxy;

use super::request::request;
use crate::canvas::app::UserMessage;
use crate::canvas::geometry::Vertex;

pub struct BufferStorage {
    pub vertices: Vec<Vertex>,
    pub indices: Vec<u32>,
    pub num_indices: u32,
}
pub struct DataLoader {
    pub event_loop_proxy: EventLoopProxy<UserMessage<'static>>,
}

impl DataLoader {
    pub fn load_data(&self, filter: heatmap_api::Filter) {
        leptos::spawn_local(load_data_async(self.event_loop_proxy.clone(), filter));
    }
}

async fn load_data_async(
    event_loop_proxy: EventLoopProxy<UserMessage<'static>>,
    filter: heatmap_api::Filter,
) {
    // Request data from the server
    let data = request(filter).await;

    // Convert the data into a triangular mesh
    web_sys::console::log_1(&"Meshing data...".into());
    let meshed_data = mesh_data(data);

    // Send the triangular mesh to the event loop
    web_sys::console::log_1(&"Sending Mesh to event loop".into());
    let _ = event_loop_proxy.send_event(UserMessage::IncomingData(meshed_data));
}

fn mesh_data(data_exterior: heatmap_api::HeatmapData) -> BufferStorage {
    let data = data_exterior.data;

    let mut weights = VecDeque::from(data.weights);

    let mut polygons: Vec<geo::Polygon> = data
        .positions
        .iter()
        .map(|poly| {
            poly.iter()
                .map(|(x, y)| {
                    coord! {x: *x, y: *y}
                })
                .collect()
        })
        .map(|mut exterior: Vec<geo::Coord>| {
            // Last entry is a duplicate of the first
            let _ = exterior.pop();
            geo::Polygon::new(geo::LineString(exterior.clone()), Vec::new())
        })
        .collect();

    let mut total_vertices: Vec<Vertex> = Vec::new();
    let mut indices: Vec<u32> = Vec::new();

    for poly in polygons.iter_mut() {
        // Run the ear cutting algorithm, triangles contains a list of indices after
        let triangles_raw = poly.earcut_triangles_raw();

        // Append current indices to the end of prior indices with offset
        let offset = total_vertices.len();
        for indice in triangles_raw.triangle_indices.iter() {
            indices.push(
                (indice + offset)
                    .try_into()
                    .expect("ERROR: Failed to convert usize to u32"),
            );
        }

        // Place data for each vertex into a vertex struct
        let weight = weights
            .pop_front()
            .expect("Weights was not equal to the number of polygons");
        let mut i = 0;
        while i < triangles_raw.vertices.len() {
            total_vertices.push(Vertex {
                position: [
                    triangles_raw.vertices[i] as f32,
                    triangles_raw.vertices[i + 1] as f32 as f32,
                    0.0,
                ],
                weight: weight as u32,
            });

            i += 2;
        }

        i += 1;
    }

    let num_indices = indices
        .len()
        .try_into()
        .expect("ERROR: Failed to convert usize into u32");

    BufferStorage {
        vertices: total_vertices,
        indices,
        num_indices,
    }
}
