use fregate::axum::response::IntoResponse;
use fregate::hyper::StatusCode;
use fregate::{axum, bootstrap, health::Health, Application};
use fregate::{tokio, AppConfig};
use std::sync::atomic::{AtomicU8, Ordering};
use std::sync::Arc;

#[derive(Default, Debug, Clone)]
pub struct CustomHealth {
    status: Arc<AtomicU8>,
}

#[axum::async_trait]
impl Health for CustomHealth {
    async fn alive(&self) -> axum::response::Response {
        (StatusCode::OK, "OK").into_response()
    }

    async fn ready(&self) -> axum::response::Response {
        match self.status.fetch_add(1, Ordering::SeqCst) {
            0..=2 => (StatusCode::SERVICE_UNAVAILABLE, "UNAVAILABLE").into_response(),
            _ => (StatusCode::OK, "OK").into_response(),
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
