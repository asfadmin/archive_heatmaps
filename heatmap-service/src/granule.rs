use chrono::NaiveDateTime;
use geo::Polygon;
use geojson::{Feature, FeatureCollection};
use log::warn;
use serde::Serialize;

use crate::dataset::Dataset;

#[derive(Debug, Serialize, Clone)]
pub struct Granule {
    pub polygons: Vec<Polygon>,
    pub ancestors: Vec<Ancestor>,
}

impl TryFrom<&Feature> for Granule {
    type Error = Box<dyn std::error::Error>;

    fn try_from(feature: &Feature) -> Result<Granule, Self::Error> {
        let mut ancestors: Vec<serde_json::Map<String, serde_json::Value>> = Vec::new();
        if let Some(ancestors_properties) = feature
            .properties
            .clone()
            .ok_or("feature has no properties associated with it")?
            .get("ancestors")
        {
            ancestors = ancestors_properties
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
        }

        let mut polygons: Vec<Polygon> = Vec::new();

        if let Some(geometry) = feature.geometry.clone() {
            match geometry.value {
                geojson::Value::Polygon(_) => {
                    polygons.push(
                        geometry
                            .value
                            .try_into()
                            .expect("failed to convert geometry into polygon"),
                    );
                }

                geojson::Value::MultiPolygon(_) => {
                    let multi_polygon: geo::MultiPolygon = geometry
                        .try_into()
                        .expect("failed to convert geometry to multipolygon");
                    polygons = multi_polygon.0;
                }

                _ => {
                    warn!("geometry exists but is unparsed in granule");
                }
            }
        }

        Ok(Granule {
            polygons,
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
                "%Y-%m-%d %H:%M:%S%.f",
            )
            .expect("failed to parse start time from string"),
            end_time: NaiveDateTime::parse_from_str(
                properties
                    .get("END_TIME")
                    .expect("failed to get end time")
                    .as_str()
                    .expect("failed to convert end time to str"),
                "%Y-%m-%d %H:%M:%S%.f",
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

    pub fn get_polygons(&self) -> Vec<Vec<(f64, f64)>> {
        self.polygons
            .iter()
            .map(|granule| {
                granule.exterior().clone().into_inner().iter().fold(
                    Vec::new(),
                    |mut inner_collector: Vec<(f64, f64)>, coord| {
                        inner_collector.push((coord.x, coord.y));
                        inner_collector
                    },
                )
            })
            .collect()
    }
}
