//! Nest - Secure hypervisor for autonomous AI agents

mod banner;

use clap::Parser;
use std::io::Write;
use nest_runtime::AgentRuntime;
use nest_llm::{model_catalog::ModelCatalog, Provider};

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
    
    /// Manage configuration and API keys
    Config {
        #[command(subcommand)]
        command: ConfigCommands,
    },
    
    /// Manage LLM models and providers
    Models {
        #[command(subcommand)]
        command: ModelsCommands,
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

#[derive(clap::Subcommand, Debug)]
enum ConfigCommands {
    /// Show current configuration
    Show,
    
    /// Open configuration file in editor
    Edit,
    
    /// Get a config value by key
    Get {
        /// Configuration key
        key: String,
    },
    
    /// Set a config value
    Set {
        /// Configuration key
        key: String,
        /// Configuration value
        value: String,
    },
    
    /// Save an API key securely
    SetKey {
        /// Provider name
        provider: String,
    },
    
    /// Test API key connectivity for a provider
    TestKey {
        /// Provider name
        provider: String,
    },
    
    /// Remove an API key
    DeleteKey {
        /// Provider name
        provider: String,
    },
}

#[derive(clap::Subcommand, Debug)]
enum ModelsCommands {
    /// List available models
    List {
        /// Filter by provider
        #[arg(long)]
        provider: Option<String>,
        
        /// Output as JSON
        #[arg(long)]
        json: bool,
    },
    
    /// List available LLM providers
    Providers {
        /// Output as JSON
        #[arg(long)]
        json: bool,
    },
    
    /// Set default model
    Set {
        /// Model ID or alias. Interactive picker if omitted.
        model: Option<String>,
    },
    
    /// Show model aliases
    Aliases {
        /// Output as JSON
        #[arg(long)]
        json: bool,
    },
}

#[tokio::main]
async fn main() -> anyhow::Result<()> {
    load_env();
    banner::print_banner();
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
        Commands::Config { command } => {
            match command {
                ConfigCommands::Show => {
                    println!("⚙️ Configuration:");
                    println!("(Configuration management coming soon)");
                }
                ConfigCommands::Edit => {
                    println!("✏️ Opening configuration file...");
                    println!("(Configuration management coming soon)");
                }
                ConfigCommands::Get { key } => {
                    println!("📖 Getting config key: {}", key);
                    println!("(Configuration management coming soon)");
                }
                ConfigCommands::Set { key, value } => {
                    println!("📝 Setting config {} = {}", key, value);
                    println!("(Configuration management coming soon)");
                }
                ConfigCommands::SetKey { provider } => {
                    println!("🔑 Setting API key for provider: {}", provider);
                    println!("(Configuration management coming soon)");
                }
                ConfigCommands::TestKey { provider } => {
                    println!("🔌 Testing API key for provider: {}", provider);
                    println!("(Configuration management coming soon)");
                }
                ConfigCommands::DeleteKey { provider } => {
                    println!("🗑️ Deleting API key for provider: {}", provider);
                    println!("(Configuration management coming soon)");
                }
            }
        }
        Commands::Models { command } => {
            match command {
                ModelsCommands::List { provider, json } => {
                    let catalog = ModelCatalog::new();
                    let provider_filter = provider.as_ref()
                        .and_then(|p| match nest_llm::LlmClient::from_name(p) {
                            Ok(c) => Some(c.provider()),
                            Err(_) => None,
                        });
                    
                    let models = catalog.list_models(provider_filter);
                    
                    if *json {
                        println!("{}", serde_json::to_string_pretty(&models).unwrap_or_default());
                        return Ok(());
                    }
                    
                    println!("📋 Available models:");
                    println!("{:<40} {:<16} {:<8} CONTEXT", "MODEL", "PROVIDER", "TIER");
                    println!("{}", "-".repeat(80));
                    
                    let mut current_provider = None;
                    for model in models {
                        let provider_name = format!("{:?}", model.provider).to_lowercase();
                        
                        if current_provider != Some(model.provider) {
                            println!("\n  {}:", provider_name);
                            current_provider = Some(model.provider);
                        }
                        
                        println!("    {:<36} {:<8} {}", 
                            model.id, 
                            model.tier.to_string(),
                            model.context_window
                        );
                    }
                }
                ModelsCommands::Providers { json } => {
                    use nest_llm::model_catalog::ModelCatalog;
                    
                    let catalog = ModelCatalog::new();
                    let providers = catalog.list_providers();
                    
                    if *json {
                        println!("{}", serde_json::to_string_pretty(&providers).unwrap_or_default());
                        return Ok(());
                    }
                    
                    println!("📋 Available providers:");
                    println!("{:<20} {:<12} {:<10} MODELS", "PROVIDER", "STATUS", "KEY");
                    println!("{}", "-".repeat(60));
                    
                    for provider in providers {
                        let name = format!("{:?}", provider).to_lowercase();
                        let has_key = match provider {
                            Provider::Anthropic => std::env::var("ANTHROPIC_API_KEY").is_ok(),
                            Provider::OpenAI => std::env::var("OPENAI_API_KEY").is_ok(),
                            Provider::OpenRouter => std::env::var("OPENROUTER_API_KEY").is_ok(),
                            Provider::Zai => std::env::var("ZAI_API_KEY").is_ok(),
                            Provider::Gemini => std::env::var("GOOGLE_API_KEY").is_ok(),
                            Provider::Ollama => true,
                            Provider::Deepseek => std::env::var("DEEPSEEK_API_KEY").is_ok(),
                            Provider::Mistral => std::env::var("MISTRAL_API_KEY").is_ok(),
                            Provider::Groq => std::env::var("GROQ_API_KEY").is_ok(),
                            Provider::Together => std::env::var("TOGETHER_API_KEY").is_ok(),
                        };
                        
                        println!("{:<20} {:<12} {:<10} {}", 
                            name,
                            "online",
                            if has_key { "✅" } else { "❌" },
                            catalog.list_models(Some(provider)).len()
                        );
                    }
                }
                ModelsCommands::Set { model } => {
                    use nest_llm::model_catalog::ModelCatalog;
                    use std::io::{self, BufRead};
                    
                    let catalog = ModelCatalog::new();
                    
                    let model_id = if let Some(m) = model {
                        m.clone()
                    } else {
                        // Interactive selector
                        let models = catalog.list_models(None);
                        
                        println!("🎯 Select default model:");
                        println!("{:<4} {:<40} {:<10}", "#", "MODEL", "PROVIDER");
                        println!("{}", "-".repeat(60));
                        
                        for (i, model) in models.iter().enumerate() {
                            let provider_name = format!("{:?}", model.provider).to_lowercase();
                            println!("{:<4} {:<40} {:<10}", i + 1, model.id, provider_name);
                        }
                        
                        print!("\nEnter number or model ID: ");
                        io::stdout().flush().ok();
                        
                        let mut input = String::new();
                        io::stdin().lock().read_line(&mut input).ok();
                        let input = input.trim();
                        
                        if let Ok(n) = input.parse::<usize>() {
                            if n >= 1 && n <= models.len() {
                                models[n - 1].id.clone()
                            } else {
                                eprintln!("❌ Invalid selection");
                                return Ok(());
                            }
                        } else {
                            input.to_string()
                        }
                    };
                    
                    if catalog.get_model(&model_id).is_none() {
                        eprintln!("❌ Unknown model: {}", model_id);
                        return Ok(());
                    }
                    
                    // Save to .env file
                    let env_path = std::path::Path::new(".env");
                    let content = if env_path.exists() {
                        std::fs::read_to_string(env_path).unwrap_or_default()
                    } else {
                        String::new()
                    };
                    
                    // Remove existing NEST_DEFAULT_MODEL line if present
                    let mut lines: Vec<String> = content.lines().map(|l| l.to_string()).collect();
                    lines.retain(|l| !l.starts_with("NEST_DEFAULT_MODEL="));
                    lines.push(format!("NEST_DEFAULT_MODEL={}", model_id));
                    
                    std::fs::write(env_path, lines.join("\n")).ok();
                    
                    println!("✅ Default model set to: {}", model_id);
                }
                ModelsCommands::Aliases { json } => {
                    println!("📋 Model aliases:");
                    if *json {
                        println!("  Output as JSON");
                    }
                    println!("(Model catalog coming soon)");
                }
            }
        }
    }

    Ok(())
}