use fregate::{axum, AppConfig, Application, ApplicationStatus, Health};
use std::sync::atomic::{AtomicU8, Ordering};
use std::sync::Arc;

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
    Application::new(&AppConfig::default())
        .health_indicator(CustomHealth::default())
        .serve()
        .await
        .unwrap();
}

/*
    curl http://0.0.0.0:8000/health
    curl http://0.0.0.0:8000/live
    curl http://0.0.0.0:8000/ready
*/
