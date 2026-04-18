//! Hand agent system for Nest
//!
//! Implements the production-proven Hand/Agent pattern from OpenFang.
//! Agents are defined via simple TOML manifests and run in isolated sandboxes.

use std::path::PathBuf;
use nest_tools::MCPProxy;
use nest_permissions::PermissionEngine;
use nest_llm::{LlmRegistry, LlmRequest, Message, Role, ToolDefinition};
use nest_llm::sanitize::ContentSanitizer;
use std::collections::VecDeque;

#[derive(Debug, Clone, serde::Serialize, serde::Deserialize)]
pub struct HandManifest {
    pub name: String,
    
    #[serde(default)]
    pub version: String,
    
    #[serde(default)]
    pub description: String,
    
    #[serde(default)]
    pub author: String,
    
    #[serde(default)]
    pub icon: String,
    
    #[serde(default)]
    pub tags: Vec<String>,

    #[serde(default)]
    pub model: HandModelConfig,

    #[serde(default)]
    pub resources: HandResourceLimits,

    #[serde(default)]
    pub capabilities: HandCapabilities,

    #[serde(default)]
    pub settings: Vec<HandSetting>,

    #[serde(default)]
    pub dashboard: HandDashboard,

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
    pub max_iterations: u32,
    #[serde(default)]
    pub heartbeat_interval_secs: u32,
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

#[derive(Debug, Clone, serde::Serialize, serde::Deserialize)]
pub struct HandSetting {
    pub key: String,
    pub label: String,
    pub description: String,
    pub setting_type: String,
    pub default: String,
    #[serde(default)]
    pub options: Vec<HandSettingOption>,
}

#[derive(Debug, Clone, serde::Serialize, serde::Deserialize)]
pub struct HandSettingOption {
    pub value: String,
    pub label: String,
}

#[derive(Debug, Clone, serde::Serialize, serde::Deserialize, Default)]
pub struct HandDashboard {
    #[serde(default)]
    pub metrics: Vec<HandDashboardMetric>,
}

#[derive(Debug, Clone, serde::Serialize, serde::Deserialize)]
pub struct HandDashboardMetric {
    pub label: String,
    pub memory_key: String,
    pub format: String,
}

fn default_max_tokens_per_hour() -> u32 { 150000 }
fn default_memory_limit() -> u32 { 512 }
fn default_cpu_limit() -> u32 { 50 }

pub struct Hand {
    manifest: HandManifest,
    mcp_proxy: MCPProxy,
    state: HandState,
    llm_registry: LlmRegistry,
    conversation_history: Vec<Message>,
    task_queue: VecDeque<String>,
    loop_guard: super::LoopGuard,
    depth_guard: super::DepthGuard,
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
        let mcp_proxy = MCPProxy::new(permission_engine);
        let llm_registry = LlmRegistry::new();
        
