//! PDS member operations.
//!
//! Manages members within partitioned datasets: list, create, delete, rename.

use std::fs;
use std::path::PathBuf;

use crate::catalog::Catalog;
use crate::dataset::Dsorg;
use crate::dsn::{Dsn, MemberName};
use crate::error::CatalogError;

/// Metadata for a PDS member.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct PdsMemberInfo {
    /// Member name (uppercase, 1–8 chars).
    pub name: MemberName,
    /// File size in bytes.
    pub size: u64,
    /// Last modification time (ISO 8601).
    pub modified: Option<String>,
}

impl Catalog {
    /// List all members of a PDS, sorted alphabetically.
    pub fn list_members(&self, dsn: &Dsn) -> Result<Vec<PdsMemberInfo>, CatalogError> {
        let record = self.lookup(dsn)?;
        if record.dsorg != Dsorg::PO {
            return Err(CatalogError::TypeMismatch {
                dsn: dsn.as_str().to_string(),
                actual_type: record.dsorg.to_string(),
                expected_type: "PO".to_string(),
                operation: "list_members".to_string(),
            });
        }

        let pds_dir = self.repository().root().join(&record.storage_path);
        if !pds_dir.exists() {
            return Ok(Vec::new());
        }

        let mut members = Vec::new();
        let entries = fs::read_dir(&pds_dir).map_err(|e| CatalogError::IoError {
            operation: "list_members".to_string(),
            source: e,
        })?;

        for entry in entries.flatten() {
            if let Ok(metadata) = entry.metadata() {
                if metadata.is_file() {
                    let name_str = entry.file_name().to_string_lossy().to_string();
                    if let Ok(name) = MemberName::parse(&name_str) {
                        let modified = metadata.modified().ok().map(|t| {
                            let dt: chrono::DateTime<chrono::Utc> = t.into();
                            dt.to_rfc3339()
                        });
                        members.push(PdsMemberInfo {
                            name,
                            size: metadata.len(),
                            modified,
                        });
                    }
                }
            }
        }

        members.sort_by(|a, b| a.name.as_str().cmp(b.name.as_str()));
        Ok(members)
    }

    /// Get the physical path to a PDS member.
    pub fn member_path(&self, dsn: &Dsn, member: &MemberName) -> Result<PathBuf, CatalogError> {
        let record = self.lookup(dsn)?;
        if record.dsorg != Dsorg::PO {
            return Err(CatalogError::TypeMismatch {
                dsn: dsn.as_str().to_string(),
                actual_type: record.dsorg.to_string(),
                expected_type: "PO".to_string(),
                operation: "member_path".to_string(),
            });
        }
        Ok(self
            .repository()
            .root()
            .join(&record.storage_path)
            .join(member.as_str()))
    }

    /// Create a new member in a PDS.
    pub fn create_member(
        &self,
        dsn: &Dsn,
        member: &MemberName,
        overwrite: bool,
    ) -> Result<(), CatalogError> {
        let record = self.lookup(dsn)?;
        if record.dsorg != Dsorg::PO {
            return Err(CatalogError::TypeMismatch {
                dsn: dsn.as_str().to_string(),
                actual_type: record.dsorg.to_string(),
                expected_type: "PO".to_string(),
                operation: "create_member".to_string(),
            });
        }

        let member_path = self
            .repository()
            .root()
            .join(&record.storage_path)
            .join(member.as_str());

        if member_path.exists() && !overwrite {
            return Err(CatalogError::MemberAlreadyExists {
                dsn: dsn.as_str().to_string(),
                member: member.as_str().to_string(),
                operation: "create_member".to_string(),
            });
        }

        fs::File::create(&member_path).map_err(|e| CatalogError::IoError {
            operation: "create_member".to_string(),
            source: e,
        })?;

        // Update PDS modified date
        let now = chrono::Utc::now().to_rfc3339();
        self.connection()
            .execute(
                "UPDATE datasets SET modified = ?1 WHERE dsn = ?2",
                rusqlite::params![&now, dsn.as_str()],
            )
            .map_err(|e| CatalogError::SqliteError {
                operation: "create_member".to_string(),
                source: e,
            })?;

        Ok(())
    }

