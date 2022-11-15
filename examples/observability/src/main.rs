use axum::{routing::get, Router};
use fregate::{axum, bootstrap, tokio, Application, Empty};

#[tokio::main]
async fn main() {
    let config = bootstrap::<Empty, _>([]).unwrap();

    let rest = Router::new().route("/", get(handler));

    Application::new(&config)
        .router(rest)
        .serve()
        .await
        .unwrap();
}

async fn handler() -> &'static str {
    "Hello, World!"
}

/*
    curl http://0.0.0.0:8000
*/
