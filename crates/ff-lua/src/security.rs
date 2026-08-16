//! Security mode enforcement for macro execution.
//!
//! Controls which macros may execute based on configuration and trust levels.
//! Addresses: Requirement 7 (all criteria)

use std::path::{Path, PathBuf};

/// Security mode controlling which macros may execute.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum SecurityMode {
    /// No macros may execute.
    Disabled,
    /// Prompt user before executing untrusted macros.
    Prompt,
    /// Only macros in trusted paths may execute.
    TrustedOnly,
    /// All macros execute without restriction.
    Enabled,
}

impl Default for SecurityMode {
    /// Defaults to Prompt for new installations.
    ///
    /// Addresses: Requirement 7 AC 7
    fn default() -> Self {
        SecurityMode::Prompt
    }
}

impl std::fmt::Display for SecurityMode {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            SecurityMode::Disabled => write!(f, "Disabled"),
            SecurityMode::Prompt => write!(f, "Prompt"),
            SecurityMode::TrustedOnly => write!(f, "TrustedOnly"),
            SecurityMode::Enabled => write!(f, "Enabled"),
        }
    }
}

/// The result of a security check.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum SecurityPermission {
    /// Execution allowed.
    Allowed,
    /// User must be prompted for permission.
    NeedsPrompt,
    /// Execution denied with reason.
    Denied {
        /// Human-readable reason for denial.
        reason: String,
    },
}

/// User's response to a security prompt.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum SecurityDecision {
    /// Allow this one execution.
    AllowOnce,
    /// Add to trusted list permanently.
    AlwaysTrust,
    /// Deny execution.
    Deny,
}

/// Set of Lua standard libraries to load based on security mode.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct StdlibSet {
    /// Always loaded.
    pub base: bool,
    /// Always loaded.
    pub string: bool,
    /// Always loaded.
    pub table: bool,
    /// Always loaded.
    pub math: bool,
    /// Always loaded.
    pub utf8: bool,
    /// Always loaded.
    pub coroutine: bool,
    /// Only when Enabled.
    pub io: bool,
    /// Only when Enabled.
    pub os: bool,
    /// Only when Enabled.
    pub debug: bool,
}

impl StdlibSet {
    /// Returns the standard safe set (no io/os/debug).
    pub fn safe() -> Self {
        Self {
            base: true,
            string: true,
            table: true,
            math: true,
            utf8: true,
            coroutine: true,
            io: false,
            os: false,
            debug: false,
        }
    }

    /// Returns the full set (all libraries).
    pub fn full() -> Self {
        Self {
            base: true,
            string: true,
            table: true,
            math: true,
            utf8: true,
            coroutine: true,
            io: true,
            os: true,
            debug: true,
        }
    }
}

/// The security gate that enforces execution policy.
///
/// Addresses: Requirement 7 (all criteria)
#[derive(Debug, Clone)]
pub struct SecurityGate {
    /// Current security mode (read from configuration).
    mode: SecurityMode,
    /// List of trusted script paths.
    trusted_paths: Vec<PathBuf>,
    /// User-level macro directories (always trusted in TrustedOnly mode).
    user_directories: Vec<PathBuf>,
}

impl SecurityGate {
    /// Creates a new security gate with the given configuration.
    pub fn new(
        mode: SecurityMode,
        trusted_paths: Vec<PathBuf>,
        user_directories: Vec<PathBuf>,
    ) -> Self {
        Self {
            mode,
            trusted_paths,
            user_directories,
        }
    }

    /// Returns the current security mode.
    pub fn mode(&self) -> SecurityMode {
        self.mode
    }

    /// Check whether a script is allowed to execute under current policy.
    ///
    /// Addresses: Requirement 7 (all criteria)
    pub fn check_permission(&self, script_path: &Path) -> SecurityPermission {
        match self.mode {
            SecurityMode::Disabled => SecurityPermission::Denied {
                reason: "Macro execution is disabled by security policy.".to_string(),
            },
            SecurityMode::Enabled => SecurityPermission::Allowed,
            SecurityMode::TrustedOnly => {
                if self.is_trusted(script_path) {
                    SecurityPermission::Allowed
                } else {
                    SecurityPermission::Denied {
                        reason: format!(
                            "Script '{}' is not in trusted paths.",
                            script_path.display()
                        ),
                    }
                }
            }
            SecurityMode::Prompt => {
                if self.is_trusted(script_path) {
                    SecurityPermission::Allowed
                } else {
                    SecurityPermission::NeedsPrompt
                }
            }
        }
    }

    /// Update the security mode.
    pub fn set_mode(&mut self, mode: SecurityMode) {
        self.mode = mode;
    }

    /// Add a path to the trusted list.
    pub fn add_trusted_path(&mut self, path: PathBuf) {
        if !self.trusted_paths.contains(&path) {
            self.trusted_paths.push(path);
        }
    }

    /// Filter Lua standard libraries based on security mode.
    ///
    /// Returns which stdlib modules should be loaded.
    ///
    /// Addresses: Requirement 1 AC 2
    pub fn allowed_stdlibs(&self) -> StdlibSet {
        match self.mode {
            SecurityMode::Enabled => StdlibSet::full(),
            _ => StdlibSet::safe(),
        }
    }

