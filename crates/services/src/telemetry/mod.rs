//! Telemetry and analytics

use tracing::info;

pub struct Telemetry;

impl Telemetry {
    pub fn new() -> Self {
        Self
    }

    pub fn init(&self) {
        info!("Telemetry initialized");
    }
}

impl Default for Telemetry {
    fn default() -> Self {
        Self::new()
    }
}
