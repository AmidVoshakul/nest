//! MCP (Model Context Protocol) client implementation
//!
//! Implements full MCP protocol support with automatic server discovery,
//! tool proxying, and permission checking.

use std::process::Stdio;
use tokio::io::{AsyncBufReadExt, AsyncWriteExt, BufReader};
use tokio::process::Command;

use serde_json::{Value, json};
use nest_api::error::{Result, Error};
use nest_permissions::PermissionEngine;

#[derive(Debug, Clone)]
pub struct MCPTool {
    pub name: String,
    pub description: String,
    pub input_schema: Value,
}

#[derive(Debug)]
pub struct MCPServer {
    pub name: String,
    pub command: String,
    pub tools: Vec<MCPTool>,
    pub stdin: Option<tokio::process::ChildStdin>,
    pub stdout: Option<BufReader<tokio::process::ChildStdout>>,
}

#[derive(Debug)]
pub struct MCPClient {
    permission_engine: PermissionEngine,
    servers: Vec<MCPServer>,
    running_servers: Vec<tokio::process::Child>,
}



impl MCPClient {
    /// Create a new MCP client
    pub fn new(permission_engine: PermissionEngine) -> Self {
        Self {
            permission_engine,
            servers: Vec::new(),
            running_servers: Vec::new(),
        }
    }

    /// Add an MCP server configuration
    pub fn add_server(&mut self, name: &str, command: &str) {
        self.servers.push(MCPServer {
            name: name.to_string(),
            command: command.to_string(),
            tools: Vec::new(),
            stdin: None,
            stdout: None,
        });
    }

    /// Start the MCP client background worker
    pub async fn start(&mut self) -> Result<()> {
        // Auto-discover MCP servers from tools/ directory
        self.discover_servers().await?;
        
        // Start all configured MCP servers
        let len = self.servers.len();
        for i in 0..len {
            self.initialize_server(i).await?;
        }
        Ok(())
    }

    /// Auto-discover MCP servers from tools/ directory
    async fn discover_servers(&mut self) -> Result<()> {
        if let Ok(entries) = std::fs::read_dir("./tools") {
            for entry in entries.flatten() {
                if entry.path().is_dir() {
                    let name = entry.file_name().to_string_lossy().to_string();
                    let command = format!("cargo run --manifest-path ./tools/{}/Cargo.toml", name);
                    self.add_server(&name, &command);
                    println!("✅ Discovered MCP server: {}", name);
                }
            }
        }
        Ok(())
    }

    /// Initialize a single MCP server
    async fn initialize_server(&mut self, i: usize) -> Result<()> {
        let _command = self.servers[i].command.clone();
        let name = self.servers[i].name.clone();
        
        // Use release binary if available, otherwise cargo run
        let release_bin = format!("./tools/{}/target/release/nest-tool-{}", name, name);
        let command = if std::path::Path::new(&release_bin).exists() {
            release_bin
        } else {
            format!("cd tools/{} && cargo run --quiet", name)
        };

        let mut child = Command::new("sh")
            .arg("-c")
            .arg(&command)
            .stdin(Stdio::piped())
            .stdout(Stdio::piped())
            .stderr(Stdio::piped())
            .kill_on_drop(true)
            .spawn()
            .map_err(|e| Error::Sandbox(format!("Failed to start MCP server {}: {}", name, e)))?;

        let mut stdin = child.stdin.take().unwrap();
        let mut stdout = BufReader::new(child.stdout.take().unwrap());

        // Give server time to start
        tokio::time::sleep(tokio::time::Duration::from_millis(500)).await;

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
        stdin.flush().await?;

        // Read initialize response with timeout
        let mut line = String::new();
        match tokio::time::timeout(tokio::time::Duration::from_secs(30), stdout.read_line(&mut line)).await {
            Ok(Ok(0)) => {
                let _ = child.kill().await;
                return Err(Error::Sandbox(format!("MCP server {} exited prematurely", name)));
            }
            Ok(Err(e)) => {
                let _ = child.kill().await;
                return Err(Error::Sandbox(format!("Failed to read from MCP server {}: {}", name, e)));
            }
            Err(_) => {
                let _ = child.kill().await;
                return Err(Error::Sandbox(format!("MCP server {} initialization timed out", name)));
            }
            Ok(Ok(_)) => {}
        }

        // Send tools/list request
        let tools_request = json!({
            "jsonrpc": "2.0",
            "id": 2,
            "method": "tools/list"
        });

        stdin.write_all(serde_json::to_string(&tools_request)?.as_bytes()).await?;
        stdin.write_all(b"\n").await?;
        stdin.flush().await?;

        // We're not killing the child anymore - keep it running
        // Read tools list before pushing into running_servers
        line.clear();
        match tokio::time::timeout(tokio::time::Duration::from_secs(30), stdout.read_line(&mut line)).await {
            Ok(Ok(0)) => {
                let _ = child.kill().await;
                return Err(Error::Sandbox(format!("MCP server {} exited while sending tools", name)));
            }
            Ok(Err(e)) => {
                let _ = child.kill().await;
                return Err(Error::Sandbox(format!("Failed to read tools list from {}: {}", name, e)));
            }
            Err(_) => {
                let _ = child.kill().await;
                return Err(Error::Sandbox(format!("MCP server {} tools list timed out", name)));
            }
            Ok(Ok(_)) => {}
        }

        self.running_servers.push(child);
        
        // Store the stdin/stdout handles for future use
        self.servers[i].stdin = Some(stdin);
        self.servers[i].stdout = Some(stdout);

        let tools_response: Value = match serde_json::from_str(&line) {
            Ok(v) => v,
            Err(e) => {
                return Err(Error::Sandbox(format!("Invalid JSON from MCP server {}: {}", name, e)));
            }
        };

        if let Some(tools) = tools_response["result"]["tools"].as_array() {
            for tool in tools {
                self.servers[i].tools.push(MCPTool {
                    name: tool["name"].as_str().unwrap_or_default().to_string(),
                    description: tool["description"].as_str().unwrap_or_default().to_string(),
                    input_schema: tool["inputSchema"].clone(),
                });
            }
        }

        // Keep server running for subsequent calls
        println!("✅ Initialized MCP server: {} ({} tools)", name, self.servers[i].tools.len());

        Ok(())
    }

