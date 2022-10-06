use config::{
    builder::{AsyncState, DefaultState},
    ConfigBuilder, Environment, File, FileFormat,
};
use sealed::sealed;

const DEFAULT_SEPARATOR: &str = "_";
const DEFAULT_CONFIG: &str = include_str!("../resources/default_conf.toml");

/// Extends [`ConfigBuilder`]
#[sealed]
pub trait ConfigExt {
    /// Add prefixed environment variables to [`ConfigBuilder`] with default separator
    fn add_env_prefixed(self, prefix: &str) -> Self;

    /// Add fregate default config to [`ConfigBuilder`]
    fn add_fregate_defaults(self) -> Self;
}

#[sealed]
impl ConfigExt for ConfigBuilder<DefaultState> {
    fn add_env_prefixed(self, prefix: &str) -> Self {
        self.add_source(
            Environment::with_prefix(prefix)
                .try_parsing(true)
                .separator(DEFAULT_SEPARATOR),
        )
    }

    fn add_fregate_defaults(self) -> Self {
        self.add_source(File::from_str(DEFAULT_CONFIG, FileFormat::Toml))
            .add_env_prefixed("OTEL")
    }
}

#[sealed]
impl ConfigExt for ConfigBuilder<AsyncState> {
    fn add_env_prefixed(self, prefix: &str) -> Self {
        self.add_source(
            Environment::with_prefix(prefix)
                .try_parsing(true)
                .separator(DEFAULT_SEPARATOR),
        )
    }

    fn add_fregate_defaults(self) -> Self {
        self.add_source(File::from_str(DEFAULT_CONFIG, FileFormat::Toml))
            .add_env_prefixed("OTEL")
    }
}
