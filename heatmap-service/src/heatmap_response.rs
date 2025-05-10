use bincode::{Decode, Encode};
use chrono::NaiveDate;
use rayon::prelude::*;
use serde::{Deserialize, Serialize};

use crate::{granule::Granule, heatmap_data::HeatmapData};

#[derive(Serialize, Deserialize, Decode, Encode, Debug, PartialEq)]
pub struct HeatmapResponse {
    pub data: HeatmapData,
}

impl HeatmapResponse {
    pub fn from_geojson(filter: heatmap_api::Filter, granules: &[Granule]) -> Self {
        let mut granules = granules.to_owned();

        let data_type = filter.product_type;
        let platform_type = filter.platform_type;
        let start_date = NaiveDate::parse_from_str(&filter.start_date, "%Y-%m-%d")
            .expect("Faile to parse start_date");
        let end_date = NaiveDate::parse_from_str(&filter.end_date, "%Y-%m-%d")
            .expect("Faile to parse end_date");

        granules = granules
            .par_iter_mut()
            .map(|granule| {
                // Retain only those ancestors who fall within the filter
                granule.ancestors.retain(|ancestor| {
                    let granule_date = ancestor.start_time.date();

                    data_type.contains(&ancestor.product_type)
                        && platform_type.contains(&ancestor.platform_type)
                        && granule_date >= start_date
                        && granule_date <= end_date
                });

                granule.clone()
            })
            .collect();

        granules.retain(|granule| !granule.ancestors.is_empty());

        Self::from_granules(granules)
    }

    fn from_granules(granules: Vec<Granule>) -> Self {
        Self {
            data: HeatmapData::from_granules(granules),
        }
    }
}
