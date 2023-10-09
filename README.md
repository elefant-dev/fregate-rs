# fregate-rs

Set of instruments to simplify http server set-up.

## Work in progress 
This project is in progress and might change a lot from version to version.

## Example:
```rust
use fregate::{
    axum::{routing::get, Router},
    bootstrap, tokio, AppConfig, Application,
};

async fn handler() -> &'static str {
    "Hello, World!"
}

#[tokio::main]
async fn main() {
    let config: AppConfig = bootstrap([]).unwrap();

    Application::new(config)
        .router(Router::new().route("/", get(handler)))
        .serve()
        .await
        .unwrap();
}
```

### More examples can be found [`here`](https://github.com/elefant-dev/fregate-rs/tree/main/examples).