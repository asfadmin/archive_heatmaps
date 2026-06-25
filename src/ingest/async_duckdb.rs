use std::io::Cursor;

use arrow::ipc::reader::FileReader;
use arrow::record_batch::RecordBatch;
use geo::convex_hull::qhull;
use js_sys::{Promise, Reflect, Uint8Array, WebAssembly};
use leptos::{attr::Async, logging::log};
use wasm_bindgen::prelude::*;
use wasm_bindgen_futures::JsFuture;

#[wasm_bindgen(module = "/assets/duckdb-browser.mjs")]
extern "C" {
    #[wasm_bindgen(js_name = "AsyncDuckDB")]
    #[derive(Debug)]
    type AsyncDuckDB;
    #[wasm_bindgen(method, js_name = "runQuery")]
    async fn run_query(this: &AsyncDuckDB, conn: u32, text: &str) -> Uint8Array;

    #[wasm_bindgen(js_name = "AsyncDuckDBConnection")]
    #[derive(Debug)]
    type AsyncDuckDBConnection;
    #[wasm_bindgen(method, getter, js_name = "bindings")]
    fn bindings(this: &AsyncDuckDBConnection) -> AsyncDuckDB;
}

impl AsyncDuckDBConnection {
    fn conn(&self) -> Result<u32, JsValue> {
        return js_sys::Reflect::get(&self, &"_conn".into())?
            .as_f64()
            .map(|x| x as u32)
            .ok_or("Failed to get connection number for AsyncDuckDBConnection".into());
    }

    async fn query(&self, sql: &str) -> Result<Vec<RecordBatch>, js_sys::Error> {
        let res = self.bindings().run_query(self.conn()?, sql).await.to_vec();
        let cursor = Cursor::new(res);
        let reader = FileReader::try_new(cursor, None).unwrap();
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
    Ok(js_sys::eval("
    (async () => {
        const duckdb = await import('./snippets/heatmap-client-752fa26ee35a9ffd/assets/duckdb-browser.mjs'); // TO-DO: Figure out how to make hash dynamic in case it changes! shouldnt be a problem unless .mjs gets a version bump?

        const JSDELIVR_BUNDLES = duckdb.getJsDelivrBundles();

        // Select a bundle based on browser checks
        const bundle = await duckdb.selectBundle(JSDELIVR_BUNDLES);

        const worker_url = URL.createObjectURL(
        new Blob([`importScripts(\"${bundle.mainWorker}\");`], {type: 'text/javascript'})
        );

        // Instantiate the asynchronous version of DuckDB-Wasm
        const worker = new Worker(worker_url);
        const logger = new duckdb.ConsoleLogger();
        const db = new duckdb.AsyncDuckDB(logger, worker);
        await db.instantiate(bundle.mainModule, bundle.pthreadWorker);
        URL.revokeObjectURL(worker_url);
        
        const conn = await db.connect();

        console.log('conn is AsyncDuckDBConnection: ', conn instanceof duckdb.AsyncDuckDBConnection)


        return conn
    })()
    ")
        .unwrap()
        .dyn_into::<Promise>()?
        .await?
        .dyn_into::<AsyncDuckDBConnection>()?
    )
}
