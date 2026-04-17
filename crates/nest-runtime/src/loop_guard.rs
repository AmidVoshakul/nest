//! Loop guard for detecting stuck tool call loops

use serde_json::Value;
use sha2::{Digest, Sha256};
use std::collections::HashMap;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum LoopGuardVerdict {
    /// Allow the tool call
    Allow,
    /// Warn about repeated call but allow
    Warn,
    /// Block the tool call
    Block,
    /// Terminate the entire agent loop
    CircuitBreak,
}

#[derive(Debug, Clone)]
pub struct LoopGuardConfig {
    /// Number of identical calls before warning
    pub warn_threshold: u32,
    /// Number of identical calls before blocking
    pub block_threshold: u32,
    /// Total tool calls before circuit breaker
    pub global_circuit_breaker: u32,
}

impl Default for LoopGuardConfig {
    fn default() -> Self {
        Self {
            warn_threshold: 3,
            block_threshold: 5,
            global_circuit_breaker: 30,
        }
    }
}

/// Loop guard detects when an agent is stuck calling the same tool repeatedly
#[derive(Debug, Default)]
pub struct LoopGuard {
    config: LoopGuardConfig,
    call_counts: HashMap<String, u32>,
    total_calls: u32,
}

impl LoopGuard {
    /// Create new loop guard with default configuration
    pub fn new() -> Self {
        Self::with_config(LoopGuardConfig::default())
    }

    /// Create new loop guard with custom configuration
    pub fn with_config(config: LoopGuardConfig) -> Self {
        Self {
            config,
            call_counts: HashMap::new(),
            total_calls: 0,
        }
    }

    /// Compute hash for tool call
    fn compute_hash(tool_name: &str, params: &Value) -> String {
        let mut hasher = Sha256::new();
        hasher.update(tool_name.as_bytes());
        hasher.update(b"|");
        hasher.update(serde_json::to_string(params).unwrap_or_default().as_bytes());

        format!("{:x}", hasher.finalize())
    }

    /// Check a tool call and return verdict
    pub fn check(&mut self, tool_name: &str, params: &Value) -> LoopGuardVerdict {
        self.total_calls += 1;

        // Global circuit breaker
        if self.total_calls > self.config.global_circuit_breaker {
            return LoopGuardVerdict::CircuitBreak;
        }

        let hash = Self::compute_hash(tool_name, params);
        let count = self.call_counts.entry(hash).or_insert(0);
        *count += 1;

        if *count >= self.config.block_threshold {
            LoopGuardVerdict::Block
        } else if *count >= self.config.warn_threshold {
            LoopGuardVerdict::Warn
        } else {
            LoopGuardVerdict::Allow
        }
    }

    /// Reset call counters for new think cycle
    pub fn reset(&mut self) {
        self.call_counts.clear();
        self.total_calls = 0;
    }

    /// Get total number of calls tracked
    pub fn total_calls(&self) -> u32 {
        self.total_calls
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use serde_json::json;

    #[test]
    fn test_loop_guard_basic() {
        let mut guard = LoopGuard::new();
        let params = json!({"query": "test"});

        // First 2 calls are allowed
        assert_eq!(guard.check("web_search", &params), LoopGuardVerdict::Allow);
        assert_eq!(guard.check("web_search", &params), LoopGuardVerdict::Allow);

        // 3rd call warns
        assert_eq!(guard.check("web_search", &params), LoopGuardVerdict::Warn);

        // 4th call still warns
        assert_eq!(guard.check("web_search", &params), LoopGuardVerdict::Warn);

        // 5th call blocks
        assert_eq!(guard.check("web_search", &params), LoopGuardVerdict::Block);
    }

    #[test]
    fn test_loop_guard_different_params() {
        let mut guard = LoopGuard::new();

        assert_eq!(
            guard.check("web_search", &json!({"query": "test1"})),
            LoopGuardVerdict::Allow
        );
        assert_eq!(
            guard.check("web_search", &json!({"query": "test2"})),
            LoopGuardVerdict::Allow
        );
        assert_eq!(
            guard.check("web_search", &json!({"query": "test3"})),
            LoopGuardVerdict::Allow
        );

        // Different parameters, so no warning
        assert_eq!(guard.total_calls(), 3);
    }

    #[test]
    fn test_loop_guard_circuit_breaker() {
        let mut guard = LoopGuard::new();
        let params = json!({"query": "test"});

        // 30 calls should trigger circuit breaker
        for i in 0..30 {
            assert_ne!(
                guard.check("web_search", &json!({"query": i})),
                LoopGuardVerdict::CircuitBreak
            );
        }

        // 31st call triggers circuit break
        assert_eq!(
            guard.check("web_search", &params),
            LoopGuardVerdict::CircuitBreak
        );
    }

    #[test]
    fn test_loop_guard_reset() {
        let mut guard = LoopGuard::new();
        let params = json!({"query": "test"});

        for _ in 0..5 {
            guard.check("web_search", &params);
        }

        assert_eq!(guard.check("web_search", &params), LoopGuardVerdict::Block);

        guard.reset();

        assert_eq!(guard.check("web_search", &params), LoopGuardVerdict::Allow);
        assert_eq!(guard.total_calls(), 1);
    }
}
