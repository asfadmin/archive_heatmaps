extern crate earcutr;
use std::collections::VecDeque;
use std::rc::Rc;
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
use crate::ingest::async_duckdb::{AsyncDuckDBConnection, generate_duckdb_connection};
use crate::ingest::sql::generate_ingest_world_outline_sql;
use crate::ingest::sql::generate_populate_sql;
use crate::types::{self, DateRange};
use crate::types::{Filter, Granule};
use crate::ingest::mesh::mesh_data;

pub enum Data {
    Outline(Vec<Polygon>),
    Heatmap(Vec<Granule>),
}

#[derive(Clone, Debug)]
pub struct BufferStorage {
    pub vertices: Vec<BlendVertex>,
    pub indices: Vec<u32>,
    pub num_indices: u32,
}

// Struct that is responsible for submitting storing/subsetting data with DuckDB
pub struct DataLoader {
    pub event_loop_proxy: EventLoopProxy<UserMessage<'static>>,
    pub active_requests: leptos::prelude::ReadSignal<u32>,
    pub set_active_requests: leptos::prelude::WriteSignal<u32>,
    pub set_ready: leptos::prelude::WriteSignal<bool>,
    pub connection: Rc<AsyncDuckDBConnection>,
    ingested_data: Vec<DateRange>,    
}

impl DataLoader {
    pub async fn new(
        event_loop_proxy: EventLoopProxy<UserMessage<'static>>,
        set_ready: leptos::prelude::WriteSignal<bool>,
        filter: &Filter
    ) -> Self {
        let (active_requests, set_active_requests) = signal(0);
        // TO-DO: Make data ingest incremental based on needed data
        let connection = Rc::new(
            generate_duckdb_connection()
                .await
                .expect("Failed to get connection to DuckDB"),
        );
        connection
            .query("LOAD httpfs;")
            .await
            .expect("Failed to install/load httpfs in DuckDB");
        connection
            .query("LOAD spatial;")
            .await
            .expect("Failed to install/load spatial in DuckDB");

        connection
            .query(&generate_populate_sql(&filter.date_range))
            .await
            .expect("Failed to populate DuckDB wit satellite data");
        connection
            .query(&generate_ingest_world_outline_sql())
            .await
            .expect("Failed to ingest world outlines in DuckDB");
        DataLoader {
            event_loop_proxy,
            active_requests,
            set_active_requests,
            set_ready,
            connection,
            ingested_data: vec![filter.date_range.clone()]
        }
    }

    // Updates signals and starts the process of requesting new data based on filter
    pub fn load_data(&self, filter: Filter) {
        self.set_active_requests.update(|n| *n += 1);
        self.set_ready.set(false);

        // Check for missing data in DuckDB
        let missing: Vec<DateRange> = self.ingested_data
            .iter()
            .flat_map(|x| {
                x.get_disjoint(&filter.date_range)
            })
            .flatten()
            .fold(Vec::<DateRange>::new(), |mut acc, x| {
                log!("acc: {acc:?}");
                if acc.len() == 0 {
                    log!("First DateRange: {x:?}");
                    acc.push(x);
                    acc
                } else {
                    acc.into_iter().flat_map(|mut y| {
                        if let Err(_) = y.merge(&x) {
                            log!("Failed to merge {x:?}, pushing to acc");
                            return vec![y, x.clone()]
                        } else {
                            log!("Merged {x:?} and {y:?}")
                        }
                        return vec![y]
                    }).collect()
                }
            });
        log!("Filter: {:?}", filter.date_range);
        log!("Missing: {missing:?}");
        

        leptos::task::spawn_local(load_data_async(
            self.event_loop_proxy.clone(),
            filter,
            self.active_requests,
            self.set_active_requests,
            self.connection.clone(),
        ))
    }
}

async fn load_data_async(
    event_loop_proxy: EventLoopProxy<UserMessage<'static>>,
    filter: types::Filter,
    active_requests: leptos::prelude::ReadSignal<u32>,
    set_active_requests: leptos::prelude::WriteSignal<u32>,
    connection: Rc<AsyncDuckDBConnection>,
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
