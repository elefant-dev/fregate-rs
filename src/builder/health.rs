use serde::Serialize;
use std::fmt;
use std::fmt::{Display, Formatter};

pub trait Health: Default + Send + Sync + 'static {
    fn check(&self) -> HealthStatus;
}

#[derive(Serialize, Debug, Clone, Copy)]
pub enum HealthStatus {
    Up,
    Down,
}

impl Default for HealthStatus {
    fn default() -> Self {
        HealthStatus::Down
    }
}

impl Display for HealthStatus {
    fn fmt(&self, f: &mut Formatter) -> fmt::Result {
        write!(f, "{:?}", self)
    }
}

#[derive(Default)]
pub struct DefaultHealth {}

impl Health for DefaultHealth {
    fn check(&self) -> HealthStatus {
        HealthStatus::Up
    }
}
