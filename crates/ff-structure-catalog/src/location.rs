//! Catalog location management.
//!
//! Provides [`CatalogLocationManager`] for adding, removing, renaming,
//! and switching catalog directory locations.

use crate::error::CatalogError;

/// Represents a configured catalog location with metadata.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct CatalogLocation {
    /// Filesystem path to the catalog directory.
    pub path: String,
    /// Human-readable label for display.
    pub label: String,
    /// Whether this is the active catalog location.
    pub is_active: bool,
    /// Whether the path is currently accessible.
    pub is_available: bool,
}

/// Manages configured catalog locations.
#[derive(Debug, Default)]
pub struct CatalogLocationManager {
    /// All configured locations.
    locations: Vec<CatalogLocation>,
}

impl CatalogLocationManager {
    /// Create a new empty location manager.
    pub fn new() -> Self {
        Self::default()
    }

    /// Create from an existing list of locations.
    pub fn from_locations(locations: Vec<CatalogLocation>) -> Self {
        Self { locations }
    }

    /// Get all configured locations.
    pub fn locations(&self) -> &[CatalogLocation] {
        &self.locations
    }

    /// Get the active location, if any.
    pub fn active_location(&self) -> Option<&CatalogLocation> {
        self.locations.iter().find(|loc| loc.is_active)
    }

    /// Add a new catalog location.
    ///
    /// # Errors
    ///
    /// Returns an error if the path is empty.
    pub fn add_location(&mut self, path: String, label: String) -> Result<(), CatalogError> {
        if path.is_empty() {
            return Err(CatalogError::ValidationFailed {
                detail: "catalog location path must be non-empty".to_string(),
            });
        }

        // Check for duplicates
        if self.locations.iter().any(|loc| loc.path == path) {
            return Err(CatalogError::DuplicateName { name: path });
        }

        let is_first = self.locations.is_empty();
        self.locations.push(CatalogLocation {
            path,
            label,
            is_active: is_first, // First location is active by default
            is_available: true,
        });
        Ok(())
    }

    /// Remove a catalog location by path.
    ///
    /// Does not delete the directory or its contents.
    pub fn remove_location(&mut self, path: &str) -> Result<(), CatalogError> {
        let pos = self
            .locations
            .iter()
            .position(|loc| loc.path == path)
            .ok_or_else(|| CatalogError::LocationNotFound {
                path: path.to_string(),
            })?;

        let was_active = self.locations[pos].is_active;
        self.locations.remove(pos);

        // If we removed the active location, make the first remaining one active
        if was_active {
            if let Some(first) = self.locations.first_mut() {
                first.is_active = true;
            }
        }

        Ok(())
    }

    /// Rename a catalog location's display label.
    pub fn rename_location(&mut self, path: &str, new_label: String) -> Result<(), CatalogError> {
        let loc = self
            .locations
            .iter_mut()
            .find(|loc| loc.path == path)
            .ok_or_else(|| CatalogError::LocationNotFound {
                path: path.to_string(),
            })?;

        loc.label = new_label;
        Ok(())
    }

    /// Set a location as the active catalog location.
    pub fn set_active(&mut self, path: &str) -> Result<(), CatalogError> {
        // Verify the path exists in our list
        if !self.locations.iter().any(|loc| loc.path == path) {
            return Err(CatalogError::LocationNotFound {
                path: path.to_string(),
            });
        }

        for loc in &mut self.locations {
            loc.is_active = loc.path == path;
        }
        Ok(())
    }

    /// Mark a location as unavailable (e.g., path doesn't exist).
    pub fn mark_unavailable(&mut self, path: &str) {
        if let Some(loc) = self.locations.iter_mut().find(|loc| loc.path == path) {
            loc.is_available = false;
        }
    }

    /// Return the number of configured locations.
    pub fn len(&self) -> usize {
        self.locations.len()
    }

