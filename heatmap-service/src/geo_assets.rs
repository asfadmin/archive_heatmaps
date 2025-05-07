use geojson::GeoJson;

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
                config
                    .heatmap_geo_json_path
                    .try_into()
                    .expect("Failed to get sat_data.geojson"),
            )
            .expect("Failed to convert sat_data.geojson to Granules"),
            outline_features: Granule::from_feature_collection(
                std::str::from_utf8(
                    Assets::get("outline.geojson")
                        .expect("failed to get outline.geojson")
                        .data
                        .as_ref(),
                )
                .expect("Failed to convert outline to str")
                .parse::<GeoJson>()
                .expect("Failed to parse outline GeoJson")
                .try_into()
                .expect("Failed to convert outline to a FeatureCollection"),
            )
            .unwrap(),
        }
    }
}
