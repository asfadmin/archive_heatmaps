use std::error::Error;
use leptos::{prelude::ReadSignal, reactive::traits::DefinedAt};
use chrono::NaiveDate;
use geo::Polygon;
use serde::{Deserialize, Serialize};
use strum_macros::Display;
use leptos::prelude::GetUntracked;

// Enums defining possible filter options
#[derive(Clone, Copy, Debug, PartialEq, Eq, Display)]
pub enum ProductTypes {
    #[strum(to_string = "GRD")]
    GroundRangeDetected,
    #[strum(to_string = "SLC")]
    SingleLookComplex,
    #[strum(to_string = "OCN")]
    Ocean,
}

#[derive(Deserialize, Serialize, Clone, Copy, Debug, PartialEq, Eq, Display)]
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

#[derive(Debug)]
pub struct Granule {
    pub geometry: Polygon,
    pub weight: u64,
}

#[derive(Clone, Debug)]
pub struct DateRange {
    pub start: NaiveDate,
    pub end: NaiveDate,
}

impl DateRange {
    /// Checks that start < end before constructing type
    pub fn new(start: NaiveDate, end: NaiveDate) -> Result<Self, Box<dyn Error>> {
        if start > end {
            return Err("Start is after end, this is an invalid state for a DateRange".into());
        }
        Ok(Self { start, end })
    }

    /// Merge two date ranges
    ///  
    /// Requires ranges to share exactly one temporal border:
    ///  - Does not allow overlaping or disjoint ranges
    pub fn merge(&mut self, other: &Self) -> Result<(), Box<dyn Error>> {
        if self.start == other.end {
            self.start = other.start;
        } else if self.end == other.start {
            self.end = other.end;
        } else {
            return Err("Invalid merge attempt".into());
        }
        Ok(())
    }

    /// Returns all sections of other that are disjoint from self
    pub fn get_disjoint(&self, other: &Self) -> Option<Vec<Self>> {
        if self.end < other.start || other.end < self.start {
            // Non overlapping ranges
            return Some(vec![other.clone()]);
        }
        if self.start > other.start && self.end < other.end {
            // Self is a subset of other
            return Some(vec![
                Self {
                    start: other.start,
                    end: self.start,
                },
                Self {
                    start: self.end,
                    end: other.end,
                },
            ]);
        }
        if self.end < other.end {
            // Self comes first and partially overlaps other
            return Some(vec![Self {
                start: self.end,
                end: other.end,
            }]);
        }
        if self.start > other.start {
            // Other comes first and partially overlaps self
            return Some(vec![Self {
                start: other.start,
                end: self.start,
            }]);
        }
        None
    }
}

// Describes a heatmap to generate
#[derive(Clone)]
pub struct Filter {
    pub date_range: DateRange,
    pub product_type: Vec<ProductTypes>,
    pub platform_type: Vec<PlatformType>,
}

#[derive(Clone)]
pub struct ReadySignal(pub ReadSignal<bool>);

#[derive(Clone)]
pub struct GeneratePngSignal(pub ReadSignal<bool>);

// impl GetUntracked for GeneratePngSignal {
//     type Value = bool;
//     fn try_get_untracked(&self) -> Option<Self::Value> {
//         self.0.try_get_untracked()
//     }
// }

// impl DefinedAt for GeneratePngSignal {
//     fn defined_at(&self) -> Option<&'static std::panic::Location<'static>> {
//         self.0.defined_at()
//     }
// }