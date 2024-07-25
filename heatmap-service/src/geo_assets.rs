use geojson::{FeatureCollection, GeoJson};

use crate::assets::Assets;
use crate::config::Config;

#[derive(Clone)]
pub struct GeoAssets {
    pub heatmap_features: FeatureCollection,
    pub outline_features: FeatureCollection,
}

impl GeoAssets {
    pub fn from_config(config: Config) -> Self {
        Self {
            heatmap_features: config.heatmap_geo_json_path.try_into().unwrap(),
            outline_features: std::str::from_utf8(
                Assets::get("outline.geojson").unwrap().data.as_ref(),
            )
            .unwrap()
            .parse::<GeoJson>()
            .unwrap()
            .try_into()
            .unwrap(),
        }
    }
}