    /// Returns whether a script path is in the trusted set.
    fn is_trusted(&self, script_path: &Path) -> bool {
        // Check explicit trusted paths
        for trusted in &self.trusted_paths {
            if script_path.starts_with(trusted) || script_path == trusted {
                return true;
            }
        }
        // Check user directories (always trusted in TrustedOnly/Prompt modes)
        for dir in &self.user_directories {
            if script_path.starts_with(dir) {
                return true;
            }
        }
        false
    }

    /// Returns the list of restricted functions that should be removed
    /// for non-trusted scripts regardless of security mode.
    ///
    /// Addresses: Requirement 7 AC 6
    pub fn restricted_functions() -> &'static [&'static str] {
        &["os.execute", "io.popen", "loadfile", "dofile"]
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    // Validates: Requirement 7.2
    #[test]
    fn disabled_mode_denies_all_scripts() {
        let gate = SecurityGate::new(SecurityMode::Disabled, vec![], vec![]);
        let result = gate.check_permission(Path::new("/any/script.lua"));
        assert_eq!(
            result,
            SecurityPermission::Denied {
                reason: "Macro execution is disabled by security policy.".to_string()
            }
        );
    }

    // Validates: Requirement 7.5
    #[test]
    fn enabled_mode_allows_all_scripts() {
        let gate = SecurityGate::new(SecurityMode::Enabled, vec![], vec![]);
        let result = gate.check_permission(Path::new("/any/script.lua"));
        assert_eq!(result, SecurityPermission::Allowed);
    }

    // Validates: Requirement 7.4
    #[test]
    fn trusted_only_mode_allows_trusted_scripts() {
        let gate = SecurityGate::new(
            SecurityMode::TrustedOnly,
            vec![PathBuf::from("/trusted/macros")],
            vec![],
        );
        let result = gate.check_permission(Path::new("/trusted/macros/script.lua"));
        assert_eq!(result, SecurityPermission::Allowed);
    }

    // Validates: Requirement 7.4
    #[test]
    fn trusted_only_mode_denies_untrusted_scripts() {
        let gate = SecurityGate::new(
            SecurityMode::TrustedOnly,
            vec![PathBuf::from("/trusted/macros")],
            vec![],
        );
        let result = gate.check_permission(Path::new("/untrusted/script.lua"));
        assert!(matches!(result, SecurityPermission::Denied { .. }));
    }

    // Validates: Requirement 7.3
    #[test]
    fn prompt_mode_returns_needs_prompt_for_untrusted() {
        let gate = SecurityGate::new(SecurityMode::Prompt, vec![], vec![]);
        let result = gate.check_permission(Path::new("/some/script.lua"));
        assert_eq!(result, SecurityPermission::NeedsPrompt);
    }

    // Validates: Requirement 7.3
    #[test]
    fn prompt_mode_allows_trusted_scripts_without_prompt() {
        let gate = SecurityGate::new(
            SecurityMode::Prompt,
            vec![PathBuf::from("/trusted")],
            vec![],
        );
        let result = gate.check_permission(Path::new("/trusted/script.lua"));
        assert_eq!(result, SecurityPermission::Allowed);
    }

    // Validates: Requirement 7.4
    #[test]
    fn trusted_only_mode_allows_user_directory_scripts() {
        let gate = SecurityGate::new(
            SecurityMode::TrustedOnly,
            vec![],
            vec![PathBuf::from("/home/user/.config/ffworkbench/macros")],
        );
        let result = gate.check_permission(Path::new(
            "/home/user/.config/ffworkbench/macros/myscript.lua",
        ));
        assert_eq!(result, SecurityPermission::Allowed);
    }

    // Validates: Requirement 7.7
    #[test]
    fn default_security_mode_is_prompt() {
        assert_eq!(SecurityMode::default(), SecurityMode::Prompt);
    }

    // Validates: Requirement 1.2
    #[test]
    fn allowed_stdlibs_safe_when_not_enabled() {
        let gate = SecurityGate::new(SecurityMode::Prompt, vec![], vec![]);
        let libs = gate.allowed_stdlibs();
        assert!(libs.base);
        assert!(libs.string);
        assert!(libs.table);
        assert!(libs.math);
        assert!(libs.utf8);
        assert!(libs.coroutine);
        assert!(!libs.io);
        assert!(!libs.os);
        assert!(!libs.debug);
    }

    // Validates: Requirement 1.2
    #[test]
    fn allowed_stdlibs_full_when_enabled() {
        let gate = SecurityGate::new(SecurityMode::Enabled, vec![], vec![]);
        let libs = gate.allowed_stdlibs();
        assert!(libs.io);
        assert!(libs.os);
        assert!(libs.debug);
    }

    // Validates: Requirement 7.6
    #[test]
    fn restricted_functions_include_dangerous_apis() {
        let restricted = SecurityGate::restricted_functions();
        assert!(restricted.contains(&"os.execute"));
        assert!(restricted.contains(&"io.popen"));
        assert!(restricted.contains(&"loadfile"));
        assert!(restricted.contains(&"dofile"));
    }
}
