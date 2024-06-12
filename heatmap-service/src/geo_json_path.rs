use geojson::{FeatureCollection, GeoJson};
use serde::Deserialize;

#[derive(Clone, Deserialize, Debug, Default)]
pub struct GeoJsonPath(String);

impl TryInto<FeatureCollection> for GeoJsonPath {
    type Error = geojson::Error;

    fn try_into(self) -> Result<FeatureCollection, Self::Error> {
        std::fs::read_to_string(self.0)?
            .parse::<GeoJson>()?
            .try_into()
    }
}
