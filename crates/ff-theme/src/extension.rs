//! Plugin theme extensions registration and resolution.
//!
//! Plugins can register additional colour tokens with the theme system
//! so their custom UI elements respect the user's theme and participate
//! in mode switching and hot-reload.

use std::collections::HashMap;

use serde::{Deserialize, Serialize};

use crate::colour::ColourRGBA;
use crate::error::ThemeError;
use crate::mode::VisualMode;

/// A plugin-registered set of additional colour tokens.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ThemeExtension {
    /// Plugin identifier (matches plugin-architecture plugin_id).
    pub plugin_id: String,
    /// Registered tokens with per-mode defaults.
    pub tokens: Vec<ExtensionToken>,
}

/// A single extension token with per-mode default colours.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ExtensionToken {
    /// Token name (relative to plugin namespace, e.g., "result_grid_header").
    pub name: String,
    /// Default colour for Dark mode.
    pub dark_default: ColourRGBA,
    /// Default colour for Light mode.
    pub light_default: ColourRGBA,
    /// Default colour for High-Contrast mode.
    pub high_contrast_default: ColourRGBA,
    /// Human-readable description of this token.
    pub description: String,
}

impl ExtensionToken {
    /// Get the default colour for the given visual mode.
    pub fn default_for_mode(&self, mode: VisualMode) -> ColourRGBA {
        match mode {
            VisualMode::Dark => self.dark_default,
            VisualMode::Light => self.light_default,
            VisualMode::HighContrast => self.high_contrast_default,
            // Legacy is a dark-background mode; fall back to the dark default.
            VisualMode::Legacy => self.dark_default,
        }
    }
}

/// Registry of plugin theme extensions.
///
/// Manages registered tokens, resolves them against user-defined overrides
/// in the theme file, and handles mode-aware resolution.
#[derive(Debug, Clone, Default)]
pub struct ExtensionRegistry {
    /// Registered extensions by plugin_id.
    extensions: HashMap<String, ThemeExtension>,
    /// User-defined overrides from TOML: plugin_id → token_name → mode → colour.
    user_overrides: HashMap<String, HashMap<String, HashMap<VisualMode, ColourRGBA>>>,
}

impl ExtensionRegistry {
    /// Create an empty extension registry.
    pub fn new() -> Self {
        Self::default()
    }

    /// Register a plugin's theme extension tokens.
    ///
    /// # Errors
    ///
    /// Returns `ThemeError::ExtensionCollision` if any token name collides
    /// with a core palette token name.
    pub fn register(&mut self, extension: ThemeExtension) -> Result<(), ThemeError> {
        // Check for collisions with core tokens
        for token in &extension.tokens {
            if is_core_token_name(&token.name) {
                return Err(ThemeError::ExtensionCollision {
                    plugin_id: extension.plugin_id.clone(),
                    token: token.name.clone(),
                });
            }
        }

        self.extensions
            .insert(extension.plugin_id.clone(), extension);
        Ok(())
    }

    /// Deregister a plugin's extension tokens.
    ///
    /// Removes the plugin from the active registry but preserves any
    /// user-defined overrides in the theme file data.
    pub fn deregister(&mut self, plugin_id: &str) {
        self.extensions.remove(plugin_id);
    }

    /// Check if a plugin is currently registered.
    pub fn is_registered(&self, plugin_id: &str) -> bool {
        self.extensions.contains_key(plugin_id)
    }

    /// Set user-defined override for a plugin token.
    pub fn set_user_override(
        &mut self,
        plugin_id: &str,
        token_name: &str,
        mode: VisualMode,
        colour: ColourRGBA,
    ) {
        self.user_overrides
            .entry(plugin_id.to_string())
            .or_default()
            .entry(token_name.to_string())
            .or_default()
            .insert(mode, colour);
    }

    /// Resolve a plugin token's colour for the given mode.
    ///
    /// Resolution order: user override → plugin default for mode.
    /// Returns `None` if the plugin is not registered or the token doesn't exist.
    pub fn resolve(
        &self,
        plugin_id: &str,
        token_name: &str,
        mode: VisualMode,
    ) -> Option<ColourRGBA> {
        // Check user override first
        if let Some(colour) = self
            .user_overrides
            .get(plugin_id)
            .and_then(|tokens| tokens.get(token_name))
            .and_then(|modes| modes.get(&mode))
        {
            return Some(*colour);
        }

        // Fall back to plugin default
        let extension = self.extensions.get(plugin_id)?;
        let token = extension.tokens.iter().find(|t| t.name == token_name)?;
        Some(token.default_for_mode(mode))
    }

