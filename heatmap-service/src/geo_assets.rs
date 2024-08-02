use geojson::{FeatureCollection, GeoJson};

use crate::assets::Assets;
use crate::config::Config;
use crate::granule::Granule;

#[derive(Clone)]
pub struct GeoAssets {
    pub heatmap_features: Vec<Granule>,
    pub outline_features: Vec<Granule>,
}

impl GeoAssets {
    pub fn from_config(config: Config) -> Self {
        Self {
            heatmap_features: Granule::from_feature_collection(
                config.heatmap_geo_json_path.try_into().unwrap(),
            )
            .unwrap(),
            outline_features: Granule::from_feature_collection(
                std::str::from_utf8(Assets::get("outline.geojson").unwrap().data.as_ref())
                    .unwrap()
                    .parse::<GeoJson>()
                    .unwrap()
                    .try_into()
                    .unwrap(),
            )
            .unwrap(),
        }
    }
}
