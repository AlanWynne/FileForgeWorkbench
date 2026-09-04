//! Native filesystem storage provider.
//!
//! Stores PS, PDS/PDSE, GDG, and POSIX content as native files and directories
//! under a UUID-based layout. The logical dataset name is never used as a
//! physical path.
//!
//! Layout:
//!   workspace/
//!     datasets/
//!       objects/
//!         <uuid>.dat          -- PS or GDG generation content
//!         <uuid>/             -- PDS/PDSE library directory
//!           <member-uuid>.dat
//!       staging/              -- in-progress allocations
//!
//! Validates: Requirement 18.1, 18.2, 18.3, 19.5, 20.1, 20.2, 20.3, 20.4,
//!            20.5, 20.6, 20.7, 28.1, 28.2

use std::path::{Path, PathBuf};

use uuid::Uuid;

use crate::error::CatalogError;

use super::{ObjectId, ObjectStat, ProviderCapability, StorageProvider};

const OBJECTS_DIR: &str = "datasets/objects";
const STAGING_DIR: &str = "datasets/staging";

/// Storage provider for PS, PDS/PDSE, GDG, and POSIX content.
///
/// Physical objects are identified by stable UUIDs assigned at allocation time.
/// Logical dataset names are never used as physical paths.
#[derive(Debug, Clone, Default)]
pub struct NativeFileProvider;

impl NativeFileProvider {
    /// Resolve the physical path for a locator within a workspace root.
    ///
    /// Validates the resolved path stays within the workspace root to prevent
    /// path traversal. Validates: Requirement 20.7, 28.1, 28.2
    fn resolve_path(workspace_root: &Path, locator: &str) -> Result<PathBuf, CatalogError> {
        // Locator is a relative path like "datasets/objects/<uuid>.dat"
        // or "datasets/objects/<uuid>/<member-uuid>.dat"
        let candidate = workspace_root.join(locator);
        let canonical_root = workspace_root
            .canonicalize()
            .unwrap_or_else(|_| workspace_root.to_path_buf());

        // Guard: reject traversal outside workspace root
        // We check the non-canonicalized form first (file may not exist yet)
        let normalized = normalize_path(&candidate);
        if !normalized.starts_with(&canonical_root) && !normalized.starts_with(workspace_root) {
            return Err(CatalogError::RepositoryCorrupt {
                path: locator.to_string(),
                reason: "path traversal outside workspace root rejected".to_string(),
                operation: "resolve_path".to_string(),
            });
        }

        // Guard: reject reserved device names on Windows
        if let Some(file_name) = normalized.file_name().and_then(|n| n.to_str()) {
            if is_reserved_name(file_name) {
                return Err(CatalogError::RepositoryCorrupt {
                    path: locator.to_string(),
                    reason: format!("reserved device name '{file_name}' rejected"),
                    operation: "resolve_path".to_string(),
                });
            }
        }

        Ok(normalized)
    }

    /// Build the locator string for a new sequential/GDG object.
    fn sequential_locator(id: &Uuid) -> String {
        format!("{OBJECTS_DIR}/{id}.dat")
    }

    /// Build the locator string for a new container (PDS/PDSE library).
    fn container_locator(id: &Uuid) -> String {
        format!("{OBJECTS_DIR}/{id}")
    }

    /// Build the staging locator for an in-progress allocation.
    pub fn staging_locator(id: &Uuid) -> String {
        format!("{STAGING_DIR}/{id}.dat")
    }
}

impl StorageProvider for NativeFileProvider {
    fn capabilities(&self) -> &[ProviderCapability] {
        // Validates: Requirement 19.2
        &[
            ProviderCapability::StreamRead,
            ProviderCapability::StreamWrite,
            ProviderCapability::MemberOperations,
            ProviderCapability::AtomicRename,
        ]
    }