    /// Get all resolved extension colours for the given mode.
    ///
    /// Returns a map of plugin_id → token_name → colour.
    pub fn resolve_all(&self, mode: VisualMode) -> HashMap<String, HashMap<String, ColourRGBA>> {
        let mut result = HashMap::new();
        for (plugin_id, extension) in &self.extensions {
            let mut tokens = HashMap::new();
            for token in &extension.tokens {
                if let Some(colour) = self.resolve(plugin_id, &token.name, mode) {
                    tokens.insert(token.name.clone(), colour);
                }
            }
            if !tokens.is_empty() {
                result.insert(plugin_id.clone(), tokens);
            }
        }
        result
    }
}

/// Check if a token name collides with core palette token names.
fn is_core_token_name(name: &str) -> bool {
    // Core tokens use dotted paths like "editor.background"
    // Plugin tokens are simple names like "result_grid_header"
    // Collision occurs if the name matches any ColourToken key path segment
    let core_prefixes = [
        "editor",
        "syntax",
        "file_tree",
        "tab_bar",
        "chrome",
        "decorations",
        "indicators",
        "ui",
    ];
    core_prefixes.contains(&name)
}

#[cfg(test)]
mod tests {
    use super::*;

    fn sample_extension() -> ThemeExtension {
        ThemeExtension {
            plugin_id: "sql-viewer".to_string(),
            tokens: vec![ExtensionToken {
                name: "result_grid_header".to_string(),
                dark_default: ColourRGBA::rgb(100, 150, 200),
                light_default: ColourRGBA::rgb(50, 100, 150),
                high_contrast_default: ColourRGBA::rgb(255, 255, 255),
                description: "Header row background in query results".to_string(),
            }],
        }
    }

    #[test]
    fn register_extension_succeeds_for_valid_tokens() {
        // Validates: Requirement 11.1
        let mut registry = ExtensionRegistry::new();
        let result = registry.register(sample_extension());
        assert!(result.is_ok());
        assert!(registry.is_registered("sql-viewer"));
    }

    #[test]
    fn register_extension_rejects_core_collision() {
        // Validates: Requirement 11.6
        let mut registry = ExtensionRegistry::new();
        let extension = ThemeExtension {
            plugin_id: "bad-plugin".to_string(),
            tokens: vec![ExtensionToken {
                name: "editor".to_string(),
                dark_default: ColourRGBA::rgb(0, 0, 0),
                light_default: ColourRGBA::rgb(0, 0, 0),
                high_contrast_default: ColourRGBA::rgb(0, 0, 0),
                description: "Collision test".to_string(),
            }],
        };
        let result = registry.register(extension);
        assert!(result.is_err());
    }

    #[test]
    fn resolve_returns_plugin_default_for_mode() {
        // Validates: Requirement 11.4
        let mut registry = ExtensionRegistry::new();
        registry.register(sample_extension()).unwrap();
        let colour = registry
            .resolve("sql-viewer", "result_grid_header", VisualMode::Dark)
            .unwrap();
        assert_eq!(colour, ColourRGBA::rgb(100, 150, 200));

        let colour = registry
            .resolve("sql-viewer", "result_grid_header", VisualMode::Light)
            .unwrap();
        assert_eq!(colour, ColourRGBA::rgb(50, 100, 150));
    }

    #[test]
    fn resolve_prefers_user_override() {
        // Validates: Requirement 11.3
        let mut registry = ExtensionRegistry::new();
        registry.register(sample_extension()).unwrap();
        registry.set_user_override(
            "sql-viewer",
            "result_grid_header",
            VisualMode::Dark,
            ColourRGBA::rgb(255, 0, 0),
        );
        let colour = registry
            .resolve("sql-viewer", "result_grid_header", VisualMode::Dark)
            .unwrap();
        assert_eq!(colour, ColourRGBA::rgb(255, 0, 0));
    }

    #[test]
    fn deregister_removes_plugin() {
        // Validates: Requirement 11.5
        let mut registry = ExtensionRegistry::new();
        registry.register(sample_extension()).unwrap();
        registry.deregister("sql-viewer");
        assert!(!registry.is_registered("sql-viewer"));
        assert_eq!(
            registry.resolve("sql-viewer", "result_grid_header", VisualMode::Dark),
            None
        );
    }
}