    /// Return whether no locations are configured.
    pub fn is_empty(&self) -> bool {
        self.locations.is_empty()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    // Validates: Requirement 14.2 — add location
    #[test]
    fn add_location_creates_entry() {
        let mut mgr = CatalogLocationManager::new();
        mgr.add_location("/catalogs/project".to_string(), "Project".to_string())
            .unwrap();
        assert_eq!(mgr.len(), 1);
        assert_eq!(mgr.locations()[0].path, "/catalogs/project");
        assert_eq!(mgr.locations()[0].label, "Project");
    }

    // Validates: Requirement 14.2 — first location becomes active
    #[test]
    fn first_added_location_is_active() {
        let mut mgr = CatalogLocationManager::new();
        mgr.add_location("/first".to_string(), "First".to_string())
            .unwrap();
        assert!(mgr.active_location().is_some());
        assert_eq!(mgr.active_location().unwrap().path, "/first");
    }

    // Validates: Requirement 14.3 — remove location
    #[test]
    fn remove_location_deletes_entry() {
        let mut mgr = CatalogLocationManager::new();
        mgr.add_location("/a".to_string(), "A".to_string()).unwrap();
        mgr.add_location("/b".to_string(), "B".to_string()).unwrap();
        mgr.remove_location("/a").unwrap();
        assert_eq!(mgr.len(), 1);
        assert_eq!(mgr.locations()[0].path, "/b");
    }

    // Validates: Requirement 14.3 — removing active promotes next
    #[test]
    fn removing_active_location_promotes_next() {
        let mut mgr = CatalogLocationManager::new();
        mgr.add_location("/a".to_string(), "A".to_string()).unwrap();
        mgr.add_location("/b".to_string(), "B".to_string()).unwrap();
        mgr.remove_location("/a").unwrap();
        assert_eq!(mgr.active_location().unwrap().path, "/b");
    }

    // Validates: Requirement 14.4 — rename location
    #[test]
    fn rename_location_updates_label() {
        let mut mgr = CatalogLocationManager::new();
        mgr.add_location("/path".to_string(), "Old".to_string())
            .unwrap();
        mgr.rename_location("/path", "New Label".to_string())
            .unwrap();
        assert_eq!(mgr.locations()[0].label, "New Label");
    }

    // Validates: Requirement 14.5 — set active location
    #[test]
    fn set_active_switches_active_location() {
        let mut mgr = CatalogLocationManager::new();
        mgr.add_location("/a".to_string(), "A".to_string()).unwrap();
        mgr.add_location("/b".to_string(), "B".to_string()).unwrap();
        mgr.set_active("/b").unwrap();
        assert_eq!(mgr.active_location().unwrap().path, "/b");
    }

    // Validates: Requirement 14.5 — set active with unknown path
    #[test]
    fn set_active_unknown_path_errors() {
        let mut mgr = CatalogLocationManager::new();
        let result = mgr.set_active("/unknown");
        assert!(matches!(result, Err(CatalogError::LocationNotFound { .. })));
    }

    // Validates: Requirement 14.8 — mark unavailable on missing path
    #[test]
    fn mark_unavailable_updates_flag() {
        let mut mgr = CatalogLocationManager::new();
        mgr.add_location("/missing".to_string(), "Missing".to_string())
            .unwrap();
        assert!(mgr.locations()[0].is_available);
        mgr.mark_unavailable("/missing");
        assert!(!mgr.locations()[0].is_available);
    }

    // Validates: Requirement 14.7 — startup with no config
    #[test]
    fn empty_manager_has_no_active() {
        let mgr = CatalogLocationManager::new();
        assert!(mgr.is_empty());
        assert!(mgr.active_location().is_none());
    }

    // Validates: Requirement 14.2 — reject empty path
    #[test]
    fn add_location_rejects_empty_path() {
        let mut mgr = CatalogLocationManager::new();
        let result = mgr.add_location(String::new(), "Empty".to_string());
        assert!(result.is_err());
    }

    // Validates: Requirement 14.2 — reject duplicate path
    #[test]
    fn add_location_rejects_duplicate_path() {
        let mut mgr = CatalogLocationManager::new();
        mgr.add_location("/path".to_string(), "First".to_string())
            .unwrap();
        let result = mgr.add_location("/path".to_string(), "Second".to_string());
        assert!(result.is_err());
    }
}
