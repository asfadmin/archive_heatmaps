#[allow(clippy::module_inception)]
#[cfg(test)]
mod tests {
    use actix_web::{
        middleware::{Compress, Logger},
        test,
        web::Data,
        App,
    };
    use chrono::{NaiveDate, TimeZone, Utc};
    use tokio_postgres::NoTls;

    use crate::{
        config::Config,
        database::{Dataset, Granule, HeatmapData, HeatmapResponse},
        middleware::{RedisCacheGet, RedisCacheSet},
        query::{heatmap_query, HeatmapQuery},
    };

    macro_rules! test_server {
        ($config:ident, $postgres_pool:ident, $redis_pool:ident) => {
            test::init_service(
                App::new()
                    .wrap(Logger::default())
                    .wrap(Compress::default())
                    .wrap(RedisCacheSet::default())
                    .wrap(RedisCacheGet::default())
                    .app_data(Data::new($postgres_pool.clone()))
                    .app_data(Data::new($redis_pool.clone()))
                    .app_data(Data::new($config.clone()))
                    .service(heatmap_query),
            )
            .await
        };
    }

    async fn initialize_test_service() -> (Config, deadpool_postgres::Pool, deadpool_redis::Pool) {
        dotenv::from_filename(".env_tests").ok();

        let config: Config = ::config::Config::builder()
            .add_source(::config::Environment::default())
            .build()
            .expect("config loading failed")
            .try_deserialize()
            .expect("invalid configuration");

        let postgres_pool = config
            .postgres
            .create_pool(None, NoTls)
            .expect("postgres connection failed");

        let redis_pool = config
            .redis
            .create_pool(None)
            .expect("redis connection failed");

        {
            // initialize temporary CMR table with some data
            let client = postgres_pool.get().await.unwrap();

            let cleanup_query = client
                .prepare(include_str!("testing_cleanup.sql"))
                .await
                .unwrap();

            client.query(&cleanup_query, &[]).await.unwrap();

            let init_query = client
                .prepare(include_str!("testing_init.sql"))
                .await
                .unwrap();

            client.query(&init_query, &[]).await.unwrap();

            let data_query = client
                .prepare(include_str!("add_granule.sql"))
                .await
                .unwrap();

            let granules = vec![
                Granule {
                    dataset: "ALOS PALSAR".to_string(),
                    midpoint: geo_types::Point::new(37.7125, 154.1848),
                    data_transferred: 5793,
                    time: NaiveDate::from_ymd(2021, 3, 14).and_hms(9, 33, 45),
                },
                Granule {
                    dataset: "ALOS PALSAR".to_string(),
                    midpoint: geo_types::Point::new(-172.7641, 76.3660),
                    data_transferred: 8459,
                    time: NaiveDate::from_ymd(2021, 8, 14).and_hms(11, 46, 21),
                },
                Granule {
                    dataset: "ALOS PALSAR".to_string(),
                    midpoint: geo_types::Point::new(112.5456, 161.1708),
                    data_transferred: 59,
                    time: NaiveDate::from_ymd(2022, 2, 16).and_hms(12, 15, 53),
                },
                Granule {
                    dataset: "UAVSAR TOPSAR".to_string(),
                    midpoint: geo_types::Point::new(174.0598, 153.8144),
                    data_transferred: 7802,
                    time: NaiveDate::from_ymd(2022, 6, 6).and_hms(12, 31, 39),
                },
                Granule {
                    dataset: "UAVSAR TOPSAR".to_string(),
                    midpoint: geo_types::Point::new(68.5287, 155.3284),
                    data_transferred: 145,
                    time: NaiveDate::from_ymd(2022, 10, 7).and_hms(16, 11, 20),
                },
            ];

            for granule in granules {
                client
                    .query(
                        &data_query,
                        &[
                            &granule.dataset,
                            &granule.midpoint,
                            &granule.data_transferred,
                            &granule.time,
                        ],
                    )
                    .await
                    .unwrap();
            }
        }

        {
            // clear redis database
            let mut connection = redis_pool.get().await.unwrap();

            redis::cmd("FLUSHDB")
                .query_async::<_, ()>(&mut connection)
                .await
                .unwrap();
        }

        (config, postgres_pool, redis_pool)
    }

