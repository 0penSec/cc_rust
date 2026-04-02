use serde::{Deserialize, Serialize};

/// Manager for handling tool execution permissions
#[derive(Debug, Clone)]
pub struct PermissionManager {
    mode: PermissionMode,
    history: std::collections::HashMap<String, PermissionDecision>,
}

impl PermissionManager {
    pub fn new(mode: PermissionMode) -> Self {
        Self {
            mode,
            history: std::collections::HashMap::new(),
        }
    }

    pub fn check(&self, tool_name: &str, description: &str) -> PermissionResult {
        match self.mode {
            PermissionMode::Bypass => PermissionResult::Allow,
            PermissionMode::Allow => PermissionResult::Allow,
            PermissionMode::Deny => PermissionResult::Deny,
            PermissionMode::Auto => {
                // TODO: Implement auto-decision logic
                PermissionResult::Ask(description.to_string())
            }
            PermissionMode::Ask => PermissionResult::Ask(description.to_string()),
        }
    }

    pub fn record_decision(&mut self, tool_name: String, decision: PermissionDecision) {
        self.history.insert(tool_name, decision);
    }
}

impl Default for PermissionManager {
    fn default() -> Self {
        Self::new(PermissionMode::Ask)
    }
}

/// Permission mode for tool execution
#[derive(Debug, Clone, Copy, PartialEq, Eq, Default, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum PermissionMode {
    #[default]
    Ask,
    Allow,
    Deny,
    Auto,
    Bypass,
}

/// Result of a permission check
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum PermissionResult {
    Allow,
    Deny,
    Ask(String),
}

/// A recorded permission decision
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum PermissionDecision {
    AlwaysAllow,
    AllowOnce,
    DenyOnce,
    AlwaysDeny,
}
