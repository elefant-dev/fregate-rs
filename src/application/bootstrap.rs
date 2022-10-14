use crate::logging::init_tracing;
use crate::{error::Result, *};
use ::tracing::info;
use serde::de::DeserializeOwned;
use std::fmt::Debug;
use std::sync::Arc;

/// Reads AppConfig and [`init_tracing`].\
/// Return Error if fails to read [`AppConfig`] or [`init_tracing`].\
/// Return Error if called twice because of internal call to tracing_subscriber::registry().try_init().
///```no_run
/// use fregate::*;
/// use fregate::axum::{Router, routing::get, response::IntoResponse};
/// use std::sync::Arc;
///
/// #[tokio::main]
/// async fn main() {
///    std::env::set_var("TEST_PORT", "3333");
///    std::env::set_var("TEST_NUMBER", "1010");
///
///     let config: Arc<AppConfig<Empty>> = bootstrap([
///         ConfigSource::File("./examples/configuration/app.yaml"),
///         ConfigSource::EnvPrefix("TEST"),
///     ])
///     .unwrap();
///
///     Application::new(&config)
///         .router(Router::new().route("/", get(|| async { "Hello World"})))
///         .serve()
///         .await
///         .unwrap();
/// }
/// ```
pub fn bootstrap<'a, ConfigExt, S>(sources: S) -> Result<Arc<AppConfig<ConfigExt>>>
where
    S: IntoIterator<Item = ConfigSource<'a>>,
    ConfigExt: Debug + DeserializeOwned,
{
    let config = AppConfig::<ConfigExt>::load_from(sources)?;

    let LoggerConfig {
        log_level,
        version,
        trace_level,
        service_name,
        component_name,
        traces_endpoint,
    } = &config.logger;

    init_tracing(
        log_level,
        trace_level,
        version,
        service_name,
        component_name,
        traces_endpoint.as_deref(),
    )?;

    init_metrics()?;

    info!("Configuration: `{config:?}`.");

    Ok(Arc::new(config))
}
