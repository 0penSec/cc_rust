//! /config command

use async_trait::async_trait;
use super::Command;

pub struct ConfigCommand;

#[async_trait]
impl Command for ConfigCommand {
    fn name(&self) -> &str {
        "config"
    }

    fn description(&self) -> &str {
        "Manage configuration"
    }

    async fn execute(&self, _args: &[String]) -> anyhow::Result<()> {
        println!("Configuration management");
        Ok(())
    }
}
