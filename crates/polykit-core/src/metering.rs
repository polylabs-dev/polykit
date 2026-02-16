//! 8-Dimension Metering Model
//!
//! Every Poly app uses the same metering dimensions (E/H/B/S/O/P/C/M).
//! This module provides the types, accumulation logic, and stream emission
//! for metering records.

use serde::{Deserialize, Serialize};
use crate::error::MeteringDimension;

/// A metering record for a single operation.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct MeteringRecord {
    /// User identity (16-byte truncated SHA3-256 of SPARK public key)
    pub user_id: [u8; 16],
    /// Operation name (e.g., "upload", "download", "send_message")
    pub operation: String,
    /// Dimension values consumed by this operation
    pub dimensions: DimensionValues,
    /// Unix timestamp (milliseconds)
    pub timestamp_ms: u64,
}

/// Values for each metering dimension.
/// Only non-zero dimensions need to be set.
#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct DimensionValues {
    /// E — Circuit executions
    pub executions: u64,
    /// H — Hash operations (SHA3-256, BLAKE3)
    pub hashes: u64,
    /// B — Bandwidth (bytes transferred over wire protocol)
    pub bandwidth: u64,
    /// S — Storage (bytes persisted to ESLite or scatter providers)
    pub storage: u64,
    /// O — Observable events emitted to StreamSight
    pub observables: u64,
    /// P — Proofs generated (STARK, PoVC, PoVCR)
    pub proofs: u64,
    /// C — Circuit invocations (SmartCircuit evaluate calls)
    pub circuits: u64,
    /// M — MPC sessions initiated
    pub mpc_sessions: u64,
}

impl DimensionValues {
    /// Accumulate another set of dimension values into this one.
    pub fn accumulate(&mut self, other: &DimensionValues) {
        self.executions += other.executions;
        self.hashes += other.hashes;
        self.bandwidth += other.bandwidth;
        self.storage += other.storage;
        self.observables += other.observables;
        self.proofs += other.proofs;
        self.circuits += other.circuits;
        self.mpc_sessions += other.mpc_sessions;
    }

    /// Get a specific dimension's value.
    pub fn get(&self, dim: MeteringDimension) -> u64 {
        match dim {
            MeteringDimension::Executions => self.executions,
            MeteringDimension::Hashes => self.hashes,
            MeteringDimension::Bandwidth => self.bandwidth,
            MeteringDimension::Storage => self.storage,
            MeteringDimension::Observables => self.observables,
            MeteringDimension::Proofs => self.proofs,
            MeteringDimension::Circuits => self.circuits,
            MeteringDimension::MpcSessions => self.mpc_sessions,
        }
    }

    /// Set a specific dimension's value.
    pub fn set(&mut self, dim: MeteringDimension, value: u64) {
        match dim {
            MeteringDimension::Executions => self.executions = value,
            MeteringDimension::Hashes => self.hashes = value,
            MeteringDimension::Bandwidth => self.bandwidth = value,
            MeteringDimension::Storage => self.storage = value,
            MeteringDimension::Observables => self.observables = value,
            MeteringDimension::Proofs => self.proofs = value,
            MeteringDimension::Circuits => self.circuits = value,
            MeteringDimension::MpcSessions => self.mpc_sessions = value,
        }
    }
}

/// Metering tier limits. Apps define per-tier limits.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TierLimits {
    pub tier_name: String,
    pub limits: DimensionValues,
}

/// Check if current usage exceeds tier limits on any dimension.
pub fn check_limits(current: &DimensionValues, limits: &TierLimits) -> Vec<MeteringDimension> {
    let mut violations = Vec::new();
    let all_dims = [
        MeteringDimension::Executions,
        MeteringDimension::Hashes,
        MeteringDimension::Bandwidth,
        MeteringDimension::Storage,
        MeteringDimension::Observables,
        MeteringDimension::Proofs,
        MeteringDimension::Circuits,
        MeteringDimension::MpcSessions,
    ];
    for dim in all_dims {
        let limit = limits.limits.get(dim);
        if limit > 0 && current.get(dim) > limit {
            violations.push(dim);
        }
    }
    violations
}
