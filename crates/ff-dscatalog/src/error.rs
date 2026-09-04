//! Error types for the dataset catalog subsystem.
//!
//! All errors follow the `[catalog] operation: description` format.
//! Maps to `VfsError` variants for the VFS provider interface.

use ff_vfs::VfsError;

/// Error type for all catalog operations.
///
/// Maps to `VfsError` variants for the VFS provider interface.
///
/// # Error Format
///
/// All `Display` output follows `[catalog] operation: description`.
#[derive(Debug, thiserror::Error)]
#[non_exhaustive]
pub enum CatalogError {
    /// Invalid dataset name format.
    #[error("[catalog] {operation}: invalid DSN '{input}': {reason} at position {position}")]
    DsnValidation {
        /// The input string that failed validation.
        input: String,
        /// Description of the validation failure.
        reason: String,
        /// Position of the offending character/qualifier.
        position: usize,
        /// The operation that was attempted.
        operation: String,
    },

    /// Dataset name already exists in a mounted catalog.
    #[error("[catalog] {operation}: dataset already exists: {dsn} (in catalog '{catalog}')")]
    DuplicateDataset {
        /// The duplicate DSN.
        dsn: String,
        /// The catalog containing the duplicate.
        catalog: String,
        /// The operation that was attempted.
        operation: String,
    },

    /// Dataset name does not exist in any mounted catalog.
    #[error("[catalog] {operation}: dataset not found: {dsn}")]
    DatasetNotFound {
        /// The DSN that was not found.
        dsn: String,
        /// The operation that was attempted.
        operation: String,
    },

    /// No catalog is mounted / catalog not found by name.
    #[error("[catalog] {operation}: catalog not mounted: {name}")]
    CatalogNotMounted {
        /// The catalog name or path.
        name: String,
        /// The operation that was attempted.
        operation: String,
    },

    /// Catalog is already mounted.
    #[error("[catalog] {operation}: catalog already mounted: {name}")]
    CatalogAlreadyMounted {
        /// The catalog name or path.
        name: String,
        /// The operation that was attempted.
        operation: String,
    },

    /// Repository structure is invalid or corrupt.
    #[error("[catalog] {operation}: repository invalid at '{path}': {reason}")]
    RepositoryCorrupt {
        /// The repository path.
        path: String,
        /// Description of the corruption.
        reason: String,
        /// The operation that was attempted.
        operation: String,
    },

    /// Database schema version mismatch.
    #[error("[catalog] {operation}: schema version mismatch: found {found}, expected {expected}")]
    SchemaVersionMismatch {
        /// The version found in the database.
        found: String,
        /// The expected version.
        expected: String,
        /// The operation that was attempted.
        operation: String,
    },

    /// Underlying I/O error.
    #[error("[catalog] {operation}: I/O error: {source}")]
    IoError {
        /// The operation that was attempted.
        operation: String,
        /// The underlying I/O error.
        #[source]
        source: std::io::Error,
    },

    /// SQLite database error.
    #[error("[catalog] {operation}: database error: {source}")]
    SqliteError {
        /// The operation that was attempted.
        operation: String,
        /// The underlying SQLite error.
        #[source]
        source: rusqlite::Error,
    },

    /// GDG limit exceeded or invalid reference.
    #[error("[catalog] {operation}: GDG error for '{dsn}': {reason}")]
    GdgLimitExceeded {
        /// The GDG base DSN.
        dsn: String,
        /// Description of the error.
        reason: String,
        /// The operation that was attempted.
        operation: String,
    },

    /// PDS member not found.
    #[error("[catalog] {operation}: member '{member}' not found in {dsn}")]
    MemberNotFound {
        /// The PDS DSN.
        dsn: String,
        /// The member name.
        member: String,
        /// The operation that was attempted.
        operation: String,
    },

    /// PDS member already exists.
    #[error("[catalog] {operation}: member '{member}' already exists in {dsn}")]
    MemberAlreadyExists {
        /// The PDS DSN.
        dsn: String,
        /// The member name.
        member: String,
        /// The operation that was attempted.
        operation: String,
    },

    /// Invalid allocation parameters.
    #[error("[catalog] {operation}: invalid allocation parameter: {reason}")]
    InvalidAllocationParams {
        /// Description of the invalid parameter.
        reason: String,
        /// The operation that was attempted.
        operation: String,
    },

    /// Export/import archive error.
    #[error("[catalog] {operation}: archive error: {reason}")]
    ExportError {
        /// Description of the error.
        reason: String,
        /// The operation that was attempted.
        operation: String,
    },

    /// Import error (separate from export for clarity).
    #[error("[catalog] {operation}: import error: {reason}")]
    ImportError {
        /// Description of the error.
        reason: String,
        /// The operation that was attempted.
        operation: String,
    },

