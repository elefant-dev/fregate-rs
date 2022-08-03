use axum::Router as AxumRouter;
use axum::{routing::get, Extension, Json, Router};
use bytes::Bytes;
use std::sync::Arc;
use tower_http::trace::TraceLayer;

use crate::extensions::{png, yaml};
use crate::{Health, Optional};

// TODO: MAKE IT CONFIGURABLE
const API_PATH: &str = "/v1";
const OPENAPI_PATH: &str = "/openapi";
const FAVICON_PATH: &str = "/favicon.ico";
const HEALTH_PATH: &str = "/health";

static FAVICON: Bytes = Bytes::from_static(include_bytes!("../../src/resources/favicon.png"));
const OPENAPI: &str = include_str!("../../src/resources/openapi.yaml");

pub(crate) fn build_rest_router<H: Health>(
    health_indicator: Option<Arc<H>>,
    rest_router: Option<AxumRouter>,
) -> Router {
    Router::new()
        // TODO: SET CORRECT FORMATTING FOR HTTP TRACING
        .layer(TraceLayer::new_for_http())
        .route(OPENAPI_PATH, get(|| async { yaml(OPENAPI) }))
        .route(FAVICON_PATH, get(|| async { png(&FAVICON) }))
        .nest_optional(API_PATH, rest_router)
        .merge_optional(get_health_router(health_indicator))
}

fn get_health_router<H: Health>(health_indicator: Option<Arc<H>>) -> Option<Router> {
    let health_indicator = health_indicator?;
    let health_handler =
        |Extension(health): Extension<Arc<H>>| async move { Json(health.check().await) };

    Some(
        Router::new()
            .route(HEALTH_PATH, get(health_handler))
            .layer(Extension(health_indicator)),
    )
}
