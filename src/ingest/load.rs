extern crate earcutr;
use std::collections::VecDeque;
use std::rc::Rc;
use std::sync::Mutex;
use std::sync::atomic::{AtomicBool, AtomicUsize};
use std::time::Duration;
use std::sync::Arc;
use std::sync::atomic::Ordering;
use std::cell::RefCell;

use async_std::task::{sleep, yield_now};
use geo::geometry::Polygon;
use leptos::logging::log;
use leptos::prelude::{GetUntracked, Set, Update, signal};
use winit::event_loop::EventLoopProxy;

use super::request::request;
use crate::canvas::app::UserMessage;
use crate::canvas::geometry::BlendVertex;
use crate::ingest::async_duckdb::{AsyncDuckDBConnection, generate_duckdb_connection};
use crate::ingest::mesh::mesh_data;
use crate::ingest::sql::generate_ingest_world_outline_sql;
use crate::ingest::sql::{generate_create_sat_data_sql, generate_populate_sat_data_sql};
use crate::types::{self, DateRange};
use crate::types::{Filter, Granule};

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
    ingested_data: Mutex<Vec<DateRange>>,
    ingest_queue: Rc<Mutex<VecDeque<String>>>,
    ingest_flag: Rc<RefCell<AtomicBool>>,
    ingest_filter: Rc<RefCell<Filter>>,
}

impl DataLoader {
    pub async fn new(
        event_loop_proxy: EventLoopProxy<UserMessage<'static>>,
        set_ready: leptos::prelude::WriteSignal<bool>,
        filter: &Filter,
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
            .query(&generate_create_sat_data_sql())
            .await
            .expect("Failed to create sat_data table");

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
            ingested_data: Mutex::new(vec![]),
            ingest_queue: Rc::new(Mutex::new(VecDeque::new())),
            ingest_flag: Rc::new(RefCell::new(AtomicBool::new(false))),
            ingest_filter: Rc::new(RefCell::new(filter.clone()))
        }
    }

    // Updates signals and starts the process of requesting new data based on filter
    pub fn load_data(&self, filter: Filter) {
        self.set_active_requests.update(|n| *n += 1);
        self.set_ready.set(false);

        {
            // Check for missing data in DuckDB
            let mut data_guard = self
                .ingested_data
                .lock()
                .expect("Failed to get mutex lock for ingested data, mutex poisoned");
            let mut missing: Vec<DateRange> = (*data_guard)
                .iter()
                .flat_map(|x| x.get_disjoint(&filter.date_range))
                .flatten()
                .fold(Vec::<DateRange>::new(), |mut acc, x| {
                    log!("acc: {acc:?}");
                    if acc.is_empty() {
                        log!("First DateRange: {x:?}");
                        acc.push(x);
                        acc
                    } else {
                        acc.into_iter()
                            .flat_map(|mut y| {
                                if let Err(_) = y.merge(&x) {
                                    log!("Failed to merge {x:?}, pushing to acc");
                                    return vec![y, x.clone()];
                                } else {
                                    log!("Merged {x:?} and {y:?}")
                                }
                                vec![y]
                            })
                            .collect()
                    }
                });
            if data_guard.is_empty() {
                missing = vec![filter.date_range.clone()];
            }
            log!("Filter: {:?}", filter.date_range);
            log!("Missing: {missing:?}");
            
            let mut queue_guard = self.ingest_queue.lock().expect("Failed to lock ingest queue, mutex poisoned");
            for range in missing {
                // new_range is range clipped to file resolution so start: 2020-01-05, end: 2020-01-07
                // becomes start: 2020-01-01, end 2020-02-01 since the smallest time unit we can ingest is one month
                let (sql, new_range) = generate_populate_sat_data_sql(&range);
                for stmt in sql {
                    queue_guard.push_back(stmt);
                }
                
                let mut merged = false;
                for ingested_range in &mut (*data_guard) {
                    if let Ok(_) = ingested_range.merge(&new_range) {
                        merged = true;
                    }
                }
                if !merged {
                    data_guard.push(new_range);
                }

                log!("New Ingested Data Range: {data_guard:?}");
            }
        }

        *self.ingest_filter.borrow_mut() = filter.clone();

        if !(self.ingest_flag.borrow().load(Ordering::Acquire)) {
            self.ingest_flag.borrow_mut().store(false, Ordering::Release);
            leptos::task::spawn_local(load_data_async(
                self.event_loop_proxy.clone(),
                self.active_requests,
                self.set_active_requests,
                self.connection.clone(),
                self.ingest_queue.clone(),
                self.ingest_flag.clone(),
                self.ingest_filter.clone(),
            ))
        }
        
    }
}

async fn load_data_async(
    event_loop_proxy: EventLoopProxy<UserMessage<'static>>,
    active_requests: leptos::prelude::ReadSignal<u32>,
    set_active_requests: leptos::prelude::WriteSignal<u32>,
    connection: Rc<AsyncDuckDBConnection>,
    ingest_queue: Rc<Mutex<VecDeque<String>>>,
    ingest_flag: Rc<RefCell<AtomicBool>>,
    ingest_filter: Rc<RefCell<Filter>>,
) {

    loop {  
        let mut guard = ingest_queue.lock().expect("Failed to get lock for ingest queue, mutex poisoned");
        let sql_vec = guard.pop_front().clone();
        drop(guard);

        if let Some(sql) = sql_vec {
            log!("SQL: {sql:?}");

            // Ingest Missing Data
            log!("Executing: {sql}");
            if let Err(e) = connection.query(&sql).await {
                log!("Error while ingesting data: {e:?}");
            }
        } else {
            break;
        }
    }

    // Unset ingest flag to allow future data loading
    ingest_flag.borrow_mut().store(false, Ordering::Release);

    log!("Active Requests: {:?}", active_requests.get_untracked());
    // Convert the data into a triangular mesh
    if active_requests.get_untracked() == 1 {
        // Request data from the server
        let (data, outline_data) = request(&connection, ingest_filter.borrow().clone()).await;
        
        log!("Meshing data...");
        let meshed_data = mesh_data(Data::Heatmap(data));
        let meshed_outline_data = mesh_data(Data::Outline(outline_data));

        // Send the triangular mesh to the event loop
        log!("Sending Mesh to event loop");
        let _ = event_loop_proxy
            .send_event(UserMessage::IncomingData(meshed_data, meshed_outline_data));
    }
    set_active_requests.update(|n| *n -= 1);
}
