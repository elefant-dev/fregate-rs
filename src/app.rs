use config::Config;
use serde::Deserialize;

pub mod conf;

#[derive(Default, Deserialize)]
pub struct EmptyConf;

#[derive(Deserialize)]
pub struct AppConf<T = EmptyConf> {
    _conf: T,
}

#[derive(Default)]
pub struct AppBuilder {
    _conf_prefix: Option<String>,
    _conf_file: Option<String>,
}

impl AppBuilder {
    fn _conf_prefix(mut self, prefix: impl Into<String>) -> Self {
        self._conf_prefix = Some(prefix.into());
        self
    }

    fn _conf_file(mut self, file: impl Into<String>) -> Self {
        self._conf_file = Some(file.into());
        self
    }

    fn _conf(&self) -> AppConf {
        let mut builder = Config::builder();

        // Conf file
        if let Some(file) = self._conf_file.as_ref() {
            builder = builder.add_source(config::File::with_name(file.as_str()))
        }

        // Env variables
        if let Some(prefix) = self._conf_prefix.as_ref() {
            builder = builder.add_source(config::Environment::with_prefix(prefix).separator("_"));
        }

        builder
            .build()
            .map(|config| config.get::<AppConf>("a'"))
            .expect("Failed to build config")
            .expect("Failed to get Application config")
    }
}

// impl Default for AppBuilder {
//     fn default() -> Self {
//         Self {
//             conf_prefix: None,
//             conf_file: None,
//         }
//     }
// }
