# fregate-rs

Developing a server requires to write code for logging, configuration, metrics, health checks etc.
This crate aims to solve these problems providing user with `Application` builder for setting up http or/and grpc service.

## Work in progress 
This project in progress and might change a lot from version to version.

## Usage example
```rust
async fn handler() -> &'static str {
    "Hello, World!"
}

#[tokio::main]
async fn main() {
    Application::new(&AppConfig::default())
        .router(Router::new().route("/", get(handler)))
        .serve()
        .await
        .unwrap();
}
```
