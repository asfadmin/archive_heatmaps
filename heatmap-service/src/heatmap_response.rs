use actix_web::Error;
use chrono::{DateTime, Utc};
use geojson::FeatureCollection;
use serde::{Deserialize, Serialize};

use crate::{dataset::Dataset, granule::Granule, heatmap_data::HeatmapData};

#[derive(Deserialize, Serialize, Debug, PartialEq)]
pub struct HeatmapResponse {
    pub data: HeatmapData,
}

impl HeatmapResponse {
    pub fn from_geojson(dataset: Option<Dataset>, feature_collection: &FeatureCollection) -> Self {
        let granules = Granule::from_feature_collection(dataset, feature_collection).unwrap();

        Self::from_granules(granules)
    }

    fn from_granules(granules: Vec<Granule>) -> Self {
        Self {
            data: HeatmapData::from_granules(granules),
        }
    }
}
