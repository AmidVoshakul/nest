//! Permission engine for Nest
//!
//! Implements deny-by-default permission system with granular controls
//! for agent actions and resource access.

use nest_api::permission::{Permission, PermissionResult};
use std::collections::{HashMap, HashSet};
use wildmatch::WildMatch;

#[derive(Debug, Default, Clone)]
pub struct PermissionEngine {
    /// Granted permissions by agent ID
    grants: HashMap<String, HashSet<(Permission, Option<String>)>>,
    /// Pending permission requests waiting for user approval
    pending_requests: Vec<PendingRequest>,
    /// Auto-approve all permissions for testing
    auto_approve: bool,
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
        // Auto-approve for testing
        if self.auto_approve {
            return PermissionResult::Allowed;
        }

        // Default deny policy
        let grants = match self.grants.get(agent_id) {
            Some(g) => g,
            None => return PermissionResult::NeedsApproval,
        };

        // Check for exact match first
        let exact_key = (permission, resource.map(|s| s.to_string()));
        if grants.contains(&exact_key) {
            return PermissionResult::Allowed;
        }

        // Check for glob pattern matches if resource is provided
        if let Some(resource) = resource {
            for (grant_perm, grant_resource) in grants {
                if *grant_perm != permission {
                    continue;
                }

                if let Some(pattern) = grant_resource {
                    if WildMatch::new(pattern).matches(resource) {
                        return PermissionResult::Allowed;
                    }
                } else {
                    // Grant with no resource means allow all
                    return PermissionResult::Allowed;
                }
            }
        }

        PermissionResult::NeedsApproval
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

    /// Set auto-approve mode (for testing only)
    pub fn set_auto_approve(&mut self, enabled: bool) {
        self.auto_approve = enabled;
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use nest_api::permission::Permission;

    #[test]
    fn test_permission_exact_match() {
        let mut engine = PermissionEngine::new();
        engine.grant("test-agent", Permission::FileRead, Some("/data/test.txt"));

        assert_eq!(
            engine.check("test-agent", Permission::FileRead, Some("/data/test.txt")),
            PermissionResult::Allowed
        );
    }

    #[test]
    fn test_permission_glob_match() {
        let mut engine = PermissionEngine::new();
        engine.grant("test-agent", Permission::FileRead, Some("/data/*.txt"));

        assert_eq!(
            engine.check("test-agent", Permission::FileRead, Some("/data/test.txt")),
            PermissionResult::Allowed
        );

        assert_eq!(
            engine.check("test-agent", Permission::FileRead, Some("/data/other.txt")),
            PermissionResult::Allowed
        );

        assert_eq!(
            engine.check("test-agent", Permission::FileRead, Some("/other/test.txt")),
            PermissionResult::NeedsApproval
        );
    }

    #[test]
    fn test_permission_wildcard_domain() {
        let mut engine = PermissionEngine::new();
        engine.grant(
            "test-agent",
            Permission::NetworkDomain,
            Some("*.openai.com:443"),
        );

        assert_eq!(
            engine.check(
                "test-agent",
                Permission::NetworkDomain,
                Some("api.openai.com:443")
            ),
            PermissionResult::Allowed
        );

        assert_eq!(
            engine.check(
                "test-agent",
                Permission::NetworkDomain,
                Some("other.com:443")
            ),
            PermissionResult::NeedsApproval
        );
    }

    #[test]
    fn test_permission_all_grant() {
        let mut engine = PermissionEngine::new();
        engine.grant("test-agent", Permission::FileRead, None);

        assert_eq!(
            engine.check("test-agent", Permission::FileRead, Some("/any/path.txt")),
            PermissionResult::Allowed
        );
    }
}
