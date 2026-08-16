//! JSON-based persistence for named criteria sets.
//!
//! Handles loading and saving CriteriaSets to `.criteria.json` files
//! in Criteria_Locations.

use std::fs;
use std::path::{Path, PathBuf};

use crate::error::CriteriaError;
use crate::model::CriteriaSet;

/// Metadata about a saved CriteriaSet (for catalog listing).
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct CriteriaSetMetadata {
    /// The criteria set name.
    pub name: String,
    /// Optional structure association.
    pub structure_association: Option<String>,
    /// Number of criterion rows.
    pub criteria_count: usize,
    /// Path to the `.criteria.json` file.
    pub file_path: PathBuf,
}

/// Handles loading and saving CriteriaSets to `.criteria.json` files.
///
/// Addresses: Requirement 9
pub struct CriteriaPersistence;

impl CriteriaPersistence {
    /// Load a named CriteriaSet from the given criteria location.
    ///
    /// Returns an error if the file doesn't exist or is unparseable.
    ///
    /// Addresses: Requirement 9 AC 4, 7
    pub fn load(location: &Path, name: &str) -> Result<CriteriaSet, CriteriaError> {
        let file_name = format!("{}.criteria.json", CriteriaSet::sanitise_name(name));
        let file_path = location.join(&file_name);

        if !file_path.exists() {
            return Err(CriteriaError::CriteriaNotFound {
                name: name.to_string(),
                location: location.display().to_string(),
            });
        }

        let contents = fs::read_to_string(&file_path).map_err(|e| CriteriaError::Io {
            operation: String::from("read"),
            path: file_path.display().to_string(),
            detail: e.to_string(),
        })?;

        let criteria_set: CriteriaSet =
            serde_json::from_str(&contents).map_err(|e| CriteriaError::ParseFailed {
                path: file_path.display().to_string(),
                detail: e.to_string(),
            })?;

        Ok(criteria_set)
    }

    /// Save a CriteriaSet to the given criteria location.
    ///
    /// The file name is derived from the criteria set name.
    ///
    /// Addresses: Requirement 9 AC 4, 5, 6
    pub fn save(location: &Path, criteria: &CriteriaSet) -> Result<(), CriteriaError> {
        let name = criteria.name.as_deref().unwrap_or("unnamed");
        let file_name = format!("{}.criteria.json", CriteriaSet::sanitise_name(name));
        let file_path = location.join(&file_name);

        // Ensure directory exists
        if !location.exists() {
            fs::create_dir_all(location).map_err(|e| CriteriaError::Io {
                operation: String::from("create_dir"),
                path: location.display().to_string(),
                detail: e.to_string(),
            })?;
        }

        let json =
            serde_json::to_string_pretty(criteria).map_err(|e| CriteriaError::ParseFailed {
                path: file_path.display().to_string(),
                detail: e.to_string(),
            })?;

        fs::write(&file_path, json).map_err(|e| CriteriaError::Io {
            operation: String::from("write"),
            path: file_path.display().to_string(),
            detail: e.to_string(),
        })?;

        Ok(())
    }

    /// List all saved CriteriaSets in the given criteria location.
    ///
    /// Returns metadata (name, structure_association, row count) for each.
    ///
    /// Addresses: Requirement 11 AC 2
    pub fn list(location: &Path) -> Result<Vec<CriteriaSetMetadata>, CriteriaError> {
        if !location.exists() {
            return Ok(Vec::new());
        }

        let entries = fs::read_dir(location).map_err(|e| CriteriaError::Io {
            operation: String::from("read_dir"),
            path: location.display().to_string(),
            detail: e.to_string(),
        })?;

        let mut metadata_list = Vec::new();

        for entry in entries {
            let entry = entry.map_err(|e| CriteriaError::Io {
                operation: String::from("read_dir_entry"),
                path: location.display().to_string(),
                detail: e.to_string(),
            })?;

            let path = entry.path();
            if path.extension().and_then(|e| e.to_str()) != Some("json") {
                continue;
            }

            let file_name = path.file_name().and_then(|n| n.to_str()).unwrap_or("");
            if !file_name.ends_with(".criteria.json") {
                continue;
            }

            // Try to parse for metadata
            let contents = match fs::read_to_string(&path) {
                Ok(c) => c,
                Err(_) => continue,
            };

            let criteria_set: CriteriaSet = match serde_json::from_str(&contents) {
                Ok(cs) => cs,
                Err(_) => continue,
            };

            let name = criteria_set.name.clone().unwrap_or_else(|| {
                file_name
                    .strip_suffix(".criteria.json")
                    .unwrap_or(file_name)
                    .to_string()
            });

            metadata_list.push(CriteriaSetMetadata {
                name,
                structure_association: criteria_set.structure_association,
                criteria_count: criteria_set.criteria.len(),
                file_path: path,
            });
        }

        Ok(metadata_list)
    }

