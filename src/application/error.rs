//! fregate Errors
use config::ConfigError;
use hyper::Error as HyperError;
use metrics::SetRecorderError;
use opentelemetry::trace::TraceError;
use tracing_subscriber::util::TryInitError;

/// Possible Errors which might occur in fregate
#[derive(thiserror::Error, Debug)]
pub enum Error {
    /// Error returned when AppConfigBuilder fails to build configuration
    #[error("Got ConfigError: `{0}`")]
    ConfigError(#[from] ConfigError),
    /// Error returned on init_tracing()
    #[error("Got TraceError: `{0}`")]
    TraceError(#[from] TraceError),
    /// Error returned on init_tracing()
    #[error("Got TryInitError: `{0}`")]
    TryInitError(#[from] TryInitError),
    /// Error returned on Application::serve()
    #[error("Got HyperError: `{0}`")]
    HyperError(#[from] HyperError),
    /// Error returned on init_metrics()
    #[error("Got SetRecorderError: `{0}`")]
    SetRecorderError(#[from] SetRecorderError),
    /// Error returned by serde_json crate
    #[error("Got SerdeError: `{0}`")]
    SerdeError(#[from] serde_json::Error),
    /// Custom fregate Error
    #[error("Got CustomError: `{0}`")]
    CustomError(String),
    /// Some std IO Error
    #[error("Got IoError: `{0}`")]
    IoError(#[from] std::io::Error),

    /// Error returned by native-tls
    #[cfg(feature = "native-tls")]
    #[error("Got NativeTlsError: `{0}`")]
    NativeTlsError(#[from] tonic_native_tls::native_tls::Error),
}

/// fregate Result alias
pub type Result<T> = std::result::Result<T, Error>;
