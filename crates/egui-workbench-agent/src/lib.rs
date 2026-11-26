//! MCP, permissions, accessibility, and automation for egui-workbench.
//!
//! The MCP server is transport-agnostic: [`McpServer::handle`] takes one JSON-RPC value
//! and returns one response value, and [`McpServer::serve_stdio`] wraps it in the
//! newline-delimited stdio transport. Permissions are checked before any host mutation.

pub mod access;
pub mod automation;
pub mod mcp;
mod permissions;

pub use access::{AccessNode, accessibility_tree, find_node};
pub use automation::{Automation, AutomationError, AutomationStep};
pub use mcp::{Activity, McpServer, Permissions, ToolError};
