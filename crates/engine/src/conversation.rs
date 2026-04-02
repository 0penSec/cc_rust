use claude_core::{message::ToolCall, Message, SessionId, ToolCallResult};

/// A conversation session
#[derive(Debug, Clone)]
pub struct Conversation {
    pub session_id: SessionId,
    pub messages: Vec<Message>,
    pub system_prompt: Option<String>,
    pub model: String,
    pub max_tokens: usize,
    pub total_input_tokens: usize,
    pub total_output_tokens: usize,
}

impl Conversation {
    pub fn builder() -> ConversationBuilder {
        ConversationBuilder::default()
    }

    pub fn add_message(&mut self, message: Message) {
        self.messages.push(message);
    }

    pub fn add_user_message(&mut self, content: impl Into<String>) {
        self.messages.push(Message::User {
            content: claude_core::message::MessageContent::Text(content.into()),
        });
    }

    pub fn add_assistant_message(&mut self, content: impl Into<String>) {
        self.messages.push(Message::Assistant {
            content: claude_core::message::AssistantContent::Text(content.into()),
        });
    }

    pub fn add_tool_results(&mut self, results: Vec<ToolCallResult>) {
        // Tool results are added as user messages with tool_result content
        for result in results {
            self.add_user_message(format!("Tool result: {}", result.content));
        }
    }

    pub fn pending_tool_calls(&self) -> Vec<&ToolCall> {
        // Extract pending tool calls from the last assistant message
        if let Some(Message::Assistant { content }) = self.messages.last() {
            match content {
                claude_core::message::AssistantContent::ToolCalls(calls) => calls.iter().collect(),
                _ => vec![],
            }
        } else {
            vec![]
        }
    }

    pub fn update_token_usage(&mut self, input: usize, output: usize) {
        self.total_input_tokens += input;
        self.total_output_tokens += output;
    }
}

/// Builder for conversations
#[derive(Debug, Default)]
pub struct ConversationBuilder {
    session_id: Option<SessionId>,
    system_prompt: Option<String>,
    model: Option<String>,
    max_tokens: Option<usize>,
}

impl ConversationBuilder {
    pub fn session_id(mut self, id: SessionId) -> Self {
        self.session_id = Some(id);
        self
    }

    pub fn system_prompt(mut self, prompt: impl Into<String>) -> Self {
        self.system_prompt = Some(prompt.into());
        self
    }

    pub fn model(mut self, model: impl Into<String>) -> Self {
        self.model = Some(model.into());
        self
    }

    pub fn max_tokens(mut self, tokens: usize) -> Self {
        self.max_tokens = Some(tokens);
        self
    }

    pub fn build(self) -> Conversation {
        Conversation {
            session_id: self.session_id.unwrap_or_default(),
            messages: Vec::new(),
            system_prompt: self.system_prompt,
            model: self
                .model
                .unwrap_or_else(|| "claude-sonnet-4-6".to_string()),
            max_tokens: self.max_tokens.unwrap_or(4096),
            total_input_tokens: 0,
            total_output_tokens: 0,
        }
    }
}
