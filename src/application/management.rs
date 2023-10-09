use crate::application::health::HealthExt;
use crate::observability::render_metrics;
use crate::version::VersionExt;
use crate::{AppConfig, ManagementConfig};
use axum::{routing::get, Extension, Router};
use std::sync::Arc;

pub(crate) fn build_management_router<T, H, V>(
    app_cfg: &Arc<AppConfig<T>>,
    health_indicator: H,
    version: V,
    callback: Option<Arc<dyn Fn() + Send + Sync + 'static>>,
) -> Router
where
    H: HealthExt,
    V: VersionExt<T>,
    T: Send + Sync + 'static,
{
    Router::new()
        .merge(build_health_router(
            &app_cfg.management_cfg,
            health_indicator,
        ))
        .merge(build_metrics_router(&app_cfg.management_cfg, callback))
        .merge(build_version_router(app_cfg, version))
}

fn build_health_router<H: HealthExt>(
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

fn build_version_router<T, V>(app_cfg: &Arc<AppConfig<T>>, version: V) -> Router
where
    V: VersionExt<T>,
    T: Send + Sync + 'static,
{
    let config = Arc::clone(app_cfg);
    let endpoint = app_cfg.management_cfg.endpoints.version.as_ref();
    let version_handler = |version: Extension<V>| async move { version.get_version(&config) };

    Router::new()
        .route(endpoint, get(version_handler))
        .layer(Extension(version))
}

#[cfg(test)]
#[allow(clippy::unwrap_used)]
mod management_test {
    use super::*;
    use crate::version::DefaultVersion;
    use crate::Empty;
    use axum::http::{HeaderValue, Request, StatusCode};
    use axum::Json;
    use tower::ServiceExt;

    #[derive(Debug, Clone)]
    pub struct Config {
        pub version: String,
    }

    #[derive(Default, Debug, Clone)]
    pub struct CustomHealth;

    #[derive(Default, Debug, Clone)]
    pub struct CustomVersion;

    impl Default for Config {
        fn default() -> Self {
            Self {
                version: "123.220.0".to_owned(),
            }
        }
    }

    #[axum::async_trait]
    impl HealthExt for CustomHealth {
        type HealthResponse = (StatusCode, &'static str);
        type ReadyResponse = (StatusCode, &'static str);

        async fn alive(&self) -> Self::HealthResponse {
            (StatusCode::OK, "OK")
        }

        async fn ready(&self) -> Self::ReadyResponse {
            (StatusCode::SERVICE_UNAVAILABLE, "UNAVAILABLE")
        }
    }

    impl VersionExt<Config> for CustomVersion {
        type Response = (StatusCode, Json<String>);

        fn get_version(&self, cfg: &AppConfig<Config>) -> Self::Response {
            let version = cfg.private.version.clone();
            (StatusCode::OK, Json(version))
        }
    }

    #[tokio::test]
    async fn health_test() {
        let app_cfg = Arc::new(AppConfig::<Empty>::default());

        let router = build_management_router(&app_cfg, CustomHealth, DefaultVersion, None);
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
        let app_cfg = Arc::new(AppConfig::<Empty>::default());

        let router = build_management_router(&app_cfg, CustomHealth, DefaultVersion, None);
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
        let app_cfg = Arc::new(AppConfig::<Empty>::default());

        let router = build_management_router(&app_cfg, CustomHealth, DefaultVersion, None);
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
    async fn version_test() {
        let app_cfg = Arc::new(AppConfig::<Config>::default());

        let router = build_management_router(&app_cfg, CustomHealth, CustomVersion, None);
        let request = Request::builder()
            .uri("http://0.0.0.0/version")
            .method("GET")
            .body(hyper::Body::empty())
            .unwrap();

        let response = router.oneshot(request).await.unwrap();
        let status = response.status();
        let content_type = response.headers().get("Content-Type").cloned();
        let body = hyper::body::to_bytes(response.into_body()).await.unwrap();

        assert_eq!(StatusCode::OK, status);
        assert_eq!(content_type, Some(HeaderValue::from_static("json")));
        assert_eq!(&body[..], b"123.220.0");
    }
}