    /// Operation not supported by this catalog location or transport.
    #[error("[catalog] {operation}: unsupported operation for scheme '{scheme}': {reason}")]
    UnsupportedOperation {
        /// The transport scheme that does not support the operation.
        scheme: String,
        /// Description of why it is unsupported.
        reason: String,
        /// The operation that was attempted.
        operation: String,
    },

    /// Operation attempted on wrong dataset type.
    #[error("[catalog] {operation}: dataset '{dsn}' is {actual_type}, expected {expected_type}")]
    TypeMismatch {
        /// The DSN of the dataset.
        dsn: String,
        /// The actual dataset type.
        actual_type: String,
        /// The expected dataset type.
        expected_type: String,
        /// The operation that was attempted.
        operation: String,
    },

    /// Invalid member name.
    #[error("[catalog] {operation}: invalid member name '{input}': {reason}")]
    InvalidMemberName {
        /// The invalid input.
        input: String,
        /// Description of the failure.
        reason: String,
        /// The operation that was attempted.
        operation: String,
    },
}

impl From<CatalogError> for VfsError {
    fn from(err: CatalogError) -> VfsError {
        match &err {
            CatalogError::DatasetNotFound { dsn, operation } => VfsError::NotFound {
                uri: dsn.clone(),
                operation: operation.clone(),
            },
            CatalogError::DuplicateDataset { dsn, operation, .. } => VfsError::AlreadyExists {
                uri: dsn.clone(),
                operation: operation.clone(),
            },
            CatalogError::DsnValidation { input, .. } => VfsError::InvalidUri {
                uri: input.clone(),
                reason: err.to_string(),
            },
            CatalogError::MemberNotFound {
                dsn,
                member,
                operation,
            } => VfsError::NotFound {
                uri: format!("{dsn}({member})"),
                operation: operation.clone(),
            },
            CatalogError::MemberAlreadyExists {
                dsn,
                member,
                operation,
            } => VfsError::AlreadyExists {
                uri: format!("{dsn}({member})"),
                operation: operation.clone(),
            },
            CatalogError::CatalogNotMounted { .. } => VfsError::ProviderUnavailable {
                scheme: "catalog".to_string(),
            },
            CatalogError::TypeMismatch { dsn, operation, .. } => VfsError::NotADirectory {
                uri: dsn.clone(),
                operation: operation.clone(),
            },
            _ => VfsError::Io {
                uri: String::new(),
                operation: String::new(),
                source: std::io::Error::other(err.to_string()),
            },
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn dsn_validation_error_follows_format() {
        // Validates: Requirement 2 AC 4
        let err = CatalogError::DsnValidation {
            input: "BAD..NAME".to_string(),
            reason: "consecutive dots".to_string(),
            position: 3,
            operation: "parse".to_string(),
        };
        let msg = err.to_string();
        assert!(msg.starts_with("[catalog]"));
        assert!(msg.contains("BAD..NAME"));
        assert!(msg.contains("consecutive dots"));
    }

    #[test]
    fn dataset_not_found_maps_to_vfs_not_found() {
        // Validates: Requirement 10 AC 12
        let err = CatalogError::DatasetNotFound {
            dsn: "TEST.DSN".to_string(),
            operation: "resolve".to_string(),
        };
        let vfs_err: VfsError = err.into();
        match vfs_err {
            VfsError::NotFound { uri, operation } => {
                assert_eq!(uri, "TEST.DSN");
                assert_eq!(operation, "resolve");
            }
            other => panic!("expected NotFound, got: {other:?}"),
        }
    }

    #[test]
    fn duplicate_dataset_maps_to_vfs_already_exists() {
        // Validates: Requirement 10 AC 12
        let err = CatalogError::DuplicateDataset {
            dsn: "TEST.DSN".to_string(),
            catalog: "DEV".to_string(),
            operation: "allocate".to_string(),
        };
        let vfs_err: VfsError = err.into();
        match vfs_err {
            VfsError::AlreadyExists { uri, operation } => {
                assert_eq!(uri, "TEST.DSN");
                assert_eq!(operation, "allocate");
            }
            other => panic!("expected AlreadyExists, got: {other:?}"),
        }
    }

    #[test]
    fn catalog_not_mounted_maps_to_provider_unavailable() {
        // Validates: Requirement 10 AC 12
        let err = CatalogError::CatalogNotMounted {
            name: "TEST".to_string(),
            operation: "resolve".to_string(),
        };
        let vfs_err: VfsError = err.into();
        match vfs_err {
            VfsError::ProviderUnavailable { scheme } => {
                assert_eq!(scheme, "catalog");
            }
            other => panic!("expected ProviderUnavailable, got: {other:?}"),
        }
    }
}
