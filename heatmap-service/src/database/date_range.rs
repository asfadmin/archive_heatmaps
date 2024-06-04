use actix_web::Error;
use chrono::{DateTime, Datelike, NaiveDate, NaiveDateTime, Utc};
use deadpool_postgres::Client;
use tokio_postgres::Row;

use crate::error::{ActixExpect, ActixMapResult};

use super::{Dataset, ToPartialString};

trait GetNaiveDateTime {
    fn get_naive_date_time(&self) -> Result<NaiveDateTime, Error>;
}

impl GetNaiveDateTime for Vec<Row> {
    fn get_naive_date_time(&self) -> Result<NaiveDateTime, Error> {
        Ok(self
            .get(0)
            .actix_expect("malformed cmr table")?
            .get::<&str, _>("time"))
    }
}

#[derive(Copy, Clone)]
pub struct DateRange {
    pub start: DateTime<Utc>,
    pub end: DateTime<Utc>,
}

impl DateRange {
    pub async fn from_database(dataset: Option<Dataset>, client: &Client) -> Result<Self, Error> {
        let dataset_partial = dataset.to_partial_string();

        let start_query = client
            .prepare(include_str!("../../sql/start_date.sql"))
            .await
            .actix_map_result()?;

        let end_query = client
            .prepare(include_str!("../../sql/end_date.sql"))
            .await
            .actix_map_result()?;

        let start = DateTime::<Utc>::from_utc(
            client
                .query(&start_query, &[&dataset_partial])
                .await
                .actix_map_result()?
                .get_naive_date_time()?,
            Utc,
        );

        let end = DateTime::<Utc>::from_utc(
            client
                .query(&end_query, &[&dataset_partial])
                .await
                .actix_map_result()?
                .get_naive_date_time()?,
            Utc,
        );

        Ok(Self { start, end })
    }

    pub fn transform_date(&self, date: DateTime<Utc>) -> i32 {
        (date
            - DateTime::<Utc>::from_utc(
                NaiveDate::from_ymd_opt(self.start.year(), 1, 1)
                    .unwrap()
                    .and_hms_opt(0, 0, 0)
                    .unwrap(),
                Utc,
            ))
        .num_days() as i32
    }
}

#[cfg(test)]
mod tests {
    use chrono::TimeZone;

    use super::*;

    #[test]
    fn test_transform_date() {
        let date_range = DateRange {
            start: Utc.ymd(2020, 5, 9).and_hms(18, 34, 12),
            end: Utc.ymd(2022, 9, 12).and_hms(8, 34, 49),
        };

        let date = Utc.ymd(2022, 3, 25).and_hms(12, 52, 10);

        assert_eq!(814i32, date_range.transform_date(date));
    }
}
