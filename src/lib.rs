mod extentions;
mod builder;

use axum::{routing::IntoMakeService, Router};
use config::Config;
use hyper::{server::conn::AddrIncoming, Server};
use std::net::SocketAddr;
use tokio::signal;
use tracing::info;

pub use builder::*;

pub struct Application {
    server: Server<AddrIncoming, IntoMakeService<Router>>,
    socket: SocketAddr,
    _config: Config,
}

impl Application {
    pub fn builder<'a, H: Health>() -> ApplicationBuilder<'a, H> {
        ApplicationBuilder::default()
    }

    pub async fn run(self) -> hyper::Result<()> {
        info!("Start Listening on {:?}", self.socket);
        // TODO: LISTEN ON THE BACKGROUND ?
        self.server.with_graceful_shutdown(shutdown_signal()).await
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
