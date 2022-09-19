use fregate::{
    axum::{routing::get, Router},
    bootstrap, http_trace_layer, Application, Empty,
};

#[tokio::main]
async fn main() {
    let config = bootstrap::<Empty, _>([], None);

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
