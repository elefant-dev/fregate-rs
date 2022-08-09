use axum::Router as AxumRouter;
use hyper::header::CONTENT_TYPE;
use hyper::{Body, Request, Server};
use serde::de::DeserializeOwned;
use std::net::SocketAddr;
use tokio::signal;
use tower::make::Shared;
use tower::steer::Steer;
use tracing::info;

use crate::utils::*;

#[derive(Debug)]
pub struct Application<H, T> {
    config: AppConfig<T>,
    health_indicator: Option<H>,
    rest_router: Option<AxumRouter>,
    grpc_router: Option<AxumRouter>,
}

impl<T: DeserializeOwned> Application<NoHealth, T> {
    pub fn new(config: AppConfig<T>) -> Self {
        Application::<NoHealth, T> {
            config,
            health_indicator: None,
            rest_router: None,
            grpc_router: None,
        }
    }
}

impl<H: Health, T: DeserializeOwned> Application<H, T> {
    pub fn new_with_health(config: AppConfig<T>) -> Self {
        Self {
            config,
            health_indicator: None,
            rest_router: None,
            grpc_router: None,
        }
    }

    pub fn health_indicator(mut self, health: H) -> Self {
        self.health_indicator = Some(health);
        self
    }

    pub async fn serve(mut self) -> hyper::Result<()> {
        let rest = build_application_router(self.rest_router)
            .merge(build_management_router(self.health_indicator));

        let application_socket = SocketAddr::new(self.config.server.host, self.config.server.port);

        // TODO: MAKE GRPC A FEATURE ?
        if let Some(grpc) = self.grpc_router.take() {
            run_rest_and_grpc_service(&application_socket, rest, grpc).await
        } else {
            run_rest_service(&application_socket, rest).await
        }
    }

    pub fn rest_router(mut self, router: AxumRouter) -> Self {
        self.rest_router = Some(router);
        self
    }

    pub fn grpc_router(mut self, router: AxumRouter) -> Self {
        self.grpc_router = Some(router);
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

async fn run_rest_service(socket: &SocketAddr, rest: AxumRouter) -> hyper::Result<()> {
    let server = Server::bind(socket).serve(rest.into_make_service());
    info!(target: "server", server_type = "rest", "Started: http://{socket}");

    server.with_graceful_shutdown(shutdown_signal()).await
}

async fn run_rest_and_grpc_service(
    socket: &SocketAddr,
    rest: AxumRouter,
    grpc: AxumRouter,
) -> hyper::Result<()> {
    let rest_grpc = Steer::new(vec![rest, grpc], |req: &Request<Body>, _svcs: &[_]| {
        if req.headers().get(CONTENT_TYPE).map(|v| v.as_bytes()) != Some(b"application/grpc") {
            0
        } else {
            1
        }
    });

    let server = Server::bind(socket).serve(Shared::new(rest_grpc));
    info!(target: "server", server_type = "rest + grpc", "Started: http://{socket}");

    server.with_graceful_shutdown(shutdown_signal()).await
}
