use chrono::NaiveDate;
use geojson::FeatureCollection;
use serde::{Deserialize, Serialize};

use crate::{granule::Granule, heatmap_data::HeatmapData};

#[derive(Deserialize, Serialize, Debug, PartialEq)]
pub struct HeatmapResponse {
    pub data: HeatmapData,
}

impl HeatmapResponse {
    pub fn from_geojson(
        filter: heatmap_api::Filter,
        feature_collection: &FeatureCollection,
    ) -> Self {
        let mut granules = Granule::from_feature_collection(feature_collection).unwrap();

        let data_type = filter.product_type;
        let platform_type = filter.platform_type;
        let start_date = NaiveDate::parse_from_str(&filter.start_date, "%Y-%m-%d")
            .expect("Faile to parse start_date");
        let end_date = NaiveDate::parse_from_str(&filter.end_date, "%Y-%m-%d")
            .expect("Faile to parse end_date");

        
        for granule in granules.iter_mut() {
            // Retain only those ancestors who fall within the filter
            granule.ancestors.retain(|ancestor| {
                let granule_date = ancestor.start_time.date();

                data_type.contains(&ancestor.product_type)
                    && platform_type.contains(&ancestor.platform_type)
                    && granule_date >= start_date
                    && granule_date <= end_date
            });
        }

        granules.retain(|granule| !granule.ancestors.is_empty());

        Self::from_granules(granules.clone())
    }

    fn from_granules(granules: Vec<Granule>) -> Self {
        Self {
            data: HeatmapData::from_granules(granules),
        }
    }
}
