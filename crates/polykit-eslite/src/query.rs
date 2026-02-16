//! ESLite query engine
//!
//! Runs queries in WASM and returns serialized results to the TS layer.

use serde::{Deserialize, Serialize};

/// Query result returned from WASM to the TS binding layer.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct QueryResult {
    pub columns: Vec<String>,
    pub rows: Vec<Vec<serde_json::Value>>,
    pub row_count: usize,
}

/// Query builder for ESLite.
#[derive(Debug, Clone)]
pub struct Query {
    pub table: String,
    pub select: Vec<String>,
    pub where_clauses: Vec<WhereClause>,
    pub order_by: Option<(String, Order)>,
    pub limit: Option<usize>,
    pub offset: Option<usize>,
}

#[derive(Debug, Clone)]
pub enum WhereClause {
    Eq(String, serde_json::Value),
    Lt(String, serde_json::Value),
    Gt(String, serde_json::Value),
    Like(String, String),
    In(String, Vec<serde_json::Value>),
    IsNull(String),
    IsNotNull(String),
}

#[derive(Debug, Clone, Copy)]
pub enum Order {
    Asc,
    Desc,
}

impl Query {
    pub fn from(table: &str) -> Self {
        Self {
            table: table.to_string(),
            select: vec!["*".to_string()],
            where_clauses: Vec::new(),
            order_by: None,
            limit: None,
            offset: None,
        }
    }

    pub fn select(mut self, columns: &[&str]) -> Self {
        self.select = columns.iter().map(|s| s.to_string()).collect();
        self
    }

    pub fn where_eq(mut self, column: &str, value: serde_json::Value) -> Self {
        self.where_clauses.push(WhereClause::Eq(column.to_string(), value));
        self
    }

    pub fn order_by(mut self, column: &str, order: Order) -> Self {
        self.order_by = Some((column.to_string(), order));
        self
    }

    pub fn limit(mut self, n: usize) -> Self {
        self.limit = Some(n);
        self
    }

    /// Execute query against ESLite store (in WASM).
    pub fn execute(&self) -> Result<QueryResult, String> {
        // In production: executes against ESLite via host imports
        Ok(QueryResult {
            columns: self.select.clone(),
            rows: Vec::new(),
            row_count: 0,
        })
    }
}
