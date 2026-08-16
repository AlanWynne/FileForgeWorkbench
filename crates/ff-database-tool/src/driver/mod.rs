//! Driver abstraction and registry.
//!
//! Provides a registry of available Rust database drivers with capability
//! detection and TOML persistence.

use std::collections::HashMap;

use crate::error::DatabaseToolError;
use crate::types::SqlDialect;

/// Capabilities of a database driver.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct DriverCapabilities {
    /// Driver supports read-only access.
    pub read_only: bool,
    /// Driver supports read-write access.
    pub read_write: bool,
    /// Driver supports transactions.
    pub transactions: bool,
    /// Driver supports streaming result sets.
    pub streaming: bool,
    /// Driver supports prepared statements.
    pub prepared_statements: bool,
    /// Driver supports bulk load operations.
    pub bulk_load: bool,
}

impl Default for DriverCapabilities {
    fn default() -> Self {
        Self {
            read_only: true,
            read_write: true,
            transactions: true,
            streaming: true,
            prepared_statements: true,
            bulk_load: false,
        }
    }
}

/// A driver-specific connection parameter definition.
#[derive(Debug, Clone)]
pub struct DriverParam {
    /// Parameter name (e.g., "ssl_mode").
    pub name: String,
    /// Human-readable display name.
    pub display_name: String,
    /// Parameter type.
    pub param_type: ParamType,
    /// Whether this parameter is required.
    pub required: bool,
    /// Default value (if any).
    pub default_value: Option<String>,
}

/// Type of a driver connection parameter.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum ParamType {
    String,
    Integer,
    Boolean,
    Enum(Vec<String>),
}

/// Definition of a database driver.
#[derive(Debug, Clone)]
pub struct DriverDefinition {
    /// Unique driver name (e.g., "postgresql").
    pub name: String,
    /// Human-readable display name.
    pub display_name: String,
    /// Supported SQL dialect.
    pub dialect: SqlDialect,
    /// Connection URL template with placeholders.
    pub url_template: String,
    /// Default port number.
    pub default_port: u16,
    /// Name of the Rust crate providing this driver.
    pub crate_name: String,
    /// Driver capabilities.
    pub capabilities: DriverCapabilities,
    /// Driver-specific connection parameters.
    pub params: Vec<DriverParam>,
}

impl DriverDefinition {
    /// Build a connection URL from the template and provided values.
    ///
    /// Replaces `{key}` placeholders with values from the map.
    pub fn build_url(&self, values: &HashMap<String, String>) -> String {
        let mut url = self.url_template.clone();
        for (key, value) in values {
            url = url.replace(&format!("{{{key}}}"), value);
        }
        url
    }
}

/// Registry of available database drivers.
pub struct DriverRegistry {
    drivers: HashMap<String, DriverDefinition>,
}

impl DriverRegistry {
    /// Create a new empty registry.
    pub fn new() -> Self {
        Self {
            drivers: HashMap::new(),
        }
    }

    /// Create a registry pre-populated with built-in driver definitions.
    pub fn with_builtin_drivers() -> Self {
        let mut registry = Self::new();
        registry.register(Self::postgresql_driver());
        registry.register(Self::mysql_driver());
        registry.register(Self::sqlite_driver());
        registry.register(Self::sqlserver_driver());
        registry
    }

    /// Register a driver definition.
    pub fn register(&mut self, driver: DriverDefinition) {
        self.drivers.insert(driver.name.clone(), driver);
    }

    /// Find a driver by name.
    ///
    /// # Errors
    ///
    /// Returns `DriverNotFound` if no driver with that name is registered.
    pub fn find_by_name(&self, name: &str) -> Result<&DriverDefinition, DatabaseToolError> {
        self.drivers
            .get(name)
            .ok_or_else(|| DatabaseToolError::DriverNotFound {
                driver_name: name.to_string(),
            })
    }

    /// Find drivers supporting the given SQL dialect.
    pub fn find_by_dialect(&self, dialect: SqlDialect) -> Vec<&DriverDefinition> {
        self.drivers
            .values()
            .filter(|d| d.dialect == dialect)
            .collect()
    }

    /// List all registered drivers.
    pub fn list_drivers(&self) -> Vec<&DriverDefinition> {
        let mut drivers: Vec<_> = self.drivers.values().collect();
        drivers.sort_by_key(|d| &d.name);
        drivers
    }

    /// Number of registered drivers.
    pub fn len(&self) -> usize {
        self.drivers.len()
    }

    /// Whether the registry is empty.
    pub fn is_empty(&self) -> bool {
        self.drivers.is_empty()
    }

    // ── Built-in Driver Definitions ───────────────────────────────────────

