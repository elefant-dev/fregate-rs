use std::sync::atomic::{AtomicU8, Ordering};
use std::sync::Arc;

use fregate::{axum, init_tracing, Application, Health, HealthStatus};

#[derive(Default, Debug)]
pub struct CustomHealth {
    status: AtomicU8,
}

#[axum::async_trait]
impl Health for CustomHealth {
    async fn check(&self) -> HealthStatus {
        match self.status.fetch_add(1, Ordering::SeqCst) {
            0..=2 => HealthStatus::DOWN,
            _ => HealthStatus::UP,
        }
    }
}

#[tokio::main]
async fn main() {
    init_tracing();
    let health = Arc::new(CustomHealth::default());
    Application::new_with_health(health).run().await.unwrap();
}
