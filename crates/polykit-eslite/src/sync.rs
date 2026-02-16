//! Snapshot + delta sync engine
//!
//! Manages efficient synchronization between WASM ESLite store and
//! lex stream state. Uses the snapshot+delta pattern:
//! 1. Initial load: subscribe to {topic}.snapshot → full state
//! 2. Ongoing: subscribe to {topic}.delta → incremental updates

use serde::{Deserialize, Serialize};

/// Sync state for a table.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum SyncState {
    /// Not yet synced
    Unsynced,
    /// Snapshot received, applying deltas
    Synced { last_sequence: u64 },
    /// Sync paused (offline)
    Paused { last_sequence: u64 },
    /// Sync error
    Error(String),
}

/// A delta update from a lex stream.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Delta {
    pub sequence: u64,
    pub operation: DeltaOp,
    pub table: String,
    pub key: Vec<u8>,
    pub data: Option<Vec<u8>>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum DeltaOp {
    Insert,
    Update,
    Delete,
}

/// Sync manager for a set of ESLite tables.
pub struct SyncManager {
    states: std::collections::HashMap<String, SyncState>,
}

impl SyncManager {
    pub fn new() -> Self {
        Self {
            states: std::collections::HashMap::new(),
        }
    }

    /// Register a table for sync.
    pub fn register(&mut self, table: &str) {
        self.states.insert(table.to_string(), SyncState::Unsynced);
    }

    /// Apply a snapshot (full state replace).
    pub fn apply_snapshot(&mut self, table: &str, _data: &[u8], sequence: u64) {
        self.states.insert(table.to_string(), SyncState::Synced { last_sequence: sequence });
    }

    /// Apply a delta (incremental update).
    pub fn apply_delta(&mut self, delta: &Delta) -> Result<(), String> {
        match self.states.get(&delta.table) {
            Some(SyncState::Synced { last_sequence }) => {
                if delta.sequence != last_sequence + 1 {
                    return Err(format!(
                        "sequence gap: expected {}, got {}",
                        last_sequence + 1,
                        delta.sequence
                    ));
                }
                self.states.insert(
                    delta.table.clone(),
                    SyncState::Synced { last_sequence: delta.sequence },
                );
                Ok(())
            }
            _ => Err("table not synced".to_string()),
        }
    }

    /// Get sync state for a table.
    pub fn state(&self, table: &str) -> &SyncState {
        self.states.get(table).unwrap_or(&SyncState::Unsynced)
    }
}
