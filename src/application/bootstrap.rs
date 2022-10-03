use crate::*;
use serde::de::DeserializeOwned;
// TODO(kos): redundant usess
use std::fmt::Debug;
use tracing::info;

// TODO(kos): What is usecase of parameter T?
// All calls looks similarly.
// ```
// let conf = bootstrap::<Empty, _>([]);
// ```

// FIXME(kos): redundant trailing slash after "panic".
// FIXME(kos): a snipet with example?
/// Reads AppConfig and initialise tracing.\
/// Panic if fail to read AppConfig or initialise tracing.\
/// Because of internal call to tracing_subscriber::registry().init() can't be called twice, otherwise panic.\
pub fn bootstrap<'a, T, S>(sources: S) -> Result<AppConfig<T>>
where
    S: IntoIterator<Item = ConfigSource<'a>>,
    T: Debug + DeserializeOwned,
{
    let config = AppConfig::<T>::load_from(sources)?;

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
