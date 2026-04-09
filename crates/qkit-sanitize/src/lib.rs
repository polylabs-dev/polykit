//! 3-Stage Compliance Sanitization Pipeline
//!
//! Stage 1: PII Detection — identifies sensitive data (SSN, PAN, names, emails)
//! Stage 2: Value Transform — redacts or abstracts detected values
//! Stage 3: Audit Record — creates PoVC-witnessed audit trail
//!
//! Compliance: HIPAA, PCI-DSS, GDPR, SOC2
//!
//! All stages run in WASM. Sensitive data never crosses the JS boundary.

pub mod detect;
pub mod transform;
pub mod audit;

use serde::{Deserialize, Serialize};

/// Detected sensitive data.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Detection {
    pub field_path: String,
    pub data_type: DataType,
    pub regulation: Vec<Regulation>,
    pub confidence: f64,
}

/// Sensitive data types.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum DataType {
    Ssn,
    CreditCard,
    PersonalName,
    Email,
    PhoneNumber,
    DateOfBirth,
    Address,
    MedicalRecord,
    FinancialAccount,
    BiometricData,
    Custom(String),
}

/// Applicable regulations.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum Regulation {
    Hipaa,
    PciDss,
    Gdpr,
    Soc2,
    Ccpa,
}

/// Sanitization result from the 3-stage pipeline.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SanitizationResult {
    /// Sanitized output (safe to display / cross JS boundary)
    pub sanitized_data: serde_json::Value,
    /// Audit entries for each detected item
    pub audit_entries: Vec<AuditEntry>,
}

/// Audit entry from stage 3.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AuditEntry {
    pub timestamp_ms: u64,
    pub stage: Stage,
    pub field_path: String,
    pub original_type: String,
    pub placeholder: String,
    pub regulations: Vec<String>,
    pub witness_hash: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum Stage {
    PiiDetect,
    ValueTransform,
    AuditRecord,
}

/// Run the full 3-stage sanitization pipeline on input data.
pub fn sanitize(input: &serde_json::Value) -> SanitizationResult {
    // Stage 1: Detect PII
    let detections = detect::scan(input);

    // Stage 2: Transform values
    let sanitized = transform::redact(input, &detections);

    // Stage 3: Create audit trail
    let audit_entries = audit::record(&detections);

    SanitizationResult {
        sanitized_data: sanitized,
        audit_entries,
    }
}
