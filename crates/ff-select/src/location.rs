//! Criteria_Location management (catalog path CRUD).
//!
//! Manages Criteria_Locations and the Active_Criteria_Location via
//! a persistent CriteriaStore.

use std::fs;
use std::path::{Path, PathBuf};

use serde::{Deserialize, Serialize};

use crate::config::CriteriaConfig;
use crate::error::CriteriaError;

/// The persistent store tracking Criteria_Locations and Active_Criteria_Location.
///
/// Stored as TOML in the configuration system's user layer.
///
/// Addresses: Requirement 9 AC 1, 2
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct CriteriaStore {
    /// All known Criteria_Locations.
    pub locations: Vec<CriteriaLocation>,
    /// The name/path of the Active_Criteria_Location.
    pub active_location: String,
}

/// A single Criteria_Location entry in the store.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct CriteriaLocation {
    /// User-assigned name for this location.
    pub name: String,
    /// Filesystem path to the criteria catalog directory.
    pub path: String,
}

/// Manages Criteria_Locations and the Active_Criteria_Location.
///
/// Addresses: Requirement 9 AC 1, 2, 3, 10
pub struct CriteriaLocationManager {
    store: CriteriaStore,
}

impl CriteriaLocationManager {
    /// Create from defaults using the provided configuration.
    ///
    /// Addresses: Requirement 9 AC 8
    pub fn new(config: &CriteriaConfig) -> Self {
        let default_location = CriteriaLocation {
            name: String::from("default"),
            path: config.default_location.clone(),
        };

        Self {
            store: CriteriaStore {
                locations: vec![default_location],
                active_location: config.default_location.clone(),
            },
        }
    }

    /// Load the CriteriaStore from a TOML file.
    ///
    /// Handles absent/corrupt file gracefully by returning defaults.
    ///
    /// Addresses: Requirement 9 AC 8, 9
    pub fn load(store_path: &Path, config: &CriteriaConfig) -> Result<Self, CriteriaError> {
        if !store_path.exists() {
            return Ok(Self::new(config));
        }

        let contents = fs::read_to_string(store_path).map_err(|e| CriteriaError::Io {
            operation: String::from("read"),
            path: store_path.display().to_string(),
            detail: e.to_string(),
        })?;

        let store: CriteriaStore =
            toml::from_str(&contents).map_err(|e| CriteriaError::StoreCorrupt {
                path: store_path.display().to_string(),
                detail: e.to_string(),
            })?;

        Ok(Self { store })
    }

    /// Get the Active_Criteria_Location path.
    pub fn active_location(&self) -> &Path {
        Path::new(&self.store.active_location)
    }

    /// Get the Active_Criteria_Location as a `PathBuf`.
    pub fn active_location_path(&self) -> PathBuf {
        PathBuf::from(&self.store.active_location)
    }

    /// Set the Active_Criteria_Location.
    pub fn set_active_location(&mut self, path: &str) -> Result<(), CriteriaError> {
        // Verify the location exists in our known locations
        if !self.store.locations.iter().any(|l| l.path == path) {
            return Err(CriteriaError::InvalidConfig {
                key: String::from("active_location"),
                value: path.to_string(),
            });
        }
        self.store.active_location = path.to_string();
        Ok(())
    }

    /// Add a new Criteria_Location.
    pub fn add_location(&mut self, name: &str, path: &str) -> Result<(), CriteriaError> {
        if self.store.locations.iter().any(|l| l.name == name) {
            return Err(CriteriaError::NameCollision {
                name: name.to_string(),
            });
        }
        self.store.locations.push(CriteriaLocation {
            name: name.to_string(),
            path: path.to_string(),
        });
        Ok(())
    }

    /// Remove a Criteria_Location by name.
    pub fn remove_location(&mut self, name: &str) -> Result<(), CriteriaError> {
        let initial_len = self.store.locations.len();
        self.store.locations.retain(|l| l.name != name);
        if self.store.locations.len() == initial_len {
            return Err(CriteriaError::CriteriaNotFound {
                name: name.to_string(),
                location: String::from("criteria store"),
            });
        }
        Ok(())
    }

