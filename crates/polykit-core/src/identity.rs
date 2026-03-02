//! SPARK identity runtime helpers
//!
//! Key derivation and crypto operations are now in polykit_identity.fl.
//! This module provides only the AppContext struct and topic formatting
//! helpers used by the WASM shim and React hooks.

use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AppContext {
    pub app_id: String,
    pub hkdf_context: String,
    pub lex_namespace: String,
}

pub fn create_app_context(app_id: &str, hkdf_context: &str, lex_namespace: &str) -> AppContext {
    AppContext {
        app_id: app_id.to_string(),
        hkdf_context: hkdf_context.to_string(),
        lex_namespace: lex_namespace.to_string(),
    }
}

pub fn format_user_topic(ctx: &AppContext, user_id: &[u8; 16], suffix: &str) -> String {
    let user_hex: String = user_id.iter().map(|b| format!("{:02x}", b)).collect();
    format!("{}.{}.{}", ctx.lex_namespace, user_hex, suffix)
}

pub fn format_global_topic(ctx: &AppContext, suffix: &str) -> String {
    format!("lex://estream/apps/{}/{}", ctx.lex_namespace, suffix)
}
