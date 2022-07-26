use axum::routing::get;
use axum::Router;

use fregate::application::Application;

#[tokio::main]
async fn main() {
    let app = Application::builder()
        .rest_router(Router::new().route("/", get(handler)))
        .build();

    app.run().await.unwrap();
}

async fn handler() -> &'static str {
    "Hello, World!"
}
