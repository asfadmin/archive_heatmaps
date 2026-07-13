use std::iter::successors;

use chrono::NaiveDate;
use chrono::{Datelike, Months};
use leptos::html::P;
use leptos::logging::log;

use crate::DateRange;
use crate::types::Filter;

pub fn generate_create_sat_data_sql() -> String {
    "CREATE TABLE sat_data (
        geometry BLOB,
        ancestors STRUCT(granule_name VARCHAR, platform_type VARCHAR, data_sensor_type VARCHAR, start_time TIMESTAMP)[]
    );".to_string()
}

/// Create sql to read sat data from s3 into DuckDB based on the passed DateRange
pub fn generate_populate_sat_data_sql(date_range: &DateRange) -> (String, DateRange) {
    let range_start = NaiveDate::from_ymd_opt(date_range.start.year(), date_range.start.month(), 1)
        .expect("Failed to create start month");
    let mut range_end: NaiveDate = date_range.end;
    let missing_months: Vec<NaiveDate> = successors(Some(range_start), |x| {
        log!("x: {x:?}\tend: {:?}", date_range.end);
        let next = x.checked_add_months(Months::new(1));
        if let Some(n) = next
            && n >= date_range.end
        {
            log!("x: {x:?}\tend: {:?}", date_range.end);
            range_end = next.expect("Failed to get end month for final date range");
            return None;
        }

        next
    })
    .collect();

    // Generate SQL to import missing data
    let missing_files = missing_months
        .iter()
        .map(|x| {
            let end = x
                .checked_add_months(Months::new(1))
                .expect("Failed to add a month");
            format!(
                "{}_{}.parquet",
                x.format("%Y-%m-01"),
                end.format("%Y-%m-01")
            )
            .to_string()
        })
        .enumerate()
        .fold("[".to_string(), |mut acc, (i, s)| {
            if i != 0 {
                acc += ", ";
            }
            acc += &format!("'s3://archive-heatmap-storage/sat_data/{s}'");
            acc
        })
        + "]";
    log!("Missing: {missing_files}");
    let sql = format!(
        "INSERT INTO sat_data
     SELECT * 
     FROM read_parquet({missing_files});"
    );

    // Create Vector representing the imported data, needed to clip
    //  input range to file resolution, ie 2019-12-08 imports data
    //  starting from 2019-12-01
    let missing_range = DateRange {
        start: range_start,
        end: range_end,
    };

    (sql, missing_range)
}

pub fn generate_ingest_world_outline_sql() -> String {
    "CREATE TABLE world_outline AS
     SELECT * 
     FROM read_parquet('s3://archive-heatmap-storage/world_continents.parquet');"
        .to_string()
}

/// Create sql to generate a Heatmap based on a filter and data already in DuckDB
pub fn generate_sql(filter: &Filter) -> String {
    let mut plat_str = "(".to_string();
    filter.platform_type.iter().enumerate().for_each(|(i, x)| {
        if i > 0 {
            plat_str += ", ";
        }
        plat_str += &format!("'{}'", x);
    });
    plat_str += ")";

    let mut prod_str = "(".to_string();
    filter.product_type.iter().enumerate().for_each(|(i, x)| {
        if i > 0 {
            prod_str += ", "
        }
        prod_str += &format!("'{}'", x);
    });
    prod_str += ")";

    format!(
        "
    SELECT
        geometry,
        len(list_filter(ancestors, lambda x: 
            x.start_time > '{}' AND               -- Start Time
            x.start_time < '{}' AND               -- End Time
            x.platform_type IN {} AND             -- Platform: SA, SB, 5C, 5D
            substring(x.granule_name, 8, 3) IN {} -- Product Type: SLC, GRD, OCN
        )) AS weight,
    FROM sat_data
    WHERE weight > 0;
    ",
        filter.date_range.start.format("%Y-%m-%d"),
        filter.date_range.end.format("%Y-%m-%d"),
        plat_str,
        prod_str
    )
}
