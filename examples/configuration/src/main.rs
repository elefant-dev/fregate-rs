use axum::routing::get;
use axum::Router;

use fregate::application::{Application, ApplicationConfigurationEnvironment};

#[tokio::main]
async fn main() {
    std::env::set_var("APP_TRANSPORT_ADDRESS", "::1");

    let builder = Application::builder()
        .telemetry(true)
        .configuration_environment(ApplicationConfigurationEnvironment::Prefix("APP"))
        .configuration_file("examples/configuration/app");

    let _s = builder.conf().get_string("aaa");

    let app = builder
        .rest_router(Router::new().route("/", get(handler)))
        .build();

    app.run().await.unwrap();
}

async fn handler() -> &'static str {
    "Hello, Configuration!"
}
