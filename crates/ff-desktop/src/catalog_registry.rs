//! # Catalog Registry
//!
//! Maintains the in-memory list of all defined `VirtualCatalog` entries and
//! persists them to `session.toml` under the `[[virtual_catalogs]]` array.
//!
//! Validates: Requirement 2.1–2.5

// Types and methods are wired into the UI in Tasks 4–10; suppress until then.
#![allow(dead_code)]

use serde::{Deserialize, Serialize};

// ── Types ────────────────────────────────────────────────────────────────────

/// The classification of a virtual catalog.
///
/// Validates: Requirement 3.2
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum CatalogType {
    /// z/OS-style datasets backed by `ff-dscatalog`.
    Mainframe,
    /// POSIX-style hierarchical filesystem emulation (new `posix` VFS provider).
    Posix,
    /// The host platform's local filesystem (Windows, Linux, or macOS).
    Native,
}

impl CatalogType {
    /// Human-readable label used in the UI section header.
    #[allow(dead_code)] // used in Task 4 files_panel rendering
    pub fn section_label(self) -> &'static str {
        match self {
            CatalogType::Mainframe => "Mainframe Catalogs",
            CatalogType::Posix => "POSIX Catalogs",
            CatalogType::Native => "Native Catalogs",
        }
    }
}

/// A single registered virtual catalog entry.
///
/// Validates: Requirement 2.1
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct VirtualCatalog {
    /// Unique name across all catalog types (1–32 chars, alphanumeric/hyphen/underscore).
    pub name: String,
    /// Catalog classification.
    #[serde(rename = "type")]
    pub catalog_type: CatalogType,
    /// Backing path: repository directory (Mainframe), root directory (POSIX/Native).
    pub path: String,
    /// Optional free-text description (up to 120 chars).
    pub description: Option<String>,
    /// When true, mount this catalog automatically on startup.
    pub auto_mount: bool,
    /// Mainframe only: default high-level qualifier prepended to bare DSNs.
    pub default_hlq: Option<String>,
    /// POSIX only: the POSIX path prefix (default `/`).
    pub mount_point: Option<String>,
    /// POSIX / Native: when true, all write operations are rejected.
    pub read_only: bool,
}

impl VirtualCatalog {
    /// Validate the catalog name: 1–32 chars, alphanumeric / hyphen / underscore only.
    ///
    /// Validates: Requirement 3.3, 3.8
    pub fn is_valid_name(name: &str) -> bool {
        !name.is_empty()
            && name.len() <= 32
            && name
                .chars()
                .all(|c| c.is_alphanumeric() || c == '-' || c == '_')
    }
}

// ── Registry ─────────────────────────────────────────────────────────────────

/// In-memory registry of all defined virtual catalogs.
///
/// Validates: Requirement 2.3–2.5
#[derive(Debug, Default)]
pub struct CatalogRegistry {
    catalogs: Vec<VirtualCatalog>,
}

/// Errors returned by registry mutation operations.
#[derive(Debug, PartialEq, Eq)]
pub enum RegistryError {
    /// A catalog with this name already exists.
    DuplicateName(String),
    /// No catalog with this name was found.
    NotFound(String),
    /// The catalog name is invalid.
    InvalidName(String),
}

impl std::fmt::Display for RegistryError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            RegistryError::DuplicateName(n) => write!(f, "catalog '{n}' already exists"),
            RegistryError::NotFound(n) => write!(f, "catalog '{n}' not found"),
            RegistryError::InvalidName(n) => write!(f, "invalid catalog name '{n}'"),
        }
    }
}

impl CatalogRegistry {
    /// Create an empty registry.
    pub fn new() -> Self {
        Self::default()
    }

