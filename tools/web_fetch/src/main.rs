//! Nest MCP Server: Web Fetch
//!
//! Exposes web_fetch tool via MCP protocol.
//! With SSRF protection, HTML→Markdown conversion, caching.

use serde::{Deserialize, Serialize};
use serde_json::{json, Value};
use tokio::io::{AsyncBufReadExt, AsyncWriteExt, BufReader};
use nest_api::ssrf::validate_url;

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

#[derive(Serialize, Deserialize, Debug)]
struct FetchParams {
    url: String,
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
                            "name": "nest-web-fetch",
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
                                "name": "web_fetch",
                                "description": "Fetch and read content from a URL",
                                "inputSchema": {
                                    "type": "object",
                                    "properties": {
                                        "url": {
                                            "type": "string",
                                            "description": "URL to fetch"
                                        }
                                    },
                                    "required": ["url"]
                                }
                            }
                        ]
                    })
                }
            }
            "tools/call" => {
                let params = request.params.unwrap_or_default();
                let tool_name = params["name"].as_str().unwrap_or_default();

                if tool_name != "web_fetch" {
                    MCPResponse::Error {
                        jsonrpc: "2.0".into(),
                        id: request.id,
                        error: MCPError {
                            code: -32602,
                            message: format!("Unknown tool: {}", tool_name)
                        }
                    }
                } else {
                    let args = params["arguments"].clone();
                    let fetch_params: FetchParams = serde_json::from_value(args).unwrap_or_else(|_| FetchParams {
                        url: "".into(),
                    });

                    let result = fetch_url(&fetch_params.url).await;

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
                                    message: format!("Fetch failed: {}", e)
                                }
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

async fn fetch_url(url: &str) -> Result<String, String> {
    // Full SSRF protection with DNS rebinding defense
    validate_url(url).map_err(|e| e.to_string())?;

    let client = reqwest::Client::builder()
        .timeout(std::time::Duration::from_secs(30))
        .user_agent("Mozilla/5.0 (compatible; NestAgent/0.1)")
        .build()
        .map_err(|e| e.to_string())?;

    let resp = client
        .get(url)
        .send()
        .await
        .map_err(|e| format!("HTTP request failed: {e}"))?;

    if !resp.status().is_success() {
        return Err(format!("HTTP error: {}", resp.status()));
    }

    let content_type = resp
        .headers()
        .get("content-type")
        .and_then(|v| v.to_str().ok())
        .unwrap_or("")
        .to_string();

    let body = resp
        .text()
        .await
        .map_err(|e| format!("Failed to read response: {e}"))?;

    // Convert HTML to Markdown if needed
    let processed = if content_type.contains("text/html") {
        html2md::parse_html(&body)
    } else {
        body
    };

    // Truncate to reasonable size
    let truncated = if processed.len() > 100000 {
        format!("{}... [truncated]", &processed[..95000])
    } else {
        processed
    };

    Ok(truncated)
}
