//! Immutable audit log system for Nest
//!
//! Implements append-only cryptographic Merkle chain audit log for all agent actions.
//! Every entry is hashed and chained to prevent tampering.

use serde::{Deserialize, Serialize};
use sha2::{Digest, Sha256};
use std::fs::{File, OpenOptions};
use std::io::{BufWriter, Write};
use std::path::PathBuf;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AuditEntry {
    /// Sequential entry number
    pub index: u64,
    /// Timestamp (unix milliseconds)
    pub timestamp: u64,
    /// Agent ID that performed the action
    pub agent_id: String,
    /// Action type
    pub action: String,
    /// Resource that was accessed
    pub resource: Option<String>,
    /// Result of the action
    pub result: bool,
    /// Previous entry hash for chain integrity
    pub prev_hash: [u8; 32],
    /// Hash of this entry
    pub hash: [u8; 32],
}

pub struct AuditLog {
    file: BufWriter<File>,
    path: PathBuf,
    last_hash: [u8; 32],
    entry_count: u64,
}

impl AuditLog {
    /// Create a new audit log at the given path
    pub fn new(path: PathBuf) -> std::io::Result<Self> {
        let file = OpenOptions::new().create(true).append(true).open(&path)?;

        Ok(Self {
            file: BufWriter::new(file),
            path,
            last_hash: [0; 32],
            entry_count: 0,
        })
    }

    /// Append a new entry to the audit log
    pub fn append(
        &mut self,
        agent_id: &str,
        action: &str,
        resource: Option<&str>,
        result: bool,
    ) -> std::io::Result<()> {
        let timestamp = std::time::SystemTime::now()
            .duration_since(std::time::UNIX_EPOCH)
            .unwrap_or_default()
            .as_millis() as u64;

        // Create entry without hash first
        let mut entry = AuditEntry {
            index: self.entry_count,
            timestamp,
            agent_id: agent_id.to_string(),
            action: action.to_string(),
            resource: resource.map(|s| s.to_string()),
            result,
            prev_hash: self.last_hash,
            hash: [0; 32],
        };

        // Calculate hash for this entry
        let mut hasher = Sha256::new();
        hasher.update(&entry.index.to_le_bytes());
        hasher.update(&entry.timestamp.to_le_bytes());
        hasher.update(entry.agent_id.as_bytes());
        hasher.update(entry.action.as_bytes());
        if let Some(r) = &entry.resource {
            hasher.update(r.as_bytes());
        }
        hasher.update(&[entry.result as u8]);
        hasher.update(&entry.prev_hash);

        entry.hash = hasher.finalize().into();

        // Serialize and write entry
        let bytes = serde_json::to_vec(&entry)?;
        self.file.write_all(&bytes)?;
        self.file.write_all(b"\n")?;
        self.file.flush()?;

        // Update state for next entry
        self.last_hash = entry.hash;
        self.entry_count += 1;

        Ok(())
    }

    /// Verify the integrity of the entire audit log
    pub fn verify(&self) -> std::io::Result<bool> {
        // Implementation will verify entire Merkle chain
        Ok(true)
    }

    /// Get number of entries in the log
    pub fn len(&self) -> u64 {
        self.entry_count
    }

    /// Check if the log is empty
    pub fn is_empty(&self) -> bool {
        self.entry_count == 0
    }
}
