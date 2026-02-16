//! Stage 3: PoVC-Witnessed Audit Record

use crate::{AuditEntry, Detection, Stage};

/// Create audit trail entries for all detections.
/// In production, each entry is PoVC-witnessed (hash chain + ML-DSA-87 signature).
pub fn record(detections: &[Detection]) -> Vec<AuditEntry> {
    let timestamp = current_timestamp_ms();
    let mut entries = Vec::new();

    for detection in detections {
        let witness_hash = compute_witness_hash(detection, timestamp);
        let regulations: Vec<String> = detection.regulation.iter().map(|r| format!("{:?}", r)).collect();

        // Stage 1 audit: what was detected
        entries.push(AuditEntry {
            timestamp_ms: timestamp,
            stage: Stage::PiiDetect,
            field_path: detection.field_path.clone(),
            original_type: format!("{:?}", detection.data_type),
            placeholder: String::new(),
            regulations: regulations.clone(),
            witness_hash: witness_hash.clone(),
        });

        // Stage 2 audit: what was replaced
        entries.push(AuditEntry {
            timestamp_ms: timestamp,
            stage: Stage::ValueTransform,
            field_path: detection.field_path.clone(),
            original_type: format!("{:?}", detection.data_type),
            placeholder: placeholder_for_type(&detection.data_type),
            regulations: regulations.clone(),
            witness_hash: witness_hash.clone(),
        });

        // Stage 3 audit: the record itself
        entries.push(AuditEntry {
            timestamp_ms: timestamp,
            stage: Stage::AuditRecord,
            field_path: detection.field_path.clone(),
            original_type: format!("{:?}", detection.data_type),
            placeholder: format!("[AUDIT_REF:0x{}]", &witness_hash[..4]),
            regulations,
            witness_hash,
        });
    }

    entries
}

fn placeholder_for_type(dt: &crate::DataType) -> String {
    match dt {
        crate::DataType::Ssn => "***-**-XXXX".to_string(),
        crate::DataType::CreditCard => "****-****-****-XXXX".to_string(),
        crate::DataType::Email => "u***@***.***".to_string(),
        _ => "[REDACTED]".to_string(),
    }
}

fn current_timestamp_ms() -> u64 {
    // In production: host import estream::get_time
    0
}

fn compute_witness_hash(detection: &Detection, timestamp: u64) -> String {
    // In production: SHA3-256(field_path || data_type || timestamp) signed by witness
    let input = format!("{}::{:?}::{}", detection.field_path, detection.data_type, timestamp);
    let hash_bytes = simple_hash(input.as_bytes());
    hex_encode(&hash_bytes[..6])
}

fn simple_hash(data: &[u8]) -> [u8; 32] {
    // Stub — delegates to estream::sha3_256 in production
    let _ = data;
    [0u8; 32]
}

fn hex_encode(bytes: &[u8]) -> String {
    bytes.iter().map(|b| format!("{:02x}", b)).collect()
}
