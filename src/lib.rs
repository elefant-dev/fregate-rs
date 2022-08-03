mod builder;
mod extensions;

use axum::{BoxError, Router as AxumRouter};
use config::Config;
use hyper::header::CONTENT_TYPE;
use hyper::{Body, Request, Server};
use std::net::SocketAddr;
use tokio::signal;
use tonic::transport::server::Router as TonicRouter;
use tower::make::Shared;
use tower::steer::Steer;
use tower::ServiceExt;
use tracing::info;

pub use builder::*;

pub struct Application {
    rest_router: AxumRouter,
    grpc_router: Option<TonicRouter>,
    socket: SocketAddr,
    _config: Config,
}

impl Application {
    pub fn builder<'a, H: Health>() -> ApplicationBuilder<'a, H> {
        ApplicationBuilder::default()
    }

    pub async fn run(mut self) -> hyper::Result<()> {
        let rest = self.rest_router;

        // TODO: MAKE GRPC A FEATURE ?
        // TODO: GENERIC FOR SERVER TYPE
        if let Some(grpc_router) = self.grpc_router.take() {
            let rest = rest.map_err(BoxError::from).boxed_clone();

            let grpc = grpc_router
                .into_service()
                .map_response(|r| r.map(axum::body::boxed))
                .boxed_clone();

            let http_grpc = Steer::new(vec![rest, grpc], |req: &Request<Body>, _svcs: &[_]| {
                if req.headers().get(CONTENT_TYPE).map(|v| v.as_bytes())
                    != Some(b"application/grpc")
                {
                    0
                } else {
                    1
                }
            });

            let server = Server::bind(&self.socket).serve(Shared::new(http_grpc));

            info!("Start Listening on {:?}", self.socket);
            server.with_graceful_shutdown(shutdown_signal()).await
        } else {
            let server = Server::bind(&self.socket).serve(rest.into_make_service());

            info!("Start Listening on {:?}", self.socket);
            server.with_graceful_shutdown(shutdown_signal()).await
        }
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
