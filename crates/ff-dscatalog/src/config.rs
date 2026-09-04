//! Configuration integration for the catalog subsystem.
//!
//! Reads/writes the `[catalog]` namespace in ff-config for mounted catalogs,
//! default HLQ, and repository paths.

use std::path::PathBuf;

use serde::{Deserialize, Serialize};

/// Configuration for the catalog subsystem.
#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub struct CatalogConfig {
    /// Default High Level Qualifier for bare qualifier expansion.
    pub default_hlq: Option<String>,
    /// Default repository root path.
    pub repository_root: Option<PathBuf>,
    /// List of catalogs to mount.
    pub mounted_catalogs: Vec<MountedCatalogEntry>,
    /// Default allocation parameters.
    pub defaults: Option<AllocationDefaults>,
}

/// A persisted catalog mount entry.
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub struct MountedCatalogEntry {
    /// Catalog name.
    pub name: String,
    /// Repository root path (used when location = "local").
    pub path: PathBuf,
    /// Priority order (higher = checked first).
    pub priority: u32,
    /// Whether to auto-mount on startup.
    pub auto_mount: bool,
    /// Transport discriminant: "local" or "remote". Defaults to "local" when absent.
    #[serde(default = "default_location")]
    pub location: String,
    /// URI for remote catalogs (required when location = "remote").
    #[serde(skip_serializing_if = "Option::is_none")]
    pub uri: Option<String>,
}

fn default_location() -> String {
    "local".to_string()
}

/// Default allocation parameters per dataset type.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AllocationDefaults {
    /// Default RECFM for PS datasets.
    pub ps_recfm: Option<String>,
    /// Default LRECL for PS datasets.
    pub ps_lrecl: Option<u32>,
    /// Default BLKSIZE for PS datasets.
    pub ps_blksize: Option<u32>,
    /// Default RECFM for PO datasets.
    pub po_recfm: Option<String>,
    /// Default LRECL for PO datasets.
    pub po_lrecl: Option<u32>,
    /// Default BLKSIZE for PO datasets.
    pub po_blksize: Option<u32>,
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn config_default_is_empty() {
        // Validates: Requirement 14 AC 1
        let config = CatalogConfig::default();
        assert!(config.default_hlq.is_none());
        assert!(config.mounted_catalogs.is_empty());
    }

    #[test]
    fn config_serializes_correctly() {
        // Validates: Requirement 14 AC 2
        let config = CatalogConfig {
            default_hlq: Some("USER".to_string()),
            repository_root: Some(PathBuf::from("/home/user/.ffworkbench/catalogs")),
            mounted_catalogs: vec![MountedCatalogEntry {
                name: "DEV".to_string(),
                path: PathBuf::from("/home/user/catalogs/dev"),
                priority: 1,
                auto_mount: true,
                location: "local".to_string(),
                uri: None,
            }],
            defaults: None,
        };
        let json = serde_json::to_string(&config).unwrap();
        assert!(json.contains("USER"));
        assert!(json.contains("DEV"));
    }
}
