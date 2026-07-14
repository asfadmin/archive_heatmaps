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