    /// Register a new catalog.
    ///
    /// Validates: Requirement 2.3, 2.4
    pub fn register(&mut self, catalog: VirtualCatalog) -> Result<(), RegistryError> {
        if !VirtualCatalog::is_valid_name(&catalog.name) {
            return Err(RegistryError::InvalidName(catalog.name));
        }
        if self.catalogs.iter().any(|c| c.name == catalog.name) {
            return Err(RegistryError::DuplicateName(catalog.name));
        }
        self.catalogs.push(catalog);
        Ok(())
    }

    /// Update an existing catalog's mutable properties.
    ///
    /// Name and type are immutable after creation.
    /// Validates: Requirement 2.3, 4.2
    pub fn update(
        &mut self,
        name: &str,
        description: Option<String>,
        auto_mount: bool,
        read_only: bool,
        default_hlq: Option<String>,
    ) -> Result<(), RegistryError> {
        let catalog = self
            .catalogs
            .iter_mut()
            .find(|c| c.name == name)
            .ok_or_else(|| RegistryError::NotFound(name.to_string()))?;
        catalog.description = description;
        catalog.auto_mount = auto_mount;
        catalog.read_only = read_only;
        catalog.default_hlq = default_hlq;
        Ok(())
    }

    /// Remove a catalog by name.
    ///
    /// Validates: Requirement 2.3, 4.4
    pub fn remove(&mut self, name: &str) -> Result<VirtualCatalog, RegistryError> {
        let idx = self
            .catalogs
            .iter()
            .position(|c| c.name == name)
            .ok_or_else(|| RegistryError::NotFound(name.to_string()))?;
        Ok(self.catalogs.remove(idx))
    }

    /// List all catalogs.
    ///
    /// Validates: Requirement 2.5
    pub fn list(&self) -> &[VirtualCatalog] {
        &self.catalogs
    }

    /// List catalogs filtered by type.
    ///
    /// Validates: Requirement 2.5
    pub fn list_by_type(&self, catalog_type: CatalogType) -> Vec<&VirtualCatalog> {
        self.catalogs
            .iter()
            .filter(|c| c.catalog_type == catalog_type)
            .collect()
    }

    /// Get a catalog by name.
    ///
    /// Validates: Requirement 2.5
    pub fn get_by_name(&self, name: &str) -> Option<&VirtualCatalog> {
        self.catalogs.iter().find(|c| c.name == name)
    }

    /// Check whether a catalog with the given name exists.
    ///
    /// Validates: Requirement 2.5
    pub fn exists(&self, name: &str) -> bool {
        self.catalogs.iter().any(|c| c.name == name)
    }

    /// Deserialise the registry from a TOML string (the `[[virtual_catalogs]]` array).
    ///
    /// Validates: Requirement 2.2
    pub fn load_from_toml(toml_str: &str) -> Self {
        #[derive(Deserialize, Default)]
        struct Root {
            #[serde(default)]
            virtual_catalogs: Vec<VirtualCatalog>,
        }
        let root: Root = toml::from_str(toml_str).unwrap_or_default();
        Self {
            catalogs: root.virtual_catalogs,
        }
    }

    /// Serialise the registry to a TOML fragment (the `[[virtual_catalogs]]` array).
    ///
    /// Validates: Requirement 2.1
    pub fn save_to_toml(&self) -> String {
        #[derive(Serialize)]
        struct Root<'a> {
            virtual_catalogs: &'a [VirtualCatalog],
        }
        toml::to_string(&Root {
            virtual_catalogs: &self.catalogs,
        })
        .unwrap_or_default()
    }
}

// ── Tests ────────────────────────────────────────────────────────────────────

#[cfg(test)]
mod tests {
    use super::*;

    fn mainframe_catalog(name: &str) -> VirtualCatalog {
        VirtualCatalog {
            name: name.to_string(),
            catalog_type: CatalogType::Mainframe,
            path: "/catalogs/payroll".to_string(),
            description: Some("Payroll datasets".to_string()),
            auto_mount: true,
            default_hlq: Some("PAYROLL".to_string()),
            mount_point: None,
            read_only: false,
        }
    }

