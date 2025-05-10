use actix_web::{
    body::BoxBody,
    web::{Data, Json},
    Error, HttpRequest, HttpResponse,
};

use crate::geo_assets::GeoAssets;
use crate::outline_response::OutlineResponse;
use crate::{config::Config, heatmap_response::HeatmapResponse, redis};

#[actix_web::post("/heatmap")]
async fn heatmap_query(
    req: HttpRequest,
    geo_assets: Data<GeoAssets>,
    query: Json<heatmap_api::HeatmapQuery>,
    config: Data<Config>,
) -> Result<HttpResponse, Error> {
    let query = query.0;
    let query_string = serde_json::to_string(&query)?;

    let redis_wrapped = req.app_data::<deadpool_redis::Pool>();

    let feature_collection = &geo_assets.heatmap_features;

    /*
    why check cache if there is already a middleware handling caching?
    the middleware handles only caching of pre-compressed results,
    if there is no pre-compressed result, the serve can access existing
    cache and recompress that result, instead of having to redo a db
    query.
    */
    if let Some(redis_pool) = redis_wrapped {
        if let Some(response) = redis::cache_get(query_string.clone(), redis_pool).await? {
            return Ok(HttpResponse::Ok()
                .content_type("application/json")
                .body(response));
        }
    }

    let response_data = HeatmapResponse::from_geojson(query.filter, feature_collection);

    let response = HttpResponse::Ok()
        .message_body(BoxBody::new(
            bincode::encode_to_vec(&response_data, bincode::config::standard()).unwrap(),
        ))
        .expect("Failed to create HttpResponse for heatmap granules");

    if let Some(redis_pool) = redis_wrapped {
        redis::cache_put(
            query_string,
            serde_json::to_string(&response_data)?,
            config.cache_ttl,
            redis_pool,
        )
        .await?;
    }

    Ok(response)
}

#[actix_web::get("/outline")]
async fn outline_query(geo_assets: Data<GeoAssets>) -> Result<HttpResponse, Error> {
    let feature_collection = &geo_assets.outline_features;

    let response_data = OutlineResponse::from_geojson(feature_collection);

    let response = HttpResponse::Ok().json(&response_data);

    Ok(response)
}
