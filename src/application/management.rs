use crate::{
    extensions::{png, yaml, RouterOptionalExt},
    health::Health,
};
use axum::{routing::get, Extension, Router};
use bytes::Bytes;

const OPENAPI_PATH: &str = "/openapi";
const HEALTH_PATH: &str = "/health";
const LIVE_PATH: &str = "/live";
const READY_PATH: &str = "/ready";

// TODO(kos): Rust is great thing, but time of compilation is Achilles' heel of
//            Rust. Whenever possible compilation time should be reduced to keep
//            it sane. More than 10s for incremental build is a high risk for a
//            project. Use the `rust-embed` crate instead of embedding assets
//            into the binary. The crate exposes a macro which loads files into
//            the rust binary at compile time during a release build only, and
//            loads the file from the filesystem during usual debug builds.
//            Example is here:
//            https://github.com/pyrossh/rust-embed/blob/master/examples/axum.rs#L64
const OPENAPI: &str = include_str!("../resources/openapi.yaml");

pub(crate) fn build_management_router<H: Health>(health_indicator: Option<H>) -> Router {
    Router::new()
        .route(OPENAPI_PATH, get(|| async { yaml(OPENAPI) }))
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
