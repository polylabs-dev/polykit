//! StreamSight observability data processors
//!
//! Generic processors for the 5 reusable observability widgets.
//! Parameterized by lex namespace — works for any Poly app.

use serde::{Deserialize, Serialize};
use crate::event_bus::PolykitEvent;
use crate::widget_data::{WidgetProcessor, WidgetPayload};

/// Deviation feed processor.
/// Subscribes to: {namespace}/metrics/deviations
pub struct DeviationFeedProcessor {
    pub app: String,
    pub namespace: String,
}

impl WidgetProcessor for DeviationFeedProcessor {
    fn widget_type(&self) -> &str { "polykit-deviation-feed" }

    fn process(
        &mut self,
        stream_data: &serde_json::Value,
        events: &[PolykitEvent],
    ) -> WidgetPayload {
        let mut data = stream_data.clone();

        // Apply circuit filter from event bus
        for event in events {
            if let PolykitEvent::ClassificationFilter { tag: Some(tag) } = event {
                // Filter deviations by classification context
                if let Some(deviations) = data.get_mut("deviations") {
                    // In production: filter array by classification
                    let _ = tag;
                }
            }
        }

        WidgetPayload {
            widget_id: format!("{}-deviation-feed", self.app),
            data,
            dirty: true,
        }
    }
}

/// Capacity forecast processor.
/// Subscribes to: {namespace}/capacity
pub struct CapacityForecastProcessor {
    pub app: String,
    pub namespace: String,
}

impl WidgetProcessor for CapacityForecastProcessor {
    fn widget_type(&self) -> &str { "polykit-capacity-forecast" }

    fn process(
        &mut self,
        stream_data: &serde_json::Value,
        events: &[PolykitEvent],
    ) -> WidgetPayload {
        let mut data = stream_data.clone();

        // Apply classification filter highlight
        for event in events {
            if let PolykitEvent::ClassificationFilter { tag } = event {
                data["highlighted_tier"] = serde_json::json!(tag);
            }
        }

        WidgetPayload {
            widget_id: format!("{}-capacity-forecast", self.app),
            data,
            dirty: true,
        }
    }
}

/// SLI dashboard processor.
/// Subscribes to: {namespace}/telemetry/sli
pub struct SliDashboardProcessor {
    pub app: String,
    pub namespace: String,
}

impl WidgetProcessor for SliDashboardProcessor {
    fn widget_type(&self) -> &str { "polykit-sli-dashboard" }

    fn process(&mut self, stream_data: &serde_json::Value, _events: &[PolykitEvent]) -> WidgetPayload {
        WidgetPayload {
            widget_id: format!("{}-sli-dashboard", self.app),
            data: stream_data.clone(),
            dirty: true,
        }
    }
}

/// Circuit health processor.
/// Subscribes to: {namespace}/telemetry
pub struct CircuitHealthProcessor {
    pub app: String,
    pub namespace: String,
}

impl WidgetProcessor for CircuitHealthProcessor {
    fn widget_type(&self) -> &str { "polykit-circuit-health" }

    fn process(&mut self, stream_data: &serde_json::Value, events: &[PolykitEvent]) -> WidgetPayload {
        let mut data = stream_data.clone();

        for event in events {
            if let PolykitEvent::InvestigateMetric { circuit, .. } = event {
                data["focused_circuit"] = serde_json::json!(circuit);
            }
        }

        WidgetPayload {
            widget_id: format!("{}-circuit-health", self.app),
            data,
            dirty: true,
        }
    }
}

/// Incident timeline processor.
/// Subscribes to: {namespace}/incidents
pub struct IncidentTimelineProcessor {
    pub app: String,
    pub namespace: String,
}

impl WidgetProcessor for IncidentTimelineProcessor {
    fn widget_type(&self) -> &str { "polykit-incident-timeline" }

    fn process(&mut self, stream_data: &serde_json::Value, events: &[PolykitEvent]) -> WidgetPayload {
        let mut data = stream_data.clone();

        for event in events {
            if let PolykitEvent::TimeRange { from_ms, to_ms } = event {
                data["time_filter"] = serde_json::json!({ "from": from_ms, "to": to_ms });
            }
        }

        WidgetPayload {
            widget_id: format!("{}-incident-timeline", self.app),
            data,
            dirty: true,
        }
    }
}
