use fregate::axum::{routing::get, Router};
use fregate::{AppConfig, Application};
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
    std::env::set_var("OTEL_EXPORTER_OTLP_TRACES_ENDPOINT", "http://0.0.0.0:4317");
    std::env::set_var("OTEL_SERVICE_NAME", "CONFIGURATION");

    /*
        // Only if default settings needed
        let _conf = AppConfig::default();

        // Read default, overwrite with envs, overwrite with file, init tracing (initialised by default)
        let _conf = AppConfig::<Empty>::builder()
            .add_default()
            .add_env_prefixed("TEST")
            .init_tracing()
            .add_file("./examples/configuration/app.yaml")
            .add_str(include_str!("../app.yaml"), FileFormat::Yaml)
            .build()
            .unwrap();

        // Try to deserialize with private field, here you can add any number of sources
        let conf: AppConfig<Custom> = AppConfig::builder()
            .add_default()
            .add_env_prefixed("TEST")
            .init_tracing()
            .build()
            .unwrap();

        // if do not need private field use Empty struct from crate
        let _conf: AppConfig<Empty> =
            AppConfig::default_with("./examples/configuration/app.yaml", "TEST").unwrap();
    */

    // Or most popular use cases:
    let conf: AppConfig<Custom> =
        AppConfig::default_with("./examples/configuration/app.yaml", "TEST").unwrap();

    Application::new(&conf)
        .router(Router::new().route("/", get(handler)))
        .serve()
        .await
        .unwrap();
}
