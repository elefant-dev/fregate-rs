use fregate::axum::routing::get;
use fregate::axum::Router;
use fregate::config::{Config, Environment, File};
use fregate::{init_tracing, Application, DefaultConfig, DeserializeAndLog};

async fn handler() -> &'static str {
    "Hello, Configuration!"
}

#[tokio::main]
async fn main() {
    init_tracing();

    std::env::set_var("APP_SERVER_HOST", "0.0.0.0");
    // This one will be overwritten by default_conf.toml
    std::env::set_var("APP_SERVER_PORT", "1234");

    let config = Config::builder()
        .add_source(Environment::with_prefix("APP").separator("_"))
        .add_source(File::with_name("./src/resources/default_conf.toml"))
        .build()
        .unwrap()
        .try_deserialize_and_log::<DefaultConfig>()
        .unwrap();

    Application::new_without_health()
        .ip_addr(config.server.ip_addr)
        .port(config.server.port)
        .rest_router(Router::new().route("/", get(handler)))
        .run()
        .await
        .unwrap();
}
