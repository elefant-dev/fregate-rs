# fregate-rs

Developing an HTTP server requires to add code for logging, configuration, metrics, health checks etc.
This crate aims to solve these problems providing user with `Application` builder for setting up HTTP service.

## Work in progress 
This project is in progress and might change a lot from version to version.

## Usage example
```rust
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
```
