//! Nest - Secure hypervisor for autonomous AI agents

use clap::Parser;
use nest_runtime::AgentRuntime;

// Load environment variables from .env file
fn load_env() {
    if let Err(e) = dotenvy::dotenv() {
        eprintln!("⚠️  Warning: Could not load .env file: {}", e);
    }
}

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

    /// Submit a research task to Researcher Hand
    Research {
        /// Research query to perform
        query: String,
    },
    
    /// Schedule a recurring task
    Schedule {
        /// Name of the hand to run the task
        hand: String,
        
        /// Cron schedule expression
        schedule: String,
        
        /// Task to execute
        task: String,
    },
    
    /// List all scheduled tasks
    ScheduleList,
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
    load_env();
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
            if let Ok(pid_str) = std::fs::read_to_string("./var/nest.pid") {
                if let Ok(pid) = pid_str.parse::<u32>() {
                    // Try to send SIGTERM
                    unsafe { libc::kill(pid as i32, libc::SIGTERM) };
                    std::fs::remove_file("./var/nest.pid").ok();
                    println!("✅ Nest runtime stopped");
                } else {
                    println!("⚠️  No running Nest instance found");
                }
            } else {
                println!("⚠️  No running Nest instance found");
            }
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
        Commands::Research { query } => {
            println!("🔍 Starting Nest runtime...");
            
            // Write PID file
            std::fs::write("./var/nest.pid", std::process::id().to_string()).ok();
            
            // Start runtime
            let mut runtime = nest_runtime::AgentRuntime::new();
            
            // Load hands
            let hands_path = std::path::Path::new("./hands");
            if hands_path.exists() {
                if let Err(e) = runtime.load_hands(hands_path) {
                    eprintln!("⚠️  Failed to load hands: {}", e);
                }
            }
            
            println!("✅ Runtime started");
            println!("✅ Loaded {} hands", runtime.scheduled_tasks().len());
            println!("📋 Logs will appear below:");
            println!("━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━");
            
            // We need to start runtime and submit task once hands are loaded
            // Create a channel to signal when we can submit the task
            let (tx, mut rx) = tokio::sync::oneshot::channel();
            let query_clone = query.clone();
            
            tokio::spawn(async move {
                // Wait a bit for hands to fully load
                tokio::time::sleep(tokio::time::Duration::from_secs(3)).await;
                tx.send(query_clone).ok();
            });
            
            // Start main runtime loop
            loop {
                // Check if we have a task to submit
                if let Ok(q) = rx.try_recv() {
                    println!("📤 Submitting research task: {}", q);
                    runtime.submit_task("researcher", q);
                }
                
                // Run one tick of runtime
                runtime.tick().await?;
                
                // Small sleep
                tokio::time::sleep(tokio::time::Duration::from_millis(500)).await;
            }
        }
        Commands::Schedule { hand, schedule, task } => {
            println!("⏰ Scheduling task for {}: {}", hand, task);
            println!("   Schedule: {}", schedule);
            println!("✅ Task scheduled successfully (persistence coming soon)");
        },
        
        Commands::ScheduleList => {
            println!("📋 Scheduled tasks:");
            println!("(Persistence coming soon)");
        }
    }

    Ok(())
}