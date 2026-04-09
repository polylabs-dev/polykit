//! ESZ Demo Fixture Engine
//!
//! Provides realistic mock data for widgets when running in demo mode (?demo=true).
//! Fixtures are defined in Rust and serialized to the same JSON format as live data.

use serde::{Deserialize, Serialize};

/// Demo mode detection.
pub fn is_demo_mode() -> bool {
    // In WASM: check via host import or init parameter
    false // Default off — set by TS bridge on init
}

/// A demo fixture for a widget.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Fixture {
    pub widget_id: String,
    pub data: serde_json::Value,
}

/// Fixture registry. Apps register fixtures at init time.
pub struct FixtureRegistry {
    fixtures: std::collections::HashMap<String, serde_json::Value>,
    demo_mode: bool,
}

impl FixtureRegistry {
    pub fn new(demo_mode: bool) -> Self {
        Self {
            fixtures: std::collections::HashMap::new(),
            demo_mode,
        }
    }

    /// Register a fixture for a widget.
    pub fn register(&mut self, widget_id: &str, data: serde_json::Value) {
        self.fixtures.insert(widget_id.to_string(), data);
    }

    /// Get fixture data for a widget (returns None if not in demo mode or no fixture).
    pub fn get(&self, widget_id: &str) -> Option<&serde_json::Value> {
        if !self.demo_mode {
            return None;
        }
        self.fixtures.get(widget_id)
    }

    pub fn is_demo(&self) -> bool {
        self.demo_mode
    }
}
