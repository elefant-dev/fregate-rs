use crate::{
    build_management_router,
    error::Result,
    extensions::RouterOptionalExt,
    health::{AlwaysReadyAndAlive, Health},
    AppConfig,
};
use axum::Router;
use hyper::Server;
use std::fmt::Debug;
use std::net::SocketAddr;
use tokio::signal;
use tracing::info;

#[cfg(feature = "tls")]
use {
    crate::error::Error,
    axum::extract::connect_info::Connected,
    futures_util::StreamExt,
    hyper::server::accept,
    hyper::server::conn::AddrIncoming,
    hyper::server::conn::AddrStream,
    native_tls::{Identity, TlsAcceptor},
    std::future::ready,
    tls_listener::TlsListener,
    tokio_native_tls::TlsStream,
    tracing::log::warn,
};

// TODO(kos): Consider avoiding doing framework and eliminating `Application`.
//            Better than the alternative.

// TODO(kos): Alternatively:
//            - hide as much as possible, proxying what is needed by the
//              `Application`.
//            - accept constructor function in `Application::router`, that
//              allows getting proxy accessors to various extensions, like
//              `logging` or `opentelemetry` context.
//            - provide a function like
//              `with_application_state(impl FnOnce(ApplicationState))`, that
//              would be a single access point to the global state.
//            - `init_metrics` and `init_tracing` should be methods of
//              `Application`.
//            - `bootstrap()` is redundant, move its content into
//              `Application::new()`
//            - make `Application::new()` async if necessary.
//            But removing the `Application` is a better option.

/// Application to set up HTTP server with given config [`AppConfig`]
#[derive(Debug)]
pub struct Application<'a, H, T> {
    config: &'a AppConfig<T>,
    health_indicator: Option<H>,
    router: Option<Router>,
}

impl<'a, T> Application<'a, AlwaysReadyAndAlive, T> {
    /// Creates new Application with health checks always returning [200 OK]
    pub fn new(config: &'a AppConfig<T>) -> Self {
        Application::<'a, AlwaysReadyAndAlive, T> {
            config,
            health_indicator: Some(AlwaysReadyAndAlive {}),
            router: None,
        }
    }
}

impl<'a, H, T> Application<'a, H, T> {
    /// Set up new health indicator
    pub fn health_indicator<Hh: Health>(self, health: Hh) -> Application<'a, Hh, T> {
        Application::<'a, Hh, T> {
            config: self.config,
            health_indicator: Some(health),
            router: self.router,
        }
    }

    /// Start serving at specified host and port in [AppConfig] accepting both HTTP1 and HTTP2
    pub async fn serve(self) -> Result<()>
    where
        H: Health,
    {
        let app = build_management_router(self.health_indicator).merge_optional(self.router);
        let app = app.into_make_service_with_connect_info::<SocketAddr>();
        let socket = SocketAddr::new(self.config.host, self.config.port);

        let server = Server::bind(&socket).serve(app);
        info!(target: "server", "Started: http://{socket}");

        Ok(server.with_graceful_shutdown(shutdown_signal()).await?)
    }

    /// Set up Router Application will serve to
    pub fn router(mut self, router: Router) -> Self {
        self.router = Some(router);
        self
    }

    /// Serve TLS
    #[cfg(feature = "tls")]
    pub async fn serve_tls(self, pfx: &[u8], password: &str) -> Result<()>
    where
        H: Health,
    {
        let app = build_management_router(self.health_indicator).merge_optional(self.router);
        let app = app.into_make_service_with_connect_info::<RemoteAddr>();

        let socket = SocketAddr::new(self.config.host, self.config.port);

        let identity = Identity::from_pkcs12(pfx, password).map_err(Error::NativeTlsError)?;
        let acceptor = TlsAcceptor::builder(identity).build()?;
        let addr = AddrIncoming::bind(&socket)?;

        let listener =
            TlsListener::<_, tokio_native_tls::TlsAcceptor>::new_hyper(acceptor.into(), addr)
                .filter(|conn| {
                    if let Err(err) = conn {
                        warn!("TLS connect error: {err}");
                        ready(false)
                    } else {
                        ready(true)
                    }
                });

        let server = Server::builder(accept::from_stream(listener)).serve(app);

        info!(target: "server", "Started: https://{socket}");
        Ok(server.with_graceful_shutdown(shutdown_signal()).await?)
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

#[cfg(feature = "tls")]
#[derive(Debug, Clone)]
/// Wrapper for SocketAddr to implement [`axum::extract::connect_info::Connected`] so
/// we can run [`axum::routing::Router::into_make_service_with_connect_info`] with [`TlsStream<AddrStream>`]
pub struct RemoteAddr(pub SocketAddr);

#[cfg(feature = "tls")]
impl Connected<&TlsStream<AddrStream>> for RemoteAddr {
    fn connect_info(target: &TlsStream<AddrStream>) -> Self {
        RemoteAddr(target.get_ref().get_ref().get_ref().remote_addr())
    }
}
