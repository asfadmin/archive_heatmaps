use actix_web::Error;
use deadpool_redis::{Connection, Pool};
use redis::AsyncCommands;

use crate::error::ActixMapResult;

pub async fn cache_get(query: String, redis_pool: &Pool) -> Result<Option<String>, Error> {
    let mut connection: Connection = redis_pool.get().await.actix_map_result()?;

    connection.get(query).await.actix_map_result()
}

pub async fn cache_put(
    query: String,
    body: String,
    expiry: usize,
    redis_pool: &Pool,
) -> Result<(), Error> {
    let mut connection: Connection = redis_pool.get().await.actix_map_result()?;

    connection
        .set_ex(query, body, expiry)
        .await
        .actix_map_result()
}
