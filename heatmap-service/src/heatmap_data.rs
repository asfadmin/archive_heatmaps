use serde::{Deserialize, Serialize};

use crate::granule::Granule;

#[derive(Deserialize, Serialize, Debug, PartialEq)]
pub struct HeatmapData {
    pub length: i32,
    pub positions: Vec<Vec<f64>>,
    pub weights: Vec<u64>,
}

impl HeatmapData {
    pub fn from_granules(granules: Vec<Granule>) -> Self {
        Self {
            length: granules.len() as i32,
            positions: granules
                .iter()
                .map(|granule| {
                    granule.polygon.exterior().clone().into_inner().iter().fold(
                        Vec::new(),
                        |mut inner_collector, coord| {
                            inner_collector.push(coord.x);
                            inner_collector.push(coord.y);
                            inner_collector
                        },
                    )
                })
                .collect(),
            weights: granules
                .iter()
                .map(|granule| granule.ancestors.len() as u64)
                .collect(),
        }
    }
}
