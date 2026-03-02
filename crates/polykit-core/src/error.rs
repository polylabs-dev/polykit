use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum PolykitError {
    IdentityDerivation(String),
    Crypto(String),
    MeteringLimit { dimension: MeteringDimension, current: u64, limit: u64 },
    ClassificationViolation(String),
    Wire(String),
    Storage(String),
    Sanitization(String),
    Unauthorized { required_role: String, actual_roles: Vec<String> },
}

#[derive(Debug, Clone, Copy, Serialize, Deserialize, PartialEq, Eq, Hash)]
pub enum MeteringDimension {
    Executions,
    Hashes,
    Bandwidth,
    Storage,
    Observables,
    Proofs,
    Circuits,
    MpcSessions,
}

pub type Result<T> = core::result::Result<T, PolykitError>;
