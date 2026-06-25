use bincode::{Decode, Encode};
use chrono::NaiveDate;
use geo::Polygon;
use serde::{Deserialize, Serialize};
use strum_macros::Display;

pub trait ToPartialString {
    fn _to_partial_string(&self) -> String;
}

// Enums defining possible filter options
#[derive(Clone, Copy, Debug, PartialEq, Display)]
pub enum ProductTypes {
    #[strum(to_string = "GRD")]
    GroundRangeDetected,
    #[strum(to_string = "SLC")]
    SingleLookComplex,
    #[strum(to_string = "OCN")]
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

#[derive(Deserialize, Serialize, Clone, Copy, Debug, PartialEq, Display)]
pub enum PlatformType {
    #[strum(to_string = "SA")]
    Sentinel1A,
    #[strum(to_string = "SB")]
    Sentinel1B,
    #[strum(to_string = "5C")]
    Sentinel1C,
    #[strum(to_string = "5D")]
    Sentinel1D,
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

// The filter passed from client to server on a request for data
#[derive(Clone)]
pub struct Filter {
    pub product_type: Vec<ProductTypes>,
    pub platform_type: Vec<PlatformType>,
    pub start_date: NaiveDate,
    pub end_date: NaiveDate,
}

#[derive(Debug)]
pub struct Granule {
    pub geometry: Polygon,
    pub weight: u64,
}

#[derive(Encode, Decode, Deserialize, Serialize, Debug, PartialEq)]
pub struct HeatmapData {
    pub data: InteriorData,
}
#[derive(Encode, Decode, Deserialize, Serialize, Debug, PartialEq)]
pub struct InteriorData {
    pub length: i32,
    pub positions: Vec<Vec<(f64, f64)>>,
    pub weights: Vec<u64>,
}

// Server sends this back to client after a query,
// contains world outline data
#[derive(Deserialize, Serialize, Debug, PartialEq)]
pub struct OutlineResponse {
    pub data: OutlineData,
}
#[derive(Deserialize, Serialize, Debug, PartialEq)]
pub struct OutlineData {
    pub length: i32,
    pub positions: Vec<Vec<(f64, f64)>>,
}
