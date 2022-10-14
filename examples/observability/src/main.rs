use fregate::{
    axum::{middleware::from_fn, routing::get, Router},
    bootstrap,
    middleware::trace_request,
    tokio, Application, Empty,
};

#[tokio::main]
async fn main() {
    let config = bootstrap::<Empty, _>([]).unwrap();
    let conf = config.clone();

    let rest = Router::new()
        .route("/", get(handler))
        .layer(from_fn(move |req, next| {
            trace_request(req, next, conf.clone())
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
