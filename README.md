# fregate-rs

When developing a new server it usually requires to write a lot of boilerplate code for logging, configuration set up, metrics, health checks etc.
This crate aims to solve these problems providing user with `ApplicationBuilder` struct with multiple configurations for setting up http or/and grpc service.

This crate relies on multiple external crates:
`TODO list of external crates`

## Usage example
```rust
#[tokio::main]
async fn main() {
    let app = Application::builder::<DefaultHealth>()
        .init_tracing()
        .set_configuration_file("./src/resources/default_conf.toml")
        .set_rest_routes(Router::new().route("/", get(handler)))
        .build();

    app.run().await.unwrap();
}

async fn handler() -> &'static str {
    "Hello, World!"
}
```