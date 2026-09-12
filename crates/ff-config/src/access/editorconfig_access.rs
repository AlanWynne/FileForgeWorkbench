use std::path::Path;

use crate::editorconfig::parser::{
    Charset, EditorConfigProperties, EndOfLine, IndentSize, IndentStyle,
};
use crate::editorconfig::resolver::resolve_editorconfig as resolve_editorconfig_for_path;
use crate::error::ConfigError;
use crate::value::ConfigValue;

use super::ConfigAccess;

impl<'a> ConfigAccess<'a> {
    /// Resolve EditorConfig properties for a given file path.
    ///
    /// Delegates to the EditorConfig resolver, which traverses the directory
    /// hierarchy looking for `.editorconfig` files and merges matching sections.
    ///
    /// # Arguments
    ///
    /// * `file_path` -- The absolute path of the file to resolve properties for.
    ///
    /// # Returns
    ///
    /// The merged `EditorConfigProperties` for the given file path.
    pub fn resolve_editorconfig(&self, file_path: &Path) -> EditorConfigProperties {
        resolve_editorconfig_for_path(file_path)
    }

    /// Get a configuration value for a specific file, applying EditorConfig
    /// precedence for editor-scoped keys.
    ///
    /// For keys in the `editor.*` namespace, this method first resolves
    /// EditorConfig properties for the given file path. If EditorConfig
    /// provides a value for the corresponding property, that value is returned
    /// (EditorConfig overrides ALL configuration layers for editor keys).
    ///
    /// For keys outside the `editor.*` namespace (e.g., `logging.*`, `theme.*`,
    /// `plugins.*`, `vfs.*`), EditorConfig is never consulted and the normal
    /// layered resolution applies.
    ///
    /// # Arguments
    ///
    /// * `key` -- The configuration key to look up (e.g., `"editor.indent_style"`).
    /// * `file_path` -- The absolute path of the file being edited.
    ///
    /// # Returns
    ///
    /// The resolved `ConfigValue`, or a `ConfigError` if the key is undefined.
    pub fn get_for_file(&self, key: &str, file_path: &Path) -> Result<ConfigValue, ConfigError> {
        // Only editor-scoped keys consult EditorConfig (Task 17.5)
        if is_editor_key(key) {
            let ec_props = self.resolve_editorconfig(file_path);
            if let Some(value) = editorconfig_value_for_key(key, &ec_props) {
                return Ok(value);
            }
        }
        // Fall back to normal layered resolution
        self.get(key)
    }

    /// Get a string value for a specific file, applying EditorConfig precedence.
    ///
    /// Behaves like `get_string`, but for editor-scoped keys, EditorConfig
    /// values take priority over all configuration layers.
    pub fn get_string_for_file(&self, key: &str, file_path: &Path) -> Result<String, ConfigError> {
        if is_editor_key(key) {
            let ec_props = self.resolve_editorconfig(file_path);
            if let Some(ConfigValue::String(s)) = editorconfig_value_for_key(key, &ec_props) {
                return Ok(s);
            }
        }
        self.get_string(key)
    }

    /// Get an integer value for a specific file, applying EditorConfig precedence.
    ///
    /// Behaves like `get_int`, but for editor-scoped keys, EditorConfig
    /// values take priority over all configuration layers.
    pub fn get_int_for_file(&self, key: &str, file_path: &Path) -> Result<i64, ConfigError> {
        if is_editor_key(key) {
            let ec_props = self.resolve_editorconfig(file_path);
            if let Some(ConfigValue::Integer(i)) = editorconfig_value_for_key(key, &ec_props) {
                return Ok(i);
            }
        }
        self.get_int(key)
    }

    /// Get a boolean value for a specific file, applying EditorConfig precedence.
    ///
    /// Behaves like `get_bool`, but for editor-scoped keys, EditorConfig
    /// values take priority over all configuration layers.
    pub fn get_bool_for_file(&self, key: &str, file_path: &Path) -> Result<bool, ConfigError> {
        if is_editor_key(key) {
            let ec_props = self.resolve_editorconfig(file_path);
            if let Some(ConfigValue::Boolean(b)) = editorconfig_value_for_key(key, &ec_props) {
                return Ok(b);
            }
        }
        self.get_bool(key)
    }
}

/// Check whether a configuration key is in the `editor.*` namespace.
///
/// Only editor-scoped keys are eligible for EditorConfig override.
/// Keys in other namespaces (logging, theme, plugins, vfs, etc.) are
/// never affected by EditorConfig settings.
pub(super) fn is_editor_key(key: &str) -> bool {
    key.starts_with("editor.")
}

/// Map a configuration key to the corresponding EditorConfig property value.
///
/// Returns `Some(ConfigValue)` if the EditorConfig properties have a value
/// for the property that maps to the given key. Returns `None` if the
/// EditorConfig has no value for this key.
pub(super) fn editorconfig_value_for_key(
    key: &str,
    props: &EditorConfigProperties,
) -> Option<ConfigValue> {
    match key {
        "editor.indent_style" => props.indent_style.map(|v| {
            ConfigValue::String(
                match v {
                    IndentStyle::Space => "space",
                    IndentStyle::Tab => "tab",
                }
                .to_string(),
            )
        }),
        "editor.indent_size" | "editor.tab_size" => props.indent_size.map(|v| match v {
            IndentSize::Value(n) => ConfigValue::Integer(i64::from(n)),
            IndentSize::Tab => ConfigValue::String("tab".to_string()),
        }),
        "editor.tab_width" => props.tab_width.map(|v| ConfigValue::Integer(i64::from(v))),
        "editor.end_of_line" | "editor.line_endings" => props.end_of_line.map(|v| {
            ConfigValue::String(
                match v {
                    EndOfLine::Lf => "lf",
                    EndOfLine::CrLf => "crlf",
                    EndOfLine::Cr => "cr",
                }
                .to_string(),
            )
        }),
        "editor.charset" => props.charset.map(|v| {
            ConfigValue::String(
                match v {
                    Charset::Utf8 => "utf-8",
                    Charset::Utf8Bom => "utf-8-bom",
                    Charset::Latin1 => "latin1",
                    Charset::Utf16Be => "utf-16be",
                    Charset::Utf16Le => "utf-16le",
                }
                .to_string(),
            )
        }),
        "editor.trim_trailing_whitespace" => {
            props.trim_trailing_whitespace.map(ConfigValue::Boolean)
        }
        "editor.insert_final_newline" => props.insert_final_newline.map(ConfigValue::Boolean),
        _ => None,
    }
}
