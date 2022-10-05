use crate::logging::init_tracing;
use crate::{error::Result, *};
use ::tracing::info;
use serde::de::DeserializeOwned;
use std::fmt::Debug;

/// Reads AppConfig and [`init_tracing`].\
/// Return Error if fails to read [`AppConfig`] or [`init_tracing`].\
/// Return Error if called twice because of internal call to tracing_subscriber::registry().try_init().
pub fn bootstrap<'a, ConfigExt, S>(sources: S) -> Result<AppConfig<ConfigExt>>
where
    S: IntoIterator<Item = ConfigSource<'a>>,
    ConfigExt: Debug + DeserializeOwned,
{
    let config = AppConfig::<ConfigExt>::load_from(sources)?;

    let LoggerConfig {
        log_level,
        trace_level,
        service_name,
        traces_endpoint,
    } = &config.logger;

    init_tracing(
        log_level,
        trace_level,
        service_name,
        traces_endpoint.as_deref(),
    )?;

    info!("Configuration: `{config:?}`.");

    Ok(config)
}
