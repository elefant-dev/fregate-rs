use fregate::{
    axum::{routing::get, Router},
    bootstrap,
    middleware::http_trace_layer,
    tokio, Application, Empty,
};

#[tokio::main]
async fn main() {
    let config = bootstrap::<Empty, _>([]).unwrap();

    let rest = Router::new()
        .route("/", get(handler))
        .layer(http_trace_layer());

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
