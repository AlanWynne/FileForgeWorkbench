//! LISTCAT and LISTDS command implementations.
//!
//! Provides wildcard matching for dataset listing and detailed dataset info.

use crate::catalog_registry::CatalogRegistry;
use crate::dataset::{DatasetRecord, Dsorg};
use crate::dsn::Dsn;
use crate::error::CatalogError;

/// Result entry from a LISTCAT operation.
#[derive(Debug, Clone)]
pub struct ListcatEntry {
    /// Dataset name.
    pub dsn: String,
    /// Organization type.
    pub dsorg: Dsorg,
    /// Record format (as string).
    pub recfm: Option<String>,
    /// Logical record length.
    pub lrecl: Option<u32>,
    /// Creation date.
    pub created: Option<String>,
    /// Catalog name containing this dataset.
    pub catalog_name: String,
}

impl CatalogRegistry {
    /// List datasets matching a filter pattern across all mounted catalogs.
    ///
    /// Supports `*` (zero or more chars) and `%` (one qualifier) wildcards.
    pub fn listcat(
        &self,
        filter: &str,
        dsorg_filter: Option<Dsorg>,
        catalog_filter: Option<&str>,
    ) -> Result<Vec<ListcatEntry>, CatalogError> {
        let mut results = Vec::new();

        for catalog in self.catalogs() {
            if let Some(cat_filter) = catalog_filter {
                if catalog.name() != cat_filter {
                    continue;
                }
            }

            let datasets = catalog.list_datasets()?;
            for record in datasets {
                // Apply DSORG filter
                if let Some(dsorg) = dsorg_filter {
                    if record.dsorg != dsorg {
                        continue;
                    }
                }

                // Apply wildcard filter
                if !filter.is_empty() && filter != "*" && !record.dsn.matches_pattern(filter) {
                    continue;
                }

                results.push(ListcatEntry {
                    dsn: record.dsn.as_str().to_string(),
                    dsorg: record.dsorg,
                    recfm: record.recfm.map(|r| r.to_string()),
                    lrecl: record.lrecl,
                    created: record.created,
                    catalog_name: catalog.name().to_string(),
                });
            }
        }

        // Sort by DSN
        results.sort_by(|a, b| a.dsn.cmp(&b.dsn));
        Ok(results)
    }

    /// Get detailed info for a single dataset (LISTDS equivalent).
    pub fn listds(&self, dsn: &Dsn) -> Result<DatasetRecord, CatalogError> {
        let result = self.resolve(dsn)?;
        Ok(result.entry)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::catalog::CatalogMount;
    use crate::dataset::{AllocParams, Recfm};
    use crate::repository::Repository;
    use tempfile::TempDir;

    fn setup_with_datasets() -> (TempDir, CatalogRegistry) {
        let tmp = TempDir::new().unwrap();
        let path = tmp.path().join("listcat-test");
        let repo = Repository::new(&path);
        repo.initialize("LISTTEST").unwrap();

        let mut registry = CatalogRegistry::new();
        registry.mount(CatalogMount::local(&path, 1)).unwrap();

        // Create some datasets
        let catalog = registry.get_catalog("LISTTEST").unwrap();
        for name in &[
            "PAYROLL.INPUT",
            "PAYROLL.OUTPUT",
            "SYS1.MACLIB",
            "USER.DATA",
        ] {
            let dsorg = if *name == "SYS1.MACLIB" {
                Dsorg::PO
            } else {
                Dsorg::PS
            };
            catalog
                .allocate(AllocParams {
                    dsn: Dsn::parse(name).unwrap(),
                    dsorg,
                    recfm: Some(Recfm::FB),
                    lrecl: Some(80),
                    blksize: Some(27920),
                    dir_blocks: None,
                    gdg_limit: None,
                    gdg_scratch: None,
                    subtype: None,
                    description: None,
                    scope: crate::hierarchy::CatalogScope::User,
                })
                .unwrap();
        }

        (tmp, registry)
    }

    #[test]
    fn listcat_all_returns_all_datasets() {
        // Validates: Requirement 13 AC 1
        let (_tmp, registry) = setup_with_datasets();
        let results = registry.listcat("*", None, None).unwrap();
        assert_eq!(results.len(), 4);
    }

    #[test]
    fn listcat_with_prefix_filter() {
        // Validates: Requirement 13 AC 2
        let (_tmp, registry) = setup_with_datasets();
        let results = registry.listcat("PAYROLL.*", None, None).unwrap();
        assert_eq!(results.len(), 2);
    }

    #[test]
    fn listcat_with_dsorg_filter() {
        // Validates: Requirement 13 AC 3
        let (_tmp, registry) = setup_with_datasets();
        let results = registry.listcat("*", Some(Dsorg::PO), None).unwrap();
        assert_eq!(results.len(), 1);
        assert_eq!(results[0].dsn, "SYS1.MACLIB");
    }

    #[test]
    fn listds_returns_record() {
        // Validates: Requirement 13 AC 4
        let (_tmp, registry) = setup_with_datasets();
        let dsn = Dsn::parse("PAYROLL.INPUT").unwrap();
        let record = registry.listds(&dsn).unwrap();
        assert_eq!(record.dsn.as_str(), "PAYROLL.INPUT");
    }

    #[test]
    fn listds_not_found() {
        // Validates: Requirement 13 AC 5
        let (_tmp, registry) = setup_with_datasets();
        let dsn = Dsn::parse("NO.EXIST").unwrap();
        let result = registry.listds(&dsn);
        assert!(matches!(result, Err(CatalogError::DatasetNotFound { .. })));
    }
}
