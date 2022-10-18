use crate::{
    build_management_router,
    error::Result,
    extensions::RouterOptionalExt,
    health::{AlwaysReadyAndAlive, Health},
    AppConfig,
};
use axum::extract::connect_info::IntoMakeServiceWithConnectInfo;
use axum::Router;
use hyper::Server;
use std::fmt::Debug;
use std::net::SocketAddr;
use tokio::signal;
use tracing::info;

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
        let (app, socket) = self.prepare_application::<SocketAddr>();

        let server = Server::bind(&socket).serve(app);
        info!(target: "server", "Started: http://{socket}");

        Ok(server.with_graceful_shutdown(shutdown_signal()).await?)
    }

    /// Serve TLS
    #[cfg(feature = "native-tls")]
    pub async fn serve_tls(self) -> Result<()>
    where
        H: Health,
    {
        use crate::error::Error;
        use futures_util::{StreamExt, TryFutureExt};
        use hyper::server::{accept, conn::AddrIncoming};
        use std::{fmt, future::ready};
        use tls_listener::TlsListener;
        use tokio::{fs, try_join};
        use tracing::warn;

        fn cant_load<Arg: fmt::Display>(r#type: &str) -> impl FnOnce(Arg) -> Error + '_ {
            move |error| Error::CustomError(format!("Cant load TLS {type}: `{error}`."))
        }

        let tls_cert_path = self
            .config
            .tls_cert_path
            .as_deref()
            .ok_or_else(|| cant_load("certificate")("No path present."))?;
        let tls_key_path = self
            .config
            .tls_key_path
            .as_deref()
            .ok_or_else(|| cant_load("key")("No path present."))?;
        let (tls_cert, tls_key) = try_join!(
            fs::read(tls_cert_path).map_err(cant_load("certificate")),
            fs::read(tls_key_path).map_err(cant_load("key"))
        )?;

        let (app, socket) = self.prepare_application::<RemoteAddr>();

        let acceptor = build_acceptor(tls_cert, tls_key)?;
        let addr = AddrIncoming::bind(&socket)?;

        let listener = TlsListener::new_hyper(acceptor, addr).filter(|conn| {
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

    /// Set up Router Application will serve to
    pub fn router(mut self, router: Router) -> Self {
        self.router = Some(router);
        self
    }

    fn prepare_application<C>(self) -> (IntoMakeServiceWithConnectInfo<Router, C>, SocketAddr)
    where
        H: Health,
    {
        let app = build_management_router(self.health_indicator)
            .merge_optional(self.router)
            .into_make_service_with_connect_info::<C>();
        let socket = SocketAddr::new(self.config.host, self.config.port);

        (app, socket)
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

#[cfg(feature = "native-tls")]
fn build_acceptor(pem: Vec<u8>, key: Vec<u8>) -> Result<tokio_native_tls::TlsAcceptor> {
    use tokio_native_tls::native_tls::{Identity, TlsAcceptor};

    let identity = Identity::from_pkcs8(&pem, &key)?;
    TlsAcceptor::builder(identity)
        .build()
        .map(From::from)
        .map_err(Into::into)
}

#[cfg(feature = "native-tls")]
#[derive(Debug, Clone)]
/// Wrapper for SocketAddr to implement [`axum::extract::connect_info::Connected`] so
/// we can run [`axum::routing::Router::into_make_service_with_connect_info`] with [`TlsStream<AddrStream>`]
pub struct RemoteAddr(pub SocketAddr);

#[cfg(feature = "native-tls")]
impl
    axum::extract::connect_info::Connected<
        &tokio_native_tls::TlsStream<hyper::server::conn::AddrStream>,
    > for RemoteAddr
{
    fn connect_info(target: &tokio_native_tls::TlsStream<hyper::server::conn::AddrStream>) -> Self {
        RemoteAddr(target.get_ref().get_ref().get_ref().remote_addr())
    }
}
