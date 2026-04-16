//! Scheduler types for Nest

use serde::{Deserialize, Serialize};
use std::collections::HashMap;

/// A scheduled task
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ScheduledTask {
    /// Unique task ID
    pub id: String,

    /// Name of the hand to execute the task
    pub hand_name: String,

    /// Task description/query
    pub task: String,

    /// Cron schedule expression
    pub schedule: String,

    /// Next execution time (unix timestamp)
    pub next_run: u64,

    /// Number of times this task has run
    pub run_count: u32,

    /// Maximum number of runs (0 = unlimited)
    pub max_runs: u32,

    /// Whether the task is enabled
    pub enabled: bool,
}

/// Scheduler state
#[derive(Debug, Clone, Default)]
pub struct SchedulerState {
    pub tasks: HashMap<String, ScheduledTask>,
}
