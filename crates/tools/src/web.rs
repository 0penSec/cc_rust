use async_trait::async_trait;
use schemars::{schema_for, JsonSchema};
use serde::{Deserialize, Serialize};
use serde_json::json;
use tracing::debug;

use claude_core::{
    Tool, ToolContext, ToolInput, ToolOutput, ToolResult, PermissionMode,
};

/// Web fetch tool for retrieving URL content
pub struct WebFetchTool;

#[derive(Debug, Serialize, Deserialize, JsonSchema)]
struct WebFetchInput {
    /// URL to fetch
    url: String,
    /// Maximum content length in characters (default: 10000)
    #[serde(default)]
    max_length: Option<usize>,
}

#[async_trait]
impl Tool for WebFetchTool {
    fn name(&self) -> &str {
        "web_fetch"
    }

    fn description(&self) -> &str {
        "Fetch content from a URL. Returns the page content as text. \
         Useful for reading documentation, articles, or any web content."
    }

    fn input_schema(&self) -> serde_json::Value {
        serde_json::to_value(schema_for!(WebFetchInput)).unwrap()
    }

    fn permission_mode(&self) -> PermissionMode {
        PermissionMode::Ask
    }

    async fn execute(&self, input: ToolInput, _ctx: &ToolContext) -> ToolResult {
        let input: WebFetchInput = input.parse()?;
        let max_length = input.max_length.unwrap_or(10000);

        debug!("Fetching URL: {}", input.url);

        let client = reqwest::Client::new();
        let response = match client.get(&input.url).send().await {
            Ok(r) => r,
            Err(e) => return Ok(ToolOutput::error(format!("Request failed: {}", e))),
        };

        let status = response.status();
        if !status.is_success() {
            return Ok(ToolOutput::error(format!(
                "HTTP error: {} {}",
                status.as_u16(),
                status.canonical_reason().unwrap_or("Unknown")
            )));
        }

        let content = match response.text().await {
            Ok(t) => t,
            Err(e) => return Ok(ToolOutput::error(format!("Failed to read response: {}", e))),
        };

        // Truncate if needed
        let truncated = if content.len() > max_length {
            format!(
                "{}\n\n[Content truncated from {} to {} characters]",
                &content[..max_length],
                content.len(),
                max_length
            )
        } else {
            content
        };

        Ok(ToolOutput::success(truncated))
    }
}

/// Web search tool (placeholder for search integration)
pub struct WebSearchTool;

#[derive(Debug, Serialize, Deserialize, JsonSchema)]
struct WebSearchInput {
    /// Search query
    query: String,
    /// Number of results (default: 5)
    #[serde(default)]
    num_results: Option<usize>,
}

#[async_trait]
impl Tool for WebSearchTool {
    fn name(&self) -> &str {
        "web_search"
    }

    fn description(&self) -> &str {
        "Search the web for information. Returns search results with titles, \
         URLs, and snippets. Requires a search API integration."
    }

    fn input_schema(&self) -> serde_json::Value {
        serde_json::to_value(schema_for!(WebSearchInput)).unwrap()
    }

    fn permission_mode(&self) -> PermissionMode {
        PermissionMode::Ask
    }

    async fn execute(&self, input: ToolInput, _ctx: &ToolContext) -> ToolResult {
        let input: WebSearchInput = input.parse()?;

        // TODO: Implement actual search using an API like:
        // - Brave Search API
        // - Google Custom Search
        // - Bing Web Search
        // - SerpAPI

        Ok(ToolOutput::error(
            "Web search not yet implemented. Consider using web_fetch with a known URL instead."
        ))
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use wiremock::{MockServer, Mock, ResponseTemplate};
    use wiremock::matchers::{method, path};

    #[tokio::test]
    async fn test_web_fetch() {
        let mock_server = MockServer::start().await;

        Mock::given(method("GET"))
            .and(path("/test"))
            .respond_with(ResponseTemplate::new(200).set_body_string("Hello from web"))
            .mount(&mock_server)
            .await;

        let tool = WebFetchTool;
        let ctx = ToolContext {
            session_id: Default::default(),
            working_directory: std::env::current_dir().unwrap(),
            env_vars: std::collections::HashMap::new(),
        };
        let input = ToolInput::new(json!({
            "url": format!("{}/test", mock_server.uri())
        }));

        let result = tool.execute(input, &ctx).await.unwrap();
        assert!(!result.is_error);
        assert!(result.content.contains("Hello from web"));
    }
}
