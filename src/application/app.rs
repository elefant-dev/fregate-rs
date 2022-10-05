use crate::*;
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

// FIXME(kos): It's impossible to create several application instances.
//             If the `Application` stays, make it a module instead of a struct.

/// Application to set up HTTP server with given config [`AppConfig`]
#[derive(Debug)]
pub struct Application<'a, H, T> {
    // TODO(kos): Consider owning the `AppConfig` rather than referring it.
    //            It would simplify `Application` code and seems natural this
    //            way.
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

// TODO(kos): it looks like you're trying to implement Builder pattern.
// But its name does not reflect it.
// https://rust-unofficial.github.io/patterns/patterns/creational/builder.html
impl<'a, H, T> Application<'a, H, T> {
    // FIXME(kos): Put here `Hh: Health` bound, so the possible error of not
    //             implementing `Health` trait is cached earlier.
    //             In this case error description will be more descriptive.
    /// Set up new health indicator
    pub fn health_indicator<Hh>(self, health: Hh) -> Application<'a, Hh, T> {
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
        // FIXME(kos): Well, while it's obviously simpler, it's not very good to expose
        //             API endpoints and infrastructure endpoints on the same port (and
        //             router) because of the following reasons:
        //             1. HTTP services using this library may expose their API to
        //                public, while at the same time, in general case, we don't want
        //                our metrics to be exposed publicly. Of course, we may forbid
        //                metrics and liveness endpoints in external load-balancer
        //                configuration, but, simply, why bother? Keeping these endpoints
        //                on a different port, not being exposed publicly, cuts off
        //                possible human factors like "forgot to forbid metrics in
        //                load-balancer" and even removes the necessity to do so.
        //             2. HTTP services using this library may want different middlewares
        //                stack for API endpoints and infrastructure endpoints, like
        //                using some sort of rate-limiting or throttling for API
        //                endpoints, while fully omit them for metrics and liveness
        //                probes. Or, for example, collect access logs of API endpoints
        //                only and don't bother with the ones of metrics and liveness
        //                probes.
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
