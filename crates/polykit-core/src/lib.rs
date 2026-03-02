//! PolyKit Core — Thin kernel for runtime plumbing
//!
//! Identity, crypto, metering, and classification logic now lives in
//! FastLang circuits (circuits/fl/*.fl). This crate provides only the
//! runtime helpers that can't be expressed as circuits: AppContext,
//! topic formatting, and error types.
//!
//! See: circuits/fl/polykit_identity.fl (replaces identity.rs + crypto.rs)
//!      circuits/fl/polykit_metering.fl (replaces metering.rs)
//!      docs/FASTLANG_REFACTOR_PLAN.md

pub mod identity;
pub mod classification;
pub mod wire;
pub mod error;
