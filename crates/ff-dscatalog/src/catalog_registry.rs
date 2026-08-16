//! Multi-catalog registry with priority-based resolution.
//!
//! Manages multiple mounted catalogs and resolves DSNs by priority order.

use std::path::{Path, PathBuf};

use crate::catalog::Catalog;
use crate::dataset::DatasetRecord;
use crate::dsn::Dsn;
use crate::error::CatalogError;
use crate::repository::Repository;

/// Result of resolving a DSN across mounted catalogs.
#[derive(Debug, Clone)]
pub struct ResolveResult {
    /// The resolved dataset entry.
    pub entry: DatasetRecord,
    /// The catalog that provided the resolution.
    pub catalog_name: String,
    /// Absolute physical path to the dataset content.
    pub physical_path: PathBuf,
}

/// Manages multiple mounted catalogs with priority ordering.
///
/// Higher priority catalogs are searched first during resolution.
pub struct CatalogRegistry {
    /// Mounted catalogs sorted by priority (highest first).
    catalogs: Vec<Catalog>,
}

impl CatalogRegistry {
    /// Create a new empty registry.
    pub fn new() -> Self {
        Self {
            catalogs: Vec::new(),
        }
    }

    /// Mount a catalog from a repository path with given priority.
    pub fn mount(&mut self, path: &Path, priority: u32) -> Result<(), CatalogError> {
        // Check if already mounted
        let path_str = path.display().to_string();
        if self.catalogs.iter().any(|c| c.repository_path() == path) {
            return Err(CatalogError::CatalogAlreadyMounted {
                name: path_str,
                operation: "mount".to_string(),
            });
        }

        let catalog = Catalog::mount(path, priority)?;
        self.catalogs.push(catalog);
        // Sort by priority descending (highest first)
        self.catalogs
            .sort_by_key(|c| std::cmp::Reverse(c.priority()));
        Ok(())
    }

    /// Unmount a catalog by name or path.
    pub fn unmount(&mut self, name_or_path: &str) -> Result<(), CatalogError> {
        let idx = self.catalogs.iter().position(|c| {
            c.name() == name_or_path || c.repository_path().display().to_string() == name_or_path
        });

        match idx {
            Some(i) => {
                self.catalogs.remove(i);
                Ok(())
            }
            None => Err(CatalogError::CatalogNotMounted {
                name: name_or_path.to_string(),
                operation: "unmount".to_string(),
            }),
        }
    }

    /// Resolve a DSN across all mounted catalogs (priority order).
    ///
    /// Returns the first match with catalog identity and physical path.
    pub fn resolve(&self, dsn: &Dsn) -> Result<ResolveResult, CatalogError> {
        for catalog in &self.catalogs {
            if let Ok(entry) = catalog.lookup(dsn) {
                let physical_path = catalog.repository_path().join(&entry.storage_path);
                return Ok(ResolveResult {
                    entry,
                    catalog_name: catalog.name().to_string(),
                    physical_path,
                });
            }
        }

        Err(CatalogError::DatasetNotFound {
            dsn: dsn.as_str().to_string(),
            operation: "resolve".to_string(),
        })
    }

    /// Check if a DSN exists in any mounted catalog.
    pub fn exists(&self, dsn: &Dsn) -> bool {
        self.catalogs.iter().any(|c| c.exists(dsn).unwrap_or(false))
    }

    /// List all mounted catalogs (names, paths, priorities).
    pub fn list_mounted(&self) -> Vec<(&str, &Path, u32)> {
        self.catalogs
            .iter()
            .map(|c| (c.name(), c.repository_path(), c.priority()))
            .collect()
    }

    /// Get a reference to a specific catalog by name.
    pub fn get_catalog(&self, name: &str) -> Option<&Catalog> {
        self.catalogs.iter().find(|c| c.name() == name)
    }

    /// Get a mutable reference to a specific catalog by name.
    pub fn get_catalog_mut(&mut self, name: &str) -> Option<&mut Catalog> {
        self.catalogs.iter_mut().find(|c| c.name() == name)
    }

    /// Get all catalogs as a slice.
    pub fn catalogs(&self) -> &[Catalog] {
        &self.catalogs
    }

    /// Create a new catalog and optionally mount it.
    pub fn create(
        &mut self,
        name: &str,
        path: &Path,
        auto_mount: bool,
        priority: u32,
    ) -> Result<(), CatalogError> {
        let repo = Repository::new(path);
        repo.initialize(name)?;

        if auto_mount {
            self.mount(path, priority)?;
        }
        Ok(())
    }

