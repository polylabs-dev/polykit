//! Typed cross-widget event bus
//!
//! Runs entirely in WASM. Events are dispatched between widgets without
//! crossing the JS boundary. The TS layer only sees render-ready data
//! after event processing is complete.

use serde::{Deserialize, Serialize};

/// Generic PolyKit events shared across all apps.
/// Apps extend this with domain-specific variants.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum PolykitEvent {
    /// Deviation clicked in feed → highlights related widgets
    DeviationSelect {
        metric: String,
        circuit: String,
        z_score: f64,
        timestamp_ms: u64,
    },
    /// Classification tag selected → filters governance + observability
    ClassificationFilter {
        tag: Option<String>,
    },
    /// Time range changed → propagates to all widgets
    TimeRange {
        from_ms: u64,
        to_ms: u64,
    },
    /// Reset all cross-widget filters
    FilterReset,
    /// ESN-AI "Investigate" → drill into observability
    InvestigateMetric {
        metric: String,
        circuit: Option<String>,
        category: String,
        recommendation_id: String,
    },
    /// Human review completed → refreshes accuracy + feedback widgets
    ReviewCompleted {
        action: ReviewAction,
        sample_hash: String,
        classification: String,
    },
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum ReviewAction {
    Accept,
    Override,
    Flag,
}

/// Event bus. Holds subscribers and dispatches events.
pub struct EventBus {
    /// App namespace (e.g., "polydata", "polymessenger")
    app: String,
    /// Pending events (consumed by widget_data processors)
    pending: Vec<PolykitEvent>,
}

impl EventBus {
    pub fn new(app: &str) -> Self {
        Self {
            app: app.to_string(),
            pending: Vec::new(),
        }
    }

    /// Emit an event. Widget data processors pick it up on next render cycle.
    pub fn emit(&mut self, event: PolykitEvent) {
        self.pending.push(event);
    }

    /// Drain all pending events (called by widget_data processors).
    pub fn drain(&mut self) -> Vec<PolykitEvent> {
        std::mem::take(&mut self.pending)
    }

    /// Get app namespace for lex topic formatting.
    pub fn app(&self) -> &str {
        &self.app
    }
}
