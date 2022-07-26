#[cfg(feature = "tls")]
pub(crate) mod tls;

use crate::middleware::{trace_request, Attributes};
use crate::{
    build_management_router,
    error::Result,
    extensions::RouterOptionalExt,
    health::{AlwaysReadyAndAlive, Health},
    AppConfig,
};
use axum::middleware::from_fn;
use axum::Router;
use hyper::Server;
use std::fmt::{Debug, Formatter};
use std::net::SocketAddr;
use std::sync::Arc;
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
pub struct Application<'a, H, T> {
    config: &'a AppConfig<T>,
    health_indicator: Option<H>,
    router: Option<Router>,
    metrics_callback: Option<Arc<dyn Fn() + Send + Sync + 'static>>,
}

impl<'a, H: Debug, T: Debug> Debug for Application<'a, H, T> {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        let Self {
            config,
            health_indicator,
            router,
            metrics_callback,
        } = self;
        f.debug_struct("Application")
            .field("config", config)
            .field("health_indicator", health_indicator)
            .field("router", router)
            .field(
                "metrics_callback",
                if metrics_callback.is_some() {
                    &"Some"
                } else {
                    &"None"
                },
            )
            .finish()
    }
}

impl<'a, T> Application<'a, AlwaysReadyAndAlive, T> {
    /// Creates new Application with health checks always returning [200 OK]
    pub fn new(config: &'a AppConfig<T>) -> Self {
        Application::<'a, AlwaysReadyAndAlive, T> {
            config,
            health_indicator: Some(AlwaysReadyAndAlive {}),
            router: None,
            metrics_callback: None,
        }
    }
}

impl<'a, H, T> Application<'a, H, T> {
    /// Set up new health indicator
    pub fn health_indicator<Hh: Health>(self, health: Hh) -> Application<'a, Hh, T> {
        let Self {
            config,
            health_indicator: _,
            router,
            metrics_callback,
        } = self;

        Application::<'a, Hh, T> {
            config,
            health_indicator: Some(health),
            router,
            metrics_callback,
        }
    }

    /// Set up Router Application will serve to
    pub fn router(self, router: Router) -> Self {
        Self {
            router: Some(router),
            ..self
        }
    }

    /// Set up callback which will be called before metrics will render.
    pub fn metrics_callback(self, metrics_callback: impl Fn() + Send + Sync + 'static) -> Self {
        Self {
            metrics_callback: Some(Arc::new(metrics_callback)),
            ..self
        }
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
    #[cfg(feature = "tls")]
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

        let tls_handshake_timeout = self.config.tls.handshake_timeout;

        let tls_cert_path = self
            .config
            .tls
            .cert_path
            .as_deref()
            .ok_or_else(|| cant_load("certificate")("No path present."))?;

        let tls_key_path = self
            .config
            .tls
            .key_path
            .as_deref()
            .ok_or_else(|| cant_load("key")("No path present."))?;

        let (tls_cert, tls_key) = try_join!(
            fs::read(tls_cert_path).map_err(cant_load("certificate")),
            fs::read(tls_key_path).map_err(cant_load("key"))
        )?;

        let (router, application_socket) = self.prepare_router();

        tls::run_service(
            &application_socket,
            router,
            tls_handshake_timeout,
            tls_cert,
            tls_key,
        )
        .await
    }

    fn prepare_router(self) -> (Router, SocketAddr)
    where
        H: Health,
    {
        let attributes = Attributes::new_from_config(self.config);
        let router = self.router.map(|r| {
            r.layer(from_fn(move |req, next| {
                trace_request(req, next, attributes.clone())
            }))
        });

        let router = build_management_router(self.health_indicator, self.metrics_callback)
            .merge_optional(router);
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