    fn native_catalog(name: &str) -> VirtualCatalog {
        VirtualCatalog {
            name: name.to_string(),
            catalog_type: CatalogType::Native,
            path: "/home/user/projects".to_string(),
            description: None,
            auto_mount: true,
            default_hlq: None,
            mount_point: None,
            read_only: false,
        }
    }

    /// Validates: Requirement 2.3 — register adds a catalog to the list.
    #[test]
    fn register_adds_catalog_to_list() {
        // Validates: Requirement 2.3
        let mut reg = CatalogRegistry::new();
        reg.register(mainframe_catalog("PAYROLL")).unwrap();
        assert_eq!(reg.list().len(), 1);
        assert_eq!(reg.list()[0].name, "PAYROLL");
    }

    /// Validates: Requirement 2.4 — duplicate name is rejected.
    #[test]
    fn register_rejects_duplicate_name() {
        // Validates: Requirement 2.4
        let mut reg = CatalogRegistry::new();
        reg.register(mainframe_catalog("PAYROLL")).unwrap();
        let err = reg.register(mainframe_catalog("PAYROLL")).unwrap_err();
        assert_eq!(err, RegistryError::DuplicateName("PAYROLL".to_string()));
    }

    /// Validates: Requirement 3.8 — invalid name characters are rejected.
    #[test]
    fn register_rejects_invalid_name() {
        // Validates: Requirement 3.8
        let mut reg = CatalogRegistry::new();
        let mut bad = mainframe_catalog("bad name!");
        bad.name = "bad name!".to_string();
        let err = reg.register(bad).unwrap_err();
        assert!(matches!(err, RegistryError::InvalidName(_)));
    }

    /// Validates: Requirement 3.8 — empty name is rejected.
    #[test]
    fn register_rejects_empty_name() {
        // Validates: Requirement 3.8
        let mut reg = CatalogRegistry::new();
        let mut bad = mainframe_catalog("");
        bad.name = String::new();
        let err = reg.register(bad).unwrap_err();
        assert!(matches!(err, RegistryError::InvalidName(_)));
    }

    /// Validates: Requirement 3.8 — name over 32 chars is rejected.
    #[test]
    fn register_rejects_name_over_32_chars() {
        // Validates: Requirement 3.8
        let mut reg = CatalogRegistry::new();
        let long_name = "A".repeat(33);
        let mut cat = mainframe_catalog(&long_name);
        cat.name = long_name.clone();
        let err = reg.register(cat).unwrap_err();
        assert!(matches!(err, RegistryError::InvalidName(_)));
    }

    /// Validates: Requirement 2.3 — remove deletes a catalog by name.
    #[test]
    fn remove_deletes_catalog_by_name() {
        // Validates: Requirement 2.3
        let mut reg = CatalogRegistry::new();
        reg.register(mainframe_catalog("PAYROLL")).unwrap();
        reg.remove("PAYROLL").unwrap();
        assert!(reg.list().is_empty());
    }

    /// Validates: Requirement 2.3 — remove on unknown name returns NotFound.
    #[test]
    fn remove_unknown_name_returns_not_found() {
        // Validates: Requirement 2.3
        let mut reg = CatalogRegistry::new();
        let err = reg.remove("NOSUCH").unwrap_err();
        assert_eq!(err, RegistryError::NotFound("NOSUCH".to_string()));
    }

    /// Validates: Requirement 2.5 — list_by_type filters correctly.
    #[test]
    fn list_by_type_returns_only_matching_type() {
        // Validates: Requirement 2.5
        let mut reg = CatalogRegistry::new();
        reg.register(mainframe_catalog("MF1")).unwrap();
        reg.register(native_catalog("NAT1")).unwrap();
        let mf = reg.list_by_type(CatalogType::Mainframe);
        assert_eq!(mf.len(), 1);
        assert_eq!(mf[0].name, "MF1");
        let nat = reg.list_by_type(CatalogType::Native);
        assert_eq!(nat.len(), 1);
        assert_eq!(nat[0].name, "NAT1");
        let posix = reg.list_by_type(CatalogType::Posix);
        assert!(posix.is_empty());
    }

