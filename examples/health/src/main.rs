use fregate::hyper::StatusCode;
use fregate::{axum, bootstrap, health::HealthExt, Application};
use fregate::{tokio, AppConfig};
use std::sync::atomic::{AtomicU8, Ordering};
use std::sync::Arc;

#[derive(Default, Debug, Clone)]
pub struct CustomHealth {
    status: Arc<AtomicU8>,
}

#[axum::async_trait]
impl HealthExt for CustomHealth {
    type HealthResponse = (StatusCode, &'static str);
    type ReadyResponse = StatusCode;

    async fn alive(&self) -> Self::HealthResponse {
        (StatusCode::OK, "OK")
    }

    async fn ready(&self) -> Self::ReadyResponse {
        match self.status.fetch_add(1, Ordering::SeqCst) {
            0..=2 => StatusCode::SERVICE_UNAVAILABLE,
            _ => StatusCode::OK,
        }
    }
}

#[tokio::main]
async fn main() {
    let config: AppConfig = bootstrap([]).unwrap();

    Application::new(config)
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
