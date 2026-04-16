//! Nest - Secure hypervisor for autonomous AI agents

use clap::Parser;
use nest_runtime::AgentRuntime;

#[derive(Parser, Debug)]
#[command(author, version, about, long_about = None)]
struct Cli {
    #[command(subcommand)]
    command: Commands,
}

#[derive(clap::Subcommand, Debug)]
enum Commands {
    /// Start the Nest hypervisor daemon
    Start,

    /// Stop the Nest hypervisor
    Stop,

    /// Show status of all running agents
    Status,

    /// Run an agent
    Run {
        /// Path to agent manifest
        manifest: String,
    },

    /// List all available agents
    List,

    /// Show audit log
    Log {
        /// Number of lines to show
        #[arg(short, long, default_value_t = 20)]
        lines: usize,
    },

    /// Manage permissions
    Permissions {
        #[command(subcommand)]
        command: PermissionCommands,
    },
}

#[derive(clap::Subcommand, Debug)]
enum PermissionCommands {
    /// List pending permission requests
    List,

    /// Approve a pending permission request
    Approve {
        /// Index of the request to approve
        index: usize,
    },

    /// Deny a pending permission request
    Deny {
        /// Index of the request to deny
        index: usize,
    },
}

#[tokio::main]
async fn main() -> anyhow::Result<()> {
    let cli = Cli::parse();

    match &cli.command {
        Commands::Start => {
            println!("Starting Nest hypervisor...");
            let mut runtime = AgentRuntime::new();
            
            // Register example Researcher Hand
            runtime.register_agent("researcher-hand");
            println!("✅ Registered Researcher Hand agent");
            
            runtime.run().await?;
        }
        Commands::Stop => {
            println!("Stopping Nest hypervisor...");
        }
        Commands::Status => {
            println!("Nest hypervisor status:");
            println!("  Status: Running");
            println!("  Agents: 0");
            println!("  Uptime: 0s");
        }
        Commands::Run { manifest } => {
            println!("Running agent from manifest: {}", manifest);
        }
        Commands::List => {
            println!("Available agents:");
        }
        Commands::Log { lines } => {
            println!("Last {} log entries:", lines);
        }
        Commands::Permissions { command } => {
            match command {
                PermissionCommands::List => {
                    println!("Pending permission requests:");
                }
                PermissionCommands::Approve { index } => {
                    println!("Approving permission request {}", index);
                }
                PermissionCommands::Deny { index } => {
                    println!("Denying permission request {}", index);
                }
            }
        }
    }

    Ok(())
}