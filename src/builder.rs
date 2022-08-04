use axum::{BoxError, Router as AxumRouter};
use hyper::header::CONTENT_TYPE;
use hyper::{Body, Request, Server};
use std::net::{IpAddr, Ipv4Addr, SocketAddr};
use std::sync::Arc;
use tokio::signal;
use tonic::transport::server::Router as TonicRouter;
use tower::make::Shared;
use tower::steer::Steer;
use tower::ServiceExt;
use tracing::info;

use crate::utils::*;

const DEFAULT_PORT: u16 = 8000;

pub struct Application<H: Health> {
    health_indicator: Option<Arc<H>>,
    host: Option<IpAddr>,
    port: Option<u16>,
    rest_router: Option<AxumRouter>,
    grpc_router: Option<TonicRouter>,
}

impl Application<NoHealth> {
    pub fn new_without_health() -> Application<NoHealth> {
        Application::<NoHealth> {
            health_indicator: None,
            host: None,
            port: None,
            rest_router: None,
            grpc_router: None,
        }
    }
}

impl<H: Health> Application<H> {
    pub fn new_with_health(health: Arc<H>) -> Self {
        Self {
            health_indicator: Some(health),
            host: None,
            port: None,
            rest_router: None,
            grpc_router: None,
        }
    }

    pub async fn run(mut self) -> hyper::Result<()> {
        let socket = SocketAddr::new(
            self.host.unwrap_or(IpAddr::V4(Ipv4Addr::UNSPECIFIED)),
            self.port.unwrap_or(DEFAULT_PORT),
        );

        let rest = build_rest_router(self.health_indicator, self.rest_router);

        // TODO: MAKE GRPC A FEATURE ?
        // TODO: GENERIC FOR SERVER TYPE TO REMOVE DIFFERENT FUNCTIONS
        if let Some(grpc) = self.grpc_router.take() {
            run_rest_and_grpc(&socket, rest, grpc).await
        } else {
            run_rest_only(&socket, rest).await
        }
    }

    pub fn rest_router(mut self, router: AxumRouter) -> Self {
        self.rest_router = Some(router);
        self
    }

    pub fn grpc_router(mut self, router: TonicRouter) -> Self {
        self.grpc_router = Some(router);
        self
    }

    pub fn host(mut self, host: impl Into<IpAddr>) -> Self {
        self.host = Some(host.into());
        self
    }

    pub fn port(mut self, port: impl Into<u16>) -> Self {
        self.port = Some(port.into());
        self
    }
}

async fn shutdown_signal() {
    let ctrl_c = async {
        signal::ctrl_c()
            .await
            .expect("failed to install Ctrl+C handler");
    };

    #[cfg(unix)]
    let terminate = async {
        signal::unix::signal(signal::unix::SignalKind::terminate())
            .expect("failed to install SIGTERM signal handler")
            .recv()
            .await;
    };

    #[cfg(not(unix))]
    let terminate = std::future::pending::<()>();

    tokio::select! {
        _ = ctrl_c => {},
        _ = terminate => {},
    }

    info!("Termination signal, starting shutdown...");
}

async fn run_rest_only(socket: &SocketAddr, rest: AxumRouter) -> hyper::Result<()> {
    let server = Server::bind(socket).serve(rest.into_make_service());

    info!("Start Listening on {:?}", socket);
    server.with_graceful_shutdown(shutdown_signal()).await
}

async fn run_rest_and_grpc(
    socket: &SocketAddr,
    rest: AxumRouter,
    grpc: TonicRouter,
) -> hyper::Result<()> {
    let rest = rest.map_err(BoxError::from).boxed_clone();

    let grpc = grpc
        .into_service()
        .map_response(|r| r.map(axum::body::boxed))
        .boxed_clone();

    let http_grpc = Steer::new(vec![rest, grpc], |req: &Request<Body>, _svcs: &[_]| {
        if req.headers().get(CONTENT_TYPE).map(|v| v.as_bytes()) != Some(b"application/grpc") {
            0
        } else {
            1
        }
    });

    let server = Server::bind(socket).serve(Shared::new(http_grpc));

    info!("Start Listening on {:?}", socket);
    server.with_graceful_shutdown(shutdown_signal()).await
}