    #[actix_web::test]
    async fn null_dataset_query_should_return_all_granules() {
        let (config, postgres_pool, redis_pool) = initialize_test_service().await;
        let app = test_server!(config, postgres_pool, redis_pool);

        let req = test::TestRequest::post()
            .uri("/query")
            .set_json(HeatmapQuery { dataset: None })
            .to_request();
        let resp: HeatmapResponse = test::call_and_read_body_json(&app, req).await;

        assert_eq!(
            HeatmapResponse {
                data: HeatmapData {
                    length: 5,
                    positions: vec![
                        37.7125, 154.1848, -172.7641, 76.366, 112.5456, 161.1708, 174.0598,
                        153.8144, 68.5287, 155.3284,
                    ],
                    weights: vec![5793, 8459, 59, 7802, 145,],
                    times: vec![72, 225, 411, 521, 644,],
                },
                date_start: Utc.ymd(2021, 3, 14).and_hms(9, 33, 45),
                date_end: Utc.ymd(2022, 10, 7).and_hms(16, 11, 20),
            },
            resp
        );
    }

    #[actix_web::test]
    async fn different_dataset_queries_should_return_different_datasets() {
        let (config, postgres_pool, redis_pool) = initialize_test_service().await;
        let app = test_server!(config, postgres_pool, redis_pool);

        {
            let req = test::TestRequest::post()
                .uri("/query")
                .set_json(HeatmapQuery {
                    dataset: Some(Dataset::Alos),
                })
                .to_request();
            let resp: HeatmapResponse = test::call_and_read_body_json(&app, req).await;

            assert_eq!(
                HeatmapResponse {
                    data: HeatmapData {
                        length: 3,
                        positions: vec![37.7125, 154.1848, -172.7641, 76.366, 112.5456, 161.1708,],
                        weights: vec![5793, 8459, 59,],
                        times: vec![72, 225, 411,],
                    },
                    date_start: Utc.ymd(2021, 3, 14).and_hms(9, 33, 45),
                    date_end: Utc.ymd(2022, 2, 16).and_hms(12, 15, 53),
                },
                resp
            );
        }

        {
            let req = test::TestRequest::post()
                .uri("/query")
                .set_json(HeatmapQuery {
                    dataset: Some(Dataset::Uavsar),
                })
                .to_request();
            let resp: HeatmapResponse = test::call_and_read_body_json(&app, req).await;

            assert_eq!(
                HeatmapResponse {
                    data: HeatmapData {
                        length: 2,
                        positions: vec![174.0598, 153.8144, 68.5287, 155.3284,],
                        weights: vec![7802, 145,],
                        times: vec![156, 279,],
                    },
                    date_start: Utc.ymd(2022, 6, 6).and_hms(12, 31, 39),
                    date_end: Utc.ymd(2022, 10, 7).and_hms(16, 11, 20),
                },
                resp
            );
        }
    }

    // for now disabled, rt::spawn silently fails in tests causing the compression middleware to fail silently
    // #[actix_web::test]
    // async fn concurrent_requests_should_use_preferred_encoding() {
    //     let (config, postgres_pool, redis_pool) = initialize_test_service().await;
    //     let app = test_server!(config, postgres_pool, redis_pool);

    //     for _ in 0..3 {
    //         let req = test::TestRequest::post()
    //             .uri("/api/query")
    //             .set_json(HeatmapQuery {
    //                 dataset: Some(Dataset::Alos),
    //             })
    //             .append_header(AcceptEncoding(vec![QualityItem::max(
    //                 Preference::Specific(Encoding::gzip()),
    //             )]))
    //             .to_request();
    //         let resp = test::call_service(&app, req).await;

    //         assert_eq!(resp.status(), StatusCode::OK);
    //         assert_eq!(
    //             resp.headers().get(header::CONTENT_ENCODING).unwrap(),
    //             "gzip"
    //         );

    //         let req = test::TestRequest::post()
    //             .uri("/api/query")
    //             .set_json(HeatmapQuery {
    //                 dataset: Some(Dataset::Alos),
    //             })
    //             .append_header(AcceptEncoding(vec![QualityItem::max(
    //                 Preference::Specific(Encoding::zstd()),
    //             )]))
    //             .to_request();
    //         let resp = test::call_service(&app, req).await;

    //         assert_eq!(resp.status(), StatusCode::OK);
    //         assert_eq!(
    //             resp.headers().get(header::CONTENT_ENCODING).unwrap(),
    //             "zstd"
    //         );

    //         let req = test::TestRequest::post()
    //             .uri("/api/query")
    //             .set_json(HeatmapQuery {
    //                 dataset: Some(Dataset::Alos),
    //             })
    //             .append_header(AcceptEncoding(vec![
    //                 QualityItem::max(Preference::Specific(Encoding::deflate())),
    //                 QualityItem::new(Preference::Specific(Encoding::zstd()), q(0.5)),
    //             ]))
    //             .to_request();
    //         let resp = test::call_service(&app, req).await;

    //         assert_eq!(resp.status(), StatusCode::OK);
    //         assert_eq!(
    //             resp.headers().get(header::CONTENT_ENCODING).unwrap(),
    //             "zstd"
    //         );
    //     }
    // }
}
