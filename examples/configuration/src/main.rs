use axum::routing::get;
use axum::Router;

use fregate::{Application, DefaultHealth, Environment};

#[tokio::main]
async fn main() {
    std::env::set_var("APP_SERVER_HOST", "0.0.0.0");
    std::env::set_var("APP_SERVER_PORT", "8005");

    let app = Application::builder::<DefaultHealth>()
        .init_tracing()
        .set_configuration_environment(Environment::Prefixed("APP"))
        .set_rest_routes(Router::new().route("/", get(handler)))
        .build();

    app.run().await.unwrap();
}

async fn handler() -> &'static str {
    "Hello, Configuration!"
}
