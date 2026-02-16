//! Stage 1: PII Detection

use crate::{DataType, Detection, Regulation};

/// Scan input JSON for sensitive data patterns.
pub fn scan(input: &serde_json::Value) -> Vec<Detection> {
    let mut detections = Vec::new();
    scan_recursive(input, "", &mut detections);
    detections
}

fn scan_recursive(value: &serde_json::Value, path: &str, detections: &mut Vec<Detection>) {
    match value {
        serde_json::Value::String(s) => {
            if let Some(detection) = detect_pii(path, s) {
                detections.push(detection);
            }
        }
        serde_json::Value::Object(map) => {
            for (key, val) in map {
                let child_path = if path.is_empty() {
                    key.clone()
                } else {
                    format!("{}.{}", path, key)
                };
                scan_recursive(val, &child_path, detections);
            }
        }
        serde_json::Value::Array(arr) => {
            for (i, val) in arr.iter().enumerate() {
                let child_path = format!("{}[{}]", path, i);
                scan_recursive(val, &child_path, detections);
            }
        }
        _ => {}
    }
}

fn detect_pii(path: &str, value: &str) -> Option<Detection> {
    // SSN pattern: XXX-XX-XXXX
    if value.len() == 11 && value.chars().filter(|c| *c == '-').count() == 2 {
        let parts: Vec<&str> = value.split('-').collect();
        if parts.len() == 3 && parts[0].len() == 3 && parts[1].len() == 2 && parts[2].len() == 4 {
            if parts.iter().all(|p| p.chars().all(|c| c.is_ascii_digit())) {
                return Some(Detection {
                    field_path: path.to_string(),
                    data_type: DataType::Ssn,
                    regulation: vec![Regulation::Hipaa, Regulation::Gdpr],
                    confidence: 0.95,
                });
            }
        }
    }

    // Credit card pattern: 16 digits (possibly with separators)
    let digits_only: String = value.chars().filter(|c| c.is_ascii_digit()).collect();
    if digits_only.len() >= 13 && digits_only.len() <= 19 {
        if luhn_check(&digits_only) {
            return Some(Detection {
                field_path: path.to_string(),
                data_type: DataType::CreditCard,
                regulation: vec![Regulation::PciDss],
                confidence: 0.98,
            });
        }
    }

    // Email pattern
    if value.contains('@') && value.contains('.') && value.len() > 5 {
        return Some(Detection {
            field_path: path.to_string(),
            data_type: DataType::Email,
            regulation: vec![Regulation::Gdpr],
            confidence: 0.90,
        });
    }

    None
}

fn luhn_check(digits: &str) -> bool {
    let mut sum = 0;
    let mut double = false;
    for c in digits.chars().rev() {
        if let Some(d) = c.to_digit(10) {
            let mut d = d;
            if double {
                d *= 2;
                if d > 9 { d -= 9; }
            }
            sum += d;
            double = !double;
        }
    }
    sum % 10 == 0
}
