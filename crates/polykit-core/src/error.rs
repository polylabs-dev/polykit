use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum PolykitError {
    /// SPARK identity derivation failed
    IdentityDerivation(String),
    /// Cryptographic operation failed
    Crypto(String),
    /// Metering limit exceeded
    MeteringLimit { dimension: MeteringDimension, current: u64, limit: u64 },
    /// Classification policy violation
    ClassificationViolation(String),
    /// Wire protocol error
    Wire(String),
    /// ESLite storage error
    Storage(String),
    /// Sanitization pipeline error
    Sanitization(String),
    /// RBAC authorization denied
    Unauthorized { required_role: String, actual_roles: Vec<String> },
}

#[derive(Debug, Clone, Copy, Serialize, Deserialize, PartialEq, Eq, Hash)]
pub enum MeteringDimension {
    /// E — Executions
    Executions,
    /// H — Hash operations
    Hashes,
    /// B — Bandwidth (bytes)
    Bandwidth,
    /// S — Storage (bytes)
    Storage,
    /// O — Observable events
    Observables,
    /// P — Proofs generated
    Proofs,
    /// C — Circuit invocations
    Circuits,
    /// M — MPC sessions
    MpcSessions,
}

pub type Result<T> = core::result::Result<T, PolykitError>;
