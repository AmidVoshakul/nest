//! Simple test binary to verify MCP client works end-to-end

use nest_tools::MCPClient;
use nest_permissions::PermissionEngine;
use serde_json::json;

#[tokio::main]
async fn main() -> anyhow::Result<()> {
    println!("🔍 Testing MCP client end-to-end");
    
    let mut permission_engine = PermissionEngine::new();
    permission_engine.set_auto_approve(true);
    
    let mut client = MCPClient::new(permission_engine);
    
    println!("✅ Created MCP client");
    
    // Start client and discover servers
    println!("🔍 Starting MCP client and discovering servers...");
    client.start().await?;
    
    println!("✅ MCP client started successfully");
    
    // List discovered tools
    let tools = client.tools();
    println!("\n📋 Discovered tools:");
    for tool in tools {
        println!("  - {}: {}", tool.name, tool.description);
    }
    
    println!("\n✅ MCP client test completed successfully");
    
    Ok(())
}