use fregate::axum::routing::get;
use fregate::axum::Router;
use fregate::config::{Config, Environment};
use fregate::{init_tracing, Application, DeserializeAndLog};
use serde::Deserialize;
use std::net::SocketAddr;

async fn handler() -> &'static str {
    "Hello, Configuration!"
}

#[derive(Deserialize, Debug)]
struct CustomConfig {
    server: ServerConfig,
}

#[derive(Deserialize, Debug)]
struct ServerConfig {
    socket: SocketAddr,
}

#[tokio::main]
async fn main() {
    init_tracing();

    std::env::set_var("APP_SERVER_SOCKET", "0.0.0.0:9998");

    // You might want to read DefaultConfig providing only path to default conf file and environment
    // variables prefix, separator is "_" by default.
    // This function will read file and overwrite it with given in environment variables.
    // let config = read_default_config("./src/resources/default_conf.toml", "APP").unwrap();

    // For more configuration you might want to do something like this:
    let config = Config::builder()
        .add_source(Environment::with_prefix("APP").separator("_"))
        .build()
        .unwrap()
        .try_deserialize_and_log::<CustomConfig>()
        .unwrap();

    Application::new_without_health()
        .host(config.server.socket.ip())
        .port(config.server.socket.port())
        .management_port(9999u16)
        .rest_router(Router::new().route("/", get(handler)))
        .run()
        .await
        .unwrap();
}
