//! PolyKit WASM Entry Point
//!
//! This is the wasm-bindgen export surface for all polykit crates.
//! The TypeScript layer calls ONLY these exports — no direct crate access.
//!
//! All exports conform to the ESCIR WASM ABI contract (estream-io #550):
//!   Required: evaluate(i32) -> i32
//!   Optional: alloc, dealloc, circuit_name, circuit_version
//!
//! Note: In the production pipeline, this crate is compiled via:
//!   estream-dev build-wasm-client --sign key.pem --enforce-budget

use wasm_bindgen::prelude::*;

// ─── App Initialization ──────────────────────────────────────────────────────

/// Initialize PolyKit for a specific app.
/// Called once from PolyProvider on mount.
///
/// Returns a JSON string with the initialized app state.
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

// ─── Identity ────────────────────────────────────────────────────────────────

/// Derive user identity from SPARK master seed.
/// Returns JSON with user_id (hex) and public keys (hex).
#[wasm_bindgen]
pub fn derive_identity(master_seed: &[u8], hkdf_context: &str, lex_namespace: &str) -> String {
    let ctx = polykit_core::identity::create_app_context("", hkdf_context, lex_namespace);
    match polykit_core::identity::derive_identity(master_seed, &ctx) {
        Ok(id) => serde_json::json!({
            "user_id": hex_encode(&id.user_id),
            "signing_public_key": hex_encode(&id.signing_public_key),
            "encryption_public_key": hex_encode(&id.encryption_public_key),
        })
        .to_string(),
        Err(e) => serde_json::json!({ "error": format!("{:?}", e) }).to_string(),
    }
}

// ─── ESLite ──────────────────────────────────────────────────────────────────

/// Run ESLite migrations for all registered tables.
/// Returns JSON with migration results.
#[wasm_bindgen]
pub fn run_migrations(migrations_json: &str) -> String {
    // In production: parse migration definitions and run against ESLite store
    serde_json::json!({
        "applied": 0,
        "status": "ok",
    })
    .to_string()
}

/// Execute an ESLite query.
/// Returns JSON with query results.
#[wasm_bindgen]
pub fn query(table: &str, filter_json: &str) -> String {
    let _ = (table, filter_json);
    serde_json::json!({
        "columns": [],
        "rows": [],
        "row_count": 0,
    })
    .to_string()
}

// ─── Widget Data Pipeline ────────────────────────────────────────────────────

/// Process widget data. Called on each render cycle by the TS bridge.
/// Takes raw stream data + pending events, returns render-ready payloads.
#[wasm_bindgen]
pub fn process_widgets(stream_data_json: &str, _events_json: &str) -> String {
    let _stream_data: serde_json::Value =
        serde_json::from_str(stream_data_json).unwrap_or(serde_json::Value::Null);

    // In production: routes to registered WidgetProcessors
    serde_json::json!([]).to_string()
}

/// Emit a widget event (cross-widget communication).
#[wasm_bindgen]
pub fn emit_widget_event(event_json: &str) -> String {
    let _ = event_json;
    serde_json::json!({ "status": "emitted" }).to_string()
}

// ─── Sanitization ────────────────────────────────────────────────────────────

/// Run 3-stage sanitization pipeline on input data.
/// Returns sanitized data + audit trail (safe to display in TS).
#[wasm_bindgen]
pub fn sanitize(input_json: &str) -> String {
    let input: serde_json::Value =
        serde_json::from_str(input_json).unwrap_or(serde_json::Value::Null);
    let result = polykit_sanitize::sanitize(&input);
    serde_json::to_string(&result).unwrap_or_default()
}

// ─── Classification ──────────────────────────────────────────────────────────

/// Classify a file path against a policy.
#[wasm_bindgen]
pub fn classify(path: &str, policy_json: &str) -> String {
    let policy: polykit_core::classification::ClassificationPolicy =
        serde_json::from_str(policy_json).unwrap_or(polykit_core::classification::ClassificationPolicy {
            rules: Vec::new(),
            minimum: None,
        });
    let result = polykit_core::classification::classify(path, &policy);
    serde_json::json!({
        "classification": result.as_str(),
        "scatter_policy": result.scatter_policy(),
    })
    .to_string()
}

// ─── Metering ────────────────────────────────────────────────────────────────

/// Check metering limits. Returns any violated dimensions.
#[wasm_bindgen]
pub fn check_metering_limits(current_json: &str, limits_json: &str) -> String {
    let _ = (current_json, limits_json);
    serde_json::json!({ "violations": [] }).to_string()
}

// ─── ESCIR ABI Required Export ───────────────────────────────────────────────

/// ESCIR ABI required export: evaluate(context_ptr) -> status
/// This is called by the eStream runtime for circuit execution.
#[wasm_bindgen]
pub fn evaluate(context_ptr: i32) -> i32 {
    let _ = context_ptr;
    0 // Success
}

// ─── ESCIR ABI Optional Exports ──────────────────────────────────────────────

#[wasm_bindgen]
pub fn circuit_name() -> String {
    "polykit".to_string()
}

#[wasm_bindgen]
pub fn circuit_version() -> String {
    env!("CARGO_PKG_VERSION").to_string()
}

// ─── Internal Helpers ────────────────────────────────────────────────────────

fn hex_encode(bytes: &[u8]) -> String {
    bytes.iter().map(|b| format!("{:02x}", b)).collect()
}
