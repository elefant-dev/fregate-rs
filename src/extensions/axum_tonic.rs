use axum::body::boxed;
use axum::Router;
use hyper::{Body, Request, Response};
use sealed::sealed;
use std::convert::Infallible;
use tonic::body::BoxBody;
use tonic::transport::NamedService;
use tower::{Service, ServiceBuilder};
use tower_http::ServiceBuilderExt;

/// Takes Tonic [`Service`] and converts it into [`Router`]
#[sealed]
pub trait RouterTonicExt {
    /// Takes Tonic [`Service`] and converts it into [`Router`]
    fn from_tonic_service<S>(service: S) -> Self
    where
        Self: Sized,
        S: Service<Request<Body>, Response = Response<BoxBody>, Error = Infallible>
            + NamedService
            + Clone
            + Send
            + 'static,
        S::Future: Send + 'static;
}

#[sealed]
impl RouterTonicExt for Router {
    fn from_tonic_service<S>(service: S) -> Self
    where
        Self: Sized,
        S: Service<Request<Body>, Response = Response<BoxBody>, Error = Infallible>
            + NamedService
            + Clone
            + Send
            + 'static,
        S::Future: Send + 'static,
    {
        // this piece of code is taken from:
        // https://github.com/EmbarkStudios/server-framework/blob/83ff44b0ad19e4fcbc163bc652f00e04f4143365/src/server.rs#L679-L685
        let svc = ServiceBuilder::new()
            .map_err(|err: Infallible| match err {})
            .map_response_body(boxed)
            .service(service);

        Router::new().route_service(&format!("/{}/*rest", S::NAME), svc)
    }
}