    /// Call an MCP tool with permission checking
    pub async fn call_tool(&mut self, agent_id: &str, tool_name: &str, params: Value) -> Result<Value> {
        // First check permission for this tool
        let permission = match tool_name {
            "file_read" | "filesystem_read_file" => nest_api::permission::Permission::FileRead,
            "file_write" | "filesystem_write_file" => nest_api::permission::Permission::FileWrite,
            "shell_execute" => nest_api::permission::Permission::CommandExecute,
            "web_fetch" | "web_search" => nest_api::permission::Permission::NetworkAccess,
            _ => return Err(Error::Sandbox(format!("Unknown tool: {}", tool_name))),
        };

        match self.permission_engine.check(agent_id, permission, None) {
            nest_api::permission::PermissionResult::Allowed => {
                // Find the server that has this tool
                let mut server_index = None;
                for (i, server) in self.servers.iter().enumerate() {
                    if server.tools.iter().any(|t| t.name == tool_name) {
                        server_index = Some(i);
                        break;
                    }
                }

                let server_index = server_index.ok_or_else(|| {
                    Error::Sandbox(format!("Tool {} not found in any MCP server", tool_name))
                })?;

                // Execute the tool call
                self.execute_tool_call(server_index, tool_name, params).await
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

    /// Execute actual tool call against MCP server
    async fn execute_tool_call(&mut self, server_idx: usize, tool_name: &str, params: Value) -> Result<Value> {
        let server = &mut self.servers[server_idx];
        
        let stdin = server.stdin.as_mut().ok_or_else(|| {
            Error::Sandbox(format!("MCP server {} not initialized", server.name))
        })?;
        
        let stdout = server.stdout.as_mut().ok_or_else(|| {
            Error::Sandbox(format!("MCP server {} not initialized", server.name))
        })?;

        // Send tools/call request
        let request = json!({
            "jsonrpc": "2.0",
            "id": 1,
            "method": "tools/call",
            "params": {
                "name": tool_name,
                "arguments": params
            }
        });

        stdin.write_all(serde_json::to_string(&request)?.as_bytes()).await?;
        stdin.write_all(b"\n").await?;
        stdin.flush().await?;

        // Read response
        let mut line = String::new();
        stdout.read_line(&mut line).await?;
        
        let response: Value = serde_json::from_str(&line)
            .map_err(|e| Error::Sandbox(format!("Failed to parse tool response: {}", e)))?;

        if let Some(error) = response.get("error") {
            return Err(Error::Sandbox(format!("Tool error: {}", error)));
        }

        Ok(response["result"].clone())
    }

    /// Get list of all available tools
    pub fn tools(&self) -> Vec<&MCPTool> {
        self.servers.iter().flat_map(|s| &s.tools).collect()
    }
}
