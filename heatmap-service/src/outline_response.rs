use serde::{Deserialize, Serialize};

use crate::granule::Granule;

#[derive(Deserialize, Serialize, Debug, PartialEq)]
pub struct OutlineData {
    pub length: i32,
    pub positions: Vec<Vec<(f64, f64)>>,
}

impl OutlineData {
    pub fn from_granules(granules: &[Granule]) -> Self {
        let mut positions = Vec::new();

        for granule in granules.iter() {
            let mut polygons = granule.get_polygons();
            positions.append(polygons.as_mut());
        }

        Self {
            length: granules.len() as i32,
            positions,
        }
    }
}

#[derive(Deserialize, Serialize, Debug, PartialEq)]
pub struct OutlineResponse {
    pub data: OutlineData,
}

impl OutlineResponse {
    pub fn from_geojson(granules: &[Granule]) -> Self {
        Self::from_granules(granules)
    }

    fn from_granules(granules: &[Granule]) -> Self {
        Self {
            data: OutlineData::from_granules(granules),
        }
    }
}
