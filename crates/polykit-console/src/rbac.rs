//! Role-based access control for console widgets
//!
//! Runs in WASM. The TS layer calls check_access() before rendering
//! a widget — if denied, the widget shell shows an "Unauthorized" state.

use serde::{Deserialize, Serialize};

/// Standard roles available across all Poly apps.
/// Apps may define additional roles specific to their domain.
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq, Hash)]
pub enum StandardRole {
    /// Read-only access to observability data
    Viewer,
    /// Full operational access (deploy, configure, review)
    Operator,
    /// Compliance audit access (sanitization logs, regulatory export)
    Compliance,
}

/// Check if a user's roles satisfy a widget's required roles.
pub fn check_access(user_roles: &[String], required_roles: &[String]) -> bool {
    if required_roles.is_empty() {
        return true;
    }
    required_roles.iter().any(|req| user_roles.contains(req))
}

/// Format a role name with app prefix.
/// e.g., ("polydata", StandardRole::Viewer) → "polydata-viewer"
pub fn format_role(app: &str, role: StandardRole) -> String {
    let suffix = match role {
        StandardRole::Viewer => "viewer",
        StandardRole::Operator => "operator",
        StandardRole::Compliance => "compliance",
    };
    format!("{}-{}", app, suffix)
}
