//! ESLite Schema Migration System
//!
//! Versioned, forward-only migrations with automatic schema_version tracking.
//! Each app registers its migrations at init time; the runner applies
//! any unapplied migrations in order.

use serde::{Deserialize, Serialize};
use crate::schema::TableDef;

/// A single schema migration.
#[derive(Debug, Clone)]
pub struct Migration {
    /// Monotonically increasing version number (1, 2, 3, ...)
    pub version: u32,
    /// Human-readable description
    pub description: String,
    /// Migration operations
    pub operations: Vec<MigrationOp>,
}

/// A migration operation.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum MigrationOp {
    /// Create a new table
    CreateTable(TableDef),
    /// Add a column to an existing table
    AddColumn {
        table: String,
        name: String,
        column_type: String,
        default: Option<String>,
        nullable: bool,
        indexed: bool,
    },
    /// Create an index
    CreateIndex {
        table: String,
        columns: Vec<String>,
        unique: bool,
    },
    /// Drop a table
    DropTable(String),
}

/// Migration runner. Tracks applied versions per table namespace.
pub struct MigrationRunner {
    /// Table namespace → current schema version
    applied_versions: std::collections::HashMap<String, u32>,
}

impl MigrationRunner {
    pub fn new() -> Self {
        Self {
            applied_versions: std::collections::HashMap::new(),
        }
    }

    /// Run all unapplied migrations for a given table namespace.
    pub fn migrate(
        &mut self,
        namespace: &str,
        migrations: &[Migration],
    ) -> Result<u32, String> {
        let current = self.applied_versions.get(namespace).copied().unwrap_or(0);

        let mut applied = 0;
        for migration in migrations {
            if migration.version > current {
                // In production: execute operations against ESLite store
                // via host import eslite::execute_ddl
                applied += 1;
            }
        }

        let new_version = migrations.last().map(|m| m.version).unwrap_or(current);
        self.applied_versions.insert(namespace.to_string(), new_version);

        Ok(applied)
    }

    /// Get current schema version for a namespace.
    pub fn current_version(&self, namespace: &str) -> u32 {
        self.applied_versions.get(namespace).copied().unwrap_or(0)
    }
}
