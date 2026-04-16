//! Nest API - Core trait definitions
//!
//! This crate defines all the fundamental abstractions that the Nest hypervisor
//! is built on. No implementations, no heavy dependencies. Every other crate
//! depends only on this.

#![allow(async_fn_in_trait)]

pub mod agent;
pub mod sandbox;
pub mod message;
pub mod permission;
pub mod audit;
pub mod error;
pub mod scheduler;
pub mod path;
pub mod ssrf;
pub mod subprocess;
pub mod taint;

#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord)]
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

/// Core Agent trait - every agent implements this
pub trait Agent: Send + Sync + 'static {
    /// Unique ID of the agent
    fn id(&self) -> &str;

    /// Human readable name
    fn name(&self) -> &str;

    /// Current state of the agent
    fn state(&self) -> AgentState;

    /// Start the agent execution
    async fn start(&mut self) -> error::Result<()>;

    /// Stop the agent immediately
    async fn stop(&mut self) -> error::Result<()>;

    /// Send a message to the agent
    async fn send_message(&mut self, message: message::Message) -> error::Result<()>;

    /// Get next message from agent's outbox
    async fn recv_message(&mut self) -> error::Result<Option<message::Message>>;
}