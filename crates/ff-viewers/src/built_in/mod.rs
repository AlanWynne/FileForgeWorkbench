//! Built-in viewer implementations.
//!
//! This module contains the four built-in viewers that are always available
//! without additional plugins: `asa-report`, `hex`, `image`, and `csv-table`.

pub mod asa_report;
pub mod csv_table;
pub mod hex;
pub mod image;

use crate::error::ViewerError;
use crate::registry::ViewerRegistry;

/// Register all built-in viewers into the provided registry.
///
/// This function is called during platform startup, before any plugin
/// initialization occurs.
///
/// # Errors
///
/// Returns an error if any built-in viewer fails to register (should not
/// happen in practice since built-in keys are hard-coded and unique).
pub fn register_built_in_viewers(registry: &ViewerRegistry) -> Result<(), ViewerError> {
    registry.register_builtin(Box::new(asa_report::AsaReportViewer::new()))?;
    registry.register_builtin(Box::new(hex::HexViewer::new()))?;
    registry.register_builtin(Box::new(image::ImageViewer::new()))?;
    registry.register_builtin(Box::new(csv_table::CsvTableViewer::new()))?;
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::key::ViewerKey;

    #[test]
    fn register_built_in_viewers_populates_registry_with_4_entries() {
        // Validates: Requirement 4 AC 5
        let registry = ViewerRegistry::new();
        register_built_in_viewers(&registry).unwrap();
        assert_eq!(registry.viewer_count(), 4);
    }

    #[test]
    fn all_built_in_keys_are_registered() {
        // Validates: Requirement 4 AC 1–4
        let registry = ViewerRegistry::new();
        register_built_in_viewers(&registry).unwrap();

        let asa_key = ViewerKey::new("asa-report").unwrap();
        let hex_key = ViewerKey::new("hex").unwrap();
        let image_key = ViewerKey::new("image").unwrap();
        let csv_key = ViewerKey::new("csv-table").unwrap();

        assert!(registry.contains(&asa_key));
        assert!(registry.contains(&hex_key));
        assert!(registry.contains(&image_key));
        assert!(registry.contains(&csv_key));
    }

    #[test]
    fn built_in_viewers_have_correct_metadata() {
        // Validates: Requirement 4 AC 1–4
        let registry = ViewerRegistry::new();
        register_built_in_viewers(&registry).unwrap();

        let list = registry.list_viewers();
        assert_eq!(list.len(), 4);

        // Verify each built-in has a non-empty display name and description
        for info in &list {
            assert!(!info.display_name.is_empty());
            assert!(!info.description.is_empty());
        }
    }
}
