use arrow::array::{BinaryArray, Int64Array};
use geo::Polygon;
use geo_traits::to_geo::ToGeoGeometry;
use leptos::logging::log;
use wkb::reader::read_wkb;

use crate::{
    ingest::{async_duckdb::AsyncDuckDBConnection, sql::generate_sql},
    types::{Filter, Granule},
};

// Send a request to the service for data based on the filter
pub async fn request(conn: &AsyncDuckDBConnection, filter: Filter) -> (Vec<Granule>, Vec<Polygon>) {
    log!("Request started...\n\t{:?}", filter.date_range);

    //////////////////////////////
    //  Process Satellite Data  //
    //////////////////////////////

    let sql = &generate_sql(&filter);
    let gran_vec: Vec<Granule> = conn
        .query(sql)
        .await
        .expect("Failed to get data from DuckDB")
        .iter()
        .flat_map(|batch| {
            batch
                .column(0)
                .as_any()
                .downcast_ref::<BinaryArray>()
                .expect("Sat_data geometry was not a BinaryArray")
                .iter()
                .zip(
                    batch
                        .column(1)
                        .as_any()
                        .downcast_ref::<Int64Array>()
                        .expect("Weights were not a Int64Array"),
                )
                .map(|(wkb_binary, weight)| {
                    use geo_traits::to_geo::ToGeoGeometry;
                    let poly = TryInto::<Polygon>::try_into(
                        read_wkb(&wkb_binary.expect("Failed to get [u8] from wkb"))
                            .expect("Failed to convert sat_data geometry to geo::geometry")
                            .to_geometry(),
                    )
                    .expect("Failed to convert sat data to Polygon");
                    Granule {
                        geometry: poly,
                        weight: weight.expect("Failed to get weight for a polygon") as u64,
                    }
                })
                .collect::<Vec<Granule>>()
        })
        .collect();

    log!("{gran_vec:?}");

    ////////////////////////////////
    //  Ingest World Border Data  //
    ////////////////////////////////

    let outline_vec: Vec<Polygon> = conn
        .query("SELECT geom FROM world_outline;")
        .await
        .expect("Failed to get world border data")
        .iter()
        .flat_map(|batch| {
            batch
                .column(0)
                .as_any()
                .downcast_ref::<BinaryArray>()
                .expect(
                    "DuckDB did not return a BinaryArray for the geometry column of world outline",
                )
                .iter()
                .flat_map(|wkb_binary| {
                    match read_wkb(
                        wkb_binary.expect("Failed to read wkb_binary from geometry column"),
                    )
                    .expect("Failed to convert wkb to geometry")
                    .to_geometry()
                    {
                        geo::Geometry::MultiPolygon(multi_poly) => {
                            multi_poly.iter().cloned().collect::<Vec<Polygon>>()
                        }
                        geo::Geometry::Polygon(poly) => vec![poly],
                        _ => vec![],
                    }
                })
                .collect::<Vec<Polygon>>()
        })
        .collect();
    log!("{outline_vec:?}");

    // Deserialize the json into a HeatmapData struct
    log!("Data succesfully deserialized");
    (gran_vec, outline_vec)
}
