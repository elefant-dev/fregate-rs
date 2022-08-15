use std::sync::atomic::{AtomicU8, Ordering};
use std::sync::Arc;

use fregate::{axum, init_tracing, AppConfig, Application, ApplicationStatus, Health};

#[derive(Default, Debug, Clone)]
pub struct CustomHealth {
    status: Arc<AtomicU8>,
}

#[axum::async_trait]
impl Health for CustomHealth {
    async fn alive(&self) -> ApplicationStatus {
        match self.status.fetch_add(1, Ordering::SeqCst) {
            0..=2 => ApplicationStatus::DOWN,
            _ => ApplicationStatus::UP,
        }
    }

    async fn ready(&self) -> ApplicationStatus {
        match self.status.load(Ordering::SeqCst) {
            0..=3 => ApplicationStatus::DOWN,
            _ => ApplicationStatus::UP,
        }
    }
}

#[tokio::main]
async fn main() {
    std::env::set_var("APP_PORT", "3333");

    init_tracing();

    let health = CustomHealth::default();
    let conf1 = AppConfig::default();
    let conf2 = AppConfig::builder()
        .add_default()
        .add_env_prefixed("APP")
        .build()
        .unwrap();

    // this will always ready and healthy
    let always_ready = Application::new(&conf1).serve();

    // this will use custom health
    let custom_health = Application::new(&conf2).health_indicator(health).serve();

    tokio::try_join!(always_ready, custom_health).unwrap();
}

/*
    curl http://0.0.0.0:8000/health
    curl http://0.0.0.0:8000/live
    curl http://0.0.0.0:8000/ready
    curl http://0.0.0.0:3333/health
    curl http://0.0.0.0:3333/live
    curl http://0.0.0.0:3333/ready
*/
