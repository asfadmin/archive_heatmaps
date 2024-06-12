use crate::{
    config::Config, dataset::Dataset, error::ActixMapResult, heatmap_response::HeatmapResponse,
    redis,
};
use actix_web::{
    web::{Data, Json},
    Error, HttpResponse,
};
use serde::{Deserialize, Serialize};

#[derive(Deserialize, Serialize)]
pub struct HeatmapQuery {
    pub dataset: Option<Dataset>,
}

#[actix_web::post("/query")]
async fn heatmap_query(
    query: Json<HeatmapQuery>,
    redis_pool: Data<deadpool_redis::Pool>,
    feature_collection: Data<geojson::FeatureCollection>,
    config: Data<Config>,
) -> Result<HttpResponse, Error> {
    let query = query.0;
    let query_string = serde_json::to_string(&query)?;

    /*
    why check cache if there is already a middleware handling caching?
    the middleware handles only caching of pre-compressed results,
    if there is no pre-compressed result, the serve can access existing
    cache and recompress that result, instead of having to redo a db
    query.
    */
    if let Some(response) = redis::cache_get(query_string.clone(), &redis_pool).await? {
        return Ok(HttpResponse::Ok()
            .content_type("application/json")
            .body(response));
    }

    let response_data = HeatmapResponse::from_geojson(query.dataset, &feature_collection);

    let response = HttpResponse::Ok().json(&response_data);

    redis::cache_put(
        query_string,
        serde_json::to_string(&response_data)?,
        config.cache_ttl,
        &redis_pool,
    )
    .await?;

    Ok(response)
}
