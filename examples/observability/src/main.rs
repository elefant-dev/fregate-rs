use axum::{middleware::from_fn, routing::get, Router};
use fregate::{
    axum, bootstrap,
    middleware::{trace_request, Attributes},
    tokio, Application, Empty,
};

#[tokio::main]
async fn main() {
    let config = bootstrap::<Empty, _>([]).unwrap();
    let attributes = Attributes::new_from_config(&config);

    let rest = Router::new()
        .route("/", get(handler))
        .layer(from_fn(move |req, next| {
            trace_request(req, next, attributes.clone())
        }));

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
