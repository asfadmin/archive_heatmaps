use bincode::{Decode, Encode};
use serde::{Deserialize, Serialize};

use crate::granule::Granule;

#[derive(Serialize, Deserialize, Decode, Encode, Debug, PartialEq)]
pub struct HeatmapData {
    pub length: i32,
    pub positions: Vec<Vec<(f64, f64)>>,
    pub weights: Vec<u64>,
}

impl HeatmapData {
    pub fn from_granules(granules: Vec<Granule>) -> Self {
        let mut positions = Vec::new();
        let mut weights = Vec::new();

        for granule in granules.iter() {
            let mut polygons = granule.get_polygons();
            let len = polygons.len();
            weights.append(vec![granule.ancestors.len() as u64; len].as_mut());
            positions.append(polygons.as_mut());
        }

        Self {
            length: granules.len() as i32,
            positions,
            weights,
        }
    }
}
