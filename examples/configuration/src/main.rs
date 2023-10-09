use fregate::axum::{routing::get, Router};
use fregate::config::FileFormat;
use fregate::{bootstrap, tokio};
use fregate::{AppConfig, Application, ConfigSource, Empty};
use serde::Deserialize;

async fn handler() -> &'static str {
    "Hello, Configuration!"
}

#[allow(dead_code)]
#[derive(Deserialize, Debug)]
struct Custom {
    number: u32,
}

#[tokio::main]
async fn main() {
    std::env::set_var("TEST_PORT", "3333");
    std::env::set_var("TEST_NUMBER", "1010");
    std::env::set_var("TEST_LOG_LEVEL", "debug");
    std::env::set_var("TEST_TRACE_LEVEL", "debug");
    std::env::set_var("TEST_COMPONENT_NAME", "configuration");
    std::env::set_var("TEST_MANAGEMENT_ENDPOINTS_VERSION", "/give/me/version");
    std::env::set_var("TEST_COMPONENT_VERSION", "0.0.0");
    std::env::set_var("OTEL_EXPORTER_OTLP_TRACES_ENDPOINT", "http://0.0.0.0:4317");
    std::env::set_var("OTEL_SERVICE_NAME", "CONFIGURATION");

    // There are multiple ways to read AppConfig:

    // This will read AppConfig and call init_tracing() with arguments read in AppConfig
    let conf_0: AppConfig = bootstrap([
        ConfigSource::File("./examples/configuration/app.yaml"),
        ConfigSource::EnvPrefix("TEST"),
    ])
    .unwrap();

    // Read default AppConfig
    let _conf = AppConfig::<Empty>::default();

    // Set up AppConfig through builder, nothing added by default
    let _conf = AppConfig::<Empty>::builder()
        .add_default()
        .add_env_prefixed("TEST")
        .add_file("./examples/configuration/app.yaml")
        .add_str(include_str!("../app.yaml"), FileFormat::Yaml)
        .build()
        .unwrap();

    // Read default config with private field struct Custom with specified file and environment variables with specified prefix and "_" separator
    let _conf: AppConfig<Custom> =
        AppConfig::default_with("./examples/configuration/app.yaml", "TEST").unwrap();

    Application::new(conf_0)
        .router(Router::new().route("/", get(handler)))
        .serve()
        .await
        .unwrap();
}

// Check version with:
// curl http://localhost:3333/configuration/give/me/version
