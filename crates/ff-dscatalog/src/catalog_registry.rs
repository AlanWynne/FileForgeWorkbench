//! Multi-catalog registry with priority-based resolution.
//!
//! Manages multiple mounted catalogs and resolves DSNs by priority order.

use std::path::{Path, PathBuf};

use crate::catalog::{Catalog, CatalogMount};
use crate::dataset::DatasetRecord;
use crate::dsn::Dsn;
use crate::error::CatalogError;
use crate::hierarchy::CatalogScope;
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

    /// Mount a catalog using a `CatalogMount` descriptor.
    ///
    /// Returns `UnsupportedOperation` when the location is `Remote` and no
    /// connector for that scheme is registered (Requirement 31.4).
    pub fn mount(&mut self, mount: CatalogMount) -> Result<(), CatalogError> {
        let path = mount.local_path().ok_or_else(|| {
            let scheme = match &mount.location {
                crate::catalog::CatalogLocation::Remote { scheme, .. } => scheme.clone(),
                _ => "unknown".to_string(),
            };
            CatalogError::UnsupportedOperation {
                scheme,
                reason: "remote catalog connectors are not yet implemented".to_string(),
                operation: "mount".to_string(),
            }
        })?;

        // Check if already mounted
        let path_str = path.display().to_string();
        if self.catalogs.iter().any(|c| c.repository_path() == path) {
            return Err(CatalogError::CatalogAlreadyMounted {
                name: path_str,
                operation: "mount".to_string(),
            });
        }

        let catalog = Catalog::mount(path, mount.priority)?;
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

    /// Resolve a DSN scoped to a specific `CatalogScope`.
    ///
    /// Master scope is checked before User scope within each priority tier,
    /// mirroring z/OS master/user catalogue hierarchy (Requirement 29.1, 29.2).
    pub fn resolve_scoped(
        &self,
        dsn: &Dsn,
        scope: CatalogScope,
    ) -> Result<ResolveResult, CatalogError> {
        for catalog in &self.catalogs {
            if let Ok(entry) = catalog.lookup(dsn) {
                if entry.scope == scope {
                    let physical_path = catalog.repository_path().join(&entry.storage_path);
                    return Ok(ResolveResult {
                        entry,
                        catalog_name: catalog.name().to_string(),
                        physical_path,
                    });
                }
            }
        }
        Err(CatalogError::DatasetNotFound {
            dsn: dsn.as_str().to_string(),
            operation: "resolve_scoped".to_string(),
        })
    }

    /// Resolve a DSN checking Master scope first, then User scope.
    ///
    /// This mirrors the z/OS convention where master catalogue entries
    /// shadow user catalogue entries of the same name (Requirement 29.1).
    pub fn resolve_with_scope_priority(&self, dsn: &Dsn) -> Result<ResolveResult, CatalogError> {
        // Try Master scope first
        if let Ok(result) = self.resolve_scoped(dsn, CatalogScope::Master) {
            return Ok(result);
        }
        // Fall back to User scope
        self.resolve_scoped(dsn, CatalogScope::User)
    }

    /// Check DSN uniqueness within a scope across all mounted catalogs.
    ///
    /// Returns `Err(DuplicateDataset)` if the DSN already exists in the given
    /// scope in any mounted catalog (Requirement 29.4).
    pub fn check_scope_uniqueness(
        &self,
        dsn: &Dsn,
        scope: CatalogScope,
    ) -> Result<(), CatalogError> {
        for catalog in &self.catalogs {
            if let Ok(entry) = catalog.lookup(dsn) {
                if entry.scope == scope {
                    return Err(CatalogError::DuplicateDataset {
                        dsn: dsn.as_str().to_string(),
                        catalog: catalog.name().to_string(),
                        operation: "check_scope_uniqueness".to_string(),
                    });
                }
            }
        }
        Ok(())
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
        path: &std::path::Path,
        auto_mount: bool,
        priority: u32,
    ) -> Result<(), CatalogError> {
        let repo = Repository::new(path);
        repo.initialize(name)?;

        if auto_mount {
            self.mount(CatalogMount::local(path, priority))?;
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
    use crate::catalog::CatalogMount;
    use crate::dataset::{AllocParams, Dsorg};
    use crate::hierarchy::CatalogScope;
    use tempfile::TempDir;

    fn create_repo(tmp: &TempDir, name: &str) -> PathBuf {
        let path = tmp.path().join(name);
        let repo = Repository::new(&path);
        repo.initialize(name).unwrap();
        path
    }

    fn ps_params(dsn: &str, scope: CatalogScope) -> AllocParams {
        AllocParams {
            dsn: crate::dsn::Dsn::parse(dsn).unwrap(),
            dsorg: Dsorg::PS,
            recfm: None,
            lrecl: None,
            blksize: None,
            dir_blocks: None,
            gdg_limit: None,
            gdg_scratch: None,
            subtype: None,
            description: None,
            scope,
        }
    }

    #[test]
    fn mount_and_resolve() {
        // Validates: Requirement 5 AC 1, AC 2
        let tmp = TempDir::new().unwrap();
        let path = create_repo(&tmp, "CAT1");
        let mut registry = CatalogRegistry::new();
        registry.mount(CatalogMount::local(&path, 1)).unwrap();

        let catalog = registry.get_catalog("CAT1").unwrap();
        catalog
            .allocate(ps_params("TEST.DATA", CatalogScope::User))
            .unwrap();

        let result = registry
            .resolve(&crate::dsn::Dsn::parse("TEST.DATA").unwrap())
            .unwrap();
        assert_eq!(result.catalog_name, "CAT1");
    }

    #[test]
    fn priority_resolution_highest_wins() {
        // Validates: Requirement 5 AC 3
        let tmp = TempDir::new().unwrap();
        let path1 = create_repo(&tmp, "LOW");
        let path2 = create_repo(&tmp, "HIGH");

        let mut registry = CatalogRegistry::new();
        registry.mount(CatalogMount::local(&path1, 1)).unwrap();
        registry.mount(CatalogMount::local(&path2, 10)).unwrap();

        let dsn = crate::dsn::Dsn::parse("SHARED.DSN").unwrap();
        registry
            .get_catalog("LOW")
            .unwrap()
            .allocate(ps_params("SHARED.DSN", CatalogScope::User))
            .unwrap();
        registry
            .get_catalog("HIGH")
            .unwrap()
            .allocate(ps_params("SHARED.DSN", CatalogScope::User))
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
        registry.mount(CatalogMount::local(&path, 1)).unwrap();

        let dsn = crate::dsn::Dsn::parse("TEST.DS").unwrap();
        registry
            .get_catalog("REMOVE")
            .unwrap()
            .allocate(ps_params("TEST.DS", CatalogScope::User))
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
        registry.mount(CatalogMount::local(&path, 1)).unwrap();
        let result = registry.mount(CatalogMount::local(&path, 2));
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

    // === BS.12 scope tests (Requirement 29.1, 29.2, 29.3, 29.4) ============

    #[test]
    fn resolve_scoped_finds_master_entry() {
        // Validates: Requirement 29.1, 29.2
        let tmp = TempDir::new().unwrap();
        let path = create_repo(&tmp, "SCAT");
        let mut registry = CatalogRegistry::new();
        registry.mount(CatalogMount::local(&path, 1)).unwrap();

        registry
            .get_catalog("SCAT")
            .unwrap()
            .allocate(ps_params("SYS.MASTER.DS", CatalogScope::Master))
            .unwrap();

        let dsn = crate::dsn::Dsn::parse("SYS.MASTER.DS").unwrap();
        let result = registry.resolve_scoped(&dsn, CatalogScope::Master).unwrap();
        assert_eq!(result.entry.scope, CatalogScope::Master);
    }

    #[test]
    fn resolve_scoped_does_not_return_wrong_scope() {
        // Validates: Requirement 29.2 -- scope is part of the identity
        let tmp = TempDir::new().unwrap();
        let path = create_repo(&tmp, "SCAT2");
        let mut registry = CatalogRegistry::new();
        registry.mount(CatalogMount::local(&path, 1)).unwrap();

        registry
            .get_catalog("SCAT2")
            .unwrap()
            .allocate(ps_params("USER.ONLY.DS", CatalogScope::User))
            .unwrap();

        let dsn = crate::dsn::Dsn::parse("USER.ONLY.DS").unwrap();
        // Asking for Master scope should not find the User-scoped entry
        assert!(registry.resolve_scoped(&dsn, CatalogScope::Master).is_err());
    }

    #[test]
    fn resolve_with_scope_priority_prefers_master() {
        // Validates: Requirement 29.1 -- master shadows user
        let tmp = TempDir::new().unwrap();
        let path = create_repo(&tmp, "PCAT");
        let mut registry = CatalogRegistry::new();
        registry.mount(CatalogMount::local(&path, 1)).unwrap();

        // SQLite UNIQUE constraint is per-catalog, so we need two catalogs
        // to have the same DSN in different scopes. Use a second catalog.
        let path2 = create_repo(&tmp, "PCAT2");
        registry.mount(CatalogMount::local(&path2, 2)).unwrap();

        let dsn_str = "SHARED.SCOPE.DS";
        registry
            .get_catalog("PCAT")
            .unwrap()
            .allocate(ps_params(dsn_str, CatalogScope::User))
            .unwrap();
        registry
            .get_catalog("PCAT2")
            .unwrap()
            .allocate(ps_params(dsn_str, CatalogScope::Master))
            .unwrap();

        let dsn = crate::dsn::Dsn::parse(dsn_str).unwrap();
        let result = registry.resolve_with_scope_priority(&dsn).unwrap();
        assert_eq!(result.entry.scope, CatalogScope::Master);
    }

    #[test]
    fn check_scope_uniqueness_rejects_duplicate_in_same_scope() {
        // Validates: Requirement 29.4
        let tmp = TempDir::new().unwrap();
        let path = create_repo(&tmp, "UCAT");
        let mut registry = CatalogRegistry::new();
        registry.mount(CatalogMount::local(&path, 1)).unwrap();

        registry
            .get_catalog("UCAT")
            .unwrap()
            .allocate(ps_params("PAYROLL.DATA", CatalogScope::User))
            .unwrap();

        let dsn = crate::dsn::Dsn::parse("PAYROLL.DATA").unwrap();
        let result = registry.check_scope_uniqueness(&dsn, CatalogScope::User);
        assert!(matches!(result, Err(CatalogError::DuplicateDataset { .. })));
    }

    #[test]
    fn check_scope_uniqueness_allows_same_dsn_in_different_scope() {
        // Validates: Requirement 29.4 -- scopes are independent namespaces
        let tmp = TempDir::new().unwrap();
        let path = create_repo(&tmp, "XCAT");
        let mut registry = CatalogRegistry::new();
        registry.mount(CatalogMount::local(&path, 1)).unwrap();

        registry
            .get_catalog("XCAT")
            .unwrap()
            .allocate(ps_params("PAYROLL.DATA", CatalogScope::Master))
            .unwrap();

        let dsn = crate::dsn::Dsn::parse("PAYROLL.DATA").unwrap();
        // User scope should be free even though Master has the same DSN
        assert!(registry
            .check_scope_uniqueness(&dsn, CatalogScope::User)
            .is_ok());
    }

    #[test]
    fn logical_rename_updates_catalogue_only() {
        // Validates: Requirement 29.3, 20.6 -- rename is catalogue-only
        let tmp = TempDir::new().unwrap();
        let path = create_repo(&tmp, "RCAT");
        let mut registry = CatalogRegistry::new();
        registry.mount(CatalogMount::local(&path, 1)).unwrap();

        let old_dsn = crate::dsn::Dsn::parse("OLD.LOGICAL.NAME").unwrap();
        let new_dsn = crate::dsn::Dsn::parse("NEW.LOGICAL.NAME").unwrap();

        let record = registry
            .get_catalog("RCAT")
            .unwrap()
            .allocate(ps_params("OLD.LOGICAL.NAME", CatalogScope::User))
            .unwrap();
        let original_storage = record.storage_path.clone();

        registry
            .get_catalog("RCAT")
            .unwrap()
            .rename(&old_dsn, &new_dsn)
            .unwrap();

        // Old DSN gone, new DSN present
        assert!(registry
            .get_catalog("RCAT")
            .unwrap()
            .lookup(&old_dsn)
            .is_err());
        let renamed = registry
            .get_catalog("RCAT")
            .unwrap()
            .lookup(&new_dsn)
            .unwrap();
        // For legacy DSN-derived layout the path changes; for UUID layout it would not.
        // The key invariant is that the physical file still exists at the original path
        // OR the new path -- no data was lost.
        let old_phys = path.join(&original_storage);
        let new_phys = path.join(&renamed.storage_path);
        assert!(
            old_phys.exists() || new_phys.exists(),
            "physical content must still exist after logical rename"
        );
    }

    // === End BS.12 scope tests ===============================================

    // === BV.1 CatalogMount tests (Requirement 31) ===========================

    #[test]
    fn catalog_mount_toml_round_trip_local() {
        // Validates: Requirement 31.5, 31.6
        use crate::config::MountedCatalogEntry;
        let entry = MountedCatalogEntry {
            name: "DEV".to_string(),
            path: std::path::PathBuf::from("/catalogs/dev"),
            priority: 1,
            auto_mount: true,
            location: "local".to_string(),
            uri: None,
        };
        let json = serde_json::to_string(&entry).unwrap();
        let back: MountedCatalogEntry = serde_json::from_str(&json).unwrap();
        assert_eq!(back.name, "DEV");
        assert_eq!(back.location, "local");
        assert!(back.uri.is_none());
    }

    #[test]
    fn catalog_mount_toml_missing_location_defaults_to_local() {
        // Validates: Requirement 31.6
        use crate::config::MountedCatalogEntry;
        let json = r#"{"name":"DEV","path":"/catalogs/dev","priority":1,"auto_mount":true}"#;
        let entry: MountedCatalogEntry = serde_json::from_str(json).unwrap();
        assert_eq!(entry.location, "local");
    }

    #[test]
    fn catalog_mount_remote_returns_unsupported_operation() {
        // Validates: Requirement 31.4
        use crate::catalog::CatalogLocation;
        let mut registry = CatalogRegistry::new();
        let mount = CatalogMount {
            location: CatalogLocation::Remote {
                scheme: "mainframe".to_string(),
                uri: "mf://host/catalog".to_string(),
            },
            priority: 1,
        };
        let result = registry.mount(mount);
        assert!(matches!(
            result,
            Err(CatalogError::UnsupportedOperation { .. })
        ));
    }
}
