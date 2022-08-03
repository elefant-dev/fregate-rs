# fregate-rs

When developing a new server it usually requires to write a lot of boilerplate code for logging, configuration set up, metrics, health checks etc.
This crate aims to solve these problems providing user with `ApplicationBuilder` struct with multiple configurations for setting up http or/and grpc service.

This crate relies on multiple external crates:
`TODO list of external crates`

## Usage example
```rust
#[tokio::main]
async fn main() {
    init_tracing();

    Application::new_with_health(Arc::new(AlwaysHealthy::default()))
        .rest_router(Router::new().route("/", get(handler)))
        .run()
        .await
        .unwrap();
}

async fn handler() -> &'static str {
    "Hello, World!"
}
```