//! /commit command

use async_trait::async_trait;
use super::Command;

pub struct CommitCommand;

#[async_trait]
impl Command for CommitCommand {
    fn name(&self) -> &str {
        "commit"
    }

    fn description(&self) -> &str {
        "Create a git commit"
    }

    async fn execute(&self, _args: &[String]) -> anyhow::Result<()> {
        println!("Creating commit...");
        Ok(())
    }
}
