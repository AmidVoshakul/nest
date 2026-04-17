//! Sandbox implementation for Nest
//!
//! Implements process isolation using Linux namespaces, cgroups v2,
//! and seccomp-bpf filters. Each agent runs in a completely isolated
//! environment with no access to host resources unless explicitly granted.

mod sandbox;
pub mod metering;
pub mod hardening;

use std::path::PathBuf;
pub use metering::{DualMeter, MeteringConfig};
pub use hardening::*;

pub use sandbox::Sandbox;

#[derive(Debug, Clone)]
pub struct SandboxConfig {
    /// Maximum memory the agent can use (bytes)
    pub memory_limit: u64,

    /// Maximum CPU time the agent can use (percent)
    pub cpu_limit: u32,

    /// List of files the agent is allowed to read
    pub allowed_read_paths: Vec<PathBuf>,

    /// List of files the agent is allowed to write
    pub allowed_write_paths: Vec<PathBuf>,

    /// List of domains the agent is allowed to access
    pub allowed_domains: Vec<String>,

    /// Maximum number of processes the agent can spawn
    pub max_processes: u32,

    /// Allow network access at all
    pub network_allowed: bool,
    
    /// Dual metering configuration
    pub metering: MeteringConfig,
}

impl Default for SandboxConfig {
    fn default() -> Self {
        Self {
            memory_limit: 1024 * 1024 * 1024, // 1GB
            cpu_limit: 50, // 50% CPU
            allowed_read_paths: Vec::new(),
            allowed_write_paths: Vec::new(),
            allowed_domains: Vec::new(),
            max_processes: 10,
            network_allowed: false,
            metering: MeteringConfig::default(),
        }
    }
}

pub struct SandboxOutput {
    pub stdout: Vec<u8>,
    pub stderr: Vec<u8>,
    pub exit_code: i32,
}