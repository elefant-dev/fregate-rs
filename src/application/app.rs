#[cfg(feature = "tls")]
pub(crate) mod tls;

use crate::middleware::config::TraceRequestConfig;
use crate::middleware::trace_request;
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

/// Application to set up HTTP server with given config [`AppConfig`]
pub struct Application<'a, H, T> {
    config: &'a AppConfig<T>,
    health_indicator: Option<H>,
    router: Option<Router>,
    metrics_callback: Option<Arc<dyn Fn() + Send + Sync + 'static>>,
    use_default_trace_layer: bool,
    trace_request_config: Arc<TraceRequestConfig>,
}

impl<'a, H: Debug, T: Debug> Debug for Application<'a, H, T> {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        let Self {
            config,
            health_indicator,
            router,
            metrics_callback,
            use_default_trace_layer,
            trace_request_config,
        } = self;
        f.debug_struct("Application")
            .field("config", config)
            .field("health_indicator", health_indicator)
            .field("router", router)
            .field("use_default_trace_layer", use_default_trace_layer)
            .field("trace_request_config", trace_request_config)
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
        let trace_request_config = TraceRequestConfig::default()
            .service_name(config.logger.service_name.as_str())
            .component_name(config.logger.component_name.as_str());

        Application::<'a, AlwaysReadyAndAlive, T> {
            config,
            health_indicator: Some(AlwaysReadyAndAlive {}),
            router: None,
            metrics_callback: None,
            use_default_trace_layer: true,
            trace_request_config: Arc::new(trace_request_config),
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
            use_default_trace_layer,
            trace_request_config,
        } = self;

        Application::<'a, Hh, T> {
            config,
            health_indicator: Some(health),
            router,
            metrics_callback,
            use_default_trace_layer,
            trace_request_config,
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

    /// Example:
    /// In this case [`trace_request`] is not attached to Application so no default tracing/metrics/logging for incoming requests
    /// ```no_run
    ///   use fregate::{AppConfig, Application};
    ///
    ///    #[tokio::main]
    ///   async fn main() {
    ///        Application::new(&AppConfig::default())
    ///            .use_default_tracing_layer(false)
    ///            .serve()
    ///            .await
    ///            .unwrap();
    ///    }
    /// ```
    pub fn use_default_tracing_layer(self, use_default: bool) -> Self {
        Self {
            use_default_trace_layer: use_default,
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
        let router = self.router.map(|router| {
            if self.use_default_trace_layer {
                router.layer(from_fn(move |req, next| {
                    trace_request(req, next, self.trace_request_config.clone())
                }))
            } else {
                router
            }
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
