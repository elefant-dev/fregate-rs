//!This is a shortcut fn to read [`AppConfig`] and call [`init_tracing`] and [`init_metrics`] fn.
use crate::observability::cgroupv2::init_cgroup_metrics;
use crate::observability::sys_info::init_sys_metrics;
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
    bootstrap_with_callback(sources, |_s| {})
}

/// This has same functionalilty as [`bootstrap`] function, but calls `callback` before setting
/// logging, tracing and metrics allowing to modify some of the variables.
/// Example:
///```no_run
/// use fregate::*;
/// use fregate::axum::{Router, routing::get, response::IntoResponse};
///
/// #[tokio::main]
/// async fn main() {
///    std::env::set_var("TEST_PORT", "3333");
///    std::env::set_var("TEST_NUMBER", "1010");
///
///     let config: AppConfig = bootstrap_with_callback([
///         ConfigSource::File("./examples/configuration/app.yaml"),
///         ConfigSource::EnvPrefix("TEST")
///     ],
///     |cfg| {
///        // version is used for logging and tracing and you might want to set it from compile time env var.
///        if let Some(build_version) = option_env!("BUILD_VERSION") {
///           cfg.observability_cfg.version = build_version.to_owned();
///        }  
///     })
///     .unwrap();
///
///     Application::new(config)
///         .router(Router::new().route("/", get(|| async { "Hello World"})))
///         .serve()
///         .await
///         .unwrap();
/// }
/// ```
pub fn bootstrap_with_callback<'a, ConfigExt, S>(
    sources: S,
    callback: impl FnOnce(&mut AppConfig<ConfigExt>),
) -> Result<AppConfig<ConfigExt>>
where
    S: IntoIterator<Item = ConfigSource<'a>>,
    ConfigExt: Debug + DeserializeOwned,
{
    let mut config = AppConfig::<ConfigExt>::load_from(sources)?;
    callback(&mut config);
    let ObservabilityConfig {
        service_name,
        component_name,
        version,
        logger_config,
        cgroup_metrics,
        metrics_update_interval,
        trace_level,
        traces_endpoint,
    } = &config.observability_cfg;

    let worker_guard = init_tracing(
        logger_config,
        trace_level,
        version,
        service_name,
        component_name,
        traces_endpoint.as_deref(),
    )?;

    config.worker_guard.replace(worker_guard);
    init_metrics(*cgroup_metrics)?;

    #[cfg(feature = "tokio-metrics")]
    init_tokio_metrics_task(*metrics_update_interval);

    if *cgroup_metrics {
        init_cgroup_metrics(*metrics_update_interval)
    } else {
        init_sys_metrics(*metrics_update_interval);
    }

    tracing::info!("Configuration: `{config:?}`.");
    Ok(config)
}
