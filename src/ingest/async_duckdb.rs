use js_sys::{Object, Promise, Reflect, WebAssembly};
use leptos::logging::log;
use wasm_bindgen::prelude::*;
use wasm_bindgen_futures::JsFuture;

#[wasm_bindgen(module = "https://cdn.jsdelivr.net/npm/@duckdb/duckdb-wasm/dist/")]
extern "C" {
    
}

pub async fn download_duckdb_wasm() -> Result<(), JsValue> {
    log!("Preparing DuckDB-wasm module...");
    const DUCKDB_WASM_URL_BASE: &str = "https://cdn.jsdelivr.net/npm/@duckdb/duckdb-wasm/dist/";

    let js_source_text = "
    (async () => {
        console.log('***** Entered JS Source Code *****');
        const duckdb = await import('https://cdn.jsdelivr.net/npm/@duckdb/duckdb-wasm@1.33.1-dev57.0/dist/duckdb-browser.mjs');

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

    let conn: Promise = js_sys::eval(js_source_text)?.dyn_into()?;
    let conn_await = conn.await?;
    log!("Conn in rust: {conn_await:?}");

    log!("Evaled js_source_text");

    // // https://duckdb.org/docs/current/clients/wasm/deploying_duckdb_wasm
    // // Main Library Component
    // let duckdb_browser_mjs = reqwest::get(format!("{DUCKDB_WASM_URL_BASE}duckdb-browser.mjs")).await?.text().await?;
    // // Worker Component
    // let duckdb_browser_eh_worker_js = reqwest::get(format!("{DUCKDB_WASM_URL_BASE}duckdb-browser-eh.worker.js")).await?.text().await?;
    // // Wasm Module
    // let duckdb_eh_wasm = reqwest::get(format!("{DUCKDB_WASM_URL_BASE}duckdb-eh.wasm")).await?.bytes().await?;
    // log!("Downloaded duckdb-eh.wasm");
    // let a = JsFuture::from(WebAssembly::instantiate_buffer(&duckdb_eh_wasm, &Object::new())).await?;
    // log!("Created JsValue from .wasm");
    // let b: WebAssembly::Instance = Reflect::get(&a, &"instance".into())?.dyn_into()?;
    // log!("WebAssembly Instance created");
    // let c = b.exports();
    // log!("MJS: {duckdb_browser_mjs:?}");
    // log!("WASM Instance: {b:?}");
    // log!("WASM Exports: {c:?}");

    Ok(())
}