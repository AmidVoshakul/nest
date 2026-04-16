//! Simple demonstration agent that shows Nest in action
//!
//! This is the first working agent that demonstrates:
//! - Permission request flow
//! - Sandbox isolation
//! - Audit logging

use clap::Parser;
use nest_runtime::AgentRuntime;

#[derive(Parser, Debug)]
#[command(author, version, about, long_about = None)]
struct DemoCli {
    #[command(subcommand)]
    command: DemoCommands,
}

#[derive(clap::Subcommand, Debug)]
enum DemoCommands {
    /// Run a simple demo agent
    Run {
        /// Topic to research
        topic: String,
    },
    
    /// Show pending permission requests
    Requests,
    
    /// Approve a permission request
    Approve {
        index: usize,
    },
    
    /// Deny a permission request
    Deny {
        index: usize,
    },
}

#[tokio::main]
async fn main() -> anyhow::Result<()> {
    let cli = DemoCli::parse();
    let mut runtime = AgentRuntime::new();
    
    match &cli.command {
        DemoCommands::Run { topic } => {
            println!("🤖 Starting demo agent for topic: {}", topic);
            runtime.register_agent("demo-agent");
            
            // This will automatically request permissions when needed
            let result = runtime.execute_tool(
                "demo-agent", 
                "web_search", 
                serde_json::json!({"query": topic})
            ).await;
            
            match result {
                Ok(_) => println!("✅ Tool executed successfully"),
                Err(e) => println!("❌ Tool execution failed: {}", e),
            }
        }
        DemoCommands::Requests => {
            let pending = runtime.pending_permissions();
            println!("📋 Pending permission requests: {}", pending.len());
            for (i, req) in pending.iter().enumerate() {
                println!("  {}: {}", i, req.description);
            }
        }
        DemoCommands::Approve { index } => {
            if runtime.approve_permission(*index) {
                println!("✅ Permission request {} approved", index);
            } else {
                println!("❌ Invalid request index");
            }
        }
        DemoCommands::Deny { index } => {
            if runtime.deny_permission(*index) {
                println!("❌ Permission request {} denied", index);
            } else {
                println!("❌ Invalid request index");
            }
        }
    }
    
    Ok(())
}
