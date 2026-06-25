use geo::Polygon;
use leptos::logging::log;

use crate::{
    ingest::{
        async_duckdb::{AsyncDuckDBConnection, generate_duckdb_connection},
        sql::{generate_populate_sql, generate_sql},
    },
    types::{Filter, Granule},
};

pub fn populate_duckdb() -> () {
    let conn = (); // Connection::open_in_memory().expect("Failed to open in memory DuckDB");
    // conn.execute("LOAD '/var/runtime/httpfs.duckdb_extension';", [])?;

    log!("Populating duckdb...");
    let _sql = generate_populate_sql();
    // conn.execute(&sql, [])?;

    // conn.query_row("SELECT COUNT(*) FROM sat_data;", [], |row| {
    //     if let Ok(val) = row.get::<usize, usize>(0) {
    //         log!("Loaded {:?} Rows!", val);
    //     } else {
    //         log!("Failed to get row count");
    //     }

    //     Ok(())
    // });

    conn
}

// Send a request to the service for data based on the filter
pub async fn request(conn: &AsyncDuckDBConnection, filter: Filter) -> (Vec<Granule>, Vec<Polygon>) {
    log!("Request started...");

    let _sql = &generate_sql(&filter);
    // let mut stmt = conn.prepare(sql?);

    // let gran_vec: Vec<Granule>  = stmt
    //     .query_map([], |row| {
    //         if let Ok(wkb_binary) = hex::decode(row.get::<usize, Vec<u8>>(0)?)
    //             && let Ok(geom) = wkb::reader::read_wkb(&wkb_binary)
    //             && let Ok(poly) = geom.to_geometry().try_into()
    //         {
    //             let weight: u64 = row.get(1)?;

    //             Ok(Granule {
    //                 geometry: poly,
    //                 weight,
    //             })
    //         } else {
    //             Err(duckdb::Error::InvalidColumnType(
    //                 1,
    //                 "geometry".to_string(),
    //                 duckdb::types::Type::Enum,
    //             ))
    //         }
    //     })?
    //     .collect::<Result<Vec<Granule>, duckdb::Error>>()?;
    let gran_vec = vec![];
    let outline_vec: Vec<Polygon> = vec![];

    // Deserialize the json into a HeatmapData struct
    log!("Data succesfully deserialized");
    (gran_vec, outline_vec)
}
