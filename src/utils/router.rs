use axum::Router as AxumRouter;
use axum::{routing::get, Extension, Router};
use bytes::Bytes;
use tower_http::trace::TraceLayer;

use crate::extensions::{png, yaml};
use crate::{Health, Optional};

// TODO: MAKE IT CONFIGURABLE
const API_PATH: &str = "/v1";
const OPENAPI_PATH: &str = "/openapi";
const FAVICON_PATH: &str = "/favicon.ico";

static FAVICON: Bytes = Bytes::from_static(include_bytes!("../../src/resources/favicon.png"));
const OPENAPI: &str = include_str!("../../src/resources/openapi.yaml");

pub(crate) fn build_application_router(rest_router: Option<AxumRouter>) -> Router {
    Router::new().nest_optional(API_PATH, rest_router)
}

pub(crate) fn build_management_router<H: Health>(health_indicator: Option<H>) -> Router {
    Router::new()
        // TODO: SET CORRECT FORMATTING FOR HTTP TRACING
        .layer(TraceLayer::new_for_http())
        .route(OPENAPI_PATH, get(|| async { yaml(OPENAPI) }))
        .route(FAVICON_PATH, get(|| async { png(&FAVICON) }))
        .merge_optional(build_health_router(health_indicator))
}

fn build_health_router<H: Health>(health_indicator: Option<H>) -> Option<Router> {
    let health_indicator = health_indicator?;

    let alive_handler = |Extension(health): Extension<H>| async move { health.alive().await };
    let ready_handler = |Extension(health): Extension<H>| async move { health.ready().await };

    Some(
        Router::new()
            .route("/health/alive", get(alive_handler))
            .route("/health/ready", get(ready_handler))
            .layer(Extension(health_indicator)),
    )
}
