//! Tool implementations for Claude Code
//!
//! This crate implements all the tools that Claude can invoke, including
//! file operations, bash commands, search, web access, etc.

pub mod bash;
pub mod file;
pub mod search;
pub mod web;
pub mod registry;

pub use bash::BashTool;
pub use file::{FileReadTool, FileWriteTool, FileEditTool};
pub use search::{GlobTool, GrepTool};
pub use web::{WebFetchTool, WebSearchTool};
pub use registry::ToolRegistry;


/// Initialize the default tool registry with all built-in tools
pub fn default_registry() -> ToolRegistry {
    let mut registry = ToolRegistry::new();

    // File operations
    registry.register(Box::new(FileReadTool));
    registry.register(Box::new(FileWriteTool));
    registry.register(Box::new(FileEditTool));

    // Search
    registry.register(Box::new(GlobTool));
    registry.register(Box::new(GrepTool));

    // Shell
    registry.register(Box::new(BashTool::new()));

    // Web
    registry.register(Box::new(WebFetchTool));

    registry
}
