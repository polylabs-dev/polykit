//! Classification tiers and scatter policies
//!
//! Shared across all Poly apps that handle classified data.
//! Classification drives scatter policy (k-of-n erasure coding,
//! number of jurisdictions), retention, and access control.

use serde::{Deserialize, Serialize};

/// Data classification tiers, ordered by sensitivity.
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash, Serialize, Deserialize)]
pub enum Classification {
    Public,
    Internal,
    Confidential,
    Restricted,
    Sovereign,
}

impl Classification {
    /// Get the scatter policy for this classification tier.
    pub fn scatter_policy(&self) -> ScatterPolicy {
        match self {
            Classification::Public => ScatterPolicy { k: 2, n: 3, jurisdictions: 1 },
            Classification::Internal => ScatterPolicy { k: 3, n: 5, jurisdictions: 2 },
            Classification::Confidential => ScatterPolicy { k: 5, n: 7, jurisdictions: 3 },
            Classification::Restricted => ScatterPolicy { k: 7, n: 9, jurisdictions: 3 },
            Classification::Sovereign => ScatterPolicy { k: 9, n: 13, jurisdictions: 5 },
        }
    }

    /// Parse from string (case-insensitive).
    pub fn from_str(s: &str) -> Option<Self> {
        match s.to_uppercase().as_str() {
            "PUBLIC" => Some(Classification::Public),
            "INTERNAL" => Some(Classification::Internal),
            "CONFIDENTIAL" => Some(Classification::Confidential),
            "RESTRICTED" => Some(Classification::Restricted),
            "SOVEREIGN" => Some(Classification::Sovereign),
            _ => None,
        }
    }

    pub fn as_str(&self) -> &'static str {
        match self {
            Classification::Public => "PUBLIC",
            Classification::Internal => "INTERNAL",
            Classification::Confidential => "CONFIDENTIAL",
            Classification::Restricted => "RESTRICTED",
            Classification::Sovereign => "SOVEREIGN",
        }
    }
}

/// Scatter distribution policy derived from classification.
#[derive(Debug, Clone, Copy, Serialize, Deserialize)]
pub struct ScatterPolicy {
    /// Minimum shards needed to reconstruct (erasure coding threshold)
    pub k: u32,
    /// Total shards distributed
    pub n: u32,
    /// Minimum distinct jurisdictions for shard placement
    pub jurisdictions: u32,
}

/// A classification rule: pattern → classification.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ClassificationRule {
    /// Glob pattern to match (e.g., "*.xlsx", "/finance/**")
    pub pattern: String,
    /// Classification to assign when pattern matches
    pub classification: Classification,
}

/// Classification policy: ordered list of rules + minimum floor.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ClassificationPolicy {
    pub rules: Vec<ClassificationRule>,
    /// Minimum classification for all data (floor)
    pub minimum: Option<Classification>,
}

/// Evaluate classification for a given path against a policy.
pub fn classify(path: &str, policy: &ClassificationPolicy) -> Classification {
    let mut result = policy.minimum.unwrap_or(Classification::Public);

    for rule in &policy.rules {
        if glob_match(&rule.pattern, path) && rule.classification > result {
            result = rule.classification;
        }
    }

    result
}

fn glob_match(pattern: &str, path: &str) -> bool {
    // Simplified glob matching — production uses estream-kernel::patterns::glob
    if pattern == "**" {
        return true;
    }
    if let Some(ext) = pattern.strip_prefix("*.") {
        return path.ends_with(&format!(".{}", ext));
    }
    if let Some(prefix) = pattern.strip_suffix("/**") {
        return path.starts_with(prefix);
    }
    path == pattern
}
