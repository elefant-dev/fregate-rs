use fregate::axum::routing::get;
use fregate::axum::Router;
use fregate::{init_tracing, AppConfig, Application, Empty};
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
    init_tracing();

    std::env::set_var("TEST_PORT", "3333");
    std::env::set_var("TEST_NUMBER", "1010");
    std::env::set_var("TEST_LOG_LEVEL", "debug");
    std::env::set_var("OTEL_EXPORTER_OTLP_TRACES_ENDPOINT", "http://0.0.0.0:4317");
    std::env::set_var("OTEL_SERVICE_NAME", "CONFIGURATION");

    // Only if default settings needed
    let _conf = AppConfig::default();

    // Read default, overwrite with envs, overwrite with file
    let _conf = AppConfig::builder()
        .add_default()
        .add_env_prefixed("TEST")
        .add_file("./examples/configuration/app.yaml")
        //.add_str(include_str!("../app.yaml), FileFormat::Yaml)
        .build()
        .unwrap();

    // Or most popular use cases:
    // set type for private field
    let _conf: AppConfig<Custom> =
        AppConfig::default_with("./examples/configuration/app.yaml", "TEST").unwrap();

    // if do not need private field use Empty struct from crate
    let _conf: AppConfig<Empty> =
        AppConfig::default_with("./examples/configuration/app.yaml", "TEST").unwrap();

    // Try to deserialize with private field, here you can add any number of sources
    let conf: AppConfig<Custom> = AppConfig::builder_with_private()
        .add_default()
        .add_env_prefixed("TEST")
        .build()
        .unwrap();

    Application::new(&conf)
        .rest_router(Router::new().route("/", get(handler)))
        .serve()
        .await
        .unwrap();
}
