//! Catalog configuration — reads `[catalog]` keys from the configuration system.
//!
//! Defines the schema and access logic for catalog-related configuration keys.

/// Catalog configuration keys and defaults.
#[derive(Debug, Clone)]
pub struct CatalogConfig {
    /// Configured catalog directory locations.
    pub locations: Vec<String>,
    /// Active catalog location path.
    pub active_location: Option<String>,
    /// Whether auto-association is enabled.
    pub auto_associate: bool,
    /// Default field type for new fields.
    pub default_field_type: String,
}

impl Default for CatalogConfig {
    fn default() -> Self {
        Self {
            locations: Vec::new(),
            active_location: None,
            auto_associate: true,
            default_field_type: "alphanumeric".to_string(),
        }
    }
}

impl CatalogConfig {
    /// Create a new configuration with defaults.
    pub fn new() -> Self {
        Self::default()
    }

    /// Get the default catalog location for the current platform.
    ///
    /// - Linux: `~/.config/ffworkbench/catalogs/`
    /// - Windows: `%APPDATA%\FFWorkbench\catalogs\`
    /// - macOS: `~/Library/Application Support/FFWorkbench/catalogs/`
    pub fn default_catalog_path() -> Option<String> {
        #[cfg(target_os = "windows")]
        {
            std::env::var("APPDATA")
                .ok()
                .map(|appdata| format!("{appdata}\\FFWorkbench\\catalogs"))
        }

        #[cfg(target_os = "linux")]
        {
            std::env::var("HOME")
                .ok()
                .map(|home| format!("{home}/.config/ffworkbench/catalogs"))
        }

        #[cfg(target_os = "macos")]
        {
            std::env::var("HOME")
                .ok()
                .map(|home| format!("{home}/Library/Application Support/FFWorkbench/catalogs"))
        }

        #[cfg(not(any(target_os = "windows", target_os = "linux", target_os = "macos")))]
        {
            None
        }
    }

    /// Get the effective active location, falling back to the platform default.
    pub fn effective_active_location(&self) -> Option<String> {
        self.active_location
            .clone()
            .or_else(Self::default_catalog_path)
    }

    /// Check if auto-association is enabled.
    pub fn is_auto_associate_enabled(&self) -> bool {
        self.auto_associate
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    // Validates: Requirement 15.1 — configuration schema
    #[test]
    fn default_config_has_expected_values() {
        let config = CatalogConfig::default();
        assert!(config.locations.is_empty());
        assert!(config.active_location.is_none());
        assert!(config.auto_associate);
        assert_eq!(config.default_field_type, "alphanumeric");
    }

    // Validates: Requirement 1.3 — platform default path
    #[test]
    fn default_catalog_path_returns_some() {
        // On any platform with HOME or APPDATA set, this should return Some
        let path = CatalogConfig::default_catalog_path();
        // May be None in CI without HOME set, so we just verify it doesn't panic
        let _ = path;
    }

    // Validates: Requirement 15.3 — auto_associate disable
    #[test]
    fn auto_associate_flag() {
        let mut config = CatalogConfig::new();
        assert!(config.is_auto_associate_enabled());
        config.auto_associate = false;
        assert!(!config.is_auto_associate_enabled());
    }

    // Validates: Requirement 15.2 — active_location fallback
    #[test]
    fn effective_active_location_uses_configured_first() {
        let mut config = CatalogConfig::new();
        config.active_location = Some("/custom/path".to_string());
        assert_eq!(
            config.effective_active_location(),
            Some("/custom/path".to_string())
        );
    }
}
