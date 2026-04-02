//! Tool Loop - Core conversation loop with tool execution
//!
//! This module implements the main agent loop:
//! 1. Send messages to API
//! 2. Stream response, capturing text and tool calls
//! 3. Execute tools when requested
//! 4. Send tool results back
//! 5. Repeat until stop_reason is not "tool_use"

use std::io::Write;
use futures::StreamExt;
use tracing::{debug, error, info, trace, warn};

use claude_core::{
    ClaudeResult, Message, Tool, ToolContext, ToolInput, ToolCallResult,
    message::{ToolCall, AssistantContent, MessageContent},
};

use crate::conversation::Conversation;
use crate::client::{AnthropicClient, MessagesRequest};
use crate::stream::{EventStream, StreamEvent, TokenUsage};

/// Result of one turn in the conversation
#[derive(Debug)]
pub enum TurnResult {
    /// Conversation completed naturally
    Complete { usage: TokenUsage },
    /// Tool calls were made, need to continue
    ToolCallsMade { count: usize },
    /// Error occurred
    Error { message: String },
}

/// The main conversation loop
pub struct ToolLoop {
    client: AnthropicClient,
    tools: Vec<Box<dyn Tool>>,
    max_iterations: usize,
    streaming: bool,
}

impl ToolLoop {
    pub fn new(client: AnthropicClient) -> Self {
        Self {
            client,
            tools: Vec::new(),
            max_iterations: 50,
            streaming: true,
        }
    }

    /// Register a tool for use in the conversation
    pub fn register_tool(&mut self, tool: Box<dyn Tool>) {
        self.tools.push(tool);
    }

    /// Set maximum iterations to prevent infinite loops
    pub fn set_max_iterations(&mut self, max: usize) {
        self.max_iterations = max;
    }

    /// Enable/disable streaming
    pub fn set_streaming(&mut self, streaming: bool) {
        self.streaming = streaming;
    }

    /// Run the complete conversation loop
    pub async fn run(
        &self,
        conversation: &mut Conversation,
        tool_ctx: &ToolContext,
    ) -> ClaudeResult<TokenUsage> {
        let mut total_usage = TokenUsage::default();

        for iteration in 0..self.max_iterations {
            debug!("=== Tool loop iteration {} ===", iteration + 1);

            let turn_result = self.run_single_turn(conversation, tool_ctx).await?;

            match turn_result {
                TurnResult::Complete { usage } => {
                    total_usage.input_tokens += usage.input_tokens;
                    total_usage.output_tokens += usage.output_tokens;
                    info!("Conversation complete");
                    break;
                }
                TurnResult::ToolCallsMade { count } => {
                    debug!("Executed {} tool calls, continuing...", count);
                    continue;
                }
                TurnResult::Error { message } => {
                    error!("Error in conversation: {}", message);
                    return Err(claude_core::ClaudeError::Execution(message));
                }
            }
        }

        Ok(total_usage)
    }

    /// Run a single turn: call API, handle response, execute tools
    async fn run_single_turn(
        &self,
        conversation: &mut Conversation,
        tool_ctx: &ToolContext,
    ) -> ClaudeResult<TurnResult> {
        // Build the request
        let request = self.build_request(conversation)?;

        // Stream the response
        let mut stream = EventStream::new(&self.client, request).await?;

        let mut text_buffer = String::new();
        let mut tool_calls: Vec<ToolCall> = Vec::new();
        let mut _current_tool_call: Option<ToolCall> = None;
        let mut usage = TokenUsage::default();

        // Process stream events
        while let Some(result) = stream.next().await {
            match result? {
                StreamEvent::TextDelta { text } => {
                    print!("{}", text);
                    std::io::stdout().flush().ok();
                    text_buffer.push_str(&text);
                }
                StreamEvent::ThinkingDelta { thinking } => {
                    // Optionally display thinking
                    debug!("Thinking: {}", thinking);
                }
                StreamEvent::ToolUseStart { id, name, input } => {
                    println!("\n[Tool: {}]", name);

                    let tool_call = ToolCall { id, name, input };
                    _current_tool_call = Some(tool_call.clone());
                    tool_calls.push(tool_call);
                }
                StreamEvent::MessageComplete { stop_reason, usage: u } => {
                    usage = u;

                    // Add assistant message to conversation
                    if !text_buffer.is_empty() || !tool_calls.is_empty() {
                        let content = if tool_calls.is_empty() {
                            AssistantContent::Text(text_buffer.clone())
                        } else {
                            AssistantContent::ToolCalls(tool_calls.clone())
                        };

                        conversation.add_message(Message::Assistant { content });
                    }

                    // Handle stop reason
                    match stop_reason.as_deref() {
                        Some("tool_use") | None if !tool_calls.is_empty() => {
                            // Execute tools and continue
                            let results = self.execute_tool_calls(&tool_calls, tool_ctx
                            ).await?;

                            // Add tool results to conversation
                            for result in results {
                                conversation.add_user_message(format!(
                                    "Tool result ({}): {}",
                                    result.tool_call_id,
                                    result.content
                                ));
                            }

                            return Ok(TurnResult::ToolCallsMade {
                                count: tool_calls.len()
                            });
                        }
                        Some("end_turn") | Some("stop_sequence") | None => {
                            println!(); // Newline after response
                            return Ok(TurnResult::Complete { usage });
                        }
                        Some(reason) => {
                            warn!("Unexpected stop reason: {}", reason);
                            return Ok(TurnResult::Complete { usage });
                        }
                    }
                }
                StreamEvent::Ping => {
                    trace!("Received ping");
                }
                StreamEvent::Error { message } => {
                    return Ok(TurnResult::Error { message });
                }
                _ => {}
            }
        }

        Ok(TurnResult::Complete { usage })
    }

