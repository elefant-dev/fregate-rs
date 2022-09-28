use axum::{routing::get, Extension, Router};
use bytes::Bytes;

use crate::extensions::{png, yaml};
use crate::{Health, Optional};

const OPENAPI_PATH: &str = "/openapi";
const FAVICON_PATH: &str = "/favicon.ico";

static FAVICON: Bytes = Bytes::from_static(include_bytes!("../resources/favicon.png"));
const OPENAPI: &str = include_str!("../resources/openapi.yaml");

pub(crate) fn build_management_router<H: Health>(health_indicator: Option<H>) -> Router {
    println!("!!!!!!!build_management_router!!!!!");
    Router::new()
        .route(OPENAPI_PATH, get(|| async { yaml(OPENAPI) }))
        .route(FAVICON_PATH, get(|| async { png(&FAVICON) }))
        .merge_optional(build_health_router(health_indicator))
        .merge(build_metrics_router())
}

fn build_health_router<H: Health>(health_indicator: Option<H>) -> Option<Router> {
    let health_indicator = health_indicator?;

    let alive_handler = |Extension(health): Extension<H>| async move { health.alive().await };
    let ready_handler = |Extension(health): Extension<H>| async move { health.ready().await };

    Some(
        Router::new()
            .route("/health", get(alive_handler))
            .route("/live", get(alive_handler))
            .route("/ready", get(ready_handler))
            .layer(Extension(health_indicator)),
    )
}

fn build_metrics_router() -> Router {
    Router::new().route(
        "/metrics",
        get(move || std::future::ready(crate::get_metrics())),
    )
}
