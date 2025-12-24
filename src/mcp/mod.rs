//! MCP (Model Context Protocol) server implementation
//!
//! This module provides programmatic access to the quote service via the Model Context Protocol,
//! allowing AI models and automation tools to generate quotes without using the web interface.

pub mod quote_tools;
pub mod server;

pub use server::create_mcp_router;
