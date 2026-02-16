//! Widget data pipeline
//!
//! Transforms raw stream data + event bus state into render-ready
//! JSON payloads for the TS layer. Each widget type has a processor
//! that runs in WASM.

use serde::{Deserialize, Serialize};
use crate::event_bus::{EventBus, PolykitEvent};

/// Render-ready payload returned to TS for a specific widget instance.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct WidgetPayload {
    pub widget_id: String,
    pub data: serde_json::Value,
    /// If true, the widget should re-render
    pub dirty: bool,
}

/// Widget data processor trait. Each widget type implements this.
pub trait WidgetProcessor {
    /// Unique widget type ID (e.g., "polykit-deviation-feed")
    fn widget_type(&self) -> &str;

    /// Process incoming stream data and event bus events.
    /// Returns a render-ready payload for the TS layer.
    fn process(
        &mut self,
        stream_data: &serde_json::Value,
        events: &[PolykitEvent],
    ) -> WidgetPayload;
}

/// Registry of widget processors.
pub struct WidgetRegistry {
    processors: Vec<Box<dyn WidgetProcessor>>,
}

impl WidgetRegistry {
    pub fn new() -> Self {
        Self {
            processors: Vec::new(),
        }
    }

    pub fn register(&mut self, processor: Box<dyn WidgetProcessor>) {
        self.processors.push(processor);
    }

    /// Process all widgets with current stream data and event bus state.
    pub fn process_all(
        &mut self,
        stream_data: &serde_json::Value,
        bus: &mut EventBus,
    ) -> Vec<WidgetPayload> {
        let events = bus.drain();
        self.processors
            .iter_mut()
            .map(|p| p.process(stream_data, &events))
            .collect()
    }
}
