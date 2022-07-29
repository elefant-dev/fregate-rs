use serde::Serialize;
use std::fmt;
use std::fmt::{Display, Formatter};

pub trait Health: Default + Send + Sync + 'static {
    fn check(&self) -> HealthStatus;
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
pub struct DefaultHealth {}

impl Health for DefaultHealth {
    fn check(&self) -> HealthStatus {
        HealthStatus::UP
    }
}