    /// Execute tool calls and return results
    async fn execute_tool_calls(
        &self,
        calls: &[ToolCall],
        ctx: &ToolContext,
    ) -> ClaudeResult<Vec<ToolCallResult>> {
        let mut results = Vec::new();

        for call in calls {
            debug!("Executing tool: {} with input: {:?}", call.name, call.input);

            // Find the tool
            let tool = self.tools.iter()
                .find(|t| t.name() == call.name)
                .ok_or_else(|| {
                    claude_core::ClaudeError::Execution(
                        format!("Unknown tool: {}", call.name)
                    )
                })?;

            // Execute
            let input = ToolInput::new(call.input.clone());
            let output = tool.execute(input, ctx).await?;

            println!("[Tool result]: {}", &output.content[..output.content.len().min(200)]);

            results.push(ToolCallResult {
                tool_call_id: call.id.clone(),
                content: output.content,
                is_error: output.is_error,
            });
        }

        Ok(results)
    }

    /// Build API request from conversation
    fn build_request(&self,
        conversation: &Conversation,
    ) -> ClaudeResult<MessagesRequest> {
        // Convert messages to API format
        let messages: Vec<serde_json::Value> = conversation.messages
            .iter()
            .filter_map(|m| self.convert_message(m))
            .collect();

        // Build tool schemas
        let tools: Vec<serde_json::Value> = self.tools.iter()
            .map(|t| {
                serde_json::json!({
                    "name": t.name(),
                    "description": t.description(),
                    "input_schema": t.input_schema(),
                })
            })
            .collect();

        Ok(MessagesRequest {
            model: conversation.model.clone(),
            max_tokens: conversation.max_tokens,
            messages,
            tools: if tools.is_empty() { None } else { Some(tools) },
            system: conversation.system_prompt.clone(),
            temperature: Some(0.7),
            thinking: None,
        })
    }

    /// Convert internal Message to API format
    fn convert_message(&self,
        message: &Message,
    ) -> Option<serde_json::Value> {
        match message {
            Message::User { content } => {
                let text = match content {
                    MessageContent::Text(t) => t.clone(),
                    MessageContent::MultiContent(parts) => {
                        parts.iter()
                            .filter_map(|p| match p {
                                claude_core::message::ContentPart::Text { text } => Some(text.clone()),
                                _ => None,
                            })
                            .collect::<Vec<_>>()
                            .join("")
                    }
                };
                Some(serde_json::json!({
                    "role": "user",
                    "content": text,
                }))
            }
            Message::Assistant { content } => {
                match content {
                    AssistantContent::Text(text) => {
                        Some(serde_json::json!({
                            "role": "assistant",
                            "content": text,
                        }))
                    }
                    AssistantContent::ToolCalls(calls) => {
                        let content: Vec<serde_json::Value> = calls.iter()
                            .map(|call| {
                                serde_json::json!({
                                    "type": "tool_use",
                                    "id": call.id,
                                    "name": call.name,
                                    "input": call.input,
                                })
                            })
                            .collect();

                        Some(serde_json::json!({
                            "role": "assistant",
                            "content": content,
                        }))
                    }
                }
            }
            Message::System { .. } => None, // System is handled separately
        }
    }
}

impl Default for ToolLoop {
    fn default() -> Self {
        panic!("ToolLoop requires a client, use ToolLoop::new()");
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_turn_result_complete() {
        let result = TurnResult::Complete {
            usage: TokenUsage {
                input_tokens: 10,
                output_tokens: 20,
            },
        };

        match result {
            TurnResult::Complete { usage } => {
                assert_eq!(usage.input_tokens, 10);
                assert_eq!(usage.output_tokens, 20);
            }
            _ => panic!("Expected Complete"),
        }
    }
}
