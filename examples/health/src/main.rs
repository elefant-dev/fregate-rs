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
    init_tracing();

    let health = CustomHealth::default();
    let config = AppConfig::default();

    Application::new_with_health(config)
        .health_indicator(health)
        .serve()
        .await
        .unwrap();
}

/*
    curl http://0.0.0.0:8000/health/alive
    curl http://0.0.0.0:8000/health/ready
*/
