//! Temporary dataset registry.
//!
//! Job-scoped tracking of temporary datasets (`&&name`).
//! Temporary datasets never resolve against mounted catalogs.

use std::collections::HashMap;

use crate::diagnostic::{DiagnosticCode, LintDiagnostic};
use crate::operands::DcbAttributes;

/// A single temporary dataset registration.
#[derive(Debug, Clone)]
pub struct TempEntry {
    /// Step that created this temporary.
    pub creating_step: String,
    /// DCB attributes (if specified).
    pub attributes: Option<DcbAttributes>,
    /// Whether this temp has been deleted (DISP=(,DELETE)).
    pub deleted: bool,
}

/// Job-scoped tracking of temporary datasets.
#[derive(Debug, Clone)]
pub struct TempDatasetRegistry {
    /// Registered temporary datasets (name → entry).
    entries: HashMap<String, TempEntry>,
    /// Counter for system-generated names.
    next_sys_number: u32,
}

impl TempDatasetRegistry {
    /// Create a new empty registry.
    pub fn new() -> Self {
        Self {
            entries: HashMap::new(),
            next_sys_number: 1,
        }
    }

    /// Register a new temporary dataset.
    pub fn register(&mut self, name: &str, step_name: &str, attributes: Option<DcbAttributes>) {
        self.entries.insert(
            name.to_uppercase(),
            TempEntry {
                creating_step: step_name.to_string(),
                attributes,
                deleted: false,
            },
        );
    }

    /// Generate a system name and register it.
    ///
    /// Returns the generated name (format: `SYSnnnnn`).
    pub fn register_system_generated(
        &mut self,
        step_name: &str,
        attributes: Option<DcbAttributes>,
    ) -> String {
        let name = format!("SYS{:05}", self.next_sys_number);
        self.next_sys_number += 1;
        self.register(&name, step_name, attributes);
        name
    }

    /// Look up a temporary dataset.
    ///
    /// Returns the entry if found and not deleted, or a diagnostic if not found
    /// or already deleted.
    pub fn lookup(&self, name: &str, line: usize) -> Result<&TempEntry, LintDiagnostic> {
        let upper = name.to_uppercase();
        match self.entries.get(&upper) {
            Some(entry) if entry.deleted => Err(LintDiagnostic::new(
                DiagnosticCode::TemporaryNotFound,
                line,
                (0, 0),
                format!("Temporary dataset &&{} has been deleted", name),
            )),
            Some(entry) => Ok(entry),
            None => Err(LintDiagnostic::new(
                DiagnosticCode::TemporaryNotFound,
                line,
                (0, 0),
                format!("Temporary dataset not created in prior step: &&{}", name),
            )),
        }
    }

    /// Mark a temporary dataset as deleted.
    pub fn mark_deleted(&mut self, name: &str) {
        if let Some(entry) = self.entries.get_mut(&name.to_uppercase()) {
            entry.deleted = true;
        }
    }

    /// Returns true if the name is registered (even if deleted).
    pub fn contains(&self, name: &str) -> bool {
        self.entries.contains_key(&name.to_uppercase())
    }
}

impl Default for TempDatasetRegistry {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn register_and_lookup_temp_dataset() {
        // Validates: Requirement 6 AC 2, AC 3
        let mut registry = TempDatasetRegistry::new();
        registry.register("MYTEMP", "STEP1", None);

        let entry = registry.lookup("MYTEMP", 10).unwrap();
        assert_eq!(entry.creating_step, "STEP1");
        assert!(!entry.deleted);
    }

    #[test]
    fn lookup_unregistered_temp_produces_error() {
        // Validates: Requirement 6 AC 4
        let registry = TempDatasetRegistry::new();
        let result = registry.lookup("NOTHERE", 5);
        assert!(result.is_err());
        let diag = result.unwrap_err();
        assert_eq!(diag.code, DiagnosticCode::TemporaryNotFound);
        assert!(diag.message.contains("not created"));
    }

    #[test]
    fn system_generated_name_format() {
        // Validates: Requirement 6 AC 6
        let mut registry = TempDatasetRegistry::new();
        let name1 = registry.register_system_generated("STEP1", None);
        let name2 = registry.register_system_generated("STEP2", None);

        assert_eq!(name1, "SYS00001");
        assert_eq!(name2, "SYS00002");
        assert!(registry.contains("SYS00001"));
    }

    #[test]
    fn deleted_temp_cannot_be_referenced() {
        // Validates: Requirement 6 AC 7
        let mut registry = TempDatasetRegistry::new();
        registry.register("DELME", "STEP1", None);
        registry.mark_deleted("DELME");

        let result = registry.lookup("DELME", 10);
        assert!(result.is_err());
        assert!(result.unwrap_err().message.contains("deleted"));
    }

    #[test]
    fn temp_datasets_case_insensitive() {
        let mut registry = TempDatasetRegistry::new();
        registry.register("mytemp", "STEP1", None);
        assert!(registry.lookup("MYTEMP", 1).is_ok());
    }
}
