//! Agent runtime for Nest
//!
//! Implements the core agent execution loop, scheduler, and lifecycle management.

pub mod hand;

pub use hand::{Hand, HandManifest, HandState};

use nest_api::AgentState;
use nest_api::error::{Result, Error};
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

    /// Start the agent runtime main loop
    pub async fn run(&mut self) -> Result<()> {
        println!("🚀 Starting Nest agent runtime...");
        
        // Load all hands from default directory
        let hands_path = std::path::Path::new("./hands");
        if hands_path.exists() {
            if let Err(e) = self.load_hands(hands_path) {
                eprintln!("⚠️  Failed to load hands: {}", e);
            }
        }
        
        println!("✅ Loaded {} hands", self.hands.len());
        
        loop {
            // Process messages from the bus
            self.process_messages().await?;
            
            // Check for pending permission requests
            self.check_pending_requests().await?;
            
            // Small sleep to avoid busy waiting
            tokio::time::sleep(tokio::time::Duration::from_millis(10)).await;
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
}

