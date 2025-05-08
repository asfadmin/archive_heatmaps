use std::{
    future::{ready, Future, Ready},
    marker::PhantomData,
    pin::Pin,
    rc::Rc,
    str::FromStr,
    task::{Context, Poll},
};

use actix_http::{body::EitherBody, header};
use actix_web::{
    body::{BodySize, MessageBody},
    dev::{self, Service, ServiceRequest, ServiceResponse, Transform},
    http::header::{AcceptEncoding, Encoding, Preference},
    rt,
    web::{Bytes, BytesMut, Data},
    Error, HttpMessage, HttpResponse,
};
use base64::{engine::general_purpose, Engine as _};
use futures_util::{future::LocalBoxFuture, stream::StreamExt};
use redis::AsyncCommands;
use serde::{Deserialize, Serialize};
use serde_with::{serde_as, DisplayFromStr};

use crate::config::Config;
use crate::error::{ActixExpect, ActixMapResult};
use crate::redis::cache_get;

/*
actix-web middleware is infamously complicated, so I will explain my best here,
adding a caching middleware allows caching compressed data, this means that the
server doesn't have to re-compress the heavy json response. this must be implemented
using two middlewares due to the fact that actix-web's middleware model makes
reading both the query payload body, and the response body impossible in a single
middleware (atleast to my knowledge). to get around this, we have two middlewares,
a CacheSet, which runs after the Compression middleware runs, to set the cache on
the compression of a sucessful query, and a CacheGet which runs before the Compression
middleware runs and checks if the current request is cached, returning the cached result
if necessary.
*/

#[serde_as]
#[derive(Serialize, Deserialize, Clone)]
struct CachingKey {
    payload: String,
    #[serde_as(as = "DisplayFromStr")]
    content_encoding: Encoding,
}

#[derive(Clone)]
struct CachingInformation {
    key: CachingKey,
    cache_ttl: usize,
}

impl CachingInformation {
    fn from_bytes(body: BytesMut, cache_ttl: usize) -> Self {
        let payload = String::from_utf8_lossy(&body);
        let key = CachingKey {
            payload: payload.to_string(),
            content_encoding: Encoding::Unknown("".to_string()),
        };
        Self { key, cache_ttl }
    }
}

#[derive(Default)]
pub struct RedisCacheGet;

impl<S: 'static, B> Transform<S, ServiceRequest> for RedisCacheGet
where
    S: Service<ServiceRequest, Response = ServiceResponse<B>, Error = Error>,
    S::Future: 'static,
    B: 'static,
{
    type Response = ServiceResponse<EitherBody<B>>;
    type Error = Error;
    type InitError = ();
    type Transform = RedisCacheGetMiddleware<S>;
    type Future = Ready<Result<Self::Transform, Self::InitError>>;

    fn new_transform(&self, service: S) -> Self::Future {
        ready(Ok(RedisCacheGetMiddleware {
            service: Rc::new(service),
        }))
    }
}

pub struct RedisCacheGetMiddleware<S> {
    service: Rc<S>,
}

impl<S, B> Service<ServiceRequest> for RedisCacheGetMiddleware<S>
where
    S: Service<ServiceRequest, Response = ServiceResponse<B>, Error = Error> + 'static,
    S::Future: 'static,
    B: 'static,
{
    type Response = ServiceResponse<EitherBody<B>>;
    type Error = Error;
    type Future = LocalBoxFuture<'static, Result<Self::Response, Self::Error>>;

    dev::forward_ready!(service);

    fn call(&self, mut req: ServiceRequest) -> Self::Future {
        let svc = self.service.clone();

        Box::pin(async move {
            let mut body = BytesMut::new();
            let mut stream = req.take_payload();
            while let Some(chunk) = stream.next().await {
                body.extend_from_slice(&chunk?);
            }

            let redis_pool_wrapped = req.app_data::<Data<deadpool_redis::Pool>>();

            let config = req
                .app_data::<Data<Config>>()
                .actix_expect("no config in app data")?;
            req.extensions_mut()
                .insert::<CachingInformation>(CachingInformation::from_bytes(
                    body.clone(),
                    config.cache_ttl,
                ));

            if let Some(redis_pool) = redis_pool_wrapped {
                if let Some(accept_encoding) = req.get_header::<AcceptEncoding>() {
                    for preference in accept_encoding.ranked() {
                        let encoding = match preference {
                            Preference::Specific(encoding) => match encoding {
                                Encoding::Known(encoding) => encoding,
                                Encoding::Unknown(_) => continue,
                            },
                            Preference::Any => continue,
                        };

                        if let Some(response) = cache_get(
                            serde_json::to_string(&CachingKey {
                                payload: String::from_utf8_lossy(&body).to_string(),
                                content_encoding: Encoding::Known(encoding),
                            })
                            .actix_map_result()?,
                            redis_pool,
                        )
                        .await?
                        {
                            let (request, _pl) = req.into_parts();
                            let response = HttpResponse::Ok()
                                .content_type("application/json")
                                .append_header(encoding)
                                .body(
                                    general_purpose::STANDARD
                                        .decode(response)
                                        .actix_map_result()?,
                                )
                                .map_into_right_body();

                            return Ok(ServiceResponse::new(request, response));
                        }
                    }
                }
            }

            let (_, mut payload) = actix_http::h1::Payload::create(true);
            payload.unread_data(body.clone().into());
            req.set_payload(payload.into());

            let res = svc.call(req).await?;
            Ok(res.map_into_left_body())
        })
    }
}

