use fregate::axum::{routing::get, Router};
use fregate::config::{File, FileFormat};
use fregate::extensions::ConfigExt;
use fregate::logging::init_tracing_from_config;
use fregate::tracing::info;
use fregate::{config_builder, load_default_config_with, Application, TracingConfig};
use fregate::{tokio, ApplicationConfig};
use serde::Deserialize;

#[derive(Deserialize, Debug)]
pub struct CustomConfig {
    pub number: u32,
    #[serde(flatten)]
    pub app_config: ApplicationConfig,
    #[serde(flatten)]
    pub tracing_config: TracingConfig,
}

async fn handler() -> &'static str {
    "Hello, Configuration!"
}

#[tokio::main]
async fn main() {
    std::env::set_var("TEST_PORT", "3333");
    std::env::set_var("TEST_NUMBER", "1010");
    std::env::set_var("TEST_LOG_LEVEL", "debug");
    std::env::set_var("TEST_TRACE_LEVEL", "debug");
    std::env::set_var("OTEL_EXPORTER_OTLP_TRACES_ENDPOINT", "http://0.0.0.0:4317");
    std::env::set_var("OTEL_SERVICE_NAME", "CONFIGURATION");

    // This will load default fregate config and File + Env Vars on top of it
    let _custom_config: CustomConfig =
        load_default_config_with(Some("./examples/configuration/app.yaml"), Some("TEST")).unwrap();

    // For more flexibility you may use builder
    let custom_config: CustomConfig = config_builder()
        .add_fregate_defaults()
        .add_source(File::with_name("./examples/configuration/app.yaml"))
        .add_source(File::from_str(
            include_str!("../../configuration/app.yaml"),
            FileFormat::Yaml,
        ))
        .add_env_prefixed("TEST")
        .add_env_prefixed("OTEL")
        .build()
        .unwrap()
        .try_deserialize()
        .unwrap();

    let app_config = custom_config.app_config.clone();
    let tracing_config = custom_config.tracing_config.clone();

    init_tracing_from_config(tracing_config).unwrap();
    info!("Loaded {custom_config:?}");

    Application::new(app_config)
        .router(Router::new().route("/", get(handler)))
        .serve()
        .await
        .unwrap();
}
