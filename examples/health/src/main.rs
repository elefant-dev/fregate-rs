use std::sync::atomic::{AtomicU8, Ordering};
use std::sync::Arc;

use fregate::{Application, Health, HealthStatus};

#[derive(Default, Debug)]
pub struct CustomHealth {
    status: AtomicU8,
}

impl Health for CustomHealth {
    fn check(&self) -> HealthStatus {
        match self.status.fetch_add(1, Ordering::SeqCst) {
            0..=2 => HealthStatus::Down,
            _ => HealthStatus::Up,
        }
    }
}

#[tokio::main]
async fn main() {
    let health = Arc::new(CustomHealth::default());

    let app = Application::builder()
        .init_tracing()
        .set_configuration_file("./src/resources/default_conf.toml")
        .set_health_indicator(health)
        .build();

    app.run().await.unwrap();
}
