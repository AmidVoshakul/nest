# Nest API Reference

## Overview

Nest exposes a clean, minimal public API. All core types and traits are defined in the `nest-api` crate.

## Core Types

### `AgentState`

Status of an agent lifecycle.

```rust
pub enum AgentState {
    Stopped,
    Starting,
    Running,
    WaitingForApproval,
    Sleeping,
    Completed,
    Failed,
    Killed,
}
```

### `ScheduledTask`

Represents a task scheduled to run at specific times.

```rust
pub struct ScheduledTask {
    pub id: String,
    pub hand_name: String,
    pub task: String,
    pub schedule: String,
    pub next_run: u64,
    pub run_count: u32,
    pub max_runs: u32,
    pub enabled: bool,
}
```

## Core Traits

### `Agent`

Every agent implements this trait.

```rust
pub trait Agent: Send + Sync + 'static {
    fn id(&self) -> &str;
    fn name(&self) -> &str;
    fn state(&self) -> AgentState;
    
    async fn start(&mut self) -> Result<()>;
    async fn stop(&mut self) -> Result<()>;
    
    async fn send_message(&mut self, message: Message) -> Result<()>;
    async fn recv_message(&mut self) -> Result<Option<Message>>;
}
```

## Runtime API

### `AgentRuntime`

Main entry point to the Nest hypervisor.

```rust
impl AgentRuntime {
    /// Create new runtime instance
    pub fn new() -> Self;
    
    /// Load all hand agents from directory
    pub fn load_hands(&mut self, path: &Path) -> Result<()>;
    
    /// Start main runtime loop
    pub async fn run(&mut self) -> Result<()>;
    
    /// Submit task to hand agent
    pub fn submit_task(&mut self, hand_name: &str, task: String) -> bool;
    
    /// Schedule recurring task
    pub fn schedule_task(&mut self, task: ScheduledTask) -> Result<()>;
    
    /// Unschedule task
    pub fn unschedule_task(&mut self, task_id: &str) -> Option<ScheduledTask>;
    
    /// Get all scheduled tasks
    pub fn scheduled_tasks(&self) -> &HashMap<String, ScheduledTask>;
    
    /// Approve pending permission request
    pub fn approve_permission(&mut self, index: usize) -> bool;
    
    /// Deny pending permission request
    pub fn deny_permission(&mut self, index: usize) -> bool;
}
```

## Error Handling

All Nest operations return `nest_api::error::Result<T>` which uses the custom `Error` enum.

```rust
pub enum Error {
    Sandbox(String),
    PermissionDenied(String),
    Agent(String),
    MessageBus(String),
    Audit(String),
    Config(String),
    Io(String),
    Unknown(String),
}
```
