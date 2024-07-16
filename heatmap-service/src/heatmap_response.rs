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

        let filters: Vec<&str> = filter.split(" ").collect();
        let data_type: Vec<&str> = filters[0].split("/").collect();
        println!("Data Types Filter: {:?}", data_type);
        let platform_type: Vec<&str> = filters[1].split("/").collect();
        println!("Platform Type Filter: {:?}", platform_type);

        let mut i = 0;
        while i < granules.len() {
            let mut j = 0;
            while j < granules[i].ancestors.len() {
                let granule_name = &granules[i].ancestors[j].granule_name;

                println!("{:?}", &granule_name[0..3]);

                if !data_type.contains(&&granule_name[7..10])
                    || !platform_type.contains(&&granule_name[0..3])
                {
                    println!("Attempting to remove ancestor {:?}", j);
                    println!("Granule Name: {:?}", granule_name);
                    granules[i].ancestors.remove(j);
                    println!("{:?}", granules[i].ancestors.len());
                } else {
                    j += 1;
                }
            }

            if granules[i].ancestors.len() == 0 {
                println!("Removing granule");
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
