//! Message bus system for Nest
//!
//! Implements agent-to-agent communication via message passing.
//! No shared memory, no direct calls. Agents communicate like people
//! in messengers - they send messages to a central bus and receive
//! messages addressed to them.

use nest_api::message::Message;
use std::collections::{HashMap, VecDeque};
use std::sync::{Arc, Mutex};

#[derive(Debug, Clone, Default)]
pub struct MessageBus {
    /// Incoming message queues by agent ID
    queues: Arc<Mutex<HashMap<String, VecDeque<Message>>>>,
    /// Subscribed agent IDs
    subscribers: Arc<Mutex<HashMap<String, bool>>>,
}

impl MessageBus {
    /// Create a new message bus
    pub fn new() -> Self {
        Self::default()
    }

    /// Register an agent with the message bus
    pub fn register(&mut self, agent_id: &str) {
        let mut queues = self.queues.lock().unwrap();
        let mut subscribers = self.subscribers.lock().unwrap();

        queues.insert(agent_id.to_string(), VecDeque::new());
        subscribers.insert(agent_id.to_string(), true);
    }

    /// Unregister an agent from the message bus
    pub fn unregister(&mut self, agent_id: &str) {
        let mut queues = self.queues.lock().unwrap();
        let mut subscribers = self.subscribers.lock().unwrap();

        queues.remove(agent_id);
        subscribers.remove(agent_id);
    }

    /// Send a message to the bus
    pub fn send(&mut self, message: Message) -> bool {
        let mut queues = self.queues.lock().unwrap();

        if message.to == "all" {
            // Broadcast to all agents
            for queue in queues.values_mut() {
                queue.push_back(message.clone());
            }
            true
        } else if let Some(queue) = queues.get_mut(&message.to) {
            // Direct message to specific agent
            queue.push_back(message);
            true
        } else {
            // Recipient not found
            false
        }
    }

    /// Receive next message for an agent
    pub fn recv(&mut self, agent_id: &str) -> Option<Message> {
        let mut queues = self.queues.lock().unwrap();
        queues.get_mut(agent_id)?.pop_front()
    }

    /// Peek at the next message for an agent without removing it
    pub fn peek(&self, agent_id: &str) -> Option<Message> {
        let queues = self.queues.lock().unwrap();
        queues.get(agent_id)?.front().cloned()
    }

    /// Get number of pending messages for an agent
    pub fn pending_count(&self, agent_id: &str) -> usize {
        let queues = self.queues.lock().unwrap();
        queues.get(agent_id).map(|q| q.len()).unwrap_or(0)
    }

    /// List all registered agents
    pub fn agents(&self) -> Vec<String> {
        let subscribers = self.subscribers.lock().unwrap();
        subscribers.keys().cloned().collect()
    }
}
