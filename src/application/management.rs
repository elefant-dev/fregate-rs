use crate::application::health::Health;
use crate::observability::render_metrics;
use crate::{ManagementConfig, ObservabilityConfig};
use axum::response::IntoResponse;
use axum::{routing::get, Extension, Router};
use std::sync::Arc;

pub(crate) fn build_management_router<H: Health>(
    management_cfg: &ManagementConfig,
    observability_cfg: &ObservabilityConfig,
    health_indicator: H,
    callback: Option<Arc<dyn Fn() + Send + Sync + 'static>>,
) -> Router {
    Router::new()
        .merge(build_health_router(management_cfg, health_indicator))
        .merge(build_metrics_router(management_cfg, callback))
        .merge(build_version_router(management_cfg, observability_cfg))
}

fn build_health_router<H: Health>(
    management_cfg: &ManagementConfig,
    health_indicator: H,
) -> Router {
    // TODO: separate health and alive handlers
    let alive_handler = |health: Extension<H>| async move { health.alive().await };
    let ready_handler = |health: Extension<H>| async move { health.ready().await };

    Router::new()
        .route(management_cfg.endpoints.health.as_ref(), get(alive_handler))
        .route(management_cfg.endpoints.live.as_ref(), get(alive_handler))
        .route(management_cfg.endpoints.ready.as_ref(), get(ready_handler))
        .layer(Extension(health_indicator))
}

fn build_metrics_router(
    management_cfg: &ManagementConfig,
    callback: Option<Arc<dyn Fn() + Send + Sync + 'static>>,
) -> Router {
    Router::new().route(
        management_cfg.endpoints.metrics.as_ref(),
        get(move || std::future::ready(render_metrics(callback.as_deref()))),
    )
}

fn build_version_router(
    management_cfg: &ManagementConfig,
    observability_cfg: &ObservabilityConfig,
) -> Router {
    let path = format!(
        "/{}{}",
        observability_cfg.component_name,
        management_cfg.endpoints.version.as_ref()
    );
    let version = observability_cfg.version.clone();

    Router::new().route(
        path.as_str(),
        get(|| async move { version.into_response() }),
    )
}

#[cfg(test)]
#[allow(clippy::unwrap_used)]
mod management_test {
    use super::*;
    use crate::application::health::HealthResponse;
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
        let mngmt_cfg = ManagementConfig::default();
        let obs_cfg = ObservabilityConfig::default();

        let router = build_management_router(&mngmt_cfg, &obs_cfg, CustomHealth, None);
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
        let mngmt_cfg = ManagementConfig::default();
        let obs_cfg = ObservabilityConfig::default();

        let router = build_management_router(&mngmt_cfg, &obs_cfg, CustomHealth, None);
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
        let mngmt_cfg = ManagementConfig::default();
        let obs_cfg = ObservabilityConfig::default();

        let router = build_management_router(&mngmt_cfg, &obs_cfg, CustomHealth, None);
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

    #[tokio::test]
    #[allow(clippy::field_reassign_with_default)]
    async fn version_test() {
        let mngmt_cfg = ManagementConfig::default();
        let mut obs_cfg = ObservabilityConfig::default();
        obs_cfg.version = "123.220.0".to_owned();

        let router = build_management_router(&mngmt_cfg, &obs_cfg, CustomHealth, None);
        let request = Request::builder()
            .uri("http://0.0.0.0//version")
            .method("GET")
            .body(hyper::Body::empty())
            .unwrap();

        let response = router.oneshot(request).await.unwrap();
        let status = response.status();
        let body = hyper::body::to_bytes(response.into_body()).await.unwrap();

        assert_eq!(StatusCode::OK, status);
        assert_eq!(&body[..], b"123.220.0");
    }
}
