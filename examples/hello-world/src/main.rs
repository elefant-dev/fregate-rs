use fregate::{
    axum::{routing::get, Router},
    tokio, Application, ApplicationConfig,
};

async fn handler() -> &'static str {
    "Hello, World!"
}

#[tokio::main]
async fn main() {
    let config = ApplicationConfig::default();

    Application::new(config)
        .router(Router::new().route("/", get(handler)))
        .serve()
        .await
        .unwrap();
}
