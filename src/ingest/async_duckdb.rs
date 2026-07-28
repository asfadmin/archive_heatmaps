use std::io::Cursor;

use arrow::ipc::reader::FileReader;
use arrow::record_batch::RecordBatch;
use js_sys::{Promise, Uint8Array};
use wasm_bindgen::prelude::*;

#[wasm_bindgen(module = "/assets/duckdb-browser.mjs")]
extern "C" {
    #[wasm_bindgen(js_name = "AsyncDuckDB")]
    #[derive(Debug)]
    type AsyncDuckDB;
    #[wasm_bindgen(method, catch, js_name = "runQuery")]
    async fn run_query(this: &AsyncDuckDB, conn: u32, text: &str) -> Result<Uint8Array, JsValue>;

    #[wasm_bindgen(js_name = "AsyncDuckDBConnection")]
    #[derive(Debug)]
    pub type AsyncDuckDBConnection;
    #[wasm_bindgen(method, getter, js_name = "bindings")]
    fn bindings(this: &AsyncDuckDBConnection) -> AsyncDuckDB;
}

impl AsyncDuckDBConnection {
    fn conn(&self) -> Result<u32, JsValue> {
        js_sys::Reflect::get(self, &"_conn".into())?
            .as_f64()
            .map(|x| x as u32)
            .ok_or_else(|| "Failed to get connection number for AsyncDuckDBConnection".into())
    }

    pub async fn query(&self, sql: &str) -> Result<Vec<RecordBatch>, js_sys::Error> {
        let res = self.bindings().run_query(self.conn()?, sql).await?.to_vec();
        let cursor = Cursor::new(res);
        let reader = FileReader::try_new(cursor, None)
            .expect("Failed to create file reader around cursor for reading RecordBatches");
        let mut batches: Vec<RecordBatch> = Vec::new();
        for maybe_batch in reader {
            match maybe_batch {
                Ok(batch) => batches.push(batch),
                Err(err) => return Err(js_sys::Error::new(&err.to_string())),
            }
        }
        Ok(batches)
    }
}

pub async fn generate_duckdb_connection() -> Result<AsyncDuckDBConnection, JsValue> {
    js_sys::eval(include_str!("duckdb.js"))
        .expect("Failed to initialize DuckDB in JS")
        .dyn_into::<Promise>()?
        .await?
        .dyn_into::<AsyncDuckDBConnection>()
}
