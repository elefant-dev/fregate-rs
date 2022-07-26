use core::fmt;
use std::net::{IpAddr, Ipv4Addr, Ipv6Addr, SocketAddr};

use crate::{
    axum_ext::response::{png, yaml},
    health::{HealthIndicator, HealthIndicatorRef, HealthStatus, UpHealth},
    telemetry,
    telemetry::get_metrics,
};
use axum::{
    body::Bytes,
    http::StatusCode,
    routing::{get, IntoMakeService},
    Extension, Json, Router,
};
use config::Config;
use hyper::{server::conn::AddrIncoming, Server};
use serde::Serialize;
use tokio::signal;
use tower_http::trace::TraceLayer;
use tracing::{info, warn};

static FAVICON: Bytes = Bytes::from_static(include_bytes!("resources/favicon.png"));

const DEFAULT_PORT: u16 = 8000;
const API_PATH: &str = "/v1";
const OPENAPI_PATH: &str = "/openapi";
const OPENAPI: &str = include_str!("resources/openapi.yaml");
const FAVICON_PATH: &str = "/favicon.ico";

#[derive(Serialize, Debug)]
pub enum ApplicationStatus {
    Starting,
    Started,
    Stopping,
    Stopped,
    Unknown,
}

impl fmt::Display for ApplicationStatus {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(f, "{:?}", self)
    }
}

pub struct ApplicationContext<H = UpHealth>
where
    H: HealthIndicator,
{
    pub conf: String,
    pub health: H,
}

impl Default for ApplicationContext {
    fn default() -> Self {
        Self {
            health: UpHealth {},
            conf: String::default(),
        }
    }
}

type InnerServer = Server<AddrIncoming, IntoMakeService<Router>>;

pub struct Transport {
    socket: SocketAddr,
}

//type Svs = ServiceFn<dyn FnMut(Request<Body>) -> dyn Future<Output=Result<http::Response<BoxBody>, hyper::Error>>>;

pub struct Application {
    pub conf: Config,
    server: Box<InnerServer>,
    transport: Transport,
}

impl Application {
    pub fn builder() -> ApplicationBuilder {
        ApplicationBuilder::new()
    }

    pub async fn run(self) -> hyper::Result<()> {
        info!("Listening on {:?}", self.transport.socket);
        self.server.with_graceful_shutdown(shutdown_signal()).await
    }
}

pub enum ApplicationConfigurationEnvironment {
    Simple,
    Prefix(&'static str),
}

pub struct ApplicationBuilder {
    address: IpAddr,
    configuration_environment: Option<ApplicationConfigurationEnvironment>,
    configuration_file: Option<String>,
    health: Option<HealthIndicatorRef>,
    port: Option<u16>,
    rest_router: Router,
    service: Option<String>,
    telemetry: bool,
}

impl ApplicationBuilder {
    fn new() -> Self {
        Self {
            address: IpAddr::from(Ipv4Addr::UNSPECIFIED),
            configuration_environment: None,
            configuration_file: None,
            health: None,
            port: None,
            rest_router: Router::default(),
            service: None,
            telemetry: false,
        }
    }

    pub fn build(&self) -> Application {
        if self.telemetry {
            telemetry::init();
        }

        let conf = self.conf();

        let address: IpAddr = match conf.get::<String>("transport.address") {
            Ok(a) => {
                let ip4 = a.parse::<Ipv4Addr>();
                let ip6 = a.parse::<Ipv6Addr>();

                if let Ok(ip4) = ip4 {
                    IpAddr::V4(ip4)
                } else if let Ok(ip6) = ip6 {
                    IpAddr::V6(ip6)
                } else {
                    IpAddr::V4(Ipv4Addr::UNSPECIFIED)
                }
            }

            Err(_) => IpAddr::V4(Ipv4Addr::UNSPECIFIED),
        };

        let port = match self.port {
            Some(p) => p,
            None => match conf.get::<u16>("transport.port") {
                Ok(p) => p,
                Err(_) => DEFAULT_PORT,
            },
        };

        let transport = Transport {
            socket: SocketAddr::new(address, port),
        };
        //
        // let svr = if let Some(s) = self.service.as_ref() {
        //     Shared::new(s)
        // };

        let server = Server::bind(&transport.socket).serve(self.router().into_make_service());

        Application {
            server: Box::new(server),
            transport,
            conf,
        }
    }