    /// Validates: Requirement 2.5 — get_by_name returns the correct entry.
    #[test]
    fn get_by_name_returns_correct_entry() {
        // Validates: Requirement 2.5
        let mut reg = CatalogRegistry::new();
        reg.register(mainframe_catalog("PAYROLL")).unwrap();
        let cat = reg.get_by_name("PAYROLL").unwrap();
        assert_eq!(cat.catalog_type, CatalogType::Mainframe);
    }

    /// Validates: Requirement 2.5 — get_by_name returns None for unknown name.
    #[test]
    fn get_by_name_returns_none_for_unknown() {
        // Validates: Requirement 2.5
        let reg = CatalogRegistry::new();
        assert!(reg.get_by_name("NOSUCH").is_none());
    }

    /// Validates: Requirement 2.5 — exists returns true/false correctly.
    #[test]
    fn exists_returns_true_for_registered_catalog() {
        // Validates: Requirement 2.5
        let mut reg = CatalogRegistry::new();
        reg.register(mainframe_catalog("PAYROLL")).unwrap();
        assert!(reg.exists("PAYROLL"));
        assert!(!reg.exists("OTHER"));
    }

    /// Validates: Requirement 4.2 — update changes mutable fields only.
    #[test]
    fn update_changes_description_and_auto_mount() {
        // Validates: Requirement 4.2
        let mut reg = CatalogRegistry::new();
        reg.register(mainframe_catalog("PAYROLL")).unwrap();
        reg.update(
            "PAYROLL",
            Some("Updated desc".to_string()),
            false,
            true,
            None,
        )
        .unwrap();
        let cat = reg.get_by_name("PAYROLL").unwrap();
        assert_eq!(cat.description.as_deref(), Some("Updated desc"));
        assert!(!cat.auto_mount);
        assert!(cat.read_only);
        // Name and type must be unchanged
        assert_eq!(cat.name, "PAYROLL");
        assert_eq!(cat.catalog_type, CatalogType::Mainframe);
    }

    /// Validates: Requirement 2.1, 2.2 — TOML round-trip preserves all fields.
    #[test]
    fn toml_round_trip_preserves_catalog_fields() {
        // Validates: Requirement 2.1, 2.2
        let mut reg = CatalogRegistry::new();
        reg.register(mainframe_catalog("PAYROLL")).unwrap();
        reg.register(native_catalog("projects")).unwrap();

        let toml_str = reg.save_to_toml();
        let loaded = CatalogRegistry::load_from_toml(&toml_str);

        assert_eq!(loaded.list().len(), 2);
        let mf = loaded.get_by_name("PAYROLL").unwrap();
        assert_eq!(mf.catalog_type, CatalogType::Mainframe);
        assert_eq!(mf.default_hlq.as_deref(), Some("PAYROLL"));
        let nat = loaded.get_by_name("projects").unwrap();
        assert_eq!(nat.catalog_type, CatalogType::Native);
    }

    /// Validates: Requirement 2.2 — loading from empty string returns empty registry.
    #[test]
    fn load_from_empty_toml_returns_empty_registry() {
        // Validates: Requirement 2.2
        let reg = CatalogRegistry::load_from_toml("");
        assert!(reg.list().is_empty());
    }

    /// Validates: Requirement 2.2 — loading from corrupt TOML returns empty registry.
    #[test]
    fn load_from_corrupt_toml_returns_empty_registry() {
        // Validates: Requirement 2.2
        let reg = CatalogRegistry::load_from_toml("[[[[not valid toml");
        assert!(reg.list().is_empty());
    }
}
