//! PolyKit Core — SPARK identity, PQ crypto, metering, classification
//!
//! This crate provides the shared foundation for all Poly Labs apps.
//! All logic runs in WASM via ESCIR codegen (estream-io #550).
//! TypeScript never touches crypto, state, or wire protocol framing.

pub mod identity;
pub mod crypto;
pub mod metering;
pub mod classification;
pub mod wire;
pub mod error;
