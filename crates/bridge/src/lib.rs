//! IDE bridge integration
//!
//! Provides bidirectional communication between the CLI and IDE extensions.

pub mod protocol;
pub mod server;

pub use server::BridgeServer;
