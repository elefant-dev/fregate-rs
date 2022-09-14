use fregate::{
    axum::{routing::get, Router},
    http_trace_layer, AppConfig, Application,
};

#[tokio::main]
async fn main() {
    let rest = Router::new()
        .route("/", get(handler))
        .layer(http_trace_layer());

    Application::new(&AppConfig::default())
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
