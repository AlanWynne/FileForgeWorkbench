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
//! CR-CH-059 RC.A.3: this backend now implements the SINGLE physical seam
//! `ff_vfs::StorageProvider` (opaque `StorageLocator`, `HashSet<StorageCapability>`,
//! `VfsError`). The former workspace_root is carried behind the struct; the
//! opaque locator encodes the relative UUID path. The duplicate
//! `ff-dscatalog::storage::StorageProvider` trait is retired.
//!
//! Validates: virtual-file-system Requirement 13.1, 13.2, 13.3; dataset-catalog
//!            Requirement 20.1-20.7, 28.1, 28.2 (behaviour preserved).

use std::collections::HashSet;
use std::path::{Path, PathBuf};

use ff_vfs::{StorageCapability, StorageLocator, StorageProvider, StorageStat, VfsError};
use uuid::Uuid;

use crate::error::CatalogError;

const OBJECTS_DIR: &str = "datasets/objects";
const STAGING_DIR: &str = "datasets/staging";

/// Storage provider for PS, PDS/PDSE, GDG, and POSIX content.
///
/// Physical objects are identified by stable UUIDs assigned at allocation time.
/// Logical dataset names are never used as physical paths. The workspace root is
/// captured at construction so the `ff_vfs::StorageProvider` methods need not
/// thread it (Requirement 13.3).
#[derive(Debug, Clone, Default)]
pub struct NativeFileProvider {
    /// Workspace root under which physical objects live.
    root: PathBuf,
}

impl NativeFileProvider {
    /// Construct a provider rooted at `root` (used by the `ff_vfs::StorageProvider`
    /// methods, which do not thread a workspace root).
    pub fn with_root(root: impl Into<PathBuf>) -> Self {
        Self { root: root.into() }
    }

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

    // === Inherent record/object operations (workspace_root threaded) =========
    //
    // These preserve the pre-CR-CH-059 behaviour the catalog logic relies on.
    // They are NOT the physical seam; the seam is `impl StorageProvider` below.

    /// Allocate a new physical object, returning its stable UUID and locator.
    ///
    /// Validates: Requirement 20.1, 20.2, 20.3, 20.4, 20.5
    pub fn allocate_object(
        &self,
        workspace_root: &Path,
        is_container: bool,
    ) -> Result<(Uuid, String), CatalogError> {
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

    /// Open a physical object, returning its resolved path.
    pub fn open_path(&self, workspace_root: &Path, locator: &str) -> Result<PathBuf, CatalogError> {
        let path = Self::resolve_path(workspace_root, locator)?;
        if !path.exists() {
            return Err(CatalogError::DatasetNotFound {
                dsn: locator.to_string(),
                operation: "open".to_string(),
            });
        }
        Ok(path)
    }

    /// Delete a physical object.
    pub fn delete_object(&self, workspace_root: &Path, locator: &str) -> Result<(), CatalogError> {
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

    /// List child locators for a container object (e.g. PDS members).
    pub fn list_children(
        &self,
        workspace_root: &Path,
        locator: &str,
    ) -> Result<Vec<String>, CatalogError> {
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

    /// Compare catalogue entries with physical objects and report discrepancies.
    ///
    /// Validates: Requirement 27.1, 27.2, 27.3
    pub fn reconcile_locators(
        &self,
        workspace_root: &Path,
        known_locators: &[String],
    ) -> Result<Vec<String>, CatalogError> {
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

// === ff_vfs::StorageProvider -- the single physical seam =====================

impl StorageProvider for NativeFileProvider {
    fn capabilities(&self) -> HashSet<StorageCapability> {
        // Validates: Requirement 13.2 (capability mapping is lossless 1:1).
        [
            StorageCapability::StreamRead,
            StorageCapability::StreamWrite,
            StorageCapability::MemberOperations,
            StorageCapability::AtomicRename,
        ]
        .into_iter()
        .collect()
    }

    fn allocate(&self, name: &str) -> Result<StorageLocator, VfsError> {
        // `name` ending in '/' requests a container (PDS/PDSE library).
        let is_container = name.ends_with('/');
        let (_id, locator) = self
            .allocate_object(&self.root, is_container)
            .map_err(VfsError::from)?;
        Ok(StorageLocator::new(locator))
    }

    fn open(&self, locator: &StorageLocator) -> Result<Vec<u8>, VfsError> {
        let path = self
            .open_path(&self.root, locator.as_str())
            .map_err(VfsError::from)?;
        if path.is_dir() {
            return Ok(Vec::new());
        }
        std::fs::read(&path).map_err(|e| VfsError::Io {
            uri: locator.as_str().to_string(),
            operation: "open".to_string(),
            source: e,
        })
    }

    fn stat(&self, locator: &StorageLocator) -> Result<StorageStat, VfsError> {
        let path = NativeFileProvider::resolve_path(&self.root, locator.as_str())
            .map_err(VfsError::from)?;
        let meta = std::fs::metadata(&path).map_err(|e| VfsError::Io {
            uri: locator.as_str().to_string(),
            operation: "stat".to_string(),
            source: e,
        })?;
        Ok(StorageStat {
            name: locator.as_str().to_string(),
            size_bytes: Some(if meta.is_file() { meta.len() } else { 0 }),
            is_container: meta.is_dir(),
            attributes: Vec::new(),
        })
    }

    fn rename(&self, _locator: &StorageLocator, _new_name: &str) -> Result<(), VfsError> {
        // UUID-based layout: rename is catalogue-only, no filesystem move.
        // Validates: Requirement 20.6
        Ok(())
    }

    fn delete(&self, locator: &StorageLocator) -> Result<(), VfsError> {
        self.delete_object(&self.root, locator.as_str())
            .map_err(VfsError::from)
    }

    fn list(&self) -> Result<Vec<(StorageLocator, String)>, VfsError> {
        let objects_dir = self.root.join(OBJECTS_DIR);
        if !objects_dir.is_dir() {
            return Ok(Vec::new());
        }
        let mut entries = Vec::new();
        for entry in std::fs::read_dir(&objects_dir).map_err(|e| VfsError::Io {
            uri: OBJECTS_DIR.to_string(),
            operation: "list".to_string(),
            source: e,
        })? {
            let entry = entry.map_err(|e| VfsError::Io {
                uri: OBJECTS_DIR.to_string(),
                operation: "list".to_string(),
                source: e,
            })?;
            if let Some(name) = entry.file_name().to_str() {
                let locator = format!("{OBJECTS_DIR}/{name}");
                entries.push((StorageLocator::new(locator), name.to_string()));
            }
        }
        entries.sort_by(|a, b| a.1.cmp(&b.1));
        Ok(entries)
    }

    fn write(&self, locator: &StorageLocator, data: &[u8]) -> Result<(), VfsError> {
        let path = NativeFileProvider::resolve_path(&self.root, locator.as_str())
            .map_err(VfsError::from)?;
        if let Some(parent) = path.parent() {
            std::fs::create_dir_all(parent).map_err(|e| VfsError::Io {
                uri: locator.as_str().to_string(),
                operation: "write".to_string(),
                source: e,
            })?;
        }
        std::fs::write(&path, data).map_err(|e| VfsError::Io {
            uri: locator.as_str().to_string(),
            operation: "write".to_string(),
            source: e,
        })
    }

    fn reconcile(&self, catalogue_names: &[String]) -> Result<Vec<String>, VfsError> {
        self.reconcile_locators(&self.root, catalogue_names)
            .map_err(VfsError::from)
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
#[path = "native_tests.rs"]
mod tests;
