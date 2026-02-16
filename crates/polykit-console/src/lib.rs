//! PolyKit Console — widget data pipeline, event bus, RBAC, demo fixtures
//!
//! All data processing for console widgets runs in WASM.
//! The TS layer receives render-ready JSON payloads — no transforms in JS.

pub mod event_bus;
pub mod widget_data;
pub mod demo;
pub mod observability;
pub mod governance;
pub mod rbac;
