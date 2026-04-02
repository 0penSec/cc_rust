//! Core types, traits, and interfaces for Claude Code
//!
//! This crate defines the foundational abstractions used across the entire
//! codebase. It is intentionally minimal and has no dependencies on other
//! workspace crates.

pub mod context;
pub mod error;
pub mod message;
pub mod permission;
pub mod tool;
pub mod types;

pub use context::{Context, ConversationContext, ProjectContext, UserContext};
pub use error::{ClaudeError, ClaudeResult};
pub use message::{Message, MessageContent, MessageRole, ToolCall, ToolCallResult};
pub use permission::{PermissionManager, PermissionMode, PermissionResult};
pub use tool::{Tool, ToolContext, ToolDefinition, ToolInput, ToolOutput, ToolResult};

use serde::{Deserialize, Serialize};
use uuid::Uuid;

/// Unique identifier for a conversation session
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub struct SessionId(pub Uuid);

impl SessionId {
    pub fn new() -> Self {
        Self(Uuid::new_v4())
    }
}

impl Default for SessionId {
    fn default() -> Self {
        Self::new()
    }
}

/// Unique identifier for an agent
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub struct AgentId(pub Uuid);

impl AgentId {
    pub fn new() -> Self {
        Self(Uuid::new_v4())
    }
}

impl Default for AgentId {
    fn default() -> Self {
        Self::new()
    }
}

/// Unique identifier for a tool execution
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub struct ToolExecutionId(pub Uuid);

impl ToolExecutionId {
    pub fn new() -> Self {
        Self(Uuid::new_v4())
    }
}

impl Default for ToolExecutionId {
    fn default() -> Self {
        Self::new()
    }
}
