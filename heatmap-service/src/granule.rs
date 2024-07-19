use chrono::NaiveDateTime;
use geo::Polygon;
use geojson::{Feature, FeatureCollection};
use heatmap_api::{DataSensor, PlatformType, ProductTypes};
use serde::Serialize;

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
    pub granule_name: String,
    pub product_type: ProductTypes,
    pub platform_type: PlatformType,
    pub data_sensor: DataSensor,
    pub start_time: NaiveDateTime,
}

impl TryFrom<&serde_json::Map<String, serde_json::Value>> for Ancestor {
    type Error = actix_web::error::Error;

    fn try_from(
        properties: &serde_json::Map<String, serde_json::Value>,
    ) -> Result<Ancestor, Self::Error> {
        let granule_name = properties
            .get("GRANULE_NA")
            .expect("failed to get granule name")
            .as_str()
            .expect("failed to convert granule name to str")
            .to_string();

        let product_type =
            ProductTypes::from_string(&granule_name[7..10]).expect("Failed to parse product type");
        let platform_type = PlatformType::from_string(
            properties
                .get("PLATFORM_T")
                .expect("failed to get platform type")
                .as_str()
                .expect("failed to convert platform type to str"),
        )
        .expect("Failed to parse Platform type from passed string");

        let data_sensor = DataSensor::from_string(
            properties
                .get("DATA_SENSO")
                .expect("failed to get data sensor")
                .as_str()
                .expect("failed to convert data sensor to string"),
        )
        .expect("Failed to parse data sensor from passed string");

        Ok(Ancestor {
            granule_name,
            product_type,
            platform_type,
            data_sensor,
            start_time: NaiveDateTime::parse_from_str(
                properties
                    .get("START_TIME")
                    .expect("failed to get start time")
                    .as_str()
                    .expect("failed to convert start time to str"),
                "%Y-%m-%d %H:%M:%S%.f",
            )
            .expect("failed to parse start time from string"),
        })
    }
}

impl Granule {
    pub fn from_feature_collection(
        feature_collection: &FeatureCollection,
    ) -> Result<Vec<Granule>, Box<dyn std::error::Error>> {
        feature_collection
            .features
            .iter()
            .map(Self::try_from)
            .collect()
    }
}
