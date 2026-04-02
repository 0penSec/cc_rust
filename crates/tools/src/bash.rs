use async_trait::async_trait;
use schemars::{schema_for, JsonSchema};
use serde::{Deserialize, Serialize};
use serde_json::json;
use std::process::Stdio;
use tokio::process::Command;
use tokio::time::{timeout, Duration};
use tracing::{debug, warn};

use claude_core::{
    ClaudeResult, Tool, ToolContext, ToolInput, ToolOutput, ToolResult, PermissionMode,
};

/// Bash tool for executing shell commands
pub struct BashTool {
    timeout_seconds: u64,
}

impl BashTool {
    pub fn new() -> Self {
        Self {
            timeout_seconds: 300,
        }
    }

    pub fn with_timeout(mut self, seconds: u64) -> Self {
        self.timeout_seconds = seconds;
        self
    }
}

impl Default for BashTool {
    fn default() -> Self {
        Self::new()
    }
}

#[derive(Debug, Serialize, Deserialize, JsonSchema)]
struct BashInput {
    /// The shell command to execute
    command: String,
    /// Working directory for the command
    #[serde(default)]
    cwd: Option<String>,
    /// Environment variables
    #[serde(default)]
    env: Option<std::collections::HashMap<String, String>>,
    /// Timeout in seconds (default: 300)
    #[serde(default)]
    timeout: Option<u64>,
}

#[derive(Debug, Serialize, Deserialize)]
struct BashOutput {
    stdout: String,
    stderr: String,
    exit_code: i32,
}

#[async_trait]
impl Tool for BashTool {
    fn name(&self) -> &str {
        "bash"
    }

    fn description(&self) -> &str {
        "Execute a bash shell command. Use for running commands, scripts, and tools. \
         Returns stdout, stderr, and exit code. Commands run with a 5-minute timeout by default."
    }

    fn input_schema(&self) -> serde_json::Value {
        serde_json::to_value(schema_for!(BashInput)).unwrap()
    }

    fn permission_mode(&self) -> PermissionMode {
        PermissionMode::Ask
    }

    async fn execute(&self, input: ToolInput, ctx: &ToolContext) -> ToolResult {
        let input: BashInput = input.parse()?;
        let timeout_seconds = input.timeout.unwrap_or(self.timeout_seconds);

        debug!("Executing bash command: {}", input.command);

        let cwd = input
            .cwd
            .map(|p| std::path::PathBuf::from(p))
            .unwrap_or_else(|| ctx.working_directory.clone());

        let mut cmd = Command::new("bash");
        cmd.arg("-c")
            .arg(&input.command)
            .current_dir(&cwd)
            .stdout(Stdio::piped())
            .stderr(Stdio::piped())
            .stdin(Stdio::null());

        // Set environment variables
        cmd.env_clear();
        for (k, v) in &ctx.env_vars {
            cmd.env(k, v);
        }
        if let Some(env) = input.env {
            for (k, v) in env {
                cmd.env(k, v);
            }
        }

        // Execute with timeout
        let result = timeout(Duration::from_secs(timeout_seconds), cmd.output()).await;

        match result {
            Ok(Ok(output)) => {
                let stdout = String::from_utf8_lossy(&output.stdout).to_string();
                let stderr = String::from_utf8_lossy(&output.stderr).to_string();
                let exit_code = output.status.code().unwrap_or(-1);

                let result = BashOutput {
                    stdout: stdout.clone(),
                    stderr: stderr.clone(),
                    exit_code,
                };

                let content = serde_json::to_string_pretty(&result)?;

                if output.status.success() {
                    Ok(ToolOutput::success(content))
                } else {
                    Ok(ToolOutput::error(content))
                }
            }
            Ok(Err(e)) => Ok(ToolOutput::error(format!(
                "Failed to execute command: {}",
                e
            ))),
            Err(_) => {
                warn!("Command timed out after {} seconds", timeout_seconds);
                Ok(ToolOutput::error(format!(
                    "Command timed out after {} seconds",
                    timeout_seconds
                )))
            }
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[tokio::test]
    async fn test_bash_echo() {
        let tool = BashTool::new();
        let ctx = ToolContext {
            session_id: Default::default(),
            working_directory: std::env::current_dir().unwrap(),
            env_vars: std::collections::HashMap::new(),
        };
        let input = ToolInput::new(json!({
            "command": "echo 'Hello World'"
        }));

        let result = tool.execute(input, &ctx).await.unwrap();
        assert!(!result.is_error);
        assert!(result.content.contains("Hello World"));
    }

    #[tokio::test]
    async fn test_bash_error() {
        let tool = BashTool::new();
        let ctx = ToolContext {
            session_id: Default::default(),
            working_directory: std::env::current_dir().unwrap(),
            env_vars: std::collections::HashMap::new(),
        };
        let input = ToolInput::new(json!({
            "command": "exit 1"
        }));

        let result = tool.execute(input, &ctx).await.unwrap();
        assert!(result.is_error);
    }
}
