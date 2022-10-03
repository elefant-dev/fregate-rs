use fregate::{
    axum::{routing::get, Router},
    bootstrap, Application, Empty,
};

async fn handler() -> &'static str {
    "Hello, World!"
}

#[tokio::main]
async fn main() {
    let config = bootstrap::<Empty, _>([]).unwrap();

    Application::new(&config)
        .router(Router::new().route("/", get(handler)))
        .serve()
        .await
        .unwrap();
}
