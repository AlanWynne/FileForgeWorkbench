use crate::error::ValueType;
use crate::schema::{SchemaEntry, SchemaRegistry};
use crate::value::ConfigValue;

/// Register core schema entries for all well-known configuration keys.
///
/// Populates the schema registry with the default entries for editor, logging,
/// theme, and VFS namespaces. These defaults serve as the Defaults layer (priority 0)
/// in the six-layer model.
///
/// Addresses: Requirement 2 (AC 2.1), Requirement 9 (AC 9.1)
pub fn register_core_schema(schema: &mut SchemaRegistry) {
    let entries = [
        SchemaEntry {
            key: crate::keys::editor::TAB_SIZE.to_string(),
            value_type: ValueType::Integer,
            default: ConfigValue::Integer(4),
            description: "Number of spaces per tab stop".to_string(),
            constraints: Some(crate::schema::Constraints {
                min: Some(1.0),
                max: Some(16.0),
                allowed_values: None,
                pattern: None,
            }),
        },
        SchemaEntry {
            key: crate::keys::editor::INDENT_STYLE.to_string(),
            value_type: ValueType::String,
            default: ConfigValue::String("space".to_string()),
            description: "Indent style: space or tab".to_string(),
            constraints: Some(crate::schema::Constraints {
                min: None,
                max: None,
                allowed_values: Some(vec![
                    ConfigValue::String("space".to_string()),
                    ConfigValue::String("tab".to_string()),
                ]),
                pattern: None,
            }),
        },
        SchemaEntry {
            key: crate::keys::editor::LINE_ENDINGS.to_string(),
            value_type: ValueType::String,
            default: ConfigValue::String("lf".to_string()),
            description: "Line ending style: lf, crlf, or cr".to_string(),
            constraints: Some(crate::schema::Constraints {
                min: None,
                max: None,
                allowed_values: Some(vec![
                    ConfigValue::String("lf".to_string()),
                    ConfigValue::String("crlf".to_string()),
                    ConfigValue::String("cr".to_string()),
                ]),
                pattern: None,
            }),
        },
        SchemaEntry {
            key: crate::keys::editor::TRIM_TRAILING_WHITESPACE.to_string(),
            value_type: ValueType::Boolean,
            default: ConfigValue::Boolean(false),
            description: "Whether to trim trailing whitespace on save".to_string(),
            constraints: None,
        },
        SchemaEntry {
            key: crate::keys::editor::INSERT_FINAL_NEWLINE.to_string(),
            value_type: ValueType::Boolean,
            default: ConfigValue::Boolean(true),
            description: "Whether to insert a final newline on save".to_string(),
            constraints: None,
        },
        SchemaEntry {
            key: crate::keys::logging::LEVEL.to_string(),
            value_type: ValueType::String,
            default: ConfigValue::String("info".to_string()),
            description: "Logging level: trace, debug, info, warn, error".to_string(),
            constraints: Some(crate::schema::Constraints {
                min: None,
                max: None,
                allowed_values: Some(vec![
                    ConfigValue::String("trace".to_string()),
                    ConfigValue::String("debug".to_string()),
                    ConfigValue::String("info".to_string()),
                    ConfigValue::String("warn".to_string()),
                    ConfigValue::String("error".to_string()),
                ]),
                pattern: None,
            }),
        },
        SchemaEntry {
            key: crate::keys::logging::DIRECTORY.to_string(),
            value_type: ValueType::String,
            default: ConfigValue::String(String::new()),
            description: "Directory for log file output".to_string(),
            constraints: None,
        },
        SchemaEntry {
            key: crate::keys::logging::MAX_FILE_SIZE_MB.to_string(),
            value_type: ValueType::Integer,
            default: ConfigValue::Integer(10),
            description: "Maximum log file size in megabytes before rotation".to_string(),
            constraints: Some(crate::schema::Constraints {
                min: Some(1.0),
                max: Some(1024.0),
                allowed_values: None,
                pattern: None,
            }),
        },
        SchemaEntry {
            key: crate::keys::logging::MAX_RETAINED_FILES.to_string(),
            value_type: ValueType::Integer,
            default: ConfigValue::Integer(5),
            description: "Maximum number of retained rotated log files".to_string(),
            constraints: Some(crate::schema::Constraints {
                min: Some(1.0),
                max: Some(100.0),
                allowed_values: None,
                pattern: None,
            }),
        },
        SchemaEntry {
            key: crate::keys::theme::ACTIVE.to_string(),
            value_type: ValueType::String,
            default: ConfigValue::String("default".to_string()),
            description: "Active theme name".to_string(),
            constraints: None,
        },
        SchemaEntry {
            key: crate::keys::theme::FOLLOW_OS.to_string(),
            value_type: ValueType::Boolean,
            default: ConfigValue::Boolean(false),
            description: "Follow OS dark/light mode automatically".to_string(),
            constraints: None,
        },
        SchemaEntry {
            key: crate::keys::theme::FONT_SIZE.to_string(),
            value_type: ValueType::Integer,
            default: ConfigValue::Integer(14),
            description: "Font size in points".to_string(),
            constraints: Some(crate::schema::Constraints {
                min: Some(6.0),
                max: Some(72.0),
                allowed_values: None,
                pattern: None,
            }),
        },
        SchemaEntry {
            key: crate::keys::vfs::DEFAULT_PROVIDER.to_string(),
            value_type: ValueType::String,
            default: ConfigValue::String("local".to_string()),
            description: "Default virtual file system provider".to_string(),
            constraints: None,
        },
    ];

    for entry in entries {
        // Core schema registration should never conflict -- unwrap is safe here
        // since we control all entries and they have unique keys.
        schema
            .register(entry)
            .expect("core schema entries must not conflict");
    }
}

/// Register catalog schema entries with resolved default paths.
///
/// Called from `ff-desktop` after the user data directory is resolved,
/// so the defaults are concrete filesystem paths rather than templates.
///
/// Addresses: Requirement 12.3, 12.4, 12.5
pub fn register_catalog_schema(
    schema: &mut SchemaRegistry,
    mainframe_root: &str,
    posix_root: &str,
) {
    let entries = [
        SchemaEntry {
            key: crate::keys::catalogs::DEFAULT_MAINFRAME_ROOT.to_string(),
            value_type: ValueType::String,
            default: ConfigValue::String(mainframe_root.to_string()),
            description: "Default repository root directory for new Mainframe catalogs".to_string(),
            constraints: None,
        },
        SchemaEntry {
            key: crate::keys::catalogs::DEFAULT_POSIX_ROOT.to_string(),
            value_type: ValueType::String,
            default: ConfigValue::String(posix_root.to_string()),
            description: "Default root directory for new POSIX catalogs".to_string(),
            constraints: None,
        },
    ];
    for entry in entries {
        schema
            .register(entry)
            .expect("catalog schema entries must not conflict");
    }
}
