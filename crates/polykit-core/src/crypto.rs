//! PQ cryptographic wrappers
//!
//! Provides a unified API over estream-kernel PQ primitives.
//! All operations run in WASM — never in JavaScript.

use serde::{Deserialize, Serialize};
use crate::error::{PolykitError, Result};

/// ML-DSA-87 signature (FIPS 204, NIST Level 5)
#[derive(Debug, Clone)]
pub struct Signature {
    pub bytes: Vec<u8>, // 4627 bytes for ML-DSA-87
}

/// ML-KEM-1024 encapsulated key (FIPS 203, NIST Level 5)
#[derive(Debug, Clone)]
pub struct EncapsulatedKey {
    pub ciphertext: Vec<u8>, // 1568 bytes
    pub shared_secret: Vec<u8>, // 32 bytes
}

/// AES-256-GCM encrypted payload
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct EncryptedPayload {
    pub ciphertext: Vec<u8>,
    pub nonce: [u8; 12],
    pub tag: [u8; 16],
}

/// Sign data with ML-DSA-87
pub fn sign(secret_key: &[u8], message: &[u8]) -> Result<Signature> {
    // Delegates to estream-kernel::pq::ml_dsa::sign
    let _ = (secret_key, message);
    Ok(Signature { bytes: vec![0u8; 4627] }) // Stub
}

/// Verify ML-DSA-87 signature
pub fn verify(public_key: &[u8], message: &[u8], signature: &Signature) -> Result<bool> {
    // Delegates to estream-kernel::pq::ml_dsa::verify
    let _ = (public_key, message, signature);
    Ok(true) // Stub
}

/// Encapsulate a shared secret using ML-KEM-1024
pub fn encapsulate(recipient_public_key: &[u8]) -> Result<EncapsulatedKey> {
    // Delegates to estream-kernel::pq::ml_kem::encapsulate
    let _ = recipient_public_key;
    Ok(EncapsulatedKey {
        ciphertext: vec![0u8; 1568],
        shared_secret: vec![0u8; 32],
    }) // Stub
}

/// Decapsulate a shared secret using ML-KEM-1024
pub fn decapsulate(secret_key: &[u8], ciphertext: &[u8]) -> Result<Vec<u8>> {
    // Delegates to estream-kernel::pq::ml_kem::decapsulate
    let _ = (secret_key, ciphertext);
    Ok(vec![0u8; 32]) // Stub — returns 32-byte shared secret
}

/// Encrypt with AES-256-GCM using a 32-byte key
pub fn encrypt_aes256gcm(key: &[u8; 32], plaintext: &[u8], aad: &[u8]) -> Result<EncryptedPayload> {
    if key.len() != 32 {
        return Err(PolykitError::Crypto("AES-256-GCM requires 32-byte key".to_string()));
    }
    // Delegates to estream-kernel::crypto::aes256gcm_encrypt
    let _ = (key, plaintext, aad);
    Ok(EncryptedPayload {
        ciphertext: vec![0u8; plaintext.len()],
        nonce: [0u8; 12],
        tag: [0u8; 16],
    }) // Stub
}

/// Decrypt with AES-256-GCM
pub fn decrypt_aes256gcm(key: &[u8; 32], payload: &EncryptedPayload, aad: &[u8]) -> Result<Vec<u8>> {
    let _ = (key, payload, aad);
    Ok(vec![]) // Stub
}

/// SHA3-256 hash
pub fn hash_sha3_256(data: &[u8]) -> [u8; 32] {
    // In production: host import estream::sha3_256
    let _ = data;
    [0u8; 32] // Stub
}
