//! Wire protocol framing and session management
//!
//! Wraps the eStream Wire Protocol (UDP / QUIC / WebTransport) for use
//! from WASM. All wire operations happen in WASM — TypeScript never
//! frames, signs, or encrypts wire messages.

use serde::{Deserialize, Serialize};
use crate::error::Result;
use crate::identity::AppContext;

/// Wire protocol transport preference.
#[derive(Debug, Clone, Copy, Serialize, Deserialize)]
pub enum Transport {
    /// UDP :5000 (primary, full PQ)
    Udp,
    /// WebTransport :4433 (browser fallback)
    WebTransport,
}

/// Wire session state.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct WireSession {
    /// Session token from SPARK authentication
    pub session_token: Vec<u8>,
    /// Active transport
    pub transport: Transport,
    /// Connected edge node
    pub edge_node: String,
}

/// SPARK authentication message types (wire protocol opcodes)
pub mod opcodes {
    pub const SPARK_CHALLENGE_REQUEST: u8 = 0x50;
    pub const SPARK_CHALLENGE: u8 = 0x51;
    pub const SPARK_AUTH_REQUEST: u8 = 0x52;
    pub const SPARK_SESSION_GRANT: u8 = 0x53;
}

/// Perform SPARK authentication over wire protocol.
/// Returns a WireSession on success.
pub fn authenticate(
    _ctx: &AppContext,
    _signing_key: &[u8],
    _transport: Transport,
) -> Result<WireSession> {
    // In production: sends SparkChallengeRequest (0x50), receives challenge (0x51),
    // signs with ML-DSA-87, sends SparkAuthRequest (0x52), receives session grant (0x53)
    Ok(WireSession {
        session_token: vec![0u8; 32],
        transport: Transport::WebTransport,
        edge_node: String::new(),
    }) // Stub
}

/// Subscribe to a lex stream topic.
pub fn subscribe(_session: &WireSession, _topic: &str) -> Result<SubscriptionHandle> {
    Ok(SubscriptionHandle { id: 0 }) // Stub
}

/// Emit a message to a lex stream topic.
pub fn emit(_session: &WireSession, _topic: &str, _payload: &[u8]) -> Result<()> {
    Ok(()) // Stub
}

/// Handle for an active stream subscription.
#[derive(Debug, Clone)]
pub struct SubscriptionHandle {
    pub id: u64,
}
