//! ESLM governance data processors
//!
//! Generic processors for the 4 reusable governance widgets.
//! Parameterized by lex namespace.

use serde::{Deserialize, Serialize};
use crate::event_bus::PolykitEvent;
use crate::widget_data::{WidgetProcessor, WidgetPayload};

/// ESLM review queue processor.
/// Subscribes to: {namespace}/eslm/classification
pub struct EslmReviewQueueProcessor {
    pub app: String,
    pub namespace: String,
}

impl WidgetProcessor for EslmReviewQueueProcessor {
    fn widget_type(&self) -> &str { "polykit-eslm-review-queue" }

    fn process(
        &mut self,
        stream_data: &serde_json::Value,
        events: &[PolykitEvent],
    ) -> WidgetPayload {
        let mut data = stream_data.clone();

        // Apply classification filter
        for event in events {
            if let PolykitEvent::ClassificationFilter { tag } = event {
                data["classification_filter"] = serde_json::json!(tag);
            }
        }

        WidgetPayload {
            widget_id: format!("{}-eslm-review-queue", self.app),
            data,
            dirty: true,
        }
    }
}

/// Sanitization log processor.
/// Subscribes to: {namespace}/eslm/sanitization
pub struct SanitizationLogProcessor {
    pub app: String,
    pub namespace: String,
}

impl WidgetProcessor for SanitizationLogProcessor {
    fn widget_type(&self) -> &str { "polykit-sanitization-log" }

    fn process(
        &mut self,
        stream_data: &serde_json::Value,
        events: &[PolykitEvent],
    ) -> WidgetPayload {
        let mut data = stream_data.clone();

        for event in events {
            if let PolykitEvent::TimeRange { from_ms, to_ms } = event {
                data["time_filter"] = serde_json::json!({ "from": from_ms, "to": to_ms });
            }
        }

        WidgetPayload {
            widget_id: format!("{}-sanitization-log", self.app),
            data,
            dirty: true,
        }
    }
}

/// ESLM feedback dashboard processor.
/// Subscribes to: {namespace}/eslm/classification
pub struct EslmFeedbackProcessor {
    pub app: String,
    pub namespace: String,
}

impl WidgetProcessor for EslmFeedbackProcessor {
    fn widget_type(&self) -> &str { "polykit-eslm-feedback" }

    fn process(
        &mut self,
        stream_data: &serde_json::Value,
        events: &[PolykitEvent],
    ) -> WidgetPayload {
        let mut data = stream_data.clone();

        for event in events {
            if let PolykitEvent::ReviewCompleted { action, classification, .. } = event {
                data["last_review"] = serde_json::json!({
                    "action": action,
                    "classification": classification,
                });
            }
        }

        WidgetPayload {
            widget_id: format!("{}-eslm-feedback", self.app),
            data,
            dirty: true,
        }
    }
}

/// ESN-AI recommendations processor.
/// Subscribes to: {namespace}/eslm/recommendation
pub struct EsnAiRecommendationsProcessor {
    pub app: String,
    pub namespace: String,
}

impl WidgetProcessor for EsnAiRecommendationsProcessor {
    fn widget_type(&self) -> &str { "polykit-esn-ai-recommendations" }

    fn process(
        &mut self,
        stream_data: &serde_json::Value,
        events: &[PolykitEvent],
    ) -> WidgetPayload {
        let mut data = stream_data.clone();

        // Highlight correlated recommendations when deviation is selected
        for event in events {
            if let PolykitEvent::DeviationSelect { metric, circuit, .. } = event {
                data["correlated_metric"] = serde_json::json!(metric);
                data["correlated_circuit"] = serde_json::json!(circuit);
            }
        }

        WidgetPayload {
            widget_id: format!("{}-esn-ai-recommendations", self.app),
            data,
            dirty: true,
        }
    }
}
