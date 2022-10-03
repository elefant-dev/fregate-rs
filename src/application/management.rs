use axum::{routing::get, Extension, Router};
use bytes::Bytes;

use crate::extensions::{png, yaml};
use crate::{Health, Optional};

const OPENAPI_PATH: &str = "/openapi";
const FAVICON_PATH: &str = "/favicon.ico";
const HEALTH_PATH: &str = "/health";
const LIVE_PATH: &str = "/live";
const READY_PATH: &str = "/ready";

// TODO(kos): Rust is great thing, but time of compilation is Achilles' heel of Rust.
// Whenever possible compilation time should be reduced to keep it sane.
// More than 10s for incremental build is hight risk for a project.
// Use crate `rust-embed` instead of embedding assets into binary.
// The crate expose macro which loads files into the rust binary at compile time during release and loads the file from the fs during dev.
//
// Example is here: https://github.com/pyrossh/rust-embed/blob/master/examples/axum.rs#L64
static FAVICON: Bytes = Bytes::from_static(include_bytes!("../resources/favicon.png"));
const OPENAPI: &str = include_str!("../resources/openapi.yaml");

pub(crate) fn build_management_router<H: Health>(health_indicator: Option<H>) -> Router {
    Router::new()
        .route(OPENAPI_PATH, get(|| async { yaml(OPENAPI) }))
        .route(FAVICON_PATH, get(|| async { png(&FAVICON) }))
        .merge_optional(build_health_router(health_indicator))
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
