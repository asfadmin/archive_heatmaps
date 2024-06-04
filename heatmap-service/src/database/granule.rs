use actix_web::Error;
use chrono::NaiveDateTime;
use deadpool_postgres::Client;
use geo_types::Point;
use serde::{Deserialize, Serialize};
use tokio_pg_mapper::FromTokioPostgresRow;
use tokio_pg_mapper_derive::PostgresMapper;

use crate::error::ActixMapResult;

use super::{Dataset, ToPartialString};

#[derive(Debug, Deserialize, PostgresMapper, Serialize, Clone)]
#[pg_mapper(table = "cmr")]
pub struct Granule {
    pub dataset: String,
    pub data_transferred: i64,
    pub midpoint: Point<f64>,
    pub time: NaiveDateTime,
}

impl Granule {
    pub async fn from_database(
        dataset: Option<Dataset>,
        client: &Client,
    ) -> Result<Vec<Self>, Error> {
        let query = client
            .prepare(include_str!("../../sql/fetch_dataset.sql"))
            .await
            .actix_map_result()?;

        let granules = client
            .query(&query, &[&dataset.to_partial_string()])
            .await
            .actix_map_result()?;

        granules
            .iter()
            .map(Self::from_row_ref)
            .collect::<Result<Vec<Granule>, tokio_pg_mapper::Error>>()
            .actix_map_result()
    }
}
