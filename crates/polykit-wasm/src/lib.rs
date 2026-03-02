//! PolyKit WASM Entry Point
//!
//! Thin wasm-bindgen shim that routes JS calls to codegen'd circuit exports.
//! Most exports are auto-generated from FastLang circuits via ESCIR codegen.
//!
//! In the production pipeline, this crate is compiled via:
//!   estream-dev build-wasm-client --from-fl circuits/fl/ --sign key.pem --enforce-budget
//!
//! The codegen pipeline generates typed WASM exports from circuits with
//! `wasm_abi` annotations. This file provides only the bootstrap and
//! any hand-written glue that can't be expressed in FastLang.

use wasm_bindgen::prelude::*;

// --- App Initialization (hand-written: not a circuit) ---

#[wasm_bindgen]
pub fn init_app(app_id: &str, hkdf_context: &str, lex_namespace: &str, demo_mode: bool) -> String {
    let ctx = polykit_core::identity::create_app_context(app_id, hkdf_context, lex_namespace);
    serde_json::json!({
        "app_id": ctx.app_id,
        "lex_namespace": ctx.lex_namespace,
        "demo_mode": demo_mode,
        "status": "initialized",
    })
    .to_string()
}

// --- ESCIR ABI Required Export ---

#[wasm_bindgen]
pub fn evaluate(context_ptr: i32) -> i32 {
    let _ = context_ptr;
    0
}

#[wasm_bindgen]
pub fn circuit_name() -> String {
    "polykit".to_string()
}

#[wasm_bindgen]
pub fn circuit_version() -> String {
    env!("CARGO_PKG_VERSION").to_string()
}

// --- Codegen'd exports below ---
// The FastLang codegen pipeline (estream-dev build-wasm-client --from-fl)
// generates additional #[wasm_bindgen] exports for each circuit function:
//   - derive_keys, sign_message, verify_signature, encapsulate_key, ...
//   - record_usage, check_limits, get_usage_summary, ...
//   - check_rate, record_operation, ...
//   - sanitize, detect_only, ...
//   - classify_content, submit_feedback, get_thresholds, ...
//   - delta_encode, delta_decode, verify_exclusion, ...
//   - governed_emit, check_field_visibility, ...
//
// These are generated into a separate file (codegen_exports.rs) by the
// build pipeline and included here at compile time when available.
#[cfg(feature = "codegen")]
include!(concat!(env!("OUT_DIR"), "/codegen_exports.rs"));
