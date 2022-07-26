use config::Config;

pub mod conf;

#[derive(Default, Deserialize)]
pub struct EmptyConf;

pub struct AppConf<T = EmptyConf> {
    conf: T,
}

#[derive(Default)]
pub struct AppBuilder {
    conf_prefix: Option<String>,
    conf_file: Option<String>,
}

impl AppBuilder {
    fn conf_prefix(mut self, prefix: impl Into<String>) -> Self {
        self.conf_prefix = Some(prefix.into());
        self
    }

    fn conf_file(mut self, file: impl Into<String>) -> Self {
        self.conf_file = Some(file.into());
        self
    }

    fn conf(&self) -> AppConf {
        let mut builder = Config::builder();

        // Conf file
        if let Some(file) = self.conf_file.as_ref() {
            builder = builder.add_source(config::File::with_name(file.as_str()))
        }

        // Env variables
        if let Some(prefix) = self.conf_prefix.as_ref() {
            builder = builder.add_source(config::Environment::with_prefix(prefix).separator("_"));
        }

        let conf = builder.build().map(|config| config.get::<AppConf>("a'"));

        AppConf {
            conf: EmptyConf::default(),
        }
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
