//! Trait to implement custom version response
use crate::AppConfig;
use axum::response::IntoResponse;

/// Trait to implement custom /version response
pub trait VersionExt<PrivateCfg>: Send + Sync + 'static + Clone {
    /// return type for health check
    type Response: IntoResponse;

    /// returns [`Self::Response`] in response to configured endpoint. Default: `/version`.
    /// For more information see [`crate::configuration::ManagementConfig`]
    fn get_version(&self, cfg: &AppConfig<PrivateCfg>) -> Self::Response;
}

/// Returns plain text version of service.
#[derive(Default, Debug, Clone, Copy)]
pub struct DefaultVersion;

impl<T> VersionExt<T> for DefaultVersion {
    type Response = String;

    fn get_version(&self, cfg: &AppConfig<T>) -> Self::Response {
        cfg.observability_cfg.version.clone()
    }
}
