//! LLM Query Engine - Core conversation management
//!
//! This crate implements the query engine that manages conversations with
//! the Anthropic API, including streaming responses, tool call loops, and
//! retry logic.

pub mod client;
pub mod conversation;
pub mod r#loop;
pub mod retry;
pub mod stream;
pub mod token;

pub use client::{AnthropicClient, ClientConfig, MessagesRequest};
pub use conversation::{Conversation, ConversationBuilder};
pub use r#loop::{ToolLoop, TurnResult};
pub use stream::{EventStream, StreamEvent, TokenUsage};

use claude_core::{ClaudeResult, SessionId};

/// Engine configuration
#[derive(Debug, Clone)]
pub struct EngineConfig {
    pub client: ClientConfig,
    pub max_retries: u32,
    pub retry_delay_ms: u64,
    pub enable_streaming: bool,
}

impl Default for EngineConfig {
    fn default() -> Self {
        Self {
            client: ClientConfig::default(),
            max_retries: 3,
            retry_delay_ms: 1000,
            enable_streaming: true,
        }
    }
}

/// Main query engine
pub struct QueryEngine {
    _config: EngineConfig,
    _client: AnthropicClient,
    conversations: std::collections::HashMap<SessionId, Conversation>,
}

impl QueryEngine {
    pub fn new(config: EngineConfig) -> ClaudeResult<Self> {
        let client = AnthropicClient::new(config.client.clone())?;
        Ok(Self {
            _config: config,
            _client: client,
            conversations: std::collections::HashMap::new(),
        })
    }

    pub fn create_conversation(&mut self, session_id: SessionId) -> &mut Conversation {
        let conversation = Conversation::builder()
            .session_id(session_id)
            .build();
        self.conversations.insert(session_id, conversation);
        self.conversations.get_mut(&session_id).unwrap()
    }

    pub fn get_conversation(&self, session_id: &SessionId) -> Option<&Conversation> {
        self.conversations.get(session_id)
    }

    pub fn get_conversation_mut(&mut self, session_id: &SessionId) -> Option<&mut Conversation> {
        self.conversations.get_mut(session_id)
    }
}
