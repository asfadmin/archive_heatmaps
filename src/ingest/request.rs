use arrow::{
    array::{BinaryArray, Int8Array, Int64Array, RecordBatch},
    ipc::Binary,
};
use geo::{Geometry, MultiPolygon, Polygon};
use geo_traits::to_geo::ToGeoGeometry;
use leptos::logging::log;

use crate::{
    ingest::{
        async_duckdb::{AsyncDuckDBConnection, generate_duckdb_connection},
        sql::{generate_populate_sql, generate_sql},
    },
    types::{Filter, Granule},
};

// Send a request to the service for data based on the filter
pub async fn request(conn: &AsyncDuckDBConnection, filter: Filter) -> (Vec<Granule>, Vec<Polygon>) {
    log!("Request started...");

    //////////////////////////////
    //  Process Satellite Data  //
    //////////////////////////////

    let sql = &generate_sql(&filter);
    let gran_vec: Vec<Granule> = conn
        .query(sql)
        .await
        .expect("Failed to get data from DuckDB")
        .iter()
        .map(|batch| {
            batch
                .column(0)
                .as_any()
                .downcast_ref::<BinaryArray>()
                .unwrap()
                .iter()
                .zip(
                    batch
                        .column(1)
                        .as_any()
                        .downcast_ref::<Int64Array>()
                        .unwrap(),
                )
                .map(|(wkb_binary, weight)| {
                    use geo_traits::to_geo::ToGeoGeometry;
                    let poly = TryInto::<Polygon>::try_into(
                        wkb::reader::read_wkb(&hex::decode(wkb_binary.unwrap()).unwrap())
                            .unwrap()
                            .to_geometry(),
                    )
                    .unwrap();
                    Granule {
                        geometry: poly,
                        weight: weight.unwrap() as u64,
                    }
                })
                .collect::<Vec<Granule>>()
        })
        .flatten()
        .collect();

    log!("{gran_vec:?}");

    ////////////////////////////////
    //  Ingest World Border Data  //
    ////////////////////////////////

    let outline_vec: Vec<Polygon> = conn.query("SELECT geom FROM world_outline;").await.expect("Failed to get world border data").iter().map(|batch| {
        batch.column(0).as_any().downcast_ref::<BinaryArray>().unwrap().iter().map(|wkb_binary| {
                match wkb::reader::read_wkb(wkb_binary.unwrap())
                    .unwrap()
                    .to_geometry() {
                        geo::Geometry::MultiPolygon(multi_poly) => multi_poly.iter().map(|x| x.clone()).collect::<Vec<Polygon>>(),
                        geo::Geometry::Polygon(poly) => vec![poly],
                        _ => vec![]
                    }
        }).flatten().collect::<Vec<Polygon>>()
    }).flatten().collect();
    log!("{outline_vec:?}");

    // Deserialize the json into a HeatmapData struct
    log!("Data succesfully deserialized");
    (gran_vec, outline_vec)
}
