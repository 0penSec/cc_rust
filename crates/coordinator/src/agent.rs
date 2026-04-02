//! Agent definitions

use claude_core::AgentId;

pub struct Agent {
    pub id: AgentId,
    pub name: String,
}

impl Agent {
    pub fn new(name: impl Into<String>) -> Self {
        Self {
            id: AgentId::new(),
            name: name.into(),
        }
    }
}