        Ok(Self {
            manifest,
            mcp_proxy,
            state: HandState::Stopped,
            llm_registry,
            conversation_history: Vec::new(),
            task_queue: VecDeque::new(),
            loop_guard: super::LoopGuard::new(),
            depth_guard: super::DepthGuard::new(),
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

    pub async fn think_cycle(&mut self) -> anyhow::Result<()> {
        // Reset guards at start of each think cycle
        self.loop_guard.reset();
        self.depth_guard.reset();
        
        // Process pending tasks from queue
        eprintln!("🧠 [{}] Think cycle started, queue size: {}", self.manifest.name, self.task_queue.len());
        
        if let Some(task) = self.task_queue.pop_front() {
            eprintln!("\n🔍 [{}] Starting task: {}", self.manifest.name, task);
            self.conversation_history.push(Message {
                role: Role::User,
                content: task,
                tool_calls: Vec::new(),
                tool_call_id: None,
            });
        }

        // Don't run LLM if no conversation history
        if self.conversation_history.is_empty() {
            return Ok(());
        }

        eprintln!("🤖 [{}] Running LLM inference...", self.manifest.name);

        // Get LLM client for this hand
        let client = self.llm_registry.get_client(
            &self.manifest.name,
            &self.manifest.model.provider,
            self.manifest.resources.max_llm_tokens_per_hour
        )?;

        // Get available tools from MCP proxy
        let tools: Vec<ToolDefinition> = self.mcp_proxy
            .list_tools()
            .into_iter()
            .filter(|t| self.manifest.capabilities.tools.contains(&t.name))
            .map(|t| ToolDefinition {
                name: t.name,
                description: t.description,
                parameters: t.input_schema,
            })
            .collect();

        // Use default model if not specified
        let model = if self.manifest.model.model == "default" {
            client.default_model().to_string()
        } else {
            self.manifest.model.model.clone()
        };

        // Build request
        let request = LlmRequest {
            model,
            messages: self.conversation_history.clone(),
            max_tokens: self.manifest.model.max_tokens,
            temperature: self.manifest.model.temperature,
            tools,
            system_prompt: Some(self.manifest.system_prompt.clone()),
        };

        eprintln!("📤 [{}] Sending request to LLM provider ({} messages)", self.manifest.name, self.conversation_history.len());

        // Execute LLM call
        let response = client.chat_completion(request).await?;
        
        eprintln!("📥 [{}] Got LLM response, usage: {} tokens", self.manifest.name, response.usage.total_tokens);

        // Process response
        if let Some(content) = &response.content {
            println!("[{}] Response: {}", self.manifest.name, content);
            self.conversation_history.push(Message {
                role: Role::Assistant,
                content: content.clone(),
                tool_calls: Vec::new(),
                tool_call_id: None,
            });
        }

        // Execute tool calls sequentially for now
        if !response.tool_calls.is_empty() {
            println!("[{}] Executing {} tool calls...", self.manifest.name, response.tool_calls.len());
            
            for call in &response.tool_calls {
                // Check loop guard
                match self.loop_guard.check(&call.name, &call.arguments) {
                    super::LoopGuardVerdict::Allow => {},
                    super::LoopGuardVerdict::Warn => {
                        eprintln!("[{}] ⚠️  Warning: Repeated call to tool '{}'", self.manifest.name, call.name);
                    },
                    super::LoopGuardVerdict::Block => {
                        eprintln!("[{}] 🚫 Blocked repeated call to tool '{}'", self.manifest.name, call.name);
                        self.conversation_history.push(Message {
                            role: Role::Tool,
                            content: "Error: Tool call blocked due to repeated execution. You seem to be in a loop.".to_string(),
                            tool_calls: Vec::new(),
                            tool_call_id: Some(call.id.clone()),
                        });
                        continue;
                    },
                    super::LoopGuardVerdict::CircuitBreak => {
                        eprintln!("[{}] 🚨 Circuit breaker triggered, terminating agent", self.manifest.name);
                        self.state = HandState::Failed;
                        break;
                    }
                }
                
                // Check depth guard
                if self.depth_guard.would_exceed() {
                    eprintln!("[{}] 🚫 Maximum tool call depth exceeded", self.manifest.name);
                    self.conversation_history.push(Message {
                        role: Role::Tool,
                        content: "Error: Maximum tool call depth exceeded. Recursion depth limit reached.".to_string(),
                        tool_calls: Vec::new(),
                        tool_call_id: Some(call.id.clone()),
                    });
                    continue;
                }
                
                // Enter depth level
                let _ = self.depth_guard.enter();
                
                let result = self.mcp_proxy.call_tool(
                    &self.manifest.name,
                    &call.name,
                    call.arguments.clone()
                ).await;

                let content = match result {
                    Ok(v) => serde_json::to_string(&v).unwrap_or_else(|_| "Error serializing result".into()),
                    Err(e) => format!("Error: {}", e),
                };
                
                // Sanitize tool output for indirect prompt injection
                let sanitizer = ContentSanitizer::new();
                let (sanitized_content, result) = sanitizer.sanitize(&content);
                
                if result == nest_llm::sanitize::SanitizationResult::InjectionDetected {
                    eprintln!("[{}] ⚠️  Detected indirect prompt injection in tool output, sanitized", self.manifest.name);
                }
                
                let content = sanitized_content;

                self.conversation_history.push(Message {
                    role: Role::Tool,
                    content,
                    tool_calls: Vec::new(),
                    tool_call_id: Some(call.id.clone()),
                });
            }
        }

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

    /// Submit a new task to this hand
    pub fn submit_task(&mut self, task: String) {
        eprintln!("[{}] New task queued: {}", self.manifest.name, task);
        self.task_queue.push_back(task);
    }
}
