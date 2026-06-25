use crate::types::Filter;

/// Create sql to read sat data from s3 into DuckDB
pub fn generate_populate_sql() -> String {
    "CREATE TABLE sat_data AS 
     SELECT * 
     FROM read_parquet('s3://archive-heatmap-storage/sat_data/2021-01-01_2021-02-01.parquet');"
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
        filter.start_date.format("%Y-%m-%d"),
        filter.end_date.format("%Y-%m-%d"),
        plat_str,
        prod_str
    )
}
