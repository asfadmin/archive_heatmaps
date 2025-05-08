use actix_cors::Cors;
use actix_web::{
    http::header,
    middleware::{Compress, Logger},
    web::Data,
    App, HttpServer,
};
use middleware::{RedisCacheGet, RedisCacheSet};
use query::{heatmap_query, outline_query};

use crate::config::Config;
use crate::geo_assets::GeoAssets;

mod assets;
mod config;
mod error;
mod geo_assets;
mod geo_json_path;
mod granule;
mod heatmap_data;
mod heatmap_response;
mod middleware;
mod outline_response;
mod query;
mod redis;

#[actix_web::main]
async fn main() -> std::io::Result<()> {
    println!("Service starting up");

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

    let bind_address = config.server_address.clone();

    let geo_assets = GeoAssets::from_config(config.clone());

    println!("Service Running!");

    HttpServer::new(move || {
        let cors = Cors::default()
            .allowed_origin("https://asfadmin.github.io") // The client is hosted on github pages
            .allowed_origin("http://localhost:8080") // Allowed for debug purposes
            .allowed_methods(vec!["GET", "POST"])
            .allowed_headers(vec![header::AUTHORIZATION, header::ACCEPT])
            .allowed_header(header::CONTENT_TYPE)
            .max_age(3600);

        let mut app = App::new()
            .wrap(Logger::default())
            .wrap(Compress::default())
            .wrap(RedisCacheSet)
            .wrap(RedisCacheGet)
            .wrap(cors)
            .service(heatmap_query)
            .service(outline_query);

        if let Some(redis_pool_unwrapped) = redis_pool.clone() {
            app = app.app_data(Data::new(redis_pool_unwrapped.clone()));
        }

        app.app_data(Data::new(config.clone()))
            .app_data(Data::new(geo_assets.clone()))
    })
    .bind(bind_address)?
    .run()
    .await
}
