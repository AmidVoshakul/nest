//! GCRA (Generic Cell Rate Algorithm) rate limiter
//!
//! Cost-aware rate limiter that prevents abuse of expensive tools
//! while allowing burst usage of cheap operations.

use std::collections::HashMap;
use std::time::{Duration, Instant};

/// Different cost levels for tool calls
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum ToolCost {
    /// Very cheap operations (local memory)
    Low,

    /// Moderate cost operations (filesystem)
    Medium,

    /// Expensive operations (network)
    High,

    /// Very expensive operations (LLM calls)
    Critical,
}

impl ToolCost {
    /// Get weight for cost level
    pub const fn weight(&self) -> u64 {
        match self {
            ToolCost::Low => 1,
            ToolCost::Medium => 5,
            ToolCost::High => 25,
            ToolCost::Critical => 100,
        }
    }
}

/// Rate limiter for agent tool calls
#[derive(Debug)]
pub struct CostAwareRateLimiter {
    max_burst: u64,
    emission_interval: u64,
    agent_states: HashMap<String, AgentRateState>,
    tool_costs: HashMap<String, ToolCost>,
}

/// GCRA rate limiter state per agent
#[derive(Debug, Clone)]
struct AgentRateState {
    last_arrival: Instant,
    bucket: u64,
}

impl Default for CostAwareRateLimiter {
    fn default() -> Self {
        Self::new()
    }
}

impl CostAwareRateLimiter {
    /// Create new cost-aware rate limiter with default quota (1000 units/sec)
    pub fn new() -> Self {
        Self::with_quota(1000, 1000)
    }

    /// Create rate limiter with custom quota
    pub fn with_quota(per_second: u64, max_burst: u64) -> Self {
        Self {
            max_burst,
            emission_interval: Duration::from_secs(1).as_nanos() as u64 / per_second,
            agent_states: HashMap::new(),
            tool_costs: HashMap::new(),
        }
    }

    /// Register a tool with specific cost level
    pub fn register_tool(&mut self, tool_name: &str, cost: ToolCost) {
        self.tool_costs.insert(tool_name.to_string(), cost);
    }

    /// Check if a tool call is allowed
    pub fn check(&mut self, agent_id: &str, tool_name: &str) -> Result<(), Duration> {
        let cost = self
            .tool_costs
            .get(tool_name)
            .copied()
            .unwrap_or(ToolCost::Medium);

        let weight = cost.weight();
        let now = Instant::now();

        let state = self
            .agent_states
            .entry(agent_id.to_string())
            .or_insert_with(|| AgentRateState {
                last_arrival: now,
                bucket: self.max_burst,
            });

        // Update bucket with elapsed time
        let elapsed = now.duration_since(state.last_arrival).as_nanos() as u64;
        state.bucket = u64::min(
            self.max_burst,
            state.bucket + elapsed / self.emission_interval,
        );
        state.last_arrival = now;

        if state.bucket >= weight {
            state.bucket -= weight;
            Ok(())
        } else {
            let needed = weight - state.bucket;
            let wait_time = needed * self.emission_interval;
            Err(Duration::from_nanos(wait_time))
        }
    }

    /// Get remaining capacity for an agent
    pub fn remaining_capacity(&self, agent_id: &str) -> u64 {
        self.agent_states
            .get(agent_id)
            .map(|s| s.bucket)
            .unwrap_or(self.max_burst)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_rate_limiter_basic() {
        let mut limiter = CostAwareRateLimiter::with_quota(100, 100);

        limiter.register_tool("web_search", ToolCost::High);
        limiter.register_tool("file_read", ToolCost::Low);

        // Low cost operations are cheap
        for _ in 0..100 {
            assert!(limiter.check("agent1", "file_read").is_ok());
        }

        // High cost operations are expensive
        assert!(limiter.check("agent1", "web_search").is_err());
    }

    #[test]
    fn test_cost_weights() {
        assert_eq!(ToolCost::Low.weight(), 1);
        assert_eq!(ToolCost::Medium.weight(), 5);
        assert_eq!(ToolCost::High.weight(), 25);
        assert_eq!(ToolCost::Critical.weight(), 100);
    }

    #[test]
    fn test_different_agents_independent() {
        let mut limiter = CostAwareRateLimiter::with_quota(100, 100);

        // Agent 1 consumes all capacity
        for _ in 0..100 {
            let _ = limiter.check("agent1", "file_read");
        }

        // Agent 1 should be rate limited now
        assert!(limiter.check("agent1", "file_read").is_err());

        // Agent 2 still has full capacity
        assert!(limiter.check("agent2", "file_read").is_ok());
    }
}
