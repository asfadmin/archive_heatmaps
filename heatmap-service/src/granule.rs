use chrono::NaiveDateTime;
use geo::Polygon;
use geojson::{Feature, FeatureCollection};
use serde::Serialize;

use crate::dataset::Dataset;

#[derive(Debug, Serialize, Clone)]
pub struct Granule {
    pub polygon: Polygon,
    pub ancestors: Vec<Ancestor>,
}

impl TryFrom<&Feature> for Granule {
    type Error = Box<dyn std::error::Error>;

    fn try_from(feature: &Feature) -> Result<Granule, Self::Error> {
        let ancestors: Vec<serde_json::Map<String, serde_json::Value>> = feature
            .properties
            .clone()
            .ok_or("feature has no properties associated with it")?
            .get("ancestors")
            .expect("failed to get ancestors")
            .clone()
            .as_array()
            .expect("failed to convert to array")
            .iter()
            .map(|feature| {
                feature
                    .as_object()
                    .expect("failed to map ancestors")
                    .clone()
            })
            .collect();

        Ok(Granule {
            polygon: feature
                .geometry
                .clone()
                .ok_or("no geometry attached to feature")?
                .value
                .try_into()
                .expect("failed to convert geomery to polygon"),
            ancestors: ancestors
                .iter()
                .map(|feature| feature.try_into().expect("failed to map ancestors"))
                .collect(),
        })
    }
}

#[derive(Debug, Serialize, Clone)]
pub struct Ancestor {
    granule_name: String,
    platform_type: String,
    data_sensor: String,
    start_time: NaiveDateTime,
    end_time: NaiveDateTime,
}

impl TryFrom<&serde_json::Map<String, serde_json::Value>> for Ancestor {
    type Error = actix_web::error::Error;

    fn try_from(
        properties: &serde_json::Map<String, serde_json::Value>,
    ) -> Result<Ancestor, Self::Error> {
        Ok(Ancestor {
            granule_name: properties
                .get("GRANULE_NA")
                .expect("failed to get granule name")
                .as_str()
                .expect("failed to convert granule name to str")
                .to_string(),
            platform_type: properties
                .get("PLATFORM_T")
                .expect("failed to get platform type")
                .as_str()
                .expect("failed to convert platform type to str")
                .to_string(),
            data_sensor: properties
                .get("DATA_SENSO")
                .expect("failed to get data sensor")
                .as_str()
                .expect("failed to convert data sensor to string")
                .to_string(),
            start_time: NaiveDateTime::parse_from_str(
                properties
                    .get("START_TIME")
                    .expect("failed to get start time")
                    .as_str()
                    .expect("failed to convert start time to str"),
                "%Y-%m-%d %H:%M:%S",
            )
            .expect("failed to parse start time from string"),
            end_time: NaiveDateTime::parse_from_str(
                properties
                    .get("END_TIME")
                    .expect("failed to get end time")
                    .as_str()
                    .expect("failed to convert end time to str"),
                "%Y-%m-%d %H:%M:%S",
            )
            .expect("failed to parse end time from string"),
        })
    }
}

impl Granule {
    pub fn from_feature_collection(
        _dataset: Option<Dataset>,
        feature_collection: &FeatureCollection,
    ) -> Result<Vec<Granule>, Box<dyn std::error::Error>> {
        feature_collection
            .features
            .iter()
            .map(Self::try_from)
            .collect()
    }
}
