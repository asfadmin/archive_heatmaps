use js_sys::{Object, Promise, Reflect, WebAssembly};
use leptos::{attr::Async, logging::log};
use wasm_bindgen::prelude::*;
use wasm_bindgen_futures::JsFuture;

#[wasm_bindgen(module = "/assets/duckdb-browser.mjs")]
extern "C" {
    #[wasm_bindgen(js_name = "AsyncDuckDBConnection")]
    #[derive(Debug)]
    type AsyncDuckDBConnection;

    #[wasm_bindgen(method, js_name = "query")]
    async fn query(this: &AsyncDuckDBConnection, text: &str) -> JsValue;
}


pub async fn download_duckdb_wasm() -> Result<(), JsValue> {
    log!("Preparing DuckDB-wasm module...");

    let js_source_text = "
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
    ";

    let conn = js_sys::eval(js_source_text)
        .unwrap()
        .dyn_into::<Promise>()
        .unwrap()
        .await
        .unwrap()
        .dyn_into::<AsyncDuckDBConnection>()
        .unwrap();

    log!("Conn in rust is AsyncDuckDB: {:?}", conn);

    let res = conn.query("SELECT version()").await;
    log!("DuckDB Query resolved: {:?}", res);
    Ok(())
}