#[derive(Default)]
pub struct RedisCacheSet;

impl<S: 'static, B> Transform<S, ServiceRequest> for RedisCacheSet
where
    S: Service<ServiceRequest, Response = ServiceResponse<B>, Error = Error>,
    B: MessageBody + 'static,
{
    type Response = ServiceResponse<BodyCacher<B>>;
    type Error = Error;
    type InitError = ();
    type Transform = RedisCacheSetMiddleware<S>;
    type Future = Ready<Result<Self::Transform, Self::InitError>>;

    fn new_transform(&self, service: S) -> Self::Future {
        ready(Ok(RedisCacheSetMiddleware { service }))
    }
}

pub struct RedisCacheSetMiddleware<S> {
    service: S,
}

impl<S, B> Service<ServiceRequest> for RedisCacheSetMiddleware<S>
where
    S: Service<ServiceRequest, Response = ServiceResponse<B>, Error = Error>,
    B: MessageBody,
{
    type Response = ServiceResponse<BodyCacher<B>>;
    type Error = Error;
    type Future = WrapperStream<S, B>;

    dev::forward_ready!(service);

    fn call(&self, req: ServiceRequest) -> Self::Future {
        let caching_information = req.extensions().get::<CachingInformation>().cloned();
        let redis_pool = req.app_data::<Data<deadpool_redis::Pool>>().cloned();

        WrapperStream {
            fut: self.service.call(req),
            _t: PhantomData,
            caching_information,
            redis_pool,
        }
    }
}

#[pin_project::pin_project]
pub struct WrapperStream<S, B>
where
    B: MessageBody,
    S: Service<ServiceRequest>,
{
    #[pin]
    fut: S::Future,
    _t: PhantomData<(B,)>,
    caching_information: Option<CachingInformation>,
    redis_pool: Option<Data<deadpool_redis::Pool>>,
}

impl<S, B> Future for WrapperStream<S, B>
where
    B: MessageBody,
    S: Service<ServiceRequest, Response = ServiceResponse<B>, Error = Error>,
{
    type Output = Result<ServiceResponse<BodyCacher<B>>, Error>;

    fn poll(self: Pin<&mut Self>, cx: &mut Context<'_>) -> Poll<Self::Output> {
        let this = self.project();
        let res = futures_util::ready!(this.fut.poll(cx));

        let mut caching_information = this.caching_information.clone();
        if let Ok(response) = res.as_ref() {
            let content_encoding = response
                .headers()
                .get(header::CONTENT_ENCODING)
                .and_then(|e| e.to_str().ok())
                .and_then(|e| Encoding::from_str(e).ok());

            if let Some(content_encoding) = content_encoding {
                caching_information = caching_information.map(|caching_information| {
                    let mut caching_information = caching_information;
                    caching_information.key.content_encoding = content_encoding;
                    caching_information
                });
            } else {
                caching_information = None;
            }
        }

        Poll::Ready(res.map(|res| {
            res.map_body(move |_, body| BodyCacher {
                body,
                body_accum: BytesMut::new(),
                caching_information,
                redis_pool: this.redis_pool.clone(),
            })
        }))
    }
}

#[pin_project::pin_project(PinnedDrop)]
pub struct BodyCacher<B> {
    #[pin]
    body: B,
    body_accum: BytesMut,
    caching_information: Option<CachingInformation>,
    redis_pool: Option<Data<deadpool_redis::Pool>>,
}

#[pin_project::pinned_drop]
impl<B> PinnedDrop for BodyCacher<B> {
    fn drop(self: Pin<&mut Self>) {
        if let Some(caching_information) = self.caching_information.clone() {
            let redis_pool_wrapped = self.redis_pool.clone();
            let body = general_purpose::STANDARD.encode(&self.body_accum);

            rt::spawn(async move {
                // there isnt much we can do for proper error handling here :/
                if let Some(redis_pool) = redis_pool_wrapped {
                    let mut connection = redis_pool.get().await.expect("redis connection failed");

                    connection
                        .set_ex::<_, _, ()>(
                            serde_json::to_string(&caching_information.key)
                                .expect("serde json serialization failed"),
                            body,
                            caching_information.cache_ttl,
                        )
                        .await
                        .expect("setting redis cache failed");
                }
            });
        }
    }
}

impl<B: MessageBody> MessageBody for BodyCacher<B> {
    type Error = B::Error;

    fn size(&self) -> BodySize {
        self.body.size()
    }

    fn poll_next(
        self: Pin<&mut Self>,
        cx: &mut Context<'_>,
    ) -> Poll<Option<Result<Bytes, Self::Error>>> {
        let this = self.project();

        match this.body.poll_next(cx) {
            Poll::Ready(Some(Ok(chunk))) => {
                this.body_accum.extend_from_slice(&chunk);
                Poll::Ready(Some(Ok(chunk)))
            }
            Poll::Ready(Some(Err(e))) => Poll::Ready(Some(Err(e))),
            Poll::Ready(None) => Poll::Ready(None),
            Poll::Pending => Poll::Pending,
        }
    }
}
