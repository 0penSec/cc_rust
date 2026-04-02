//! SSE Stream handling for Anthropic API
//!
//! This module handles streaming responses from the Anthropic API,
//! parsing Server-Sent Events (SSE) into structured stream events.

use futures::{Stream, StreamExt};
use reqwest_eventsource::{Event, EventSource};
use serde::Deserialize;
use std::pin::Pin;
use std::task::{Context, Poll};
use tracing::{debug, error, trace, warn};

use crate::client::{AnthropicClient, MessagesRequest};
use claude_core::ClaudeResult;

/// Events emitted from the API stream
#[derive(Debug, Clone)]
pub enum StreamEvent {
    /// Raw text content delta
    TextDelta { text: String },
    /// Thinking content delta
    ThinkingDelta { thinking: String },
    /// Tool use started
    ToolUseStart {
        id: String,
        name: String,
        input: serde_json::Value,
    },
    /// Tool use input delta (partial JSON)
    ToolUseDelta { id: String, input_delta: String },
    /// Message complete
    MessageComplete {
        stop_reason: Option<String>,
        usage: TokenUsage,
    },
    /// Ping/keepalive
    Ping,
    /// Error occurred
    Error { message: String },
}

/// Token usage information
#[derive(Debug, Clone, Default)]
pub struct TokenUsage {
    pub input_tokens: usize,
    pub output_tokens: usize,
}

/// Internal SSE event from Anthropic API
#[derive(Debug, Deserialize)]
struct SseEvent {
    #[serde(rename = "type")]
    event_type: String,
    #[serde(flatten)]
    data: serde_json::Value,
}

/// Stream of events from the Anthropic API
pub struct EventStream {
    es: EventSource,
    current_tool_use: Option<ToolUseBuilder>,
}

/// Builder for accumulating tool use input
#[derive(Debug)]
struct ToolUseBuilder {
    id: String,
    name: String,
    input_json: String,
}

impl EventStream {
    /// Create a new event stream from a request
    pub async fn new(client: &AnthropicClient, request: MessagesRequest) -> ClaudeResult<Self> {
        let url = format!("{}/v1/messages", client.config().api_base);

        debug!("Starting SSE stream to {}", url);

        // Create request builder with proper headers
        let request_builder = client
            .http()
            .post(&url)
            .header("x-api-key", client.config().api_key.clone())
            .header("anthropic-version", client.config().version.clone())
            .header("content-type", "application/json");

        // Create EventSource from request builder
        let body = serde_json::to_string(&request)
            .map_err(|e| claude_core::ClaudeError::Serialization(e.to_string()))?;

        let es = EventSource::new(request_builder.body(body))
            .map_err(|e| claude_core::ClaudeError::Network(e.to_string()))?;

        Ok(Self {
            es,
            current_tool_use: None,
        })
    }

    /// Parse a raw SSE event into our StreamEvent
    fn parse_event(&mut self, data: &str) -> Option<StreamEvent> {
        trace!("Parsing SSE event: {}", data);

        let event: SseEvent = match serde_json::from_str(data) {
            Ok(e) => e,
            Err(e) => {
                warn!("Failed to parse SSE event: {}", e);
                return None;
            }
        };

        match event.event_type.as_str() {
            "message_start" => {
                debug!("Message started");
                None
            }
            "content_block_start" => {
                let block_type = event.data.get("type")?.as_str()?;
                match block_type {
                    "tool_use" => {
                        let id = event.data.get("id")?.as_str()?.to_string();
                        let name = event.data.get("name")?.as_str()?.to_string();

                        self.current_tool_use = Some(ToolUseBuilder {
                            id,
                            name,
                            input_json: String::new(),
                        });

                        None // Wait for input to accumulate
                    }
                    _ => None,
                }
            }
            "content_block_delta" => {
                let delta = event.data.get("delta")?;
                let delta_type = delta.get("type")?.as_str()?;

                match delta_type {
                    "text_delta" => {
                        let text = delta.get("text")?.as_str()?.to_string();
                        Some(StreamEvent::TextDelta { text })
                    }
                    "thinking_delta" => {
                        let thinking = delta.get("thinking")?.as_str()?.to_string();
                        Some(StreamEvent::ThinkingDelta { thinking })
                    }
                    "input_json_delta" => {
                        let partial = delta.get("partial_json")?.as_str()?.to_string();

                        if let Some(ref mut tool) = self.current_tool_use {
                            tool.input_json.push_str(&partial);
                        }

                        None // Accumulating input
                    }
                    _ => None,
                }
            }
            "content_block_stop" => {
                // Check if we have a pending tool use
                if let Some(tool) = self.current_tool_use.take() {
                    // Parse the accumulated JSON
                    let input = if tool.input_json.is_empty() {
                        serde_json::json!({})
                    } else {
                        serde_json::from_str(&tool.input_json).unwrap_or_else(|_| {
                            warn!("Failed to parse tool input JSON: {}", tool.input_json);
                            serde_json::json!({})
                        })
                    };

                    Some(StreamEvent::ToolUseStart {
                        id: tool.id,
                        name: tool.name,
                        input,
                    })
                } else {
                    None
                }
            }
            "message_delta" => {
                let stop_reason = event
                    .data
                    .get("stop_reason")
                    .and_then(|v| v.as_str())
                    .map(|s| s.to_string());

                let usage = event
                    .data
                    .get("usage")
                    .map(|u| TokenUsage {
                        input_tokens: u.get("input_tokens").and_then(|v| v.as_u64()).unwrap_or(0)
                            as usize,
                        output_tokens: u.get("output_tokens").and_then(|v| v.as_u64()).unwrap_or(0)
                            as usize,
                    })
                    .unwrap_or_default();

                Some(StreamEvent::MessageComplete { stop_reason, usage })
            }
            "message_stop" => {
                debug!("Message stopped");
                None
            }
            "ping" => Some(StreamEvent::Ping),
            "error" => {
                let msg = event
                    .data
                    .get("error")
                    .and_then(|e| e.get("message"))
                    .and_then(|m| m.as_str())
                    .unwrap_or("Unknown error")
                    .to_string();
                Some(StreamEvent::Error { message: msg })
            }
            _ => {
                trace!("Unknown event type: {}", event.event_type);
                None
            }
        }
    }
}

impl Stream for EventStream {
    type Item = ClaudeResult<StreamEvent>;

    fn poll_next(mut self: Pin<&mut Self>, cx: &mut Context<'_>) -> Poll<Option<Self::Item>> {
        match self.es.poll_next_unpin(cx) {
            Poll::Ready(Some(Ok(Event::Message(message)))) => {
                match self.parse_event(&message.data) {
                    Some(event) => Poll::Ready(Some(Ok(event))),
                    None => {
                        // Need to poll again for next event
                        cx.waker().wake_by_ref();
                        Poll::Pending
                    }
                }
            }
            Poll::Ready(Some(Ok(Event::Open))) => {
                trace!("SSE connection opened");
                cx.waker().wake_by_ref();
                Poll::Pending
            }
            Poll::Ready(Some(Err(e))) => {
                error!("SSE error: {}", e);
                Poll::Ready(Some(Err(claude_core::ClaudeError::Network(e.to_string()))))
            }
            Poll::Ready(None) => Poll::Ready(None),
            Poll::Pending => Poll::Pending,
        }
    }
}

impl Drop for EventStream {
    fn drop(&mut self) {
        debug!("Closing SSE stream");
        self.es.close();
    }
}