    fn router(&self) -> Router {
        Router::new()
            .nest(API_PATH, self.rest_router.clone())
            .layer(TraceLayer::new_for_http())
            .merge(self.health_router())
            .merge(self.metrics_router())
            .route(OPENAPI_PATH, get(|| async { yaml(OPENAPI) }))
            .route(FAVICON_PATH, get(|| async { png(&FAVICON) }))
    }

    fn health_router(&self) -> Router {
        async fn health_handler(
            Extension(health): Extension<HealthIndicatorRef>,
        ) -> Json<HealthStatus> {
            Json(health.health())
        }
        async fn live_handler(Extension(health): Extension<HealthIndicatorRef>) -> StatusCode {
            if health.live() {
                StatusCode::NO_CONTENT
            } else {
                StatusCode::SERVICE_UNAVAILABLE
            }
        }
        async fn ready_handler(Extension(health): Extension<HealthIndicatorRef>) -> StatusCode {
            if health.ready() {
                StatusCode::NO_CONTENT
            } else {
                StatusCode::SERVICE_UNAVAILABLE
            }
        }

        self.health
            .clone()
            .map(|h| {
                Router::new()
                    .route("/health", get(health_handler))
                    .route("/ready", get(ready_handler))
                    .route("/live", get(live_handler))
                    .route("/", get(live_handler))
                    .layer(Extension(h))
            })
            .unwrap_or_default()
    }

    fn metrics_router(&self) -> Router {
        if self.telemetry {
            Router::new().route("/metrics", get(|| async { get_metrics() }))
        } else {
            Router::new()
        }
    }

    pub fn conf(&self) -> Config {
        let mut builder = Config::builder();

        if let Some(file) = self.configuration_file.as_ref() {
            builder = builder.add_source(config::File::with_name(file.as_str()))
        }

        if let Some(env) = self.configuration_environment.as_ref() {
            builder = builder.add_source(match env {
                ApplicationConfigurationEnvironment::Simple => {
                    config::Environment::default().separator("_")
                }
                ApplicationConfigurationEnvironment::Prefix(prefix) => {
                    config::Environment::with_prefix(prefix).separator("_")
                }
            })
        };

        let conf = builder.build();
        if let Err(err) = conf.as_ref() {
            warn!("Configuration error: {:?}", err);
        }

        conf.unwrap_or_default()
    }

    pub fn address(mut self, address: impl Into<IpAddr>) -> Self {
        self.address = address.into();
        self
    }

    pub fn configuration_environment(
        mut self,
        env: impl Into<ApplicationConfigurationEnvironment>,
    ) -> Self {
        self.configuration_environment = Some(env.into());
        self
    }

    pub fn configuration_file(mut self, file: impl Into<String>) -> Self {
        self.configuration_file = Some(file.into());
        self
    }

    pub fn port(mut self, port: impl Into<u16>) -> Self {
        self.port = Some(port.into());
        self
    }

    pub fn rest_router(mut self, router: impl Into<Router>) -> Self {
        self.rest_router = router.into();
        self
    }

    pub fn service(mut self, service: impl Into<String>) -> Self {
        self.service = Some(service.into());
        self
    }

    pub fn telemetry(mut self, telemetry: impl Into<bool>) -> Self {
        self.telemetry = telemetry.into();
        self
    }

    pub fn health(mut self, health: Option<HealthIndicatorRef>) -> Self {
        self.health = health;
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
            .expect("failed to install signal handler")
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
