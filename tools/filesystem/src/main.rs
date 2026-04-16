//! Nest MCP Server: Filesystem
//!
//! Exposes file_read, file_write, file_list tools via MCP protocol.

use serde::{Deserialize, Serialize};
use serde_json::{json, Value};
use tokio::io::{AsyncBufReadExt, AsyncWriteExt, BufReader};
use walkdir::WalkDir;

#[derive(Serialize, Deserialize, Debug)]
struct MCPRequest {
    jsonrpc: String,
    id: Option<Value>,
    method: String,
    params: Option<Value>,
}

#[derive(Serialize, Deserialize, Debug)]
#[serde(untagged)]
enum MCPResponse {
    Result {
        jsonrpc: String,
        id: Option<Value>,
        result: Value,
    },
    Error {
        jsonrpc: String,
        id: Option<Value>,
        error: MCPError,
    },
}

#[derive(Serialize, Deserialize, Debug)]
struct MCPError {
    code: i32,
    message: String,
}

const PROTOCOL_VERSION: &str = "2024-11-05";

#[tokio::main]
async fn main() -> anyhow::Result<()> {
    let mut reader = BufReader::new(tokio::io::stdin());
    let mut stdout = tokio::io::stdout();
    let mut line = String::new();

    loop {
        line.clear();
        if reader.read_line(&mut line).await? == 0 {
            break;
        }

        let request: MCPRequest = match serde_json::from_str(&line) {
            Ok(r) => r,
            Err(_) => continue,
        };

        let response = match request.method.as_str() {
            "initialize" => {
                MCPResponse::Result {
                    jsonrpc: "2.0".into(),
                    id: request.id,
                    result: json!({
                        "protocolVersion": PROTOCOL_VERSION,
                        "capabilities": { "tools": {} },
                        "serverInfo": {
                            "name": "nest-filesystem",
                            "version": env!("CARGO_PKG_VERSION")
                        }
                    })
                }
            }
            "notifications/initialized" => continue,
            "tools/list" => {
                MCPResponse::Result {
                    jsonrpc: "2.0".into(),
                    id: request.id,
                    result: json!({
                        "tools": [
                            {
                                "name": "file_read",
                                "description": "Read content from a file",
                                "inputSchema": {
                                    "type": "object",
                                    "properties": {
                                        "path": {
                                            "type": "string",
                                            "description": "Path to file"
                                        }
                                    },
                                    "required": ["path"]
                                }
                            },
                            {
                                "name": "file_write",
                                "description": "Write content to a file",
                                "inputSchema": {
                                    "type": "object",
                                    "properties": {
                                        "path": {
                                            "type": "string",
                                            "description": "Path to file"
                                        },
                                        "content": {
                                            "type": "string",
                                            "description": "Content to write"
                                        }
                                    },
                                    "required": ["path", "content"]
                                }
                            },
                            {
                                "name": "file_list",
                                "description": "List files in a directory",
                                "inputSchema": {
                                    "type": "object",
                                    "properties": {
                                        "path": {
                                            "type": "string",
                                            "description": "Directory path"
                                        }
                                    },
                                    "required": ["path"]
                                }
                            }
                        ]
                    })
                }
            }
            "tools/call" => {
                let params = request.params.unwrap_or_default();
                let tool_name = params["name"].as_str().unwrap_or_default();
                let args = params["arguments"].clone();

                let result = match tool_name {
                    "file_read" => file_read(args).await,
                    "file_write" => file_write(args).await,
                    "file_list" => file_list(args).await,
                    _ => Err(format!("Unknown tool: {}", tool_name)),
                };

                match result {
                    Ok(content) => {
                        MCPResponse::Result {
                            jsonrpc: "2.0".into(),
                            id: request.id,
                            result: json!({
                                "content": [{
                                    "type": "text",
                                    "text": content
                                }]
                            })
                        }
                    }
                    Err(e) => {
                        MCPResponse::Error {
                            jsonrpc: "2.0".into(),
                            id: request.id,
                            error: MCPError {
                                code: -32603,
                                message: format!("Tool execution failed: {}", e)
                            }
                        }
                    }
                }
            }
            _ => {
                MCPResponse::Error {
                    jsonrpc: "2.0".into(),
                    id: request.id,
                    error: MCPError {
                        code: -32601,
                        message: format!("Method not found: {}", request.method)
                    }
                }
            }
        };

        stdout.write_all(serde_json::to_string(&response)?.as_bytes()).await?;
        stdout.write_all(b"\n").await?;
        stdout.flush().await?;
    }

    Ok(())
}

async fn file_read(params: Value) -> Result<String, String> {
    let path = params["path"].as_str().ok_or("Missing path parameter")?;
    tokio::fs::read_to_string(path)
        .await
        .map_err(|e| format!("Failed to read file: {e}"))
}

async fn file_write(params: Value) -> Result<String, String> {
    let path = params["path"].as_str().ok_or("Missing path parameter")?;
    let content = params["content"].as_str().ok_or("Missing content parameter")?;
    
    tokio::fs::write(path, content)
        .await
        .map_err(|e| format!("Failed to write file: {e}"))?;
    
    Ok(format!("Successfully wrote to {}", path))
}

async fn file_list(params: Value) -> Result<String, String> {
    let path = params["path"].as_str().ok_or("Missing path parameter")?;
    
    let mut output = format!("Contents of {}:\n\n", path);
    
    for entry in WalkDir::new(path).max_depth(1) {
        match entry {
            Ok(e) => {
                let name = e.file_name().to_string_lossy();
                let ty = if e.file_type().is_dir() { "DIR" } else { "FILE" };
                output.push_str(&format!("[{}] {}\n", ty, name));
            }
            Err(e) => output.push_str(&format!("ERROR: {}\n", e)),
        }
    }
    
    Ok(output)
}
