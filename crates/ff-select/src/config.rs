//! Configuration loading from `[criteria]` TOML namespace.
//!
//! Manages criteria subsystem configuration with validation,
//! defaults, and hot-reload support.

/// Configuration for the criteria subsystem, loaded from `[criteria]` TOML namespace.
///
/// Addresses: Requirement 14
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct CriteriaConfig {
    /// Custom path for the Criteria_Store file. None uses default location.
    pub store_path: Option<String>,
    /// Default Active_Criteria_Location path.
    pub default_location: String,
    /// Whether structure-association auto-suggestion is enabled.
    pub auto_suggest: bool,
    /// Maximum criteria rows per CriteriaSet.
    pub max_criteria_rows: usize,
}

impl Default for CriteriaConfig {
    fn default() -> Self {
        Self {
            store_path: None,
            default_location: String::from("~/.config/ffworkbench/criteria/"),
            auto_suggest: true,
            max_criteria_rows: 50,
        }
    }
}

impl CriteriaConfig {
    /// Load configuration from a TOML table.
    ///
    /// Applies validation rules:
    /// - `max_criteria_rows` clamped to [1, 200]
    /// - Invalid values fall back to defaults with warnings
    ///
    /// Addresses: Requirement 14 AC 1–6
    pub fn from_toml(table: &toml::Table) -> (Self, Vec<String>) {
        let mut config = Self::default();
        let mut warnings = Vec::new();

        if let Some(v) = table.get("store_path") {
            if let Some(s) = v.as_str() {
                config.store_path = Some(s.to_string());
            } else {
                warnings.push(String::from(
                    "[record-criteria] config: 'store_path' must be a string — using default",
                ));
            }
        }

        if let Some(v) = table.get("default_location") {
            if let Some(s) = v.as_str() {
                config.default_location = s.to_string();
            } else {
                warnings.push(String::from(
                    "[record-criteria] config: 'default_location' must be a string — using default",
                ));
            }
        }

        if let Some(v) = table.get("auto_suggest") {
            if let Some(b) = v.as_bool() {
                config.auto_suggest = b;
            } else {
                warnings.push(String::from(
                    "[record-criteria] config: 'auto_suggest' must be a boolean — using default true",
                ));
            }
        }

        if let Some(v) = table.get("max_criteria_rows") {
            if let Some(n) = v.as_integer() {
                let clamped = n.clamp(1, 200) as usize;
                if clamped != n as usize {
                    warnings.push(format!(
                        "[record-criteria] config: 'max_criteria_rows' value {n} clamped to [{clamped}]"
                    ));
                }
                config.max_criteria_rows = clamped;
            } else {
                warnings.push(String::from(
                    "[record-criteria] config: 'max_criteria_rows' must be an integer — using default 50",
                ));
            }
        }

        (config, warnings)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn default_config_has_expected_values() {
        let config = CriteriaConfig::default();
        assert!(config.store_path.is_none());
        assert_eq!(config.default_location, "~/.config/ffworkbench/criteria/");
        assert!(config.auto_suggest);
        assert_eq!(config.max_criteria_rows, 50);
    }

    #[test]
    fn from_toml_empty_table_returns_defaults() {
        let table = toml::Table::new();
        let (config, warnings) = CriteriaConfig::from_toml(&table);
        assert_eq!(config, CriteriaConfig::default());
        assert!(warnings.is_empty());
    }

    #[test]
    fn from_toml_valid_values() {
        let mut table = toml::Table::new();
        table.insert(
            "store_path".to_string(),
            toml::Value::String("/custom/store.toml".to_string()),
        );
        table.insert(
            "default_location".to_string(),
            toml::Value::String("/custom/criteria/".to_string()),
        );
        table.insert("auto_suggest".to_string(), toml::Value::Boolean(false));
        table.insert("max_criteria_rows".to_string(), toml::Value::Integer(100));

        let (config, warnings) = CriteriaConfig::from_toml(&table);
        assert!(warnings.is_empty());
        assert_eq!(config.store_path, Some("/custom/store.toml".to_string()));
        assert_eq!(config.default_location, "/custom/criteria/");
        assert!(!config.auto_suggest);
        assert_eq!(config.max_criteria_rows, 100);
    }

    #[test]
    fn from_toml_clamps_max_rows_low() {
        let mut table = toml::Table::new();
        table.insert("max_criteria_rows".to_string(), toml::Value::Integer(0));

        let (config, warnings) = CriteriaConfig::from_toml(&table);
        assert_eq!(config.max_criteria_rows, 1);
        assert!(!warnings.is_empty());
    }

    #[test]
    fn from_toml_clamps_max_rows_high() {
        let mut table = toml::Table::new();
        table.insert("max_criteria_rows".to_string(), toml::Value::Integer(500));

        let (config, warnings) = CriteriaConfig::from_toml(&table);
        assert_eq!(config.max_criteria_rows, 200);
        assert!(!warnings.is_empty());
    }

    #[test]
    fn from_toml_invalid_type_warns() {
        let mut table = toml::Table::new();
        table.insert(
            "auto_suggest".to_string(),
            toml::Value::String("yes".to_string()),
        );

        let (config, warnings) = CriteriaConfig::from_toml(&table);
        assert!(config.auto_suggest); // default
        assert_eq!(warnings.len(), 1);
    }
}
