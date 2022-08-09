use fregate::axum::routing::get;
use fregate::axum::Router;
use fregate::{init_tracing, AppConfig, Application};
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

    std::env::set_var("APP_SERVER_PORT", "3333");
    std::env::set_var("APP_PRIVATE_NUMBER", "1010");

    // Only if default settings needed
    let _conf = AppConfig::default();

    // Read default, overwrite with envs, overwrite with file
    let _conf = AppConfig::builder()
        .add_default()
        .add_env_prefixed("APP")
        .add_file("./examples/configuration/app.yaml")
        .build()
        .unwrap();

    // Or most popular use case:
    let _conf = AppConfig::default_with("./examples/configuration/app.yaml", "APP").unwrap();

    // Try to deserialize with private field
    let conf: AppConfig<Custom> = AppConfig::builder_with_private()
        .add_default()
        .add_env_prefixed("APP")
        .build()
        .unwrap();

    Application::new(conf)
        .rest_router(Router::new().route("/", get(handler)))
        .serve()
        .await
        .unwrap();
}
