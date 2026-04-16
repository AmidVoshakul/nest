//! Nest MCP Server: Web Search
//!
//! Exposes web_search tool via MCP protocol.
//! Supports DuckDuckGo (zero-config), Tavily, Brave, Perplexity, SearXNG.

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
struct SearchParams {
    query: String,
    #[serde(default = "default_max_results")]
    max_results: usize,
}

fn default_max_results() -> usize { 20 }

const PROTOCOL_VERSION: &str = "2024-11-05";

#[tokio::main]
async fn main() -> anyhow::Result<()> {
    let mut reader = BufReader::new(tokio::io::stdin());
    let mut line = String::new();
    let mut stdout = tokio::io::stdout();

    loop {
        line.clear();
        let bytes_read = reader.read_line(&mut line).await?;
        if bytes_read == 0 {
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
                            "name": "nest-web-search",
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
                                "name": "web_search",
                                "description": "Search the web for information",
                                "inputSchema": {
                                    "type": "object",
                                    "properties": {
                                        "query": {
                                            "type": "string",
                                            "description": "Search query"
                                        },
                                        "max_results": {
                                            "type": "number",
                                            "description": "Maximum number of results to return",
                                            "default": 20
                                        }
                                    },
                                    "required": ["query"]
                                }
                            }
                        ]
                    })
                }
            }
            "tools/call" => {
                let params = request.params.unwrap_or_default();
                let tool_name = params["name"].as_str().unwrap_or_default();

                if tool_name != "web_search" {
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
                    let search_params: SearchParams = serde_json::from_value(args).unwrap_or_else(|_| SearchParams {
                        query: "".into(),
                        max_results: 10,
                    });

                    // DuckDuckGo HTML search (zero-config)
                    let result = search_duckduckgo(&search_params.query, search_params.max_results).await;

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
                                    message: format!("Search failed: {}", e)
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

        let resp_str = serde_json::to_string(&response)?;
        stdout.write_all(resp_str.as_bytes()).await?;
        stdout.write_all(b"\n").await?;
        stdout.flush().await?;
    }

    Ok(())
}

async fn search_duckduckgo(query: &str, max_results: usize) -> Result<String, String> {
    let client = reqwest::Client::builder()
        .timeout(std::time::Duration::from_secs(15))
        .build()
        .map_err(|e| e.to_string())?;

    let resp = client
        .get("https://html.duckduckgo.com/html/")
        .query(&[("q", query)])
        .header("User-Agent", "Mozilla/5.0 (compatible; NestAgent/0.1)")
        .send()
        .await
        .map_err(|e| format!("DuckDuckGo request failed: {e}"))?;

    let body = resp
        .text()
        .await
        .map_err(|e| format!("Failed to read response: {e}"))?;

    let results = parse_ddg_results(&body, max_results);

    if results.is_empty() {
        return Err(format!("No results found for '{query}'."));
    }

    let mut output = format!("Search results for '{query}':\n\n");
    for (i, (title, url, snippet)) in results.iter().enumerate() {
        output.push_str(&format!(
            "{}. {}\n   URL: {}\n   {}\n\n",
            i + 1,
            title,
            url,
            snippet
        ));
    }

    Ok(output)
}

fn parse_ddg_results(html: &str, max: usize) -> Vec<(String, String, String)> {
    let mut results = Vec::new();

    for chunk in html.split("class=\"result__a\"") {
        if results.len() >= max {
            break;
        }
        if !chunk.contains("href=") {
            continue;
        }

        let url = extract_between(chunk, "href=\"", "\"")
            .unwrap_or_default()
            .to_string();

        let actual_url = if url.contains("uddg=") {
            url.split("uddg=")
                .nth(1)
                .and_then(|u| u.split('&').next())
                .map(urldecode)
                .unwrap_or(url)
        } else {
            url
        };

        let title = extract_between(chunk, ">", "</a>")
            .map(strip_html_tags)
            .unwrap_or_default();

        let snippet = if let Some(snip_start) = chunk.find("class=\"result__snippet\"") {
            let after = &chunk[snip_start..];
            extract_between(after, ">", "</a>")
                .or_else(|| extract_between(after, ">", "</"))
                .map(strip_html_tags)
                .unwrap_or_default()
        } else {
            String::new()
        };

        if !title.is_empty() && !actual_url.is_empty() {
            // Validate URL before including in results
            if validate_url(&actual_url).is_ok() {
                results.push((title, actual_url, snippet));
            }
        }
    }

    results
}

fn extract_between<'a>(text: &'a str, start: &str, end: &str) -> Option<&'a str> {
    let start_idx = text.find(start)? + start.len();
    let remaining = &text[start_idx..];
    let end_idx = remaining.find(end)?;
    Some(&remaining[..end_idx])
}

fn strip_html_tags(s: &str) -> String {
    let mut result = String::with_capacity(s.len());
    let mut in_tag = false;
    for ch in s.chars() {
        match ch {
            '<' => in_tag = true,
            '>' => in_tag = false,
            _ if !in_tag => result.push(ch),
            _ => {}
        }
    }
    result
        .replace("&amp;", "&")
        .replace("&lt;", "<")
        .replace("&gt;", ">")
        .replace("&quot;", "\"")
        .replace("&#x27;", "'")
        .replace("&nbsp;", " ")
        .replace("&#39;", "'")
}

fn urldecode(s: &str) -> String {
    let mut result = String::with_capacity(s.len());
    let mut chars = s.chars();
    while let Some(ch) = chars.next() {
        if ch == '%' {
            let hex: String = chars.by_ref().take(2).collect();
            if let Ok(byte) = u8::from_str_radix(&hex, 16) {
                result.push(byte as char);
            } else {
                result.push('%');
                result.push_str(&hex);
            }
        } else if ch == '+' {
            result.push(' ');
        } else {
            result.push(ch);
        }
    }
    result
}
