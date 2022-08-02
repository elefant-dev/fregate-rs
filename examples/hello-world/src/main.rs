use axum::routing::get;
use axum::Router;

use fregate::{AlwaysHealthy, Application};

#[tokio::main]
async fn main() {
    let app = Application::builder::<AlwaysHealthy>()
        .init_tracing()
        .set_configuration_file("./src/resources/default_conf.toml")
        .set_rest_routes(Router::new().route("/", get(handler)))
        .build();

    app.run().await.unwrap();
}

async fn handler() -> &'static str {
    "Hello, World!"
}
