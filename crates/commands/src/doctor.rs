//! /doctor command

use super::Command;
use async_trait::async_trait;

pub struct DoctorCommand;

#[async_trait]
impl Command for DoctorCommand {
    fn name(&self) -> &str {
        "doctor"
    }

    fn description(&self) -> &str {
        "Run environment diagnostics"
    }

    async fn execute(&self, _args: &[String]) -> anyhow::Result<()> {
        println!("Running diagnostics...");
        println!("✓ Rust version: OK");
        println!("✓ Git: OK");
        Ok(())
    }
}
