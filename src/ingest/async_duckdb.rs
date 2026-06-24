use js_sys::{Object, Promise, Reflect, WebAssembly};
use leptos::logging::log;
use wasm_bindgen::prelude::*;
use wasm_bindgen_futures::JsFuture;

#[wasm_bindgen]
extern "C" {
    #[wasm_bindgen(js_name = "AsyncDuckDB")]
    pub type JsAsyncDuckDB;

    #[wasm_bindgen(js_name="AsyncDuckDBConnection")]
    #[derive(Debug)]
    pub type JsAsyncDuckDBConnection;
}


pub async fn download_duckdb_wasm() -> Result<(), JsValue> {
    log!("Preparing DuckDB-wasm module...");

    let js_source_text = "
    (async () => {
        const duckdb = await import('./duckdb-browser.mjs');

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


        return conn
    })()
    ";

    let conn: Promise = js_sys::eval(js_source_text)?.dyn_into().unwrap();
    log!("Got promise back from eval");
    let conn_await: JsAsyncDuckDBConnection = conn.await?.dyn_into().unwrap();
    log!("Conn in rust: {conn_await:?}");
    Ok(())
}