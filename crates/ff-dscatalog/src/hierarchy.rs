//! Master/user catalogue hierarchy -- scoped catalogue concepts.
//!
//! Implements `CatalogScope` (Requirement 29.1) and scope-aware uniqueness
//! validation (Requirement 29.4). Logical rename is catalogue-only and is
//! handled by `Catalog::rename()` which never moves physical objects
//! (Requirement 29.3, 20.6).
//!
//! Validates: Requirement 29.1, 29.2, 29.3, 29.4

use std::str::FromStr;

use crate::error::CatalogError;

// === CatalogScope ===========================================================

/// Scope discriminant for catalogue entries.
///
/// Mirrors the z/OS master/user catalogue hierarchy concept.
/// Resolution checks scope before priority order (Requirement 29.1, 29.2).
///
/// `#[non_exhaustive]` allows future scopes without breaking match arms.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
#[non_exhaustive]
pub enum CatalogScope {
    /// Master catalogue scope -- system-wide datasets visible to all users.
    Master,
    /// User catalogue scope -- user-owned datasets (default for new allocations).
    User,
}

impl CatalogScope {
    /// Return the canonical string representation stored in the database.
    pub fn as_str(&self) -> &'static str {
        match self {
            CatalogScope::Master => "master",
            CatalogScope::User => "user",
        }
    }
}

impl std::fmt::Display for CatalogScope {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_str(self.as_str())
    }
}

impl FromStr for CatalogScope {
    type Err = CatalogError;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        match s.to_lowercase().as_str() {
            "master" => Ok(CatalogScope::Master),
            "user" => Ok(CatalogScope::User),
            _ => Err(CatalogError::RepositoryCorrupt {
                path: String::new(),
                reason: format!("unknown catalogue scope: '{s}'"),
                operation: "parse_scope".to_string(),
            }),
        }
    }
}

// === Scope-aware uniqueness =================================================

/// Check that `dsn` does not already exist within `scope` across the provided
/// list of (dsn, scope) pairs.
///
/// Returns `Ok(())` when the DSN is unique within the scope, or
/// `Err(CatalogError::DuplicateDataset)` when a collision is found.
///
/// Validates: Requirement 29.4
pub fn check_scope_uniqueness(
    dsn: &str,
    scope: CatalogScope,
    existing: &[(String, CatalogScope)],
    catalog_name: &str,
) -> Result<(), CatalogError> {
    for (existing_dsn, existing_scope) in existing {
        if existing_dsn.eq_ignore_ascii_case(dsn) && *existing_scope == scope {
            return Err(CatalogError::DuplicateDataset {
                dsn: dsn.to_string(),
                catalog: catalog_name.to_string(),
                operation: "allocate".to_string(),
            });
        }
    }
    Ok(())
}

// === Tests ==================================================================

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn scope_display_and_parse_round_trip() {
        // Validates: Requirement 29.1
        for scope in [CatalogScope::Master, CatalogScope::User] {
            let s = scope.to_string();
            let back: CatalogScope = s.parse().expect("round-trip");
            assert_eq!(back, scope);
        }
    }

    #[test]
    fn scope_parse_case_insensitive() {
        // Validates: Requirement 29.1
        assert_eq!(
            "MASTER".parse::<CatalogScope>().unwrap(),
            CatalogScope::Master
        );
        assert_eq!("User".parse::<CatalogScope>().unwrap(), CatalogScope::User);
    }

    #[test]
    fn scope_parse_unknown_returns_error() {
        let result = "unknown_scope".parse::<CatalogScope>();
        assert!(result.is_err());
    }

    #[test]
    fn uniqueness_passes_when_no_collision() {
        // Validates: Requirement 29.4
        let existing = vec![("OTHER.DSN".to_string(), CatalogScope::User)];
        assert!(check_scope_uniqueness("NEW.DSN", CatalogScope::User, &existing, "CAT").is_ok());
    }

    #[test]
    fn uniqueness_fails_on_same_scope_collision() {
        // Validates: Requirement 29.4
        let existing = vec![("PAYROLL.DATA".to_string(), CatalogScope::User)];
        let result = check_scope_uniqueness("PAYROLL.DATA", CatalogScope::User, &existing, "CAT");
        assert!(matches!(result, Err(CatalogError::DuplicateDataset { .. })));
    }

    #[test]
    fn uniqueness_passes_when_same_dsn_different_scope() {
        // Validates: Requirement 29.4 -- scopes are independent namespaces
        let existing = vec![("PAYROLL.DATA".to_string(), CatalogScope::Master)];
        // Same DSN in User scope is allowed when Master already has it
        assert!(
            check_scope_uniqueness("PAYROLL.DATA", CatalogScope::User, &existing, "CAT").is_ok()
        );
    }

    #[test]
    fn uniqueness_check_is_case_insensitive() {
        // Validates: Requirement 29.4 + Requirement 2 AC 5
        let existing = vec![("PAYROLL.DATA".to_string(), CatalogScope::User)];
        let result = check_scope_uniqueness("payroll.data", CatalogScope::User, &existing, "CAT");
        assert!(matches!(result, Err(CatalogError::DuplicateDataset { .. })));
    }
}