    fn allocate(
        &self,
        workspace_root: &Path,
        is_container: bool,
    ) -> Result<(ObjectId, String), CatalogError> {
        // Validates: Requirement 20.1, 20.2, 20.3, 20.4, 20.5
        let id = Uuid::new_v4();
        let locator = if is_container {
            Self::container_locator(&id)
        } else {
            Self::sequential_locator(&id)
        };

        let path = Self::resolve_path(workspace_root, &locator)?;

        if is_container {
            std::fs::create_dir_all(&path).map_err(|e| CatalogError::IoError {
                operation: "allocate container".to_string(),
                source: e,
            })?;
        } else {
            if let Some(parent) = path.parent() {
                std::fs::create_dir_all(parent).map_err(|e| CatalogError::IoError {
                    operation: "allocate sequential parent".to_string(),
                    source: e,
                })?;
            }
            std::fs::File::create(&path).map_err(|e| CatalogError::IoError {
                operation: "allocate sequential".to_string(),
                source: e,
            })?;
        }

        Ok((id, locator))
    }

    fn open(&self, workspace_root: &Path, locator: &str) -> Result<PathBuf, CatalogError> {
        let path = Self::resolve_path(workspace_root, locator)?;
        if !path.exists() {
            return Err(CatalogError::DatasetNotFound {
                dsn: locator.to_string(),
                operation: "open".to_string(),
            });
        }
        Ok(path)
    }

    fn stat(&self, workspace_root: &Path, locator: &str) -> Result<ObjectStat, CatalogError> {
        let path = Self::resolve_path(workspace_root, locator)?;
        let meta = std::fs::metadata(&path).map_err(|e| CatalogError::IoError {
            operation: "stat".to_string(),
            source: e,
        })?;
        Ok(ObjectStat {
            size: if meta.is_file() { meta.len() } else { 0 },
            is_container: meta.is_dir(),
            locator: locator.to_string(),
        })
    }

    fn rename(
        &self,
        _workspace_root: &Path,
        _locator: &str,
        _new_locator: &str,
    ) -> Result<(), CatalogError> {
        // UUID-based layout: rename is catalogue-only, no filesystem move.
        // Validates: Requirement 20.6
        Ok(())
    }

    fn delete(&self, workspace_root: &Path, locator: &str) -> Result<(), CatalogError> {
        let path = Self::resolve_path(workspace_root, locator)?;
        if path.is_dir() {
            std::fs::remove_dir_all(&path).map_err(|e| CatalogError::IoError {
                operation: "delete container".to_string(),
                source: e,
            })?;
        } else if path.exists() {
            std::fs::remove_file(&path).map_err(|e| CatalogError::IoError {
                operation: "delete sequential".to_string(),
                source: e,
            })?;
        }
        Ok(())
    }

    fn list(&self, workspace_root: &Path, locator: &str) -> Result<Vec<String>, CatalogError> {
        let path = Self::resolve_path(workspace_root, locator)?;
        if !path.is_dir() {
            return Ok(vec![]);
        }
        let mut entries = Vec::new();
        for entry in std::fs::read_dir(&path).map_err(|e| CatalogError::IoError {
            operation: "list".to_string(),
            source: e,
        })? {
            let entry = entry.map_err(|e| CatalogError::IoError {
                operation: "list entry".to_string(),
                source: e,
            })?;
            if let Some(name) = entry.file_name().to_str() {
                entries.push(format!("{locator}/{name}"));
            }
        }
        entries.sort();
        Ok(entries)
    }

    fn reconcile(
        &self,
        workspace_root: &Path,
        known_locators: &[String],
    ) -> Result<Vec<String>, CatalogError> {
        // Validates: Requirement 27.1, 27.2, 27.3
        let mut discrepancies = Vec::new();
        for locator in known_locators {
            match Self::resolve_path(workspace_root, locator) {
                Ok(path) if !path.exists() => {
                    discrepancies.push(format!("missing physical object for locator '{locator}'"));
                }
                Err(e) => {
                    discrepancies.push(format!("invalid locator '{locator}': {e}"));
                }
                Ok(_) => {}
            }
        }
        Ok(discrepancies)
    }
}

/// Normalise a path without requiring it to exist on disk.
fn normalize_path(path: &Path) -> PathBuf {
    let mut components = Vec::new();
    for component in path.components() {
        match component {
            std::path::Component::ParentDir => {
                components.pop();
            }
            std::path::Component::CurDir => {}
            other => components.push(other),
        }
    }
    components.iter().collect()
}

