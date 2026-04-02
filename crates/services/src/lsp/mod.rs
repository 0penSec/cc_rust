//! LSP (Language Server Protocol) client

pub struct LspClient;

impl LspClient {
    pub fn new() -> Self {
        Self
    }
}

impl Default for LspClient {
    fn default() -> Self {
        Self::new()
    }
}
