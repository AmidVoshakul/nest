//! Message types for inter-agent communication

use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum MessageType {
    /// Normal text message
    Text,

    /// Request for permission to perform an action
    PermissionRequest,

    /// Response to a permission request
    PermissionResponse,

    /// Agent status update
    StatusUpdate,

    /// Task assignment
    Task,

    /// Task result
    TaskResult,

    /// System control message
    System,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Message {
    /// Unique message ID
    pub id: String,

    /// Sender agent ID
    pub from: String,

    /// Recipient agent ID, or "all" for broadcast
    pub to: String,

    /// Message type
    pub message_type: MessageType,

    /// Message content
    pub content: String,

    /// Optional metadata
    #[serde(default)]
    pub metadata: serde_json::Map<String, serde_json::Value>,

    /// Timestamp (unix milliseconds)
    pub timestamp: u64,
}

impl Message {
    /// Create a new text message
    pub fn text(from: &str, to: &str, content: &str) -> Self {
        Self {
            id: uuid::Uuid::new_v4().to_string(),
            from: from.to_string(),
            to: to.to_string(),
            message_type: MessageType::Text,
            content: content.to_string(),
            metadata: serde_json::Map::new(),
            timestamp: std::time::SystemTime::now()
                .duration_since(std::time::UNIX_EPOCH)
                .unwrap_or_default()
                .as_millis() as u64,
        }
    }
}
