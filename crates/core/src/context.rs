use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::path::PathBuf;

/// Overall context for a conversation
#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct Context {
    pub user: UserContext,
    pub project: ProjectContext,
    pub conversation: ConversationContext,
}

/// User-specific context
#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct UserContext {
    pub preferences: HashMap<String, String>,
    pub working_directory: PathBuf,
    pub shell: String,
    pub environment: HashMap<String, String>,
}

/// Project-specific context
#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct ProjectContext {
    pub root_path: PathBuf,
    pub git_remote: Option<String>,
    pub detected_language: Option<String>,
    pub claude_md_content: Option<String>,
}

/// Conversation-specific context
#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct ConversationContext {
    pub message_count: usize,
    pub total_tokens: usize,
    pub cost_usd: f64,
    pub pending_tool_calls: Vec<crate::message::ToolCall>,
}
