use actix_web::{
    web::{Data, Json},
    Error, HttpRequest, HttpResponse,
};
use serde::{Deserialize, Serialize};

use crate::{config::Config, heatmap_response::HeatmapResponse, redis};

#[derive(Deserialize, Serialize)]
pub struct HeatmapQuery {
    pub filter: String,
}

#[actix_web::post("/query")]
async fn heatmap_query(
    req: HttpRequest,
    query: Json<HeatmapQuery>,
    feature_collection: Data<geojson::FeatureCollection>,
    config: Data<Config>,
) -> Result<HttpResponse, Error> {
    let query = query.0;
    let query_string = serde_json::to_string(&query)?;

    let redis_wrapped = req.app_data::<deadpool_redis::Pool>();

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

    let response_data = HeatmapResponse::from_geojson(query.filter, &feature_collection);

    let response = HttpResponse::Ok().json(&response_data);

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
