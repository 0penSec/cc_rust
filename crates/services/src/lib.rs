//! Service layer for external integrations
//!
//! This crate provides integrations with external services including:
//! - Anthropic API client
//! - MCP (Model Context Protocol) client
//! - LSP (Language Server Protocol) client
//! - Authentication services
//! - Telemetry and analytics

pub mod api;
pub mod auth;
pub mod mcp;
pub mod lsp;
pub mod telemetry;

pub use api::AnthropicClient;
pub use auth::AuthManager;
