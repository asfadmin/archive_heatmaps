extern crate earcutr;
use std::collections::VecDeque;
use std::time::Duration;

use async_std::task::sleep;
use geo::geometry::{Coord, LineString, Polygon};
use geo::{Simplify, TriangulateEarcut, coord};
use leptos::logging::log;
use leptos::prelude::{GetUntracked, Set, Update, signal};
use winit::event_loop::EventLoopProxy;

use super::request::request;
use crate::canvas::app::UserMessage;
use crate::canvas::geometry::BlendVertex;
use crate::ingest::request::populate_duckdb;
use crate::types;
use crate::types::Granule;

enum Data {
    Outline(Vec<Polygon>),
    Heatmap(Vec<Granule>),
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
    pub active_requests: leptos::prelude::ReadSignal<u32>,
    pub set_active_requests: leptos::prelude::WriteSignal<u32>,
    pub set_ready: leptos::prelude::WriteSignal<bool>,
    pub connection: (),
}

impl DataLoader {
    pub fn new(
        event_loop_proxy: EventLoopProxy<UserMessage<'static>>,
        set_ready: leptos::prelude::WriteSignal<bool>,
    ) -> Self {
        let (active_requests, set_active_requests) = signal(0);
        let connection = populate_duckdb();
        DataLoader {
            event_loop_proxy,
            active_requests,
            set_active_requests,
            set_ready,
            connection,
        }
    }

    // Updates signals and starts the process of requesting new data based on filter
    pub fn load_data(&self, filter: types::Filter) {
        self.set_active_requests.update(|n| *n += 1);
        self.set_ready.set(false);

        leptos::task::spawn_local(load_data_async(
            self.event_loop_proxy.clone(),
            filter,
            self.active_requests,
            self.set_active_requests,
            self.connection,
        ));
    }
}

async fn load_data_async(
    event_loop_proxy: EventLoopProxy<UserMessage<'static>>,
    filter: types::Filter,
    active_requests: leptos::prelude::ReadSignal<u32>,
    set_active_requests: leptos::prelude::WriteSignal<u32>,
    connection: (),
) {
    // Request data from the server
    let (data, outline_data) = request(&connection, filter).await;

    log!("Active Requests: {:?}", active_requests.get_untracked());
    // Convert the data into a triangular mesh
    if active_requests.get_untracked() == 1 {
        log!("Meshing data...");
        let meshed_data = mesh_data(Data::Heatmap(data));
        let meshed_outline_data = mesh_data(Data::Outline(outline_data));

        // Send the triangular mesh to the event loop
        log!("Sending Mesh to event loop");
        sleep(Duration::new(10, 0)).await;
        let _ = event_loop_proxy
            .send_event(UserMessage::IncomingData(meshed_data, meshed_outline_data));
    }
    set_active_requests.update(|n| *n -= 1);
}

/// Converts the passed data into a triangular mesh using the earcutting algorithm,
///     this is done for a varying level of detail to allow for LODs, polygon simplification
///     is done using the Ramer-Douglas-Peucker algorithm
fn mesh_data(data_exterior: Data) -> Vec<BufferStorage> {
    let mut positions: Vec<Vec<(f64, f64)>>;
    let mut weights: Vec<u64>;

    match data_exterior {
        Data::Outline(_outline_data) => {
            positions = vec![];
            weights = vec![0; positions.len()];
        }

        Data::Heatmap(heatmap_data) => {
            positions = vec![];
            weights = vec![];
            for gran in heatmap_data {
                positions.push(
                    gran.geometry
                        .exterior()
                        .points()
                        .map(|x| (x.x(), x.y()))
                        .collect(),
                );
                weights.push(gran.weight);
            }
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
