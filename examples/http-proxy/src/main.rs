use fregate::axum::routing::any;
use fregate::axum::{routing::get, Router};
use fregate::{http_trace_layer, init_tracing, route_proxy, AppConfig, Application};

#[tokio::main]
async fn main() {
    init_tracing();

    std::env::set_var("APP_PROXY_HOST", "127.0.0.1");
    std::env::set_var("APP_PROXY_PORT", "3000");

    // Start server where to proxy requests
    tokio::spawn(server());

    // set up your server Routers
    let hello = Router::new().route("/hello", get(|| async { "Hello" }));
    let world = Router::new().route("/world", get(|| async { "World" }));
    let app = Router::new().nest("/app", hello.merge(world));

    // add proxy_router, if no app Router is matched check path in fallback and redirect request
    let with_proxy = app
        .fallback(route_proxy("/proxy_server/*path", "http://127.0.0.1:3000"))
        .layer(http_trace_layer());

    // Or you might want add path to always redirect to another service:
    // let with_proxy = app
    //     .merge(route_proxy("/proxy_server/*path", "http://127.0.0.1:3000"))
    //     .layer(http_trace_layer());

    Application::new(&AppConfig::default())
        .rest_router(with_proxy)
        .serve()
        .await
        .unwrap();
}

async fn server() {
    let config = AppConfig::builder()
        .add_env_prefixed("APP_PROXY")
        .build()
        .unwrap();

    let app = Router::new().route("/proxy_server/*path", any(|| async { "Hello, Proxy!" }));

    Application::new(&config)
        .rest_router(app)
        .serve()
        .await
        .unwrap();
}

/*
will be handled by app:
    curl http://0.0.0.0:8000/app/hello
    curl http://0.0.0.0:8000/app/world
will be redirected to another service:
    curl http://0.0.0.0:8000/proxy_server/hello
will not match any Router and won't be proxied:
    curl http://0.0.0.0:8000/app/proxy_server/hello
    curl http://0.0.0.0:8000/anything
*/
