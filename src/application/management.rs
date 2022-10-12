use crate::{
    extensions::{yaml, RouterOptionalExt},
    health::Health,
};
use axum::{routing::get, Extension, Router};

const OPENAPI_PATH: &str = "/openapi";
const HEALTH_PATH: &str = "/health";
const LIVE_PATH: &str = "/live";
const READY_PATH: &str = "/ready";
const METRICS_PATH: &str = "/metrics";

// TODO consider: https://github.com/pyrossh/rust-embed/blob/master/examples/axum.rs#L64
const OPENAPI: &str = include_str!("../resources/openapi.yaml");

pub(crate) fn build_management_router<H: Health>(health_indicator: Option<H>) -> Router {
    Router::new()
        .route(OPENAPI_PATH, get(|| async { yaml(OPENAPI) }))
        .merge_optional(build_health_router(health_indicator))
        .merge(build_metrics_router())
}

fn build_health_router<H: Health>(health_indicator: Option<H>) -> Option<Router> {
    let health_indicator = health_indicator?;

    let alive_handler = |health: Extension<H>| async move { health.alive().await };
    let ready_handler = |health: Extension<H>| async move { health.ready().await };

    Some(
        Router::new()
            .route(HEALTH_PATH, get(alive_handler))
            .route(LIVE_PATH, get(alive_handler))
            .route(READY_PATH, get(ready_handler))
            .layer(Extension(health_indicator)),
    )
}

fn build_metrics_router() -> Router {
    Router::new().route(
        METRICS_PATH,
        get(move || std::future::ready(crate::get_metrics())),
    )
}

#[cfg(test)]
mod management_test {
    use super::*;
    use crate::health::HealthResponse;
    use axum::http::{Request, StatusCode};
    use tower::ServiceExt;

    #[derive(Default, Debug, Clone)]
    pub struct CustomHealth;

    #[axum::async_trait]
    impl Health for CustomHealth {
        async fn alive(&self) -> HealthResponse {
            HealthResponse::OK
        }

        async fn ready(&self) -> HealthResponse {
            HealthResponse::UNAVAILABLE
        }
    }

    #[tokio::test]
    async fn health_test() {
        let router = build_management_router(Some(CustomHealth));
        let request = Request::builder()
            .uri("http://0.0.0.0/health")
            .method("GET")
            .body(hyper::Body::empty())
            .unwrap();

        let response = router.oneshot(request).await.unwrap();
        let status = response.status();
        let body = hyper::body::to_bytes(response.into_body()).await.unwrap();

        assert_eq!(StatusCode::OK, status);
        assert_eq!(&body[..], b"OK");
    }

    #[tokio::test]
    async fn live_test() {
        let router = build_management_router(Some(CustomHealth));
        let request = Request::builder()
            .uri("http://0.0.0.0/live")
            .method("GET")
            .body(hyper::Body::empty())
            .unwrap();

        let response = router.oneshot(request).await.unwrap();
        let status = response.status();
        let body = hyper::body::to_bytes(response.into_body()).await.unwrap();

        assert_eq!(StatusCode::OK, status);
        assert_eq!(&body[..], b"OK");
    }

    #[tokio::test]
    async fn ready_test() {
        let router = build_management_router(Some(CustomHealth));
        let request = Request::builder()
            .uri("http://0.0.0.0/ready")
            .method("GET")
            .body(hyper::Body::empty())
            .unwrap();

        let response = router.oneshot(request).await.unwrap();
        let status = response.status();
        let body = hyper::body::to_bytes(response.into_body()).await.unwrap();

        assert_eq!(StatusCode::SERVICE_UNAVAILABLE, status);
        assert_eq!(&body[..], b"UNAVAILABLE");
    }
}