    /// Remove a catalog (unmount and optionally delete files).
    pub fn remove(&mut self, name: &str, delete_files: bool) -> Result<(), CatalogError> {
        // Find and unmount
        let path = self
            .catalogs
            .iter()
            .find(|c| c.name() == name)
            .map(|c| c.repository_path().to_path_buf());

        if let Some(p) = &path {
            let _ = self.unmount(name);
            if delete_files {
                std::fs::remove_dir_all(p).map_err(|e| CatalogError::IoError {
                    operation: "remove".to_string(),
                    source: e,
                })?;
            }
            Ok(())
        } else {
            Err(CatalogError::CatalogNotMounted {
                name: name.to_string(),
                operation: "remove".to_string(),
            })
        }
    }
}

impl Default for CatalogRegistry {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::dataset::{AllocParams, Dsorg};
    use tempfile::TempDir;

    fn create_repo(tmp: &TempDir, name: &str) -> PathBuf {
        let path = tmp.path().join(name);
        let repo = Repository::new(&path);
        repo.initialize(name).unwrap();
        path
    }

    #[test]
    fn mount_and_resolve() {
        // Validates: Requirement 5 AC 1, AC 2
        let tmp = TempDir::new().unwrap();
        let path = create_repo(&tmp, "CAT1");
        let mut registry = CatalogRegistry::new();
        registry.mount(&path, 1).unwrap();

        // Allocate a dataset
        let catalog = registry.get_catalog("CAT1").unwrap();
        let params = AllocParams {
            dsn: Dsn::parse("TEST.DATA").unwrap(),
            dsorg: Dsorg::PS,
            recfm: None,
            lrecl: None,
            blksize: None,
            dir_blocks: None,
            gdg_limit: None,
            gdg_scratch: None,
            subtype: None,
            description: None,
        };
        catalog.allocate(params).unwrap();

        let result = registry.resolve(&Dsn::parse("TEST.DATA").unwrap()).unwrap();
        assert_eq!(result.catalog_name, "CAT1");
    }

    #[test]
    fn priority_resolution_highest_wins() {
        // Validates: Requirement 5 AC 3
        let tmp = TempDir::new().unwrap();
        let path1 = create_repo(&tmp, "LOW");
        let path2 = create_repo(&tmp, "HIGH");

        let mut registry = CatalogRegistry::new();
        registry.mount(&path1, 1).unwrap();
        registry.mount(&path2, 10).unwrap();

        // Allocate same DSN in both
        let dsn = Dsn::parse("SHARED.DSN").unwrap();
        registry
            .get_catalog("LOW")
            .unwrap()
            .allocate(AllocParams {
                dsn: dsn.clone(),
                dsorg: Dsorg::PS,
                recfm: None,
                lrecl: None,
                blksize: None,
                dir_blocks: None,
                gdg_limit: None,
                gdg_scratch: None,
                subtype: None,
                description: None,
            })
            .unwrap();
        registry
            .get_catalog("HIGH")
            .unwrap()
            .allocate(AllocParams {
                dsn: dsn.clone(),
                dsorg: Dsorg::PS,
                recfm: None,
                lrecl: None,
                blksize: None,
                dir_blocks: None,
                gdg_limit: None,
                gdg_scratch: None,
                subtype: None,
                description: None,
            })
            .unwrap();

        let result = registry.resolve(&dsn).unwrap();
        assert_eq!(result.catalog_name, "HIGH");
    }

    #[test]
    fn unmount_removes_from_resolution() {
        // Validates: Requirement 5 AC 4
        let tmp = TempDir::new().unwrap();
        let path = create_repo(&tmp, "REMOVE");
        let mut registry = CatalogRegistry::new();
        registry.mount(&path, 1).unwrap();

        let dsn = Dsn::parse("TEST.DS").unwrap();
        registry
            .get_catalog("REMOVE")
            .unwrap()
            .allocate(AllocParams {
                dsn: dsn.clone(),
                dsorg: Dsorg::PS,
                recfm: None,
                lrecl: None,
                blksize: None,
                dir_blocks: None,
                gdg_limit: None,
                gdg_scratch: None,
                subtype: None,
                description: None,
            })
            .unwrap();

        assert!(registry.resolve(&dsn).is_ok());
        registry.unmount("REMOVE").unwrap();
        assert!(registry.resolve(&dsn).is_err());
    }

    #[test]
    fn mount_already_mounted_fails() {
        // Validates: Requirement 5 AC 1
        let tmp = TempDir::new().unwrap();
        let path = create_repo(&tmp, "DUP");
        let mut registry = CatalogRegistry::new();
        registry.mount(&path, 1).unwrap();
        let result = registry.mount(&path, 2);
        assert!(matches!(
            result,
            Err(CatalogError::CatalogAlreadyMounted { .. })
        ));
    }

    #[test]
    fn unmount_nonexistent_fails() {
        let mut registry = CatalogRegistry::new();
        let result = registry.unmount("NOEXIST");
        assert!(matches!(
            result,
            Err(CatalogError::CatalogNotMounted { .. })
        ));
    }
}
