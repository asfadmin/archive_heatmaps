use actix_web::Error;
use chrono::{DateTime, Utc};
use deadpool_postgres::Client;
use serde::{Deserialize, Serialize};

use super::{Dataset, DateRange, Granule, HeatmapData};

#[derive(Deserialize, Serialize, Debug, PartialEq)]
pub struct HeatmapResponse {
    pub data: HeatmapData,
    pub date_start: DateTime<Utc>,
    pub date_end: DateTime<Utc>,
}

impl HeatmapResponse {
    pub async fn from_database(dataset: Option<Dataset>, client: &Client) -> Result<Self, Error> {
        let granules = Granule::from_database(dataset, client).await?;
        let date_range = DateRange::from_database(dataset, client).await?;

        Ok(Self::from_database_data(granules, date_range))
    }

    fn from_database_data(granules: Vec<Granule>, date_range: DateRange) -> Self {
        Self {
            data: HeatmapData::from_granules(granules, date_range),
            date_start: date_range.start,
            date_end: date_range.end,
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use chrono::{NaiveDate, TimeZone, Utc};

    use crate::database::{DateRange, Granule};

    #[test]
    fn test_from_database_data() {
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
            HeatmapResponse::from_database_data(granules, date_range),
            HeatmapResponse {
                data: HeatmapData {
                    length: 5,
                    positions: vec![
                        37.7125, 154.1848, -172.7641, 76.366, 112.5456, 161.1708, 174.0598,
                        153.8144, 68.5287, 155.3284,
                    ],
                    weights: vec![5793, 8459, 59, 7802, 145,],
                    times: vec![72, 225, 411, 521, 644,],
                },
                date_start: Utc.ymd(2021, 3, 14).and_hms(9, 33, 45),
                date_end: Utc.ymd(2022, 10, 7).and_hms(16, 11, 20)
            }
        );
    }
}
