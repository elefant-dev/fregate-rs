use core::fmt;
use std::sync::Arc;

use serde::Serialize;
use serde_with::skip_serializing_none;

#[derive(Serialize, Debug)]
#[skip_serializing_none]
pub enum HealthStatus {
    DOWN(Option<&'static str>),
    UP(Option<&'static str>),
}

impl Default for HealthStatus {
    fn default() -> Self {
        HealthStatus::UP(None)
    }
}

impl fmt::Display for HealthStatus {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(f, "{:?}", self)
    }
}

pub trait HealthIndicator {
    fn health(&self) -> HealthStatus;

    fn live(&self) -> bool {
        matches!(self.health(), HealthStatus::UP(_))
    }

    fn ready(&self) -> bool {
        matches!(self.health(), HealthStatus::UP(_))
    }
}

pub type HealthIndicatorRef = Arc<dyn HealthIndicator + Send + Sync>;

#[derive(Default)]
pub struct UpHealth {}

impl HealthIndicator for UpHealth {
    fn health(&self) -> HealthStatus {
        HealthStatus::UP(Some("OK"))
    }
}
