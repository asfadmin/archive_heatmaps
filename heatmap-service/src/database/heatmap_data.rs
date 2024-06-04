use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};

use super::{DateRange, Granule};

#[derive(Deserialize, Serialize, Debug, PartialEq)]
pub struct HeatmapData {
    pub length: i32,
    pub positions: Vec<f64>,
    pub weights: Vec<i64>,
    pub times: Vec<i32>,
}

impl HeatmapData {
    pub fn from_granules(granules: Vec<Granule>, date_range: DateRange) -> Self {
        Self {
            length: granules.len() as i32,
            positions: granules.iter().fold(Vec::new(), |mut collector, granule| {
                collector.push(granule.midpoint.x());
                collector.push(granule.midpoint.y());
                collector
            }),
            weights: granules
                .iter()
                .map(|granule| granule.data_transferred)
                .collect(),
            times: granules
                .iter()
                .map(|granule| {
                    date_range.transform_date(DateTime::<Utc>::from_utc(granule.time, Utc))
                })
                .collect(),
        }
    }
}

#[cfg(test)]
mod tests {
    use chrono::{NaiveDate, TimeZone};

    use super::*;

    #[test]
    fn test_from_granules() {
        let granules = vec![
            Granule {
                dataset: "ALOS PALSAR".to_string(),
                midpoint: geo_types::Point::new(37.7125, 154.1848),
                data_transferred: 5793,
                time: NaiveDate::from_ymd(2021, 3, 14).and_hms(9, 33, 45),
            },
            Granule {
                dataset: "ALOS PALSAR".to_string(),
                midpoint: geo_types::Point::new(-172.7641, 76.3660),
                data_transferred: 8459,
                time: NaiveDate::from_ymd(2021, 8, 14).and_hms(11, 46, 21),
            },
            Granule {
                dataset: "ALOS PALSAR".to_string(),
                midpoint: geo_types::Point::new(112.5456, 161.1708),
                data_transferred: 59,
                time: NaiveDate::from_ymd(2022, 2, 16).and_hms(12, 15, 53),
            },
            Granule {
                dataset: "UAVSAR TOPSAR".to_string(),
                midpoint: geo_types::Point::new(174.0598, 153.8144),
                data_transferred: 7802,
                time: NaiveDate::from_ymd(2022, 6, 6).and_hms(12, 31, 39),
            },
            Granule {
                dataset: "UAVSAR TOPSAR".to_string(),
                midpoint: geo_types::Point::new(68.5287, 155.3284),
                data_transferred: 145,
                time: NaiveDate::from_ymd(2022, 10, 7).and_hms(16, 11, 20),
            },
        ];

        let date_range = DateRange {
            start: Utc.ymd(2021, 3, 14).and_hms(9, 33, 45),
            end: Utc.ymd(2022, 10, 7).and_hms(16, 11, 20),
        };

        assert_eq!(
            HeatmapData::from_granules(granules, date_range),
            HeatmapData {
                length: 5,
                positions: vec![
                    37.7125, 154.1848, -172.7641, 76.366, 112.5456, 161.1708, 174.0598, 153.8144,
                    68.5287, 155.3284,
                ],
                weights: vec![5793, 8459, 59, 7802, 145,],
                times: vec![72, 225, 411, 521, 644,],
            },
        );
    }
}
