use chrono::NaiveDate;
use geojson::FeatureCollection;
use serde::{Deserialize, Serialize};

use crate::{granule::Granule, heatmap_data::HeatmapData};

#[derive(Deserialize, Serialize, Debug, PartialEq)]
pub struct HeatmapResponse {
    pub data: HeatmapData,
}

impl HeatmapResponse {
    pub fn from_geojson(filter: String, feature_collection: &FeatureCollection) -> Self {
        let mut granules = Granule::from_feature_collection(feature_collection).unwrap();

        // Parsing the filter string, " " denotes seperate categorys, / denotes different filters in each category
        let filters: Vec<&str> = filter.split(" ").collect();
        let data_type: Vec<&str> = filters[0].split("/").collect();
        let platform_type: Vec<&str> = filters[1].split("/").collect();
        let start_date: NaiveDate =
            NaiveDate::parse_from_str(filters[2], "%Y-%m-%d").expect("Faile to parse start_date");
        let end_date: NaiveDate =
            NaiveDate::parse_from_str(filters[3], "%Y-%m-%d").expect("Faile to parse end_date");

        let mut i = 0;
        while i < granules.len() {
            let mut j = 0;
            while j < granules[i].ancestors.len() {
                // Get granule name and parse the start date from this string
                let granule_name = &granules[i].ancestors[j].granule_name;
                let gran_date = parse_granule_start(granule_name);

                // Remove any ancestors outside of the filter, see Sentinel-1 granule name conventions to clarify string slices
                if !data_type.contains(&&granule_name[7..10])
                    || !platform_type.contains(&&granule_name[0..3])
                    || gran_date < start_date
                    || gran_date > end_date
                {
                    granules[i].ancestors.remove(j);
                } else {
                    j += 1;
                }
            }

            // If we removed all ancestors remove the granule
            if granules[i].ancestors.is_empty() {
                granules.remove(i);
            } else {
                i += 1;
            }
        }

        Self::from_granules(granules.clone())
    }

    fn from_granules(granules: Vec<Granule>) -> Self {
        Self {
            data: HeatmapData::from_granules(granules),
        }
    }
}

/// Parses a Sentinel-1 granule name into a NaiveDate containing the granule start date
fn parse_granule_start(granule_name: &str) -> NaiveDate {
    let year: i32 = granule_name[17..21]
        .to_string()
        .parse()
        .expect("Failed to parse gran start year");
    let month: u32 = granule_name[21..23]
        .to_string()
        .parse()
        .expect("Failed to parse gran start month");
    let day: u32 = granule_name[23..25]
        .to_string()
        .parse()
        .expect("Failed to parse gran start day");

    NaiveDate::from_ymd_opt(year, month, day).expect("Failed to parse NaiveDate from granule name")
}
