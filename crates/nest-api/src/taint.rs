//! Information Flow Taint Tracking
//!
//! Implements lattice-based taint propagation that prevents tainted values
//! from flowing into sensitive sinks without explicit declassification.
//! Guards against prompt injection, data exfiltration, and confused deputy attacks.

use serde::{Deserialize, Serialize};
use std::collections::HashSet;

/// Taint labels indicating the origin and sensitivity of data
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub enum TaintLabel {
    /// Data received from external network requests
    ExternalNetwork,

    /// Direct user input
    UserInput,

    /// Personally identifiable information
    Pii,

    /// API keys, tokens, passwords and other secrets
    Secret,

    /// Data received from sandboxed/untrusted agents
    UntrustedAgent,
}

/// A value carrying taint labels indicating its origin and sensitivity
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TaintedValue {
    /// The actual value
    pub value: String,

    /// Attached taint labels
    pub labels: HashSet<TaintLabel>,

    /// Human-readable origin of the value
    pub source: String,
}

/// Result of a taint sink check
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum TaintResult {
    /// Value is allowed to flow to this sink
    Allowed,

    /// Value is tainted and cannot flow to this sink
    Violation(TaintViolation),
}

/// Taint violation when a tainted value attempts to flow into a sensitive sink
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct TaintViolation {
    /// The offending taint label
    pub label: TaintLabel,

    /// Name of the sink that was blocked
    pub sink_name: &'static str,

    /// Source of the tainted value
    pub source: String,
}

/// A sensitive sink that only accepts certain taint levels
pub enum TaintSink {
    /// Shell command execution
    ShellExec,

    /// Network fetch operations
    NetFetch,

    /// Message to another agent
    AgentMessage,

    /// File write operation
    FileWrite,
}

impl TaintedValue {
    /// Create a new tainted value with labels
    pub fn new(value: String, labels: HashSet<TaintLabel>, source: String) -> Self {
        Self {
            value,
            labels,
            source,
        }
    }

    /// Create a clean untainted value
    pub fn clean(value: String, source: String) -> Self {
        Self {
            value,
            labels: HashSet::new(),
            source,
        }
    }

    /// Merge taint labels from another value into this one
    pub fn merge_taint(&mut self, other: &TaintedValue) {
        self.labels.extend(other.labels.iter());
    }

    /// Check if this value can flow to the specified sink
    pub fn check_sink(&self, sink: TaintSink) -> TaintResult {
        let blocked_labels = match sink {
            TaintSink::ShellExec => &[
                TaintLabel::ExternalNetwork,
                TaintLabel::UntrustedAgent,
                TaintLabel::UserInput,
            ][..],

            TaintSink::NetFetch => &[TaintLabel::Secret, TaintLabel::Pii][..],

            TaintSink::AgentMessage => &[TaintLabel::Secret][..],

            TaintSink::FileWrite => &[][..],
        };

        for &label in blocked_labels {
            if self.labels.contains(&label) {
                return TaintResult::Violation(TaintViolation {
                    label,
                    sink_name: sink.name(),
                    source: self.source.clone(),
                });
            }
        }

        TaintResult::Allowed
    }

    /// Explicitly declassify a specific taint label
    pub fn declassify(&mut self, label: TaintLabel) {
        self.labels.remove(&label);
    }

    /// Check if value has any taint labels
    pub fn is_tainted(&self) -> bool {
        !self.labels.is_empty()
    }
}

impl TaintSink {
    /// Get human readable name for this sink
    pub const fn name(&self) -> &'static str {
        match self {
            TaintSink::ShellExec => "shell_exec",
            TaintSink::NetFetch => "net_fetch",
            TaintSink::AgentMessage => "agent_message",
            TaintSink::FileWrite => "file_write",
        }
    }
}

/// Merge taint labels from multiple values
pub fn merge_taint(values: &[&TaintedValue]) -> HashSet<TaintLabel> {
    let mut result = HashSet::new();
    for value in values {
        result.extend(value.labels.iter());
    }
    result
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_tainted_value_creation() {
        let clean = TaintedValue::clean("test".into(), "internal".into());
        assert!(!clean.is_tainted());

        let mut labels = HashSet::new();
        labels.insert(TaintLabel::ExternalNetwork);

        let tainted = TaintedValue::new("test".into(), labels, "web_request".into());
        assert!(tainted.is_tainted());
    }

    #[test]
    fn test_taint_merge() {
        let mut a = TaintedValue::clean("a".into(), "a".into());
        let mut labels = HashSet::new();
        labels.insert(TaintLabel::ExternalNetwork);
        let b = TaintedValue::new("b".into(), labels, "b".into());

        assert!(!a.is_tainted());
        a.merge_taint(&b);
        assert!(a.is_tainted());
    }

    #[test]
    fn test_sink_checks() {
        // External network data cannot flow to shell exec
        let mut labels = HashSet::new();
        labels.insert(TaintLabel::ExternalNetwork);
        let tainted = TaintedValue::new("test".into(), labels, "web".into());

        assert!(matches!(
            tainted.check_sink(TaintSink::ShellExec),
            TaintResult::Violation(_)
        ));
        assert!(matches!(
            tainted.check_sink(TaintSink::NetFetch),
            TaintResult::Allowed
        ));

        // Secrets cannot flow to network
        let mut labels = HashSet::new();
        labels.insert(TaintLabel::Secret);
        let secret = TaintedValue::new("sk_test".into(), labels, "env".into());

        assert!(matches!(
            secret.check_sink(TaintSink::NetFetch),
            TaintResult::Violation(_)
        ));
        assert!(matches!(
            secret.check_sink(TaintSink::AgentMessage),
            TaintResult::Violation(_)
        ));
    }

    #[test]
    fn test_declassify() {
        let mut labels = HashSet::new();
        labels.insert(TaintLabel::ExternalNetwork);
        let mut tainted = TaintedValue::new("test".into(), labels, "web".into());

        assert!(tainted.is_tainted());
        tainted.declassify(TaintLabel::ExternalNetwork);
        assert!(!tainted.is_tainted());
    }
}
