use async_trait::async_trait;
use schemars::JsonSchema;
use serde::{Deserialize, Serialize};
use serde_json::Value;

use crate::permission::PermissionMode;

/// Trait defining a tool that Claude can invoke
#[async_trait]
pub trait Tool: Send + Sync {
    /// Returns the unique name of the tool
    fn name(&self) -> &str;

    /// Returns the description of what the tool does
    fn description(&self) -> &str;

    /// Returns the JSON schema for the tool's input parameters
    fn input_schema(&self) -> Value;

    /// Returns the permission mode for this tool
    fn permission_mode(&self) -> PermissionMode;

    /// Executes the tool with the given input and context
    async fn execute(&self, input: ToolInput, ctx: &ToolContext) -> ToolResult;
}

/// Input to a tool execution
#[derive(Debug, Clone)]
pub struct ToolInput {
    pub raw: Value,
}

impl ToolInput {
    pub fn new(raw: Value) -> Self {
        Self { raw }
    }

    pub fn parse<T: for<'de> Deserialize<'de>>(&self) -> Result<T, serde_json::Error> {
        serde_json::from_value(self.raw.clone())
    }
}

/// Output from a tool execution
#[derive(Debug, Clone)]
pub struct ToolOutput {
    pub content: String,
    pub is_error: bool,
    pub metadata: Option<Value>,
}

impl ToolOutput {
    pub fn success(content: impl Into<String>) -> Self {
        Self {
            content: content.into(),
            is_error: false,
            metadata: None,
        }
    }

    pub fn error(content: impl Into<String>) -> Self {
        Self {
            content: content.into(),
            is_error: true,
            metadata: None,
        }
    }

    pub fn with_metadata(mut self, metadata: Value) -> Self {
        self.metadata = Some(metadata);
        self
    }
}

pub type ToolResult = crate::error::ClaudeResult<ToolOutput>;

/// Context passed to tool execution
#[derive(Debug, Clone)]
pub struct ToolContext {
    pub session_id: crate::SessionId,
    pub working_directory: std::path::PathBuf,
    pub env_vars: std::collections::HashMap<String, String>,
}

/// Definition of a tool for registration
#[derive(Debug, Clone)]
pub struct ToolDefinition {
    pub name: String,
    pub description: String,
    pub input_schema: Value,
    pub permission_mode: PermissionMode,
}
