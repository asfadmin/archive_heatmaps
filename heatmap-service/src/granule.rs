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
        Ok(Granule {
            polygon: feature
                .geometry
                .clone()
                .ok_or("no geometry attached to feature")?
                .value
                .try_into()
                .unwrap(),
            ancestors: FeatureCollection::from_json_object(
                feature
                    .properties
                    .clone()
                    .ok_or("feature has no properties associated with it")?,
            )?
            .features
            .iter()
            .map(|feature| feature.try_into().unwrap())
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

impl TryFrom<&Feature> for Ancestor {
    type Error = Box<dyn std::error::Error>;

    fn try_from(feature: &Feature) -> Result<Ancestor, Self::Error> {
        let properties = feature
            .properties
            .clone()
            .ok_or("no properties on ancestor")?;
        Ok(Ancestor {
            granule_name: properties
                .get("GRANULE_NA")
                .unwrap()
                .as_str()
                .unwrap()
                .to_string(),
            platform_type: properties
                .get("PLATFORM_T")
                .unwrap()
                .as_str()
                .unwrap()
                .to_string(),
            data_sensor: properties
                .get("DATA_SENSO")
                .unwrap()
                .as_str()
                .unwrap()
                .to_string(),
            start_time: NaiveDateTime::parse_from_str(
                properties.get("START_TIME").unwrap().as_str().unwrap(),
                "%Y-%m-%d %H:%M:%S",
            )
            .unwrap(),
            end_time: NaiveDateTime::parse_from_str(
                properties.get("END_TIME").unwrap().as_str().unwrap(),
                "%Y-%m-%d %H:%M:%S",
            )
            .unwrap(),
        })
    }
}

impl Granule {
    pub fn from_feature_collection(
        dataset: Option<Dataset>,
        feature_collection: &FeatureCollection,
    ) -> Result<Vec<Granule>, Box<dyn std::error::Error>> {
        feature_collection
            .features
            .iter()
            .map(|feature| Self::try_from(feature))
            .collect()
    }
}
