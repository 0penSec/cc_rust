//! Slash command implementations
//!
//! All user-facing slash commands like /commit, /cost, /config, etc.

pub mod commit;
pub mod config;
pub mod cost;
pub mod doctor;

use async_trait::async_trait;

/// A slash command
#[async_trait]
pub trait Command: Send + Sync {
    fn name(&self) -> &str;
    fn description(&self) -> &str;
    async fn execute(&self, args: &[String]) -> anyhow::Result<()>;
}
