use serde::Serialize;
use std::fmt;
use std::fmt::{Display, Formatter};

#[async_trait::async_trait]
pub trait Health: Default + Send + Sync + 'static {
    async fn check(&self) -> HealthStatus;
}

#[derive(Serialize, Debug, Clone, Copy)]
pub enum HealthStatus {
    UP,
    DOWN,
}

impl Default for HealthStatus {
    fn default() -> Self {
        HealthStatus::DOWN
    }
}

impl Display for HealthStatus {
    fn fmt(&self, f: &mut Formatter) -> fmt::Result {
        write!(f, "{:?}", self)
    }
}

#[derive(Default)]
pub struct AlwaysHealthy {}

#[async_trait::async_trait]
impl Health for AlwaysHealthy {
    async fn check(&self) -> HealthStatus {
        HealthStatus::UP
    }
}
