use axum::{routing::get, Extension, Json, Router};
use bytes::Bytes;
use std::sync::Arc;
use tower_http::trace::TraceLayer;

use crate::builder::{health::Health, metrics::get_metrics};
use crate::extensions::{png, yaml};

const API_PATH: &str = "/v1";
const OPENAPI_PATH: &str = "/openapi";
const FAVICON_PATH: &str = "/favicon.ico";
const METRICS_PATH: &str = "/metrics";
const HEALTH_PATH: &str = "/health";

static FAVICON: Bytes = Bytes::from_static(include_bytes!("../../src/resources/favicon.png"));
const OPENAPI: &str = include_str!("../../src/resources/openapi.yaml");

#[derive(Debug, Default)]
pub struct RouterBuilder<H: Health> {
    rest_routes: Option<Router>,
    health_indicator: Option<Arc<H>>,
    init_metrics: bool,
}

impl<H: Health> RouterBuilder<H> {
    pub fn build(&mut self) -> Router {
        let default_routes = Router::new()
            // TODO: SET CORRECT FORMATTING FOR HTTP TRACING
            .layer(TraceLayer::new_for_http())
            .route(OPENAPI_PATH, get(|| async { yaml(OPENAPI) }))
            .route(FAVICON_PATH, get(|| async { png(&FAVICON) }));

        let routes = if let Some(rest_routes) = self.rest_routes.take() {
            default_routes.nest(API_PATH, rest_routes)
        } else {
            default_routes
        };

        let routes = if let Some(health_indicator) = self.health_indicator.take() {
            routes.merge(Self::get_health_router(health_indicator))
        } else {
            routes
        };

        if self.init_metrics {
            routes.merge(Router::new().route(METRICS_PATH, get(|| async { get_metrics() })))
        } else {
            routes
        }
    }

    pub fn set_rest_routes(&mut self, router: Router) {
        self.rest_routes = Some(router);
    }

    pub fn set_health_indicator(&mut self, health_indicator: Arc<H>) {
        self.health_indicator = Some(health_indicator)
    }

    pub fn init_metrics(&mut self) {
        self.init_metrics = true;
    }

    fn get_health_router(health_indicator: Arc<H>) -> Router {
        let health_handler =
            |Extension(health): Extension<Arc<H>>| async move { Json(health.check().await) };

        Router::new()
            .route(HEALTH_PATH, get(health_handler))
            .layer(Extension(health_indicator))
    }
}
