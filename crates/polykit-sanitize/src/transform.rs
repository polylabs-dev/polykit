//! Stage 2: Value Transform (redaction / abstraction)

use crate::{DataType, Detection};

/// Replace detected sensitive values with safe placeholders.
pub fn redact(input: &serde_json::Value, detections: &[Detection]) -> serde_json::Value {
    let mut output = input.clone();

    for detection in detections {
        let placeholder = placeholder_for(&detection.data_type);
        set_at_path(&mut output, &detection.field_path, serde_json::Value::String(placeholder));
    }

    output
}

/// Generate a safe placeholder for a data type.
fn placeholder_for(data_type: &DataType) -> String {
    match data_type {
        DataType::Ssn => "[PII_SSN]".to_string(),
        DataType::CreditCard => "[PCI_PAN]".to_string(),
        DataType::PersonalName => "[PII_NAME]".to_string(),
        DataType::Email => "[PII_EMAIL]".to_string(),
        DataType::PhoneNumber => "[PII_PHONE]".to_string(),
        DataType::DateOfBirth => "[PII_DOB]".to_string(),
        DataType::Address => "[PII_ADDRESS]".to_string(),
        DataType::MedicalRecord => "[HIPAA_MEDICAL]".to_string(),
        DataType::FinancialAccount => "[PII_FINANCIAL]".to_string(),
        DataType::BiometricData => "[PII_BIOMETRIC]".to_string(),
        DataType::Custom(name) => format!("[PII_{}]", name.to_uppercase()),
    }
}

fn set_at_path(value: &mut serde_json::Value, path: &str, replacement: serde_json::Value) {
    let parts: Vec<&str> = path.split('.').collect();
    let mut current = value;

    for (i, part) in parts.iter().enumerate() {
        if i == parts.len() - 1 {
            if let Some(obj) = current.as_object_mut() {
                obj.insert(part.to_string(), replacement);
                return;
            }
        } else {
            if let Some(obj) = current.as_object_mut() {
                if let Some(next) = obj.get_mut(*part) {
                    current = next;
                    continue;
                }
            }
            return;
        }
    }
}
