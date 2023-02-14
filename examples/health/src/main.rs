use fregate::{
    axum, bootstrap,
    health::{Health, HealthResponse},
    Application,
};
use fregate::{tokio, AppConfig};
use std::sync::atomic::{AtomicU8, Ordering};
use std::sync::Arc;

#[derive(Default, Debug, Clone)]
pub struct CustomHealth {
    status: Arc<AtomicU8>,
}

#[axum::async_trait]
impl Health for CustomHealth {
    async fn alive(&self) -> HealthResponse {
        match self.status.fetch_add(1, Ordering::SeqCst) {
            0..=2 => HealthResponse::OK,
            _ => HealthResponse::UNAVAILABLE,
        }
    }

    async fn ready(&self) -> HealthResponse {
        match self.status.load(Ordering::SeqCst) {
            0..=3 => HealthResponse::OK,
            _ => HealthResponse::UNAVAILABLE,
        }
    }
}

#[tokio::main]
async fn main() {
    let config: AppConfig = bootstrap([]).unwrap();

    Application::new(&config)
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
