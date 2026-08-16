//! Configuration model for the Lua macro engine.
//!
//! Reads settings from the `macro.*` namespace in the configuration system.
//! Addresses: Requirement 1 AC 3, AC 4; Requirement 7 AC 1; Requirement 8 AC 1

use std::path::PathBuf;

use crate::security::SecurityMode;

/// Default instruction limit (10 million instructions).
pub const DEFAULT_INSTRUCTION_LIMIT: u64 = 10_000_000;

/// Default memory limit (64 MB).
pub const DEFAULT_MEMORY_LIMIT: usize = 67_108_864;

/// Configuration values for the macro engine, read from ff-config.
///
/// Addresses: Requirement 1 AC 3, AC 4; Requirement 7 AC 1; Requirement 8 AC 1
#[derive(Debug, Clone)]
pub struct EngineConfig {
    /// Maximum instruction count per invocation (default: 10_000_000).
    pub instruction_limit: u64,
    /// Maximum memory in bytes per invocation (default: 67_108_864 = 64 MB).
    pub memory_limit: usize,
    /// Security mode.
    pub security_mode: SecurityMode,
    /// Macro directory paths with priority ordering.
    pub macro_directories: Vec<PathBuf>,
    /// Whether auto-reload is enabled.
    pub auto_reload: bool,
    /// Whether debug tracebacks are enabled.
    pub debug_traceback: bool,
    /// Startup script path (optional).
    pub startup_script: Option<String>,
    /// Trusted script paths for TrustedOnly mode.
    pub trusted_paths: Vec<PathBuf>,
    /// Per-extension auto-load mappings (extension → script name).
    pub auto_load_for: Vec<(String, String)>,
}

impl Default for EngineConfig {
    fn default() -> Self {
        Self {
            instruction_limit: DEFAULT_INSTRUCTION_LIMIT,
            memory_limit: DEFAULT_MEMORY_LIMIT,
            security_mode: SecurityMode::default(),
            macro_directories: Vec::new(),
            auto_reload: true,
            debug_traceback: false,
            startup_script: None,
            trusted_paths: Vec::new(),
            auto_load_for: Vec::new(),
        }
    }
}

impl EngineConfig {
    /// Creates a new configuration with default values.
    pub fn new() -> Self {
        Self::default()
    }

    /// Creates a configuration suitable for testing (Enabled mode, no limits that would interfere).
    pub fn for_testing() -> Self {
        Self {
            security_mode: SecurityMode::Enabled,
            instruction_limit: DEFAULT_INSTRUCTION_LIMIT,
            memory_limit: DEFAULT_MEMORY_LIMIT,
            auto_reload: false,
            debug_traceback: true,
            ..Self::default()
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    // Validates: Requirement 1.3
    #[test]
    fn default_instruction_limit_is_ten_million() {
        let config = EngineConfig::default();
        assert_eq!(config.instruction_limit, 10_000_000);
    }

    // Validates: Requirement 1.4
    #[test]
    fn default_memory_limit_is_64mb() {
        let config = EngineConfig::default();
        assert_eq!(config.memory_limit, 67_108_864);
    }

    // Validates: Requirement 7.7
    #[test]
    fn default_security_mode_is_prompt() {
        let config = EngineConfig::default();
        assert_eq!(config.security_mode, SecurityMode::Prompt);
    }

    // Validates: Requirement 8.1
    #[test]
    fn default_auto_reload_is_true() {
        let config = EngineConfig::default();
        assert!(config.auto_reload);
    }

    #[test]
    fn testing_config_uses_enabled_mode() {
        let config = EngineConfig::for_testing();
        assert_eq!(config.security_mode, SecurityMode::Enabled);
        assert!(config.debug_traceback);
        assert!(!config.auto_reload);
    }
}
