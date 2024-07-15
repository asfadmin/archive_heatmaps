use geojson::FeatureCollection;
use serde::{Deserialize, Serialize};

use crate::{dataset::Dataset, granule::Granule, heatmap_data::HeatmapData};

#[derive(Deserialize, Serialize, Debug, PartialEq)]
pub struct HeatmapResponse {
    pub data: HeatmapData,
}

impl HeatmapResponse {
    pub fn from_geojson(filter: String, feature_collection: &FeatureCollection) -> Self {
        let granules = Granule::from_feature_collection(filter, feature_collection).unwrap();

        Self::from_granules(granules)
    }

    fn from_granules(granules: Vec<Granule>) -> Self {
        Self {
            data: HeatmapData::from_granules(granules),
        }
    }
}
