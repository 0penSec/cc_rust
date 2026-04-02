//! Anthropic API client

use reqwest::Client;

pub struct AnthropicClient {
    _client: Client,
    _api_key: String,
}

impl AnthropicClient {
    pub fn new(api_key: String) -> Self {
        Self {
            _client: Client::new(),
            _api_key: api_key,
        }
    }
}
