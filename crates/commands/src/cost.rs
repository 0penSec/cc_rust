//! /cost command

use super::Command;
use async_trait::async_trait;

pub struct CostCommand;

#[async_trait]
impl Command for CostCommand {
    fn name(&self) -> &str {
        "cost"
    }

    fn description(&self) -> &str {
        "Show usage cost"
    }

    async fn execute(&self, _args: &[String]) -> anyhow::Result<()> {
        println!("Current session cost: $0.00");
        Ok(())
    }
}