    /// Delete a saved CriteriaSet by name from the given location.
    ///
    /// Addresses: Requirement 11 AC 7
    pub fn delete(location: &Path, name: &str) -> Result<(), CriteriaError> {
        let file_name = format!("{}.criteria.json", CriteriaSet::sanitise_name(name));
        let file_path = location.join(&file_name);

        if !file_path.exists() {
            return Err(CriteriaError::CriteriaNotFound {
                name: name.to_string(),
                location: location.display().to_string(),
            });
        }

        fs::remove_file(&file_path).map_err(|e| CriteriaError::Io {
            operation: String::from("delete"),
            path: file_path.display().to_string(),
            detail: e.to_string(),
        })?;

        Ok(())
    }

    /// Duplicate a saved CriteriaSet under a new name.
    ///
    /// Addresses: Requirement 11 AC 6
    pub fn duplicate(
        location: &Path,
        source_name: &str,
        new_name: &str,
    ) -> Result<(), CriteriaError> {
        let mut criteria_set = Self::load(location, source_name)?;
        criteria_set.name = Some(new_name.to_string());
        Self::save(location, &criteria_set)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::model::CriteriaOperator;
    use tempfile::TempDir;

    fn make_test_criteria(name: &str) -> CriteriaSet {
        CriteriaSet {
            name: Some(name.to_string()),
            structure_association: Some("TEST_STRUCT".to_string()),
            ..CriteriaSet::single("FIELD1", CriteriaOperator::Eq, "value1")
        }
    }

    #[test]
    fn save_and_load_round_trip() {
        let dir = TempDir::new().unwrap();
        let cs = make_test_criteria("test_set");

        CriteriaPersistence::save(dir.path(), &cs).unwrap();
        let loaded = CriteriaPersistence::load(dir.path(), "test_set").unwrap();
        assert_eq!(cs, loaded);
    }

    #[test]
    fn load_nonexistent_returns_not_found_error() {
        let dir = TempDir::new().unwrap();
        let result = CriteriaPersistence::load(dir.path(), "missing");
        assert!(matches!(
            result,
            Err(CriteriaError::CriteriaNotFound { .. })
        ));
    }

    #[test]
    fn list_returns_all_criteria_files() {
        let dir = TempDir::new().unwrap();
        CriteriaPersistence::save(dir.path(), &make_test_criteria("alpha")).unwrap();
        CriteriaPersistence::save(dir.path(), &make_test_criteria("beta")).unwrap();

        let list = CriteriaPersistence::list(dir.path()).unwrap();
        assert_eq!(list.len(), 2);

        let names: Vec<&str> = list.iter().map(|m| m.name.as_str()).collect();
        assert!(names.contains(&"alpha"));
        assert!(names.contains(&"beta"));
    }

    #[test]
    fn list_empty_directory_returns_empty() {
        let dir = TempDir::new().unwrap();
        let list = CriteriaPersistence::list(dir.path()).unwrap();
        assert!(list.is_empty());
    }

    #[test]
    fn list_nonexistent_directory_returns_empty() {
        let path = Path::new("/nonexistent/criteria/path");
        let list = CriteriaPersistence::list(path).unwrap();
        assert!(list.is_empty());
    }

    #[test]
    fn delete_removes_file() {
        let dir = TempDir::new().unwrap();
        CriteriaPersistence::save(dir.path(), &make_test_criteria("to_delete")).unwrap();

        CriteriaPersistence::delete(dir.path(), "to_delete").unwrap();
        let result = CriteriaPersistence::load(dir.path(), "to_delete");
        assert!(matches!(
            result,
            Err(CriteriaError::CriteriaNotFound { .. })
        ));
    }

    #[test]
    fn delete_nonexistent_returns_not_found() {
        let dir = TempDir::new().unwrap();
        let result = CriteriaPersistence::delete(dir.path(), "missing");
        assert!(matches!(
            result,
            Err(CriteriaError::CriteriaNotFound { .. })
        ));
    }

    #[test]
    fn duplicate_creates_copy_with_new_name() {
        let dir = TempDir::new().unwrap();
        CriteriaPersistence::save(dir.path(), &make_test_criteria("original")).unwrap();

        CriteriaPersistence::duplicate(dir.path(), "original", "copy").unwrap();

        let loaded = CriteriaPersistence::load(dir.path(), "copy").unwrap();
        assert_eq!(loaded.name, Some("copy".to_string()));
        assert_eq!(
            loaded.structure_association,
            Some("TEST_STRUCT".to_string())
        );
    }
}
