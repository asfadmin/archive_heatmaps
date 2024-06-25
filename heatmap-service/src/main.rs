use actix_web::{
    middleware::{Compress, Logger},
    web::Data,
    App, HttpServer,
};
use geojson::FeatureCollection;
use middleware::{RedisCacheGet, RedisCacheSet};
use query::heatmap_query;

use crate::config::Config;

mod config;
mod dataset;
mod error;
mod geo_json_path;
mod granule;
mod heatmap_data;
mod heatmap_response;
mod middleware;
mod query;
mod redis;
mod tests;

#[actix_web::main]
async fn main() -> std::io::Result<()> {
    dotenv::dotenv().ok();

    std::env::set_var("RUST_LOG", "actix_web=info");
    env_logger::init();

    let mut config: Config = ::config::Config::builder()
        .add_source(::config::Environment::default())
        .build()
        .expect("config loading failed")
        .try_deserialize()
        .expect("invalid configuration");

    let redis_pool = config.redis.map(|redis_unwrapped| {
        redis_unwrapped
            .create_pool(None)
            .expect("redis connection failed")
    });

    config.redis = None;

    let feature_collection: FeatureCollection = config
        .geo_json_path
        .clone()
        .try_into()
        .expect("malformed geojson");

    let bind_address = config.server_address.clone();

    HttpServer::new(move || {
        let mut app = App::new()
            .wrap(Logger::default())
            .wrap(Compress::default())
            .wrap(RedisCacheSet)
            .wrap(RedisCacheGet)
            .service(heatmap_query);

        if let Some(redis_pool_unwrapped) = redis_pool.clone() {
            app = app.app_data(Data::new(redis_pool_unwrapped.clone()));
        }

        app.app_data(Data::new(config.clone()))
            .app_data(Data::new(feature_collection.clone()))
    })
    .bind(bind_address)?
    .run()
    .await
}
