use crate::{
    build_management_router,
    error::Result,
    extensions::RouterOptionalExt,
    health::{AlwaysReadyAndAlive, Health},
    AppConfig,
};
use axum::Router;
use hyper::Server;
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

    /// Set up Router Application will serve to
    pub fn router(mut self, router: Router) -> Self {
        self.router = Some(router);
        self
    }

    /// Start serving at specified host and port in [AppConfig] accepting both HTTP1 and HTTP2
    pub async fn serve(self) -> Result<()>
    where
        H: Health,
    {
        let (router, application_socket) = self.prepare_router();
        run_service(&application_socket, router).await
    }

    /// Serve TLS
    #[cfg(feature = "native-tls")]
    pub async fn serve_tls(self) -> Result<()>
    where
        H: Health,
    {
        use crate::error::Error;
        use futures_util::TryFutureExt;
        use std::fmt;
        use tokio::{fs, try_join};

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

        let (router, application_socket) = self.prepare_router();

        run_tls_service(&application_socket, router, &tls_cert, &tls_key).await
    }

    fn prepare_router(self) -> (Router, SocketAddr)
    where
        H: Health,
    {
        let router = build_management_router(self.health_indicator).merge_optional(self.router);
        let application_socket = SocketAddr::new(self.config.host, self.config.port);
        (router, application_socket)
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

async fn run_service(socket: &SocketAddr, router: Router) -> Result<()> {
    let app = router.into_make_service_with_connect_info::<SocketAddr>();
    let server = Server::bind(socket).serve(app);

    info!(target: "server", "Started: http://{socket}");

    Ok(server.with_graceful_shutdown(shutdown_signal()).await?)
}

#[cfg(feature = "native-tls")]
async fn run_tls_service(
    socket: &SocketAddr,
    router: Router,
    pem: &[u8],
    key: &[u8],
) -> Result<()> {
    use hyper::server::accept;
    use native_tls::Identity;
    use tokio::net::TcpListener;
    use tokio_native_tls::TlsAcceptor;
    use tokio_stream::wrappers::TcpListenerStream;
    use tonic_native_tls::{native_tls, tokio_native_tls};

    let identity = Identity::from_pkcs8(pem, key)?;
    let acceptor = TlsAcceptor::from(native_tls::TlsAcceptor::new(identity)?);

    let listener = TcpListener::bind(socket).await?;
    let stream = TcpListenerStream::new(listener);
    let incoming = accept::from_stream(tonic_native_tls::incoming(stream, acceptor));

    let app = router.into_make_service_with_connect_info::<SocketAddr>();
    let server = Server::builder(incoming).serve(app);

    info!(target: "server", "Started: http://{socket}");

    Ok(server.with_graceful_shutdown(shutdown_signal()).await?)
}
