use crate::Optional;
use axum::Router as AxumRouter;
use hyper::Server;
use serde::de::DeserializeOwned;
use std::fmt::Debug;
use std::net::SocketAddr;
use tokio::signal;
use tracing::info;

use crate::utils::*;

#[derive(Debug)]
pub struct Application<'a, H, T> {
    config: &'a AppConfig<T>,
    health_indicator: Option<H>,
    router: Option<AxumRouter>,
}

impl<'a, T: DeserializeOwned + Debug> Application<'a, AlwaysReadyAndAlive, T> {
    pub fn new(config: &'a AppConfig<T>) -> Self {
        Application::<'a, AlwaysReadyAndAlive, T> {
            config,
            health_indicator: Some(AlwaysReadyAndAlive {}),
            router: None,
        }
    }
}

impl<'a, H: Health, T: DeserializeOwned + Debug> Application<'a, H, T> {
    pub fn health_indicator<Hh>(self, health: Hh) -> Application<'a, Hh, T> {
        Application::<'a, Hh, T> {
            config: self.config,
            health_indicator: Some(health),
            router: self.router,
        }
    }

    pub async fn serve(self) -> hyper::Result<()> {
        info!("Application starts on: `{config:?}`.", config = self.config);

        let app = build_management_router(self.health_indicator).merge_optional(self.router);
        let application_socket = SocketAddr::new(self.config.host, self.config.port);

        run_service(&application_socket, app).await
    }

    pub fn router(mut self, router: AxumRouter) -> Self {
        self.router = Some(router);
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

async fn run_service(socket: &SocketAddr, rest: AxumRouter) -> hyper::Result<()> {
    let server = Server::bind(socket).serve(rest.into_make_service());
    info!(target: "server", "Started: http://{socket}");

    server.with_graceful_shutdown(shutdown_signal()).await
}
