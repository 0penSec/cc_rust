//! MCP (Model Context Protocol) client

pub struct McpClient;

impl McpClient {
    pub fn new() -> Self {
        Self
    }
}

impl Default for McpClient {
    fn default() -> Self {
        Self::new()
    }
}
