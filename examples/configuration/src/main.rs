use fregate::axum::{routing::get, Router};
use fregate::config::FileFormat;
use fregate::{bootstrap, AppConfig, Application, ConfigSource, Empty};
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

    // FIXME(kos): That's not a practical example.
    //             It's better to rather provide a set of smoke tests.
    //             Alternatively, all these variants of usage should be in
    //             documentation of `AppConfig` as separate examples.
    //             Consider moving and splitting.

    // This will initialise tracing from environment variables read to AppConfig.
    let _conf: AppConfig<Empty> = bootstrap([
        ConfigSource::File("./examples/configuration/app.yaml"),
        ConfigSource::EnvPrefix("TEST"),
    ])
    .unwrap();

    let _conf = AppConfig::default();

    let _conf = AppConfig::<Empty>::builder()
        .add_default()
        .add_env_prefixed("TEST")
        .add_file("./examples/configuration/app.yaml")
        .add_str(include_str!("../app.yaml"), FileFormat::Yaml)
        .build()
        .unwrap();

    let _conf: AppConfig<Custom> =
        AppConfig::default_with("./examples/configuration/app.yaml", "TEST").unwrap();

    Application::new(&AppConfig::default())
        .router(Router::new().route("/", get(handler)))
        .serve()
        .await
        .unwrap();
}