    /// Delete a member from a PDS.
    pub fn delete_member(&self, dsn: &Dsn, member: &MemberName) -> Result<(), CatalogError> {
        let member_path = self.member_path(dsn, member)?;

        if !member_path.exists() {
            return Err(CatalogError::MemberNotFound {
                dsn: dsn.as_str().to_string(),
                member: member.as_str().to_string(),
                operation: "delete_member".to_string(),
            });
        }

        fs::remove_file(&member_path).map_err(|e| CatalogError::IoError {
            operation: "delete_member".to_string(),
            source: e,
        })?;

        let now = chrono::Utc::now().to_rfc3339();
        self.connection()
            .execute(
                "UPDATE datasets SET modified = ?1 WHERE dsn = ?2",
                rusqlite::params![&now, dsn.as_str()],
            )
            .map_err(|e| CatalogError::SqliteError {
                operation: "delete_member".to_string(),
                source: e,
            })?;

        Ok(())
    }

    /// Rename a member within a PDS.
    pub fn rename_member(
        &self,
        dsn: &Dsn,
        old_name: &MemberName,
        new_name: &MemberName,
    ) -> Result<(), CatalogError> {
        let record = self.lookup(dsn)?;
        if record.dsorg != Dsorg::PO {
            return Err(CatalogError::TypeMismatch {
                dsn: dsn.as_str().to_string(),
                actual_type: record.dsorg.to_string(),
                expected_type: "PO".to_string(),
                operation: "rename_member".to_string(),
            });
        }

        let pds_dir = self.repository().root().join(&record.storage_path);
        let old_path = pds_dir.join(old_name.as_str());
        let new_path = pds_dir.join(new_name.as_str());

        if !old_path.exists() {
            return Err(CatalogError::MemberNotFound {
                dsn: dsn.as_str().to_string(),
                member: old_name.as_str().to_string(),
                operation: "rename_member".to_string(),
            });
        }

        if new_path.exists() {
            return Err(CatalogError::MemberAlreadyExists {
                dsn: dsn.as_str().to_string(),
                member: new_name.as_str().to_string(),
                operation: "rename_member".to_string(),
            });
        }

        fs::rename(&old_path, &new_path).map_err(|e| CatalogError::IoError {
            operation: "rename_member".to_string(),
            source: e,
        })?;

        let now = chrono::Utc::now().to_rfc3339();
        self.connection()
            .execute(
                "UPDATE datasets SET modified = ?1 WHERE dsn = ?2",
                rusqlite::params![&now, dsn.as_str()],
            )
            .map_err(|e| CatalogError::SqliteError {
                operation: "rename_member".to_string(),
                source: e,
            })?;

        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::dataset::{AllocParams, Recfm};
    use crate::repository::Repository;
    use tempfile::TempDir;

    fn setup() -> (TempDir, Catalog) {
        let tmp = TempDir::new().unwrap();
        let path = tmp.path().join("pds-test");
        let repo = Repository::new(&path);
        repo.initialize("PDSTEST").unwrap();
        let catalog = Catalog::mount(&path, 1).unwrap();
        (tmp, catalog)
    }

    fn allocate_pds(catalog: &Catalog, dsn_str: &str) {
        let params = AllocParams {
            dsn: Dsn::parse(dsn_str).unwrap(),
            dsorg: Dsorg::PO,
            recfm: Some(Recfm::FB),
            lrecl: Some(80),
            blksize: Some(27920),
            dir_blocks: Some(10),
            gdg_limit: None,
            gdg_scratch: None,
            subtype: None,
            description: None,
        };
        catalog.allocate(params).unwrap();
    }

    #[test]
    fn create_and_list_members() {
        // Validates: Requirement 8 AC 1, AC 3
        let (_tmp, catalog) = setup();
        let dsn = Dsn::parse("SYS1.MACLIB").unwrap();
        allocate_pds(&catalog, "SYS1.MACLIB");

        let m1 = MemberName::parse("OPEN").unwrap();
        let m2 = MemberName::parse("CLOSE").unwrap();
        catalog.create_member(&dsn, &m1, false).unwrap();
        catalog.create_member(&dsn, &m2, false).unwrap();

        let members = catalog.list_members(&dsn).unwrap();
        assert_eq!(members.len(), 2);
        assert_eq!(members[0].name.as_str(), "CLOSE"); // sorted
        assert_eq!(members[1].name.as_str(), "OPEN");
    }

    #[test]
    fn create_member_on_non_pds_fails() {
        // Validates: Requirement 8 AC 4
        let (_tmp, catalog) = setup();
        let dsn = Dsn::parse("SEQ.FILE").unwrap();
        catalog
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

        let member = MemberName::parse("TEST").unwrap();
        let result = catalog.create_member(&dsn, &member, false);
        assert!(matches!(result, Err(CatalogError::TypeMismatch { .. })));
    }

    #[test]
    fn create_duplicate_member_without_overwrite_fails() {
        // Validates: Requirement 8 AC 8
        let (_tmp, catalog) = setup();
        let dsn = Dsn::parse("MY.PDS").unwrap();
        allocate_pds(&catalog, "MY.PDS");
        let member = MemberName::parse("DUP").unwrap();
        catalog.create_member(&dsn, &member, false).unwrap();
        let result = catalog.create_member(&dsn, &member, false);
        assert!(matches!(
            result,
            Err(CatalogError::MemberAlreadyExists { .. })
        ));
    }

    #[test]
    fn create_duplicate_member_with_overwrite_succeeds() {
        // Validates: Requirement 8 AC 8
        let (_tmp, catalog) = setup();
        let dsn = Dsn::parse("MY.PDS2").unwrap();
        allocate_pds(&catalog, "MY.PDS2");
        let member = MemberName::parse("OVER").unwrap();
        catalog.create_member(&dsn, &member, false).unwrap();
        catalog.create_member(&dsn, &member, true).unwrap(); // should succeed
    }

    #[test]
    fn delete_member_removes_file() {
        // Validates: Requirement 8 AC 5
        let (_tmp, catalog) = setup();
        let dsn = Dsn::parse("DEL.PDS").unwrap();
        allocate_pds(&catalog, "DEL.PDS");
        let member = MemberName::parse("GONE").unwrap();
        catalog.create_member(&dsn, &member, false).unwrap();
        catalog.delete_member(&dsn, &member).unwrap();

        let members = catalog.list_members(&dsn).unwrap();
        assert!(members.is_empty());
    }

    #[test]
    fn delete_nonexistent_member_fails() {
        // Validates: Requirement 8 AC 7
        let (_tmp, catalog) = setup();
        let dsn = Dsn::parse("DEL2.PDS").unwrap();
        allocate_pds(&catalog, "DEL2.PDS");
        let member = MemberName::parse("NOEXIST").unwrap();
        let result = catalog.delete_member(&dsn, &member);
        assert!(matches!(result, Err(CatalogError::MemberNotFound { .. })));
    }

    #[test]
    fn rename_member_works() {
        // Validates: Requirement 8 AC 6
        let (_tmp, catalog) = setup();
        let dsn = Dsn::parse("REN.PDS").unwrap();
        allocate_pds(&catalog, "REN.PDS");
        let old = MemberName::parse("OLD").unwrap();
        let new = MemberName::parse("NEW").unwrap();
        catalog.create_member(&dsn, &old, false).unwrap();
        catalog.rename_member(&dsn, &old, &new).unwrap();

        let members = catalog.list_members(&dsn).unwrap();
        assert_eq!(members.len(), 1);
        assert_eq!(members[0].name.as_str(), "NEW");
    }
}
