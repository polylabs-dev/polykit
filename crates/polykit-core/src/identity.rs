//! SPARK identity helpers
//!
//! Provides per-app HKDF key derivation from the SPARK master seed.
//! Each Poly app gets isolated keys via a unique context string
//! (e.g., "poly-data-v1", "poly-mail-v1", "poly-messenger-v1").
//!
//! Key hierarchy:
//!   SPARK master_seed
//!     └── HKDF-SHA3-256(master_seed, "{app-context}")
//!           ├── ML-DSA-87 signing key pair
//!           └── ML-KEM-1024 encryption key pair

use serde::{Deserialize, Serialize};
use crate::error::{PolykitError, Result};

/// Application identity context. Created once per app.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AppContext {
    /// Application identifier (e.g., "polydata", "polymessenger", "polymail")
    pub app_id: String,
    /// HKDF derivation context string (e.g., "poly-data-v1")
    pub hkdf_context: String,
    /// Lex namespace for stream topics (e.g., "polylabs.data")
    pub lex_namespace: String,
}

/// Derived identity for a user within an app.
/// All fields are deterministically derived from SPARK master seed + app context.
#[derive(Debug, Clone)]
pub struct DerivedIdentity {
    /// User ID: SHA3-256(ml_dsa_87_public_key)[0..16]
    pub user_id: [u8; 16],
    /// ML-DSA-87 signing key pair (FIPS 204, NIST Level 5)
    pub signing_public_key: Vec<u8>,
    pub signing_secret_key: Vec<u8>,
    /// ML-KEM-1024 encryption key pair (FIPS 203, NIST Level 5)
    pub encryption_public_key: Vec<u8>,
    pub encryption_secret_key: Vec<u8>,
}

/// Create a new app context. This is the entry point for identity in any Poly app.
pub fn create_app_context(app_id: &str, hkdf_context: &str, lex_namespace: &str) -> AppContext {
    AppContext {
        app_id: app_id.to_string(),
        hkdf_context: hkdf_context.to_string(),
        lex_namespace: lex_namespace.to_string(),
    }
}

/// Derive user identity from SPARK master seed and app context.
///
/// This runs entirely in WASM. The master_seed never leaves the secure enclave /
/// WASM boundary — it is passed in by the host (TEE/StrongBox) and used only
/// within this function.
pub fn derive_identity(master_seed: &[u8], ctx: &AppContext) -> Result<DerivedIdentity> {
    if master_seed.len() < 32 {
        return Err(PolykitError::IdentityDerivation(
            "master_seed must be at least 32 bytes".to_string(),
        ));
    }

    // HKDF-SHA3-256 expand with app-specific context
    // Produces 64 bytes: 32 for signing seed, 32 for encryption seed
    let derived_key_material = hkdf_sha3_256_expand(master_seed, ctx.hkdf_context.as_bytes(), 64)?;

    let signing_seed = &derived_key_material[..32];
    let encryption_seed = &derived_key_material[32..64];

    // ML-DSA-87 key generation from seed (deterministic)
    let (signing_pk, signing_sk) = ml_dsa_87_keygen_from_seed(signing_seed)?;

    // ML-KEM-1024 key generation from seed (deterministic)
    let (encryption_pk, encryption_sk) = ml_kem_1024_keygen_from_seed(encryption_seed)?;

    // user_id = SHA3-256(signing_public_key)[0..16]
    let pk_hash = sha3_256(&signing_pk);
    let mut user_id = [0u8; 16];
    user_id.copy_from_slice(&pk_hash[..16]);

    Ok(DerivedIdentity {
        user_id,
        signing_public_key: signing_pk,
        signing_secret_key: signing_sk,
        encryption_public_key: encryption_pk,
        encryption_secret_key: encryption_sk,
    })
}

/// Format a lex stream topic with the user's ID.
/// e.g., "polylabs.data.{user_id}.upload" → "polylabs.data.a1b2c3d4e5f6.upload"
pub fn format_user_topic(ctx: &AppContext, user_id: &[u8; 16], suffix: &str) -> String {
    let user_hex = hex_encode(user_id);
    format!("{}.{}.{}", ctx.lex_namespace, user_hex, suffix)
}

/// Format a global (non-user-specific) lex stream topic.
/// e.g., "lex://estream/apps/polylabs.data/telemetry"
pub fn format_global_topic(ctx: &AppContext, suffix: &str) -> String {
    format!("lex://estream/apps/{}/{}", ctx.lex_namespace, suffix)
}

// ── Internal crypto helpers (delegate to estream-kernel in real impl) ───────

fn hkdf_sha3_256_expand(ikm: &[u8], info: &[u8], len: usize) -> Result<Vec<u8>> {
    // In production: delegates to estream-kernel::crypto::hkdf_sha3_256
    // Placeholder: uses the host import estream::sha3_256
    let _ = (ikm, info, len);
    Ok(vec![0u8; len]) // Stub — replaced by estream-kernel integration
}

fn ml_dsa_87_keygen_from_seed(seed: &[u8]) -> Result<(Vec<u8>, Vec<u8>)> {
    // In production: delegates to estream-kernel::pq::ml_dsa::keygen_from_seed
    let _ = seed;
    Ok((vec![0u8; 2592], vec![0u8; 4896])) // ML-DSA-87 key sizes
}

fn ml_kem_1024_keygen_from_seed(seed: &[u8]) -> Result<(Vec<u8>, Vec<u8>)> {
    // In production: delegates to estream-kernel::pq::ml_kem::keygen_from_seed
    let _ = seed;
    Ok((vec![0u8; 1568], vec![0u8; 3168])) // ML-KEM-1024 key sizes
}

fn sha3_256(data: &[u8]) -> [u8; 32] {
    // In production: delegates to host import estream::sha3_256
    let _ = data;
    [0u8; 32] // Stub
}

fn hex_encode(bytes: &[u8]) -> String {
    bytes.iter().map(|b| format!("{:02x}", b)).collect()
}
