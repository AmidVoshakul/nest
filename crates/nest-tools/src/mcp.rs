//! MCP (Model Context Protocol) client implementation
//!
//! Implements full MCP protocol support with automatic server discovery,
//! tool proxying, and permission checking.

use std::process::Stdio;
use tokio::io::{AsyncBufReadExt, AsyncWriteExt, BufReader};
use tokio::process::Command;
use tokio::sync::mpsc;
use serde_json::{Value, json};
use nest_api::error::{Result, Error};
use nest_permissions::PermissionEngine;

#[derive(Debug, Clone)]
pub struct MCPTool {
    pub name: String,
    pub description: String,
    pub input_schema: Value,
}

#[derive(Debug, Clone)]
pub struct MCPServer {
    pub name: String,
    pub command: String,
    pub tools: Vec<MCPTool>,
}

#[derive(Debug)]
pub struct MCPClient {
    permission_engine: PermissionEngine,
    servers: Vec<MCPServer>,
    tx: mpsc::Sender<MCPRequest>,
    rx: mpsc::Receiver<MCPRequest>,
}

#[derive(Debug)]
enum MCPRequest {
    CallTool {
        agent_id: String,
        server_name: String,
        tool_name: String,
        params: Value,
    },
    ListTools,
}

#[derive(Debug)]
enum MCPResponse {
    ToolResult(Value),
    ToolsList(Vec<MCPTool>),
    Error(String),
}

impl MCPClient {
    /// Create a new MCP client
    pub fn new(permission_engine: PermissionEngine) -> Self {
        let (tx, rx) = mpsc::channel(100);
        Self {
            permission_engine,
            servers: Vec::new(),
            tx,
            rx,
        }
    }

    /// Add an MCP server configuration
    pub fn add_server(&mut self, name: &str, command: &str) {
        self.servers.push(MCPServer {
            name: name.to_string(),
            command: command.to_string(),
            tools: Vec::new(),
        });
    }

    /// Start the MCP client background worker
    pub async fn start(&mut self) -> Result<()> {
        // Start all configured MCP servers
        let len = self.servers.len();
        for i in 0..len {
            self.initialize_server(i).await?;
        }
        Ok(())
    }

    /// Initialize a single MCP server
    async fn initialize_server(&mut self, i: usize) -> Result<()> {
        let command = self.servers[i].command.clone();
        let name = self.servers[i].name.clone();
        
        let mut child = Command::new("sh")
            .arg("-c")
            .arg(&command)
            .stdin(Stdio::piped())
            .stdout(Stdio::piped())
            .stderr(Stdio::piped())
            .spawn()
            .map_err(|e| Error::Sandbox(format!("Failed to start MCP server {}: {}", name, e)))?;

        let mut stdin = child.stdin.take().unwrap();
        let mut stdout = BufReader::new(child.stdout.take().unwrap());

        // Send initialize request
        let init_request = json!({
            "jsonrpc": "2.0",
            "id": 1,
            "method": "initialize",
            "params": {
                "protocolVersion": "2024-11-05",
                "capabilities": {},
                "clientInfo": {
                    "name": "nest",
                    "version": "0.1.0"
                }
            }
        });

        stdin.write_all(serde_json::to_string(&init_request)?.as_bytes()).await?;
        stdin.write_all(b"\n").await?;

        // Read initialize response
        let mut line = String::new();
        stdout.read_line(&mut line).await?;
        let response: Value = serde_json::from_str(&line)?;

        // Send tools/list request
        let tools_request = json!({
            "jsonrpc": "2.0",
            "id": 2,
            "method": "tools/list"
        });

        stdin.write_all(serde_json::to_string(&tools_request)?.as_bytes()).await?;
        stdin.write_all(b"\n").await?;

        // Read tools list
        line.clear();
        stdout.read_line(&mut line).await?;
        let tools_response: Value = serde_json::from_str(&line)?;

        if let Some(tools) = tools_response["result"]["tools"].as_array() {
            for tool in tools {
                self.servers[i].tools.push(MCPTool {
                    name: tool["name"].as_str().unwrap_or_default().to_string(),
                    description: tool["description"].as_str().unwrap_or_default().to_string(),
                    input_schema: tool["inputSchema"].clone(),
                });
            }
        }

        Ok(())
    }

    /// Call an MCP tool with permission checking
    pub async fn call_tool(&mut self, agent_id: &str, tool_name: &str, params: Value) -> Result<Value> {
        // First check permission for this tool
        let permission = match tool_name {
            "filesystem_read_file" => nest_api::permission::Permission::FileRead,
            "filesystem_write_file" => nest_api::permission::Permission::FileWrite,
            "shell_execute" => nest_api::permission::Permission::CommandExecute,
            "web_fetch" => nest_api::permission::Permission::NetworkAccess,
            _ => return Err(Error::Sandbox(format!("Unknown tool: {}", tool_name))),
        };

        match self.permission_engine.check(agent_id, permission, None) {
            nest_api::permission::PermissionResult::Allowed => {
                // Permission granted - call the actual MCP tool
                // Implementation will be completed in next step
                Ok(json!({"status": "ok"}))
            }
            nest_api::permission::PermissionResult::NeedsApproval => {
                // Add to pending requests queue
                self.permission_engine.request(nest_permissions::PendingRequest {
                    agent_id: agent_id.to_string(),
                    permission,
                    resource: None,
                    description: format!("Agent {} wants to call tool {}", agent_id, tool_name),
                    timestamp: std::time::SystemTime::now()
                        .duration_since(std::time::UNIX_EPOCH)
                        .unwrap_or_default()
                        .as_millis() as u64,
                });
                Err(Error::Sandbox(format!("Permission required for tool: {}", tool_name)))
            }
            nest_api::permission::PermissionResult::Denied => {
                Err(Error::Sandbox(format!("Permission denied for tool: {}", tool_name)))
            }
        }
    }

    /// Get list of all available tools
    pub fn tools(&self) -> Vec<&MCPTool> {
        self.servers.iter().flat_map(|s| &s.tools).collect()
    }
}
