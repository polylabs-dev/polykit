//! ESLite schema DSL
//!
//! Provides a builder API for defining ESLite table schemas.

use serde::{Deserialize, Serialize};

/// Table definition.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TableDef {
    pub name: String,
    pub columns: Vec<ColumnDef>,
    /// Optional TTL configuration
    pub ttl: Option<TtlConfig>,
}

/// Column definition.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ColumnDef {
    pub name: String,
    pub column_type: ColumnType,
    pub primary_key: bool,
    pub indexed: bool,
    pub nullable: bool,
    pub default: Option<String>,
}

/// Column data types.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum ColumnType {
    Text,
    Integer,
    Real,
    Blob,
    Boolean,
}

/// TTL configuration for auto-expiring rows.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TtlConfig {
    /// Column containing expiration timestamp
    pub column: String,
    /// How often to check for expired rows (milliseconds)
    pub cleanup_interval_ms: u64,
}

/// Builder for constructing table definitions.
pub struct TableBuilder {
    name: String,
    columns: Vec<ColumnDef>,
    ttl: Option<TtlConfig>,
}

impl TableBuilder {
    pub fn new(name: &str) -> Self {
        Self {
            name: name.to_string(),
            columns: Vec::new(),
            ttl: None,
        }
    }

    pub fn column(mut self, name: &str, col_type: ColumnType) -> ColumnBuilder {
        ColumnBuilder {
            table: self,
            def: ColumnDef {
                name: name.to_string(),
                column_type: col_type,
                primary_key: false,
                indexed: false,
                nullable: false,
                default: None,
            },
        }
    }

    pub fn ttl(mut self, column: &str, cleanup_interval_ms: u64) -> Self {
        self.ttl = Some(TtlConfig {
            column: column.to_string(),
            cleanup_interval_ms,
        });
        self
    }

    pub fn build(self) -> TableDef {
        TableDef {
            name: self.name,
            columns: self.columns,
            ttl: self.ttl,
        }
    }
}

pub struct ColumnBuilder {
    table: TableBuilder,
    def: ColumnDef,
}

impl ColumnBuilder {
    pub fn primary_key(mut self) -> Self {
        self.def.primary_key = true;
        self
    }

    pub fn indexed(mut self) -> Self {
        self.def.indexed = true;
        self
    }

    pub fn nullable(mut self) -> Self {
        self.def.nullable = true;
        self
    }

    pub fn default(mut self, value: &str) -> Self {
        self.def.default = Some(value.to_string());
        self
    }

    pub fn done(mut self) -> TableBuilder {
        self.table.columns.push(self.def);
        self.table
    }
}