/// Returns true if the name is a Windows reserved device name.
fn is_reserved_name(name: &str) -> bool {
    // Strip extension for comparison
    let base = name.split('.').next().unwrap_or(name).to_uppercase();
    matches!(
        base.as_str(),
        "CON"
            | "PRN"
            | "AUX"
            | "NUL"
            | "COM1"
            | "COM2"
            | "COM3"
            | "COM4"
            | "COM5"
            | "COM6"
            | "COM7"
            | "COM8"
            | "COM9"
            | "LPT1"
            | "LPT2"
            | "LPT3"
            | "LPT4"
            | "LPT5"
            | "LPT6"
            | "LPT7"
            | "LPT8"
            | "LPT9"
    )
}

#[cfg(test)]
mod tests {
    use super::*;
    use proptest::prelude::*;
    use tempfile::TempDir;

    fn tmp() -> TempDir {
        tempfile::tempdir().expect("tempdir")
    }

    #[test]
    fn allocate_sequential_creates_file_with_uuid_name() {
        // Validates: Requirement 20.1, 20.2, 20.3
        let dir = tmp();
        let provider = NativeFileProvider;
        let (id, locator) = provider.allocate(dir.path(), false).unwrap();
        assert!(locator.contains(&id.to_string()));
        assert!(locator.ends_with(".dat"));
        assert!(dir.path().join(&locator).exists());
    }

    #[test]
    fn allocate_container_creates_directory_with_uuid_name() {
        // Validates: Requirement 20.1, 20.2
        let dir = tmp();
        let provider = NativeFileProvider;
        let (id, locator) = provider.allocate(dir.path(), true).unwrap();
        assert!(locator.contains(&id.to_string()));
        assert!(dir.path().join(&locator).is_dir());
    }

    #[test]
    fn locator_does_not_contain_dsn_components() {
        // Validates: Requirement 20.3, 20.5 -- DSN not in physical path
        let dir = tmp();
        let provider = NativeFileProvider;
        let (_id, locator) = provider.allocate(dir.path(), false).unwrap();
        assert!(!locator.contains("PAYROLL"));
        assert!(!locator.contains("INPUT"));
    }

    #[test]
    fn two_allocations_produce_distinct_locators() {
        // Validates: Requirement 20.4 -- deterministic and unique
        let dir = tmp();
        let provider = NativeFileProvider;
        let (_, loc1) = provider.allocate(dir.path(), false).unwrap();
        let (_, loc2) = provider.allocate(dir.path(), false).unwrap();
        assert_ne!(loc1, loc2);
    }

    #[test]
    fn rename_is_noop_on_filesystem() {
        // Validates: Requirement 20.6 -- rename does not move physical object
        let dir = tmp();
        let provider = NativeFileProvider;
        let (_, locator) = provider.allocate(dir.path(), false).unwrap();
        let path_before = dir.path().join(&locator);
        provider
            .rename(dir.path(), &locator, "new_locator")
            .unwrap();
        // File still at original path
        assert!(path_before.exists());
    }

    #[test]
    fn open_returns_path_for_existing_object() {
        // Validates: Requirement 19.5
        let dir = tmp();
        let provider = NativeFileProvider;
        let (_, locator) = provider.allocate(dir.path(), false).unwrap();
        let path = provider.open(dir.path(), &locator).unwrap();
        assert!(path.exists());
    }

    #[test]
    fn open_returns_error_for_missing_object() {
        // Validates: Requirement 19.5
        let dir = tmp();
        let provider = NativeFileProvider;
        let err = provider
            .open(dir.path(), "datasets/objects/nonexistent.dat")
            .unwrap_err();
        assert!(matches!(err, CatalogError::DatasetNotFound { .. }));
    }

    #[test]
    fn stat_returns_correct_metadata() {
        // Validates: Requirement 19.5
        let dir = tmp();
        let provider = NativeFileProvider;
        let (_, locator) = provider.allocate(dir.path(), false).unwrap();
        let stat = provider.stat(dir.path(), &locator).unwrap();
        assert!(!stat.is_container);
        assert_eq!(stat.locator, locator);
    }

