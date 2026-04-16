//! Hand agent system for Nest
//!
//! Implements the production-proven Hand/Agent pattern from OpenFang.
//! Agents are defined via simple TOML manifests and run in isolated sandboxes.

use serde::{Serialize, Deserialize};
use std::path::PathBuf;
use nest_tools::MCPProxy;
use nest_permissions::PermissionEngine;

#[derive(Debug, Clone, serde::Serialize, serde::Deserialize)]
pub struct HandManifest {
    pub name: String,
    pub version: String,
    pub description: String,
    pub author: String,
    
    #[serde(default)]
    pub tags: Vec<String>,

    #[serde(default)]
    pub model: HandModelConfig,

    #[serde(default)]
    pub resources: HandResourceLimits,

    #[serde(default)]
    pub capabilities: HandCapabilities,

    pub system_prompt: String,
}

#[derive(Debug, Clone, serde::Serialize, serde::Deserialize, Default)]
pub struct HandModelConfig {
    pub provider: String,
    pub model: String,
    #[serde(default)]
    pub max_tokens: u32,
    #[serde(default)]
    pub temperature: f32,
    #[serde(default)]
    pub api_key_env: Option<String>,
}

#[derive(Debug, Clone, serde::Serialize, serde::Deserialize, Default)]
pub struct HandResourceLimits {
    #[serde(default = "default_max_tokens_per_hour")]
    pub max_llm_tokens_per_hour: u32,
    #[serde(default = "default_memory_limit")]
    pub memory_limit_mb: u32,
    #[serde(default = "default_cpu_limit")]
    pub cpu_limit_percent: u32,
}

#[derive(Debug, Clone, serde::Serialize, serde::Deserialize, Default)]
pub struct HandCapabilities {
    #[serde(default)]
    pub tools: Vec<String>,
    #[serde(default)]
    pub network: Vec<String>,
    #[serde(default)]
    pub memory_read: Vec<String>,
    #[serde(default)]
    pub memory_write: Vec<String>,
}

fn default_max_tokens_per_hour() -> u32 { 150000 }
fn default_memory_limit() -> u32 { 512 }
fn default_cpu_limit() -> u32 { 50 }

pub struct Hand {
    manifest: HandManifest,
    permission_engine: PermissionEngine,
    mcp_proxy: MCPProxy,
    state: HandState,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum HandState {
    Stopped,
    Starting,
    Running,
    WaitingForPermission,
    Sleeping,
    Completed,
    Failed,
}

impl Hand {
    /// Load a Hand agent from a manifest file
    pub fn from_file(path: PathBuf) -> anyhow::Result<Self> {
        let content = std::fs::read_to_string(path)?;
        let manifest: HandManifest = toml::from_str(&content)?;
        
        let permission_engine = PermissionEngine::new();
        let mcp_proxy = MCPProxy::new(permission_engine.clone());
        
        Ok(Self {
            manifest,
            permission_engine,
            mcp_proxy,
            state: HandState::Stopped,
        })
    }

    /// Start the Hand agent main loop
    pub async fn start(&mut self) -> anyhow::Result<()> {
        self.state = HandState::Running;
        
        loop {
            // Process any pending permission requests
            self.process_pending_permissions().await?;
            
            // Run agent thinking cycle
            self.think_cycle().await?;
            
            // Sleep between cycles
            tokio::time::sleep(tokio::time::Duration::from_secs(10)).await;
        }
    }

    async fn process_pending_permissions(&mut self) -> anyhow::Result<()> {
        // TODO: Implement permission request handling
        Ok(())
    }

    async fn think_cycle(&mut self) -> anyhow::Result<()> {
        // TODO: Implement LLM inference and tool calling
        Ok(())
    }

    /// Get the current state of the Hand
    pub fn state(&self) -> HandState {
        self.state
    }

    /// Get the manifest
    pub fn manifest(&self) -> &HandManifest {
        &self.manifest
    }
}
