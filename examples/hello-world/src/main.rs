use fregate::{
    axum::{routing::get, Router},
    tokio, Application, ApplicationConfig,
};

async fn handler() -> &'static str {
    "Hello, World!"
}

#[tokio::main]
async fn main() {
    Application::new(ApplicationConfig::default())
        .router(Router::new().route("/", get(handler)))
        .serve()
        .await
        .unwrap();
}
