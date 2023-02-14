//!This is a shortcut fn to read [`AppConfig`] and call [`init_tracing`] and [`init_metrics`] fn.
#[cfg(feature = "tokio-metrics")]
use crate::observability::tokio_metrics::init_tokio_metrics_task;
use crate::observability::{init_metrics, init_tracing};
use crate::{error::Result, *};
use serde::de::DeserializeOwned;
use std::fmt::Debug;

/// Reads AppConfig and calls [`init_tracing`].
/// Return Error if fails to read [`AppConfig`] or [`init_tracing`] returns error.
/// Return Error if called twice because of internal call to [`tracing_subscriber::registry().try_init()`].
///```no_run
/// use fregate::*;
/// use fregate::axum::{Router, routing::get, response::IntoResponse};
///
/// #[tokio::main]
/// async fn main() {
///    std::env::set_var("TEST_PORT", "3333");
///    std::env::set_var("TEST_NUMBER", "1010");
///
///     let config: AppConfig = bootstrap([
///         ConfigSource::File("./examples/configuration/app.yaml"),
///         ConfigSource::EnvPrefix("TEST"),
///     ])
///     .unwrap();
///
///     Application::new(config)
///         .router(Router::new().route("/", get(|| async { "Hello World"})))
///         .serve()
///         .await
///         .unwrap();
/// }
/// ```
pub fn bootstrap<'a, ConfigExt, S>(sources: S) -> Result<AppConfig<ConfigExt>>
where
    S: IntoIterator<Item = ConfigSource<'a>>,
    ConfigExt: Debug + DeserializeOwned,
{
    let mut config = AppConfig::<ConfigExt>::load_from(sources)?;

    let ObservabilityConfig {
        log_level,
        version,
        trace_level,
        service_name,
        component_name,
        traces_endpoint,
        msg_length,
        buffered_lines_limit,
        headers_filter,
        ..
    } = &config.observability_cfg;

    let worker_guard = init_tracing(
        log_level,
        trace_level,
        version,
        service_name,
        component_name,
        traces_endpoint.as_deref(),
        *msg_length,
        *buffered_lines_limit,
        headers_filter.clone(),
    )?;

    config.worker_guard.replace(worker_guard);
    init_metrics()?;

    #[cfg(feature = "tokio-metrics")]
    init_tokio_metrics_task(config.observability_cfg.metrics_update_interval);

    tracing::info!("Configuration: `{config:?}`.");
    Ok(config)
}