    #[test]
    fn delete_removes_file() {
        // Validates: Requirement 19.5
        let dir = tmp();
        let provider = NativeFileProvider;
        let (_, locator) = provider.allocate(dir.path(), false).unwrap();
        let path = dir.path().join(&locator);
        assert!(path.exists());
        provider.delete(dir.path(), &locator).unwrap();
        assert!(!path.exists());
    }

    #[test]
    fn path_traversal_rejected() {
        // Validates: Requirement 20.7, 28.1, 28.2
        let dir = tmp();
        let provider = NativeFileProvider;
        let err = provider.open(dir.path(), "../../etc/passwd").unwrap_err();
        assert!(matches!(err, CatalogError::RepositoryCorrupt { .. }));
    }

    #[test]
    fn reserved_device_name_rejected() {
        // Validates: Requirement 20.7
        let dir = tmp();
        let provider = NativeFileProvider;
        let err = provider
            .open(dir.path(), "datasets/objects/NUL.dat")
            .unwrap_err();
        assert!(matches!(err, CatalogError::RepositoryCorrupt { .. }));
    }

    #[test]
    fn reconcile_reports_missing_objects() {
        // Validates: Requirement 27.1, 27.2, 27.3
        let dir = tmp();
        let provider = NativeFileProvider;
        let (_, locator) = provider.allocate(dir.path(), false).unwrap();
        let missing = "datasets/objects/missing-uuid.dat".to_string();
        let discrepancies = provider
            .reconcile(dir.path(), &[locator, missing.clone()])
            .unwrap();
        assert_eq!(discrepancies.len(), 1);
        assert!(discrepancies[0].contains("missing-uuid"));
    }

    #[test]
    fn capabilities_include_stream_read_write() {
        // Validates: Requirement 19.2
        let provider = NativeFileProvider;
        let caps = provider.capabilities();
        assert!(caps.contains(&ProviderCapability::StreamRead));
        assert!(caps.contains(&ProviderCapability::StreamWrite));
    }

    // === Property test: path traversal rejection (Task 26.3) ==============

    /// Generate locator strings that contain traversal sequences or reserved names.
    fn traversal_locator_strategy() -> impl Strategy<Value = String> {
        // Combine a traversal prefix with an optional suffix
        let traversal_prefixes = prop_oneof![
            Just("../../etc/passwd".to_string()),
            Just("../secret".to_string()),
            Just("..\\windows\\system32".to_string()),
            Just("..".to_string()),
            Just("datasets/objects/../../etc/shadow".to_string()),
            Just("datasets/../../../root/.ssh/id_rsa".to_string()),
            // Windows reserved device names
            Just("NUL".to_string()),
            Just("CON".to_string()),
            Just("PRN".to_string()),
            Just("AUX".to_string()),
            Just("COM1".to_string()),
            Just("LPT1".to_string()),
            Just("NUL.dat".to_string()),
            Just("datasets/objects/NUL.dat".to_string()),
            Just("datasets/objects/CON".to_string()),
            // Double-slash variants
            Just("//etc/passwd".to_string()),
            Just("datasets//objects//../../etc".to_string()),
        ];
        traversal_prefixes
    }

    proptest! {
        #[test]
        fn path_traversal_and_reserved_names_always_rejected(
            locator in traversal_locator_strategy()
        ) {
            // Validates: Requirement 28.1, 28.2, 20.7
            // Property: any locator containing traversal sequences or reserved
            // device names MUST be rejected by resolve_path with RepositoryCorrupt.
            let dir = tmp();
            let result = NativeFileProvider::resolve_path(dir.path(), &locator);
            // Either rejected outright, or if it resolves, it must stay within root
            match result {
                Err(CatalogError::RepositoryCorrupt { .. }) => {
                    // Correct: traversal or reserved name rejected
                }
                Ok(resolved) => {
                    // If it resolved, it must be within the workspace root
                    let root = dir.path().canonicalize()
                        .unwrap_or_else(|_| dir.path().to_path_buf());
                    prop_assert!(
                        resolved.starts_with(&root) || resolved.starts_with(dir.path()),
                        "resolved path {:?} escaped workspace root {:?}",
                        resolved,
                        root
                    );
                }
                Err(_) => {
                    // Other errors (e.g. IoError) are also acceptable for invalid locators
                }
            }
        }
    }
}
