//! Permission engine for Nest
//!
//! Implements deny-by-default permission system with granular controls
//! for agent actions and resource access.

use nest_api::permission::{Permission, PermissionResult};
use std::collections::{HashMap, HashSet};

#[derive(Debug, Default, Clone)]
pub struct PermissionEngine {
    /// Granted permissions by agent ID
    grants: HashMap<String, HashSet<(Permission, Option<String>)>>,
    /// Pending permission requests waiting for user approval
    pending_requests: Vec<PendingRequest>,
}

#[derive(Debug, Clone)]
pub struct PendingRequest {
    pub agent_id: String,
    pub permission: Permission,
    pub resource: Option<String>,
    pub description: String,
    pub timestamp: u64,
}

impl PermissionEngine {
    /// Create a new permission engine with deny-all default policy
    pub fn new() -> Self {
        Self::default()
    }

    /// Check if an agent has permission to perform an action
    pub fn check(
        &self,
        agent_id: &str,
        permission: Permission,
        resource: Option<&str>,
    ) -> PermissionResult {
        // Default deny policy
        let key = (permission, resource.map(|s| s.to_string()));

        if self
            .grants
            .get(agent_id)
            .map(|g| g.contains(&key))
            .unwrap_or(false)
        {
            PermissionResult::Allowed
        } else {
            PermissionResult::NeedsApproval
        }
    }

    /// Grant a permission to an agent
    pub fn grant(&mut self, agent_id: &str, permission: Permission, resource: Option<&str>) {
        let grants = self.grants.entry(agent_id.to_string()).or_default();
        grants.insert((permission, resource.map(|s| s.to_string())));
    }

    /// Revoke a permission from an agent
    pub fn revoke(&mut self, agent_id: &str, permission: Permission, resource: Option<&str>) {
        if let Some(grants) = self.grants.get_mut(agent_id) {
            grants.remove(&(permission, resource.map(|s| s.to_string())));
        }
    }

    /// Request permission for an action that requires user approval
    pub fn request(&mut self, request: PendingRequest) {
        self.pending_requests.push(request);
    }

    /// Get all pending permission requests
    pub fn pending_requests(&self) -> &[PendingRequest] {
        &self.pending_requests
    }

    /// Approve a pending permission request
    pub fn approve(&mut self, index: usize) -> bool {
        if index >= self.pending_requests.len() {
            return false;
        }

        let req = self.pending_requests[index].clone();
        self.grant(&req.agent_id, req.permission, req.resource.as_deref());
        self.pending_requests.remove(index);
        true
    }

    /// Deny a pending permission request
    pub fn deny(&mut self, index: usize) -> bool {
        if index < self.pending_requests.len() {
            self.pending_requests.remove(index);
            true
        } else {
            false
        }
    }
}
