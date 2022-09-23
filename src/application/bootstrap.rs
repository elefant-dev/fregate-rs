use crate::*;
use serde::de::DeserializeOwned;
use std::fmt::Debug;
use tracing::info;

/// Reads AppConfig and initialise tracing.\
/// Panic if fail to read AppConfig or initialise tracing.\
/// Because of internal call to tracing_subscriber::registry().init() can't be called twice, otherwise panic.\
pub fn bootstrap<'a, T, S>(sources: S) -> AppConfig<T>
where
    S: IntoIterator<Item = ConfigSource<'a>>,
    T: Debug + DeserializeOwned,
{
    let config = AppConfig::<T>::load_from(sources).expect("Failed to load AppConfig");

    let LoggerConfig {
        log_level,
        trace_level,
        service_name,
        traces_endpoint,
        metrics_endpoint,
    } = &config.logger;

    init_tracing(
        log_level,
        trace_level,
        service_name,
        traces_endpoint.as_deref(),
    );

    init_metrics(metrics_endpoint);

    info!("Configuration: `{config:?}`.", config = config);

    config
}
