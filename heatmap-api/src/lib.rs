use serde::{Deserialize, Serialize};

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