    /// List all configured Criteria_Locations.
    pub fn locations(&self) -> &[CriteriaLocation] {
        &self.store.locations
    }

    /// Persist the CriteriaStore to its configured file.
    pub fn save(&self, store_path: &Path) -> Result<(), CriteriaError> {
        let contents =
            toml::to_string_pretty(&self.store).map_err(|e| CriteriaError::ParseFailed {
                path: store_path.display().to_string(),
                detail: e.to_string(),
            })?;

        if let Some(parent) = store_path.parent() {
            if !parent.exists() {
                fs::create_dir_all(parent).map_err(|e| CriteriaError::Io {
                    operation: String::from("create_dir"),
                    path: parent.display().to_string(),
                    detail: e.to_string(),
                })?;
            }
        }

        fs::write(store_path, contents).map_err(|e| CriteriaError::Io {
            operation: String::from("write"),
            path: store_path.display().to_string(),
            detail: e.to_string(),
        })?;

        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use tempfile::TempDir;

    fn default_config() -> CriteriaConfig {
        CriteriaConfig::default()
    }

    #[test]
    fn new_creates_default_location() {
        let mgr = CriteriaLocationManager::new(&default_config());
        assert_eq!(mgr.locations().len(), 1);
        assert_eq!(mgr.locations()[0].name, "default");
    }

    #[test]
    fn active_location_returns_default() {
        let config = CriteriaConfig {
            default_location: String::from("/tmp/criteria"),
            ..Default::default()
        };
        let mgr = CriteriaLocationManager::new(&config);
        assert_eq!(mgr.active_location(), Path::new("/tmp/criteria"));
    }

    #[test]
    fn add_and_remove_location() {
        let mut mgr = CriteriaLocationManager::new(&default_config());
        mgr.add_location("custom", "/custom/path").unwrap();
        assert_eq!(mgr.locations().len(), 2);

        mgr.remove_location("custom").unwrap();
        assert_eq!(mgr.locations().len(), 1);
    }

    #[test]
    fn add_duplicate_name_returns_error() {
        let mut mgr = CriteriaLocationManager::new(&default_config());
        let result = mgr.add_location("default", "/other/path");
        assert!(matches!(result, Err(CriteriaError::NameCollision { .. })));
    }

    #[test]
    fn remove_nonexistent_returns_error() {
        let mut mgr = CriteriaLocationManager::new(&default_config());
        let result = mgr.remove_location("nonexistent");
        assert!(matches!(
            result,
            Err(CriteriaError::CriteriaNotFound { .. })
        ));
    }

    #[test]
    fn set_active_location_validates_existence() {
        let mut mgr = CriteriaLocationManager::new(&default_config());
        let result = mgr.set_active_location("/unknown/path");
        assert!(matches!(result, Err(CriteriaError::InvalidConfig { .. })));
    }

    #[test]
    fn save_and_load_round_trip() {
        let dir = TempDir::new().unwrap();
        let store_path = dir.path().join("criteria_store.toml");
        let config = CriteriaConfig {
            default_location: String::from("/tmp/criteria"),
            ..Default::default()
        };

        let mut mgr = CriteriaLocationManager::new(&config);
        mgr.add_location("extra", "/extra/path").unwrap();
        mgr.save(&store_path).unwrap();

        let loaded = CriteriaLocationManager::load(&store_path, &config).unwrap();
        assert_eq!(loaded.locations().len(), 2);
    }

    #[test]
    fn load_nonexistent_returns_defaults() {
        let path = Path::new("/nonexistent/store.toml");
        let config = default_config();
        let mgr = CriteriaLocationManager::load(path, &config).unwrap();
        assert_eq!(mgr.locations().len(), 1);
    }
}
