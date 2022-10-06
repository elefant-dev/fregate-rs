use crate::{
    build_management_router,
    error::Result,
    extensions::RouterOptionalExt,
    health::{AlwaysReadyAndAlive, Health},
    ApplicationConfig,
};
use axum::Router;
use hyper::Server;
use std::net::SocketAddr;
use tokio::signal;
use tracing::info;

/// Application to set up HTTP server with given config [`ApplicationConfig`]
#[derive(Debug)]
pub struct Application<H> {
    config: ApplicationConfig,
    health_indicator: Option<H>,
    router: Option<Router>,
}

impl Application<AlwaysReadyAndAlive> {
    /// Creates new Application with health checks always returning [200 OK]
    pub fn new(config: ApplicationConfig) -> Self {
        Application::<AlwaysReadyAndAlive> {
            config,
            health_indicator: Some(AlwaysReadyAndAlive {}),
            router: None,
        }
    }
}

impl<H> Application<H> {
    /// Set up new health indicator
    pub fn health_indicator<Hh: Health>(self, health: Hh) -> Application<Hh> {
        Application::<Hh> {
            config: self.config,
            health_indicator: Some(health),
            router: self.router,
        }
    }

    /// Start serving at specified host and port in [ApplicationConfig] accepting both HTTP1 and HTTP2
    pub async fn serve(self) -> Result<()>
    where
        H: Health,
    {
        let app = build_management_router(self.health_indicator).merge_optional(self.router);
        let application_socket = SocketAddr::new(self.config.host, self.config.port);

        run_service(&application_socket, app).await
    }

    /// Set up Router Application will serve to
    pub fn router(mut self, router: Router) -> Self {
        self.router = Some(router);
        self
    }
}

#[allow(clippy::expect_used)]
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

async fn run_service(socket: &SocketAddr, rest: Router) -> Result<()> {
    let server = Server::bind(socket).serve(rest.into_make_service());
    info!(target: "server", "Started: http://{socket}");

    Ok(server.with_graceful_shutdown(shutdown_signal()).await?)
}
