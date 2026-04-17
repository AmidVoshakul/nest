//! Agent runtime for Nest
//!
//! Implements the core agent execution loop, scheduler, and lifecycle management.

pub mod hand;
pub mod scheduler;
pub mod loop_guard;
pub mod depth_guard;

pub use hand::{Hand, HandManifest, HandState};
pub use scheduler::Scheduler;
pub use loop_guard::{LoopGuard, LoopGuardConfig, LoopGuardVerdict};
pub use depth_guard::{DepthGuard, DepthGuardConfig};

use nest_api::AgentState;
use nest_api::error::Result;
use nest_messaging::MessageBus;
use nest_permissions::PermissionEngine;
use nest_tools::MCPProxy;
use serde_json::Value;
use std::collections::HashMap;

pub struct AgentRuntime {
    message_bus: MessageBus,
    permission_engine: PermissionEngine,
    mcp_proxy: MCPProxy,
    agents: HashMap<String, AgentState>,
    hands: HashMap<String, hand::Hand>,
    scheduler: scheduler::Scheduler,
}

impl AgentRuntime {
    /// Create a new agent runtime
    pub fn new() -> Self {
        let permission_engine = PermissionEngine::new();
        Self {
            message_bus: MessageBus::new(),
            mcp_proxy: MCPProxy::new(permission_engine.clone()),
            permission_engine,
            agents: HashMap::new(),
            hands: HashMap::new(),
            scheduler: scheduler::Scheduler::new(),
        }
    }

    /// Load all Hand agents from directory
    pub fn load_hands(&mut self, path: &std::path::Path) -> Result<()> {
        if path.is_dir() {
            for entry in std::fs::read_dir(path)? {
                let entry = entry?;
                let path = entry.path();
                
                if path.extension().and_then(|e| e.to_str()) == Some("toml") {
                    match hand::Hand::from_file(path.clone()) {
                        Ok(hand) => {
                            let name = hand.manifest().name.clone();
                            println!("✅ Loaded hand: {}", name);
                            self.hands.insert(name, hand);
                        }
                        Err(e) => {
                            eprintln!("❌ Failed to load hand {}: {}", path.display(), e);
                        }
                    }
                }
            }
        }
        Ok(())
    }

    /// Run one tick of the runtime
    pub async fn tick(&mut self) -> Result<()> {
        // Check for scheduled tasks that are due
        let due_tasks = self.scheduler.check_due_tasks();
        for task in due_tasks {
            eprintln!("⏰ Scheduled task due: {}", task.id);
            self.submit_task(&task.hand_name, task.task);
        }
        
        // Run think cycles for all hands
        for (name, hand) in self.hands.iter_mut() {
            if let Err(e) = hand.think_cycle().await {
                eprintln!("[{}] Think cycle error: {}", name, e);
            }
        }
        
        // Process messages from the bus
        self.process_messages().await?;
        
        // Check for pending permission requests
        self.check_pending_requests().await?;
        
        Ok(())
    }

    /// Start the agent runtime main loop
    pub async fn run(&mut self) -> Result<()> {
        println!("🚀 Starting Nest agent runtime...");
        
        // Load all hands FIRST before doing anything else
        let hands_path = std::path::Path::new("./hands");
        if hands_path.exists() {
            if let Err(e) = self.load_hands(hands_path) {
                eprintln!("⚠️  Failed to load hands: {}", e);
            }
        }
        
        println!("✅ Loaded {} hands", self.hands.len());
        
        // Start MCP client and discover tools
        if let Err(e) = self.mcp_proxy.start().await {
            eprintln!("⚠️  Failed to start MCP client: {}", e);
        }
        
        loop {
            self.tick().await?;
            
            // Small sleep to avoid busy waiting
            tokio::time::sleep(tokio::time::Duration::from_millis(500)).await;
        }
    }

    /// Process incoming messages from the message bus
    async fn process_messages(&mut self) -> Result<()> {
        for agent_id in self.message_bus.agents() {
            while let Some(message) = self.message_bus.recv(&agent_id) {
                // Handle message based on type
                match message.message_type {
                    nest_api::message::MessageType::Text => {
                        // Process text message
                    }
                    nest_api::message::MessageType::PermissionRequest => {
                        // This will be handled by the approval system
                    }
                    _ => {}
                }
            }
        }
        Ok(())
    }

    /// Check for pending permission requests that need user attention
    async fn check_pending_requests(&mut self) -> Result<()> {
        let pending = self.mcp_proxy.pending_requests();
        if !pending.is_empty() {
            // Emit event or signal that user attention is needed
            // For now just log
            eprintln!("Pending permission requests: {}", pending.len());
        }
        Ok(())
    }

    /// Execute a tool call through the proxy with permission checks
    pub async fn execute_tool(&mut self, agent_id: &str, tool_name: &str, params: Value) -> Result<Value> {
        self.mcp_proxy.call_tool(agent_id, tool_name, params).await
    }

    /// Approve a pending permission request
    pub fn approve_permission(&mut self, index: usize) -> bool {
        self.mcp_proxy.approve_request(index)
    }

    /// Deny a pending permission request
    pub fn deny_permission(&mut self, index: usize) -> bool {
        self.mcp_proxy.deny_request(index)
    }
    
    /// Submit a task to a hand agent
    pub fn submit_task(&mut self, hand_name: &str, task: String) -> bool {
        if let Some(hand) = self.hands.get_mut(hand_name) {
            eprintln!("✅ [Runtime] Submitting task to {}: {}", hand_name, task);
            hand.submit_task(task);
            eprintln!("✅ [Runtime] Task submitted successfully");
            true
        } else {
            eprintln!("❌ [Runtime] Hand {} not found", hand_name);
            false
        }
    }

    /// Get pending permission requests
    pub fn pending_permissions(&self) -> &[nest_permissions::PendingRequest] {
        self.mcp_proxy.pending_requests()
    }

    /// Register an agent with the runtime
    pub fn register_agent(&mut self, agent_id: &str) {
        self.message_bus.register(agent_id);
        self.agents.insert(agent_id.to_string(), AgentState::Stopped);
    }

    /// Unregister an agent from the runtime
    pub fn unregister_agent(&mut self, agent_id: &str) {
        self.message_bus.unregister(agent_id);
        self.agents.remove(agent_id);
    }
    
    /// Schedule a new recurring task
    pub fn schedule_task(&mut self, task: nest_api::scheduler::ScheduledTask) -> Result<()> {
        self.scheduler.schedule_task(task).map_err(|e| nest_api::error::Error::Unknown(e.to_string()))
    }
    
    /// Unschedule a task
    pub fn unschedule_task(&mut self, task_id: &str) -> Option<nest_api::scheduler::ScheduledTask> {
        self.scheduler.unschedule_task(task_id)
    }
    
    /// Get all scheduled tasks
    pub fn scheduled_tasks(&self) -> &std::collections::HashMap<String, nest_api::scheduler::ScheduledTask> {
        self.scheduler.tasks()
    }
}
