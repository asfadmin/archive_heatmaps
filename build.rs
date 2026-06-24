use std::fs;

/// Download DuckDB-wasm main library
#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    println!("cargo::rerun-if-changed=build.rs");
    println!("cargo::rerun-if-changed=assets/duckdb-browser.mjs");

    let res = reqwest::get("https://cdn.jsdelivr.net/npm/@duckdb/duckdb-wasm@1.33.1-dev57.0/dist/duckdb-browser.mjs").await?;
    let duckdb_browser_mjs = res.text().await?;
    fs::write("./assets/duckdb-browser.mjs", duckdb_browser_mjs)?;
    Ok(())
} 