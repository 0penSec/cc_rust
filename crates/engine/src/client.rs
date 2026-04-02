use reqwest::{Client, header};
use serde::{Deserialize, Serialize};
use std::time::Duration;

use claude_core::ClaudeResult;

/// API request body for messages
#[derive(Debug, Clone, Serialize)]
pub struct MessagesRequest {
    pub model: String,
    pub max_tokens: usize,
    pub messages: Vec<serde_json::Value>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub tools: Option<Vec<serde_json::Value>>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub system: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub temperature: Option<f32>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub thinking: Option<ThinkingConfig>,
}

#[derive(Debug, Clone, Serialize)]
pub struct ThinkingConfig {
    #[serde(rename = "type")]
    pub type_: String,
    pub budget_tokens: usize,
}

/// Content block in API response
#[derive(Debug, Clone, Deserialize)]
#[serde(tag = "type", rename_all = "snake_case")]
pub enum ContentBlock {
    Text { text: String },
    Thinking { thinking: String, signature: String },
    ToolUse { id: String, name: String, input: serde_json::Value },
}

/// Token usage
#[derive(Debug, Clone, Deserialize, Default)]
pub struct Usage {
    pub input_tokens: usize,
    pub output_tokens: usize,
}

/// Configuration for the Anthropic API client
#[derive(Debug, Clone)]
pub struct ClientConfig {
    pub api_key: String,
    pub api_base: String,
    pub version: String,
    pub timeout: Duration,
}

impl Default for ClientConfig {
    fn default() -> Self {
        Self {
            api_key: String::new(),
            api_base: "https://api.anthropic.com".to_string(),
            version: "2023-06-01".to_string(),
            timeout: Duration::from_secs(300),
        }
    }
}

/// Anthropic API client
pub struct AnthropicClient {
    config: ClientConfig,
    http: Client,
}

impl AnthropicClient {
    pub fn new(config: ClientConfig) -> ClaudeResult<Self> {
        let mut headers = header::HeaderMap::new();
        headers.insert(
            "x-api-key",
            header::HeaderValue::from_str(&config.api_key)
                .map_err(|e| claude_core::ClaudeError::Config(e.to_string()))?,
        );
        headers.insert(
            "anthropic-version",
            header::HeaderValue::from_str(&config.version)
                .map_err(|e| claude_core::ClaudeError::Config(e.to_string()))?,
        );

        let http = Client::builder()
            .default_headers(headers)
            .timeout(config.timeout)
            .build()
            .map_err(|e| claude_core::ClaudeError::Network(e.to_string()))?;

        Ok(Self { config, http })
    }

    pub fn http(&self) -> &Client {
        &self.http
    }

    pub fn config(&self) -> &ClientConfig {
        &self.config
    }
}

/// API response
#[derive(Debug, Deserialize)]
pub struct MessagesResponse {
    pub id: String,
    pub r#type: String,
    pub role: String,
    pub content: Vec<ContentBlock>,
    pub model: String,
    pub stop_reason: Option<String>,
    pub usage: Usage,
}
