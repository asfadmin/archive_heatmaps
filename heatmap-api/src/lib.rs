use chrono::NaiveDate;
use serde::{Deserialize, Serialize};

use crate::granule::{Ancestor, Granule};

pub mod granule;

pub trait ToPartialString {
    fn _to_partial_string(&self) -> String;
}

#[derive(Deserialize, Serialize, Clone, Copy, Debug, PartialEq)]
pub enum ProductTypes {
    #[serde(rename = "GRD")]
    GroundRangeDetected,
    #[serde(rename = "SLC")]
    SingleLookComplex,
    #[serde(rename = "OCN")]
    Ocean,
}

impl ProductTypes {
    pub fn from_string(string: &str) -> Result<Self, std::fmt::Error> {
        match string {
            "GRD" => Ok(ProductTypes::GroundRangeDetected),
            "SLC" => Ok(ProductTypes::SingleLookComplex),
            "OCN" => Ok(ProductTypes::Ocean),
            _ => Err(std::fmt::Error),
        }
    }
}

#[derive(Deserialize, Serialize, Clone, Copy, Debug, PartialEq)]
pub enum PlatformType {
    #[serde(rename = "SA")]
    Sentinel1A,
    #[serde(rename = "SB")]
    Sentinel1B,
}

impl PlatformType {
    pub fn from_string(string: &str) -> Result<Self, std::fmt::Error> {
        match string {
            "SA" => Ok(PlatformType::Sentinel1A),
            "SB" => Ok(PlatformType::Sentinel1B),
            _ => Err(std::fmt::Error),
        }
    }
}

#[derive(Deserialize, Serialize, Clone, Copy, Debug, PartialEq)]
pub enum DataSensor {
    #[serde(rename = "S")]
    Sentinel,
}

impl DataSensor {
    pub fn from_string(string: &str) -> Result<Self, std::fmt::Error> {
        match string {
            "S" => Ok(DataSensor::Sentinel),
            _ => Err(std::fmt::Error),
        }
    }
}

#[derive(Deserialize, Serialize, Clone)]
pub struct Filter {
    pub product_type: Vec<ProductTypes>,
    pub platform_type: Vec<PlatformType>,
    pub start_date: String,
    pub end_date: String,
}

#[derive(Deserialize, Serialize)]
pub struct HeatmapQuery {
    pub filter: Filter,
}

#[derive(Deserialize, Serialize, Debug, PartialEq)]
pub struct HeatmapData {
    pub data: InteriorData,
}
#[derive(Deserialize, Serialize, Debug, PartialEq)]
pub struct InteriorData {
    pub length: i32,
    pub positions: Vec<Vec<(f64, f64)>>,
    pub weights: Vec<u64>,
}

#[derive(Deserialize, Serialize, Debug, PartialEq)]
pub struct HeatmapResponse {
    pub data: HeatmapData,
}

#[derive(Deserialize, Serialize, Debug, PartialEq)]
pub struct OutlineData {
    pub length: i32,
    pub positions: Vec<Vec<(f64, f64)>>,
}

#[derive(Deserialize, Serialize, Debug, PartialEq)]
pub struct OutlineResponse {
    pub data: OutlineData,
}

pub fn filter(filter: Filter, mut granules: Vec<Granule>) {
    let data_type = filter.product_type;
    let platform_type = filter.platform_type;
    let start_date = NaiveDate::parse_from_str(&filter.start_date, "%Y-%m-%d")
        .expect("Faile to parse start_date");
    let end_date =
        NaiveDate::parse_from_str(&filter.end_date, "%Y-%m-%d").expect("Faile to parse end_date");

    for granule in granules.iter_mut() {
        // Retain only those ancestors who fall within the filter
        granule.ancestors.retain(|ancestor| {
            let granule_date = ancestor.start_time.date();

            data_type.contains(&ancestor.product_type)
                && platform_type.contains(&ancestor.platform_type)
                && granule_date >= start_date
                && granule_date <= end_date
        });
    }
}
