extern crate earcutr;
use std::collections::VecDeque;

use geo::geometry::{Coord, LineString, Polygon};
use geo::{coord, Simplify, TriangulateEarcut};
use heatmap_api::{HeatmapData, OutlineResponse};
use leptos::{create_signal, SignalGetUntracked, SignalSet, SignalUpdate};
use winit::event_loop::EventLoopProxy;

use super::request::request;
use crate::canvas::app::UserMessage;
use crate::canvas::geometry::BlendVertex;

enum Data {
    Outline(OutlineResponse),
    Heatmap(HeatmapData),
}

#[derive(Clone)]
pub struct BufferStorage {
    pub vertices: Vec<BlendVertex>,
    pub indices: Vec<u32>,
    pub num_indices: u32,
}

// Struct that is responsible for submitting requests to the service for new data
pub struct DataLoader {
    pub event_loop_proxy: EventLoopProxy<UserMessage<'static>>,
    pub active_requests: leptos::ReadSignal<u32>,
    pub set_active_requests: leptos::WriteSignal<u32>,
    pub set_ready: leptos::WriteSignal<bool>,
}

impl DataLoader {
    pub fn new(
        event_loop_proxy: EventLoopProxy<UserMessage<'static>>,
        set_ready: leptos::WriteSignal<bool>,
    ) -> Self {
        let (active_requests, set_active_requests) = create_signal(0);

        DataLoader {
            event_loop_proxy,
            active_requests,
            set_active_requests,
            set_ready,
        }
    }

    // Updates signals and starts the process of requesting new data based on filter
    pub fn load_data(&self, filter: heatmap_api::Filter) {
        self.set_active_requests.update(|n| *n += 1);
        self.set_ready.set(false);

        leptos::spawn_local(load_data_async(
            self.event_loop_proxy.clone(),
            filter,
            self.active_requests,
            self.set_active_requests,
        ));
    }
}

async fn load_data_async(
    event_loop_proxy: EventLoopProxy<UserMessage<'static>>,
    filter: heatmap_api::Filter,
    active_requests: leptos::ReadSignal<u32>,
    set_active_requests: leptos::WriteSignal<u32>,
) {
    // Request data from the server
    let (data, outline_data) = request(filter).await;

    web_sys::console::log_1(
        &format!("Active Requests: {:?}", active_requests.get_untracked()).into(),
    );
    // Convert the data into a triangular mesh
    if active_requests.get_untracked() == 1 {
        web_sys::console::log_1(&"Meshing data...".into());
        let meshed_data = mesh_data(Data::Heatmap(data));
        let meshed_outline_data = mesh_data(Data::Outline(outline_data));

        // Send the triangular mesh to the event loop
        web_sys::console::log_1(&"Sending Mesh to event loop".into());
        let _ = event_loop_proxy
            .send_event(UserMessage::IncomingData(meshed_data, meshed_outline_data));
    }
    set_active_requests.update(|n| *n -= 1);
}

/// Converts the passed data into a triangular mesh using the earcutting algorithm,
///     this is done for a varying level of detail to allow for LODs, polygon simplification
///     is done using the Ramer-Douglas-Peucker algorithm
fn mesh_data(data_exterior: Data) -> Vec<BufferStorage> {
    let positions: Vec<Vec<(f64, f64)>>;
    let weights: Vec<u64>;

    match data_exterior {
        Data::Outline(outline_data) => {
            positions = outline_data.data.positions;
            weights = vec![0; positions.len()];
        }

        Data::Heatmap(heatmap_data) => {
            positions = heatmap_data.data.positions;
            weights = heatmap_data.data.weights;
        }
    }

    let mut lods: Vec<BufferStorage> = Vec::new();

    let mut polygons: Vec<Polygon> = positions
        .iter()
        .map(|poly| {
            poly.iter()
                .map(|(x, y)| {
                    coord! {x: *x, y: *y}
                })
                .collect()
        })
        .map(|mut exterior: Vec<Coord>| {
            // Last entry is a duplicate of the first
            let _ = exterior.pop();
            Polygon::new(LineString(exterior.clone()), Vec::new())
        })
        .collect();

    let mut level = 0.0;
    while level <= 0.4 {
        let mut weights = VecDeque::from(weights.clone());
        let mut total_vertices: Vec<BlendVertex> = Vec::new();
        let mut indices: Vec<u32> = Vec::new();

        for poly in polygons.iter_mut() {
            let simplified = poly.simplify(&level);
            // Run the ear cutting algorithm, triangles contains a list of indices after
            let triangles_raw = simplified.earcut_triangles_raw();

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
                total_vertices.push(BlendVertex {
                    position: [
                        triangles_raw.vertices[i] as f32,
                        triangles_raw.vertices[i + 1] as f32,
                        0.0,
                    ],
                    weight: weight as u32,
                });

                i += 2;
            }
        }

        let num_indices = indices
            .len()
            .try_into()
            .expect("ERROR: Failed to convert usize into u32");

        lods.push(BufferStorage {
            vertices: total_vertices,
            indices,
            num_indices,
        });

        level += 0.2;
    }
    lods
}
