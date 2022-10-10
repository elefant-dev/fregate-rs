use axum::{routing::get, Extension, Router};
use bytes::Bytes;

use crate::extensions::{png, yaml};
use crate::{Health, Optional};

const OPENAPI_PATH: &str = "/openapi";
const FAVICON_PATH: &str = "/favicon.ico";

// TODO(kos): Rust is great thing, but time of compilation is Achilles' heel of
//            Rust. Whenever possible compilation time should be reduced to keep
//            it sane. More than 10s for incremental build is a high risk for a
//            project. Use the `rust-embed` crate instead of embedding assets
//            into the binary. The crate exposes a macro which loads files into
//            the rust binary at compile time during a release build only, and
//            loads the file from the filesystem during usual debug builds.
//            Example is here:
//            https://github.com/pyrossh/rust-embed/blob/master/examples/axum.rs#L64
static FAVICON: Bytes = Bytes::from_static(include_bytes!("../resources/favicon.png"));
const OPENAPI: &str = include_str!("../resources/openapi.yaml");

pub(crate) fn build_management_router<H: Health>(health_indicator: Option<H>) -> Router {
    Router::new()
        .route(OPENAPI_PATH, get(|| async { yaml(OPENAPI) }))
        .route(FAVICON_PATH, get(|| async { png(&FAVICON) }))
        .merge_optional(build_health_router(health_indicator))
        .merge(build_metrics_router())
}

fn build_health_router<H: Health>(health_indicator: Option<H>) -> Option<Router> {
    let health_indicator = health_indicator?;

    // TODO(kos): Extension implements Deref into the inner value, so
    //            destructuring in function arguments is not needed.
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
