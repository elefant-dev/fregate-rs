# fregate-rs

Developing a server requires to write code for logging, configuration, metrics, health checks etc.
This crate aims to solve these problems providing user with `Application` builder for setting up http or/and grpc service.

## Work in progress 
This project in progress and might change a lot from version to version.

## Usage example
```rust
#[tokio::main]
async fn main() {
    init_tracing();

    Application::new_with_health(AlwaysReadyAndAlive::default())
        .rest_router(Router::new().route("/", get(handler)))
        .run()
        .await
        .unwrap();
}

async fn handler() -> &'static str {
    "Hello, World!"
}
```
