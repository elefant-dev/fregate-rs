use config::FileFormat;

/// Enum to specify configuration source type:
#[derive(Clone, Debug)]
pub enum ConfigSource<'a> {
    /// Load from string
    String(&'a str, FileFormat),
    /// Read file by given path
    File(&'a str),
    /// Read environment variables with specified prefix
    EnvPrefix(&'a str),
}
