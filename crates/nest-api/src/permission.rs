//! Permission system for Nest

use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Serialize, Deserialize, Hash)]
pub enum Permission {
    /// Read a specific file or directory
    FileRead,

    /// Write to a specific file or directory
    FileWrite,

    /// Execute a command
    CommandExecute,

    /// Access the network
    NetworkAccess,

    /// Access a specific domain
    NetworkDomain,

    /// Spawn new processes
    ProcessSpawn,

    /// Access hardware devices
    HardwareAccess,

    /// Send messages to other agents
    AgentCommunicate,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PermissionGrant {
    pub permission: Permission,
    pub resource: Option<String>,
    pub expires_at: Option<u64>,
    pub granted_by: Option<String>,
}

/// Result of a permission check
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum PermissionResult {
    Allowed,
    Denied,
    /// Needs human approval
    NeedsApproval,
}
