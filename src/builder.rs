mod configuration;
mod health;
mod metrics;
mod router;
mod tracing;

use axum::Router as AxumRouter;
use std::net::{IpAddr, SocketAddr};
use std::sync::Arc;
use tonic::transport::server::Router as TonicRouter;

use crate::{
    builder::{metrics::*, tracing::*},
    Application,
};
use router::*;
pub use {configuration::*, health::*};

#[derive(Debug, Default)]
pub struct ApplicationBuilder<'a, H> {
    configuration: ConfigurationBuilder<'a>,
    rest_router: RouterBuilder<H>,
    grpc_router: Option<TonicRouter>,
    health_indicator: Option<Arc<H>>,
    address: Option<IpAddr>,
    port: Option<u16>,
    init_tracing: bool,
    init_metrics: bool,
}

impl<'a, H: Health> ApplicationBuilder<'a, H> {
    pub fn build(&mut self) -> Application {
        if self.init_tracing {
            init_tracing();
        }

        if self.init_metrics {
            init_metrics();
            self.rest_router.init_metrics();
        }

        let health_indicator = self
            .health_indicator
            .take()
            .unwrap_or_else(|| Arc::new(H::default()));

        self.rest_router.set_health_indicator(health_indicator);

        let config = self.configuration.build();

        let socket = match (self.address.take(), self.port.take()) {
            (Some(add), Some(port)) => SocketAddr::new(add, port),
            (None, Some(port)) => SocketAddr::new(get_address(&config), port),
            (Some(add), None) => SocketAddr::new(add, get_port(&config)),
            (None, None) => SocketAddr::new(get_address(&config), get_port(&config)),
        };

        let router = self.rest_router.build();

        Application {
            rest_router: router,
            grpc_router: self.grpc_router.take(),
            socket,
            _config: config,
        }
    }

    pub fn set_configuration_file(&'a mut self, file: &'a str) -> &'a mut Self {
        self.configuration.set_path_to_file(file);
        self
    }

    pub fn set_configuration_environment(
        &'a mut self,
        environment: Environment<'a>,
    ) -> &'a mut Self {
        self.configuration.set_environment(environment);
        self
    }

    pub fn set_rest_routes(&mut self, router: AxumRouter) -> &mut Self {
        self.rest_router.set_rest_routes(router);
        self
    }

    pub fn set_grpc_routes(&mut self, router: TonicRouter) -> &mut Self {
        self.grpc_router = Some(router);
        self
    }

    pub fn set_health_indicator(&mut self, health_indicator: Arc<H>) -> &mut Self {
        self.health_indicator = Some(health_indicator);
        self
    }

    pub fn init_tracing(&mut self) -> &mut Self {
        self.init_tracing = true;
        self
    }

    pub fn init_metrics(&mut self) -> &mut Self {
        self.init_metrics = true;
        self
    }

    pub fn set_address(&mut self, address: impl Into<IpAddr>) -> &mut Self {
        self.address = Some(address.into());
        self
    }

    pub fn set_port(&mut self, port: impl Into<u16>) -> &mut Self {
        self.port = Some(port.into());
        self
    }
}
