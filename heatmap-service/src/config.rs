use serde::Deserialize;

#[derive(Debug, Default, Deserialize, Clone)]
pub struct Config {
    pub server_address: String,
    pub cache_ttl: usize,
    pub heatmap_geo_json_path: crate::geo_json_path::GeoJsonPath,
    pub redis: Option<deadpool_redis::Config>,
}
