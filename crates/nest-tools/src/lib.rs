//! MCP (Model Context Protocol) implementation for Nest
//!
//! Implements full MCP protocol client with automatic server discovery,
//! tool proxying, and permission checking. All tool calls go through
//! this layer which proxies them through the PermissionEngine before execution.

pub mod mcp;

pub use mcp::{MCPClient, MCPServer, MCPTool};
use nest_api::error::Result;
use nest_permissions::PermissionEngine;
use serde_json::Value;

#[derive(Debug)]
pub struct MCPProxy {
    permission_engine: PermissionEngine,
    mcp_client: mcp::MCPClient,
}

impl MCPProxy {
    /// Create a new MCP proxy with the given permission engine
    pub fn new(permission_engine: PermissionEngine) -> Self {
        let mcp_client = mcp::MCPClient::new(permission_engine.clone());
        Self {
            permission_engine,
            mcp_client,
        }
    }

    /// Start the MCP client and initialize all configured servers
    pub async fn start(&mut self) -> Result<()> {
        self.mcp_client.start().await
    }

    /// Add an MCP server configuration
    pub fn add_server(&mut self, name: &str, command: &str) {
        self.mcp_client.add_server(name, command);
    }

    /// Call an MCP tool with permission checking
    pub async fn call_tool(&mut self, agent_id: &str, tool_name: &str, params: Value) -> Result<Value> {
        self.mcp_client.call_tool(agent_id, tool_name, params).await
    }

    /// Get list of all available tools
    pub fn tools(&self) -> Vec<&MCPTool> {
        self.mcp_client.tools()
    }

    /// Get list of all available tools as owned values
    pub fn list_tools(&self) -> Vec<MCPTool> {
        self.mcp_client.tools().into_iter().cloned().collect()
    }

    /// Get pending permission requests
    pub fn pending_requests(&self) -> &[nest_permissions::PendingRequest] {
        self.permission_engine.pending_requests()
    }

    /// Approve a pending request
    pub fn approve_request(&mut self, index: usize) -> bool {
        self.permission_engine.approve(index)
    }

    /// Deny a pending request
    pub fn deny_request(&mut self, index: usize) -> bool {
        self.permission_engine.deny(index)
    }
}

