use crate::*;
use serde::de::DeserializeOwned;
// TODO(kos): redundant usess
use std::fmt::Debug;
use tracing::info;

// FIXME(kos): For better navigation in code docs, use intra-doc links like:
//             ```rust
//             /// Reads [`AppConfig`] and initializes [`tracing`].
//             ///
//             /// # Panics
//             ///
//             /// - If fails to read [`AppConfig`] or to initialize [`tracing`].
//             /// - If called twice (because of an internal call to
//             ///   `tracing_subscriber::registry().init()`).
//             ```

// TODO(kos): What is the use-case of the parameter `T`?
//            All calls looks similarly.
//            ```rust
//            let conf = bootstrap::<Empty, _>([]);
//            ```
//            Parameter `T` here is the custom-defined part of the `AppConfig`.
//            It's named too poorly, so is quite unobvious.
//            I suggested to remove it, but if not then consider to rename it.

// FIXME(kos): Redundant trailing slash after "panic".
// FIXME(kos): A snippet as an example?

/// Reads AppConfig and initialise tracing.\
/// Return Error if fails to read AppConfig or initialise tracing.\
/// Return Error if called twice because of internal call to tracing_subscriber::registry().try_init().\
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
