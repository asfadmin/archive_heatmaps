use std::iter::successors;

use leptos::html::P;
use chrono::Months;
use leptos::logging::log;
use crate::DateRange;

use crate::types::Filter;

/// Create sql to read sat data from s3 into DuckDB based on the passed DateRange
pub fn generate_populate_sql(date_range: &DateRange) -> String {
    let missing: String = successors(Some(date_range.start), |x| {
        log!("x: {x:?}\tend: {:?}", date_range.end);
        let next = x.checked_add_months(Months::new(1));
        if let Some(n) = next && n >= date_range.end {
            log!("x: {x:?}\tend: {:?}", date_range.end);
            return None
        }
        next
    }).map(|x| {
        let end = x.checked_add_months(Months::new(1)).expect("Failed to add a month");
        format!("{}_{}.parquet", x.format("%Y-%m-%d"), end.format("%Y-%m-%d")).to_string()
    }).enumerate()
    .fold("[".to_string(), |mut acc, (i, s)| {
        if i != 0 {
            acc += ", ";
        }
        acc += &format!("'s3://archive-heatmap-storage/sat_data/{s}'");
        acc
    }) + "]";
    log!("Missing: {missing}");
    format!("CREATE TABLE sat_data AS 
     SELECT * 
     FROM read_parquet({missing});")
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