    fn postgresql_driver() -> DriverDefinition {
        DriverDefinition {
            name: "postgresql".to_string(),
            display_name: "PostgreSQL".to_string(),
            dialect: SqlDialect::PostgreSql,
            url_template: "postgres://{user}:{password}@{host}:{port}/{database}".to_string(),
            default_port: 5432,
            crate_name: "sqlx".to_string(),
            capabilities: DriverCapabilities {
                bulk_load: true,
                ..Default::default()
            },
            params: vec![DriverParam {
                name: "ssl_mode".to_string(),
                display_name: "SSL Mode".to_string(),
                param_type: ParamType::Enum(vec![
                    "disable".into(),
                    "allow".into(),
                    "prefer".into(),
                    "require".into(),
                    "verify-ca".into(),
                    "verify-full".into(),
                ]),
                required: false,
                default_value: Some("prefer".to_string()),
            }],
        }
    }

    fn mysql_driver() -> DriverDefinition {
        DriverDefinition {
            name: "mysql".to_string(),
            display_name: "MySQL / MariaDB".to_string(),
            dialect: SqlDialect::MySql,
            url_template: "mysql://{user}:{password}@{host}:{port}/{database}".to_string(),
            default_port: 3306,
            crate_name: "sqlx".to_string(),
            capabilities: DriverCapabilities {
                bulk_load: true,
                ..Default::default()
            },
            params: vec![],
        }
    }

    fn sqlite_driver() -> DriverDefinition {
        DriverDefinition {
            name: "sqlite".to_string(),
            display_name: "SQLite".to_string(),
            dialect: SqlDialect::Sqlite,
            url_template: "sqlite://{path}".to_string(),
            default_port: 0,
            crate_name: "rusqlite".to_string(),
            capabilities: DriverCapabilities {
                bulk_load: false,
                ..Default::default()
            },
            params: vec![],
        }
    }

    fn sqlserver_driver() -> DriverDefinition {
        DriverDefinition {
            name: "sqlserver".to_string(),
            display_name: "Microsoft SQL Server".to_string(),
            dialect: SqlDialect::TSql,
            url_template: "mssql://{user}:{password}@{host}:{port}/{database}".to_string(),
            default_port: 1433,
            crate_name: "tiberius".to_string(),
            capabilities: DriverCapabilities {
                bulk_load: false,
                ..Default::default()
            },
            params: vec![],
        }
    }
}

impl Default for DriverRegistry {
    fn default() -> Self {
        Self::with_builtin_drivers()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn builtin_drivers_registered() {
        // Validates: Requirement 2 AC 2
        let registry = DriverRegistry::with_builtin_drivers();
        assert!(registry.find_by_name("postgresql").is_ok());
        assert!(registry.find_by_name("mysql").is_ok());
        assert!(registry.find_by_name("sqlite").is_ok());
        assert!(registry.find_by_name("sqlserver").is_ok());
    }

    #[test]
    fn find_by_name_not_found() {
        // Validates: Requirement 2 AC 3
        let registry = DriverRegistry::new();
        let result = registry.find_by_name("oracle");
        assert!(matches!(
            result,
            Err(DatabaseToolError::DriverNotFound { .. })
        ));
    }

    #[test]
    fn find_by_dialect_returns_matching() {
        // Validates: Requirement 2 AC 3
        let registry = DriverRegistry::with_builtin_drivers();
        let pg_drivers = registry.find_by_dialect(SqlDialect::PostgreSql);
        assert!(!pg_drivers.is_empty());
        assert!(pg_drivers
            .iter()
            .all(|d| d.dialect == SqlDialect::PostgreSql));
    }

    #[test]
    fn list_drivers_sorted() {
        // Validates: Requirement 2 AC 3
        let registry = DriverRegistry::with_builtin_drivers();
        let drivers = registry.list_drivers();
        let names: Vec<_> = drivers.iter().map(|d| d.name.as_str()).collect();
        let mut sorted = names.clone();
        sorted.sort();
        assert_eq!(names, sorted);
    }

    #[test]
    fn build_url_substitutes_placeholders() {
        // Validates: Requirement 2 AC 5
        let registry = DriverRegistry::with_builtin_drivers();
        let driver = registry.find_by_name("postgresql").unwrap();
        let mut values = HashMap::new();
        values.insert("user".to_string(), "admin".to_string());
        values.insert("password".to_string(), "secret".to_string());
        values.insert("host".to_string(), "localhost".to_string());
        values.insert("port".to_string(), "5432".to_string());
        values.insert("database".to_string(), "mydb".to_string());
        let url = driver.build_url(&values);
        assert_eq!(url, "postgres://admin:secret@localhost:5432/mydb");
    }

    #[test]
    fn postgresql_default_port() {
        let registry = DriverRegistry::with_builtin_drivers();
        let driver = registry.find_by_name("postgresql").unwrap();
        assert_eq!(driver.default_port, 5432);
    }

    #[test]
    fn register_custom_driver() {
        // Validates: Requirement 2 AC 4
        let mut registry = DriverRegistry::new();
        registry.register(DriverDefinition {
            name: "custom".to_string(),
            display_name: "Custom DB".to_string(),
            dialect: SqlDialect::Generic,
            url_template: "custom://{host}".to_string(),
            default_port: 9999,
            crate_name: "custom-crate".to_string(),
            capabilities: DriverCapabilities::default(),
            params: vec![],
        });
        assert!(registry.find_by_name("custom").is_ok());
    }
}
