//! Auto-indent mode enum and resolution logic.
//!
//! Defines the three auto-indentation modes (`None`, `Maintain`, `Smart`)
//! and the logic for resolving the effective mode from configuration.

use crate::error::AutoIndentError;

/// The auto-indentation mode for a document.
///
/// Determines how indentation is computed when a new line is created.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum AutoIndentMode {
    /// No automatic indentation applied on Enter.
    /// New lines start at column 0.
    None,
    /// New line matches the indentation of the previous line.
    /// Simple whitespace copy without pattern analysis.
    Maintain,
    /// New line indentation adjusted by language-specific patterns.
    /// Uses increase/decrease/statement patterns for intelligent indentation.
    Smart,
}

impl AutoIndentMode {
    /// Parse from a configuration string value.
    ///
    /// Accepts "none", "maintain", "smart" (case-insensitive).
    ///
    /// # Errors
    ///
    /// Returns `AutoIndentError::InvalidMode` for unrecognised values.
    pub fn from_config_str(s: &str) -> Result<Self, AutoIndentError> {
        match s.trim().to_lowercase().as_str() {
            "none" => Ok(Self::None),
            "maintain" => Ok(Self::Maintain),
            "smart" => Ok(Self::Smart),
            _ => Err(AutoIndentError::InvalidMode {
                value: s.to_string(),
            }),
        }
    }
}

impl Default for AutoIndentMode {
    /// Defaults to `Smart` — language-aware indentation when patterns are available.
    fn default() -> Self {
        Self::Smart
    }
}

impl std::fmt::Display for AutoIndentMode {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::None => write!(f, "none"),
            Self::Maintain => write!(f, "maintain"),
            Self::Smart => write!(f, "smart"),
        }
    }
}

/// Resolve the effective auto-indent mode considering global config,
/// language pattern availability, and per-language override.
///
/// Logic:
/// - If a per-language mode override exists, use it.
/// - If the global mode is Smart but the language has no patterns, fall back to Maintain.
/// - Otherwise, use the global mode as configured.
///
/// # Arguments
///
/// * `global_mode` — The mode set in global configuration (`editor.auto_indent`).
/// * `has_language_patterns` — Whether the active language has indent patterns defined.
/// * `language_mode_override` — Optional per-language mode override from the language TOML.
pub fn resolve_effective_mode(
    global_mode: AutoIndentMode,
    has_language_patterns: bool,
    language_mode_override: Option<AutoIndentMode>,
) -> AutoIndentMode {
    // Per-language override takes highest precedence
    if let Some(override_mode) = language_mode_override {
        return override_mode;
    }

    // Smart mode requires language patterns; fall back to Maintain if none
    if global_mode == AutoIndentMode::Smart && !has_language_patterns {
        return AutoIndentMode::Maintain;
    }

    global_mode
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn default_mode_is_smart() {
        // Validates: Requirement 1.1 — Smart is the default mode
        assert_eq!(AutoIndentMode::default(), AutoIndentMode::Smart);
    }

    #[test]
    fn from_config_str_parses_none() {
        // Validates: Requirement 1.3 — mode configurable via string
        assert_eq!(
            AutoIndentMode::from_config_str("none").unwrap(),
            AutoIndentMode::None
        );
    }

    #[test]
    fn from_config_str_parses_maintain() {
        // Validates: Requirement 1.3
        assert_eq!(
            AutoIndentMode::from_config_str("maintain").unwrap(),
            AutoIndentMode::Maintain
        );
    }

    #[test]
    fn from_config_str_parses_smart() {
        // Validates: Requirement 1.3
        assert_eq!(
            AutoIndentMode::from_config_str("smart").unwrap(),
            AutoIndentMode::Smart
        );
    }

    #[test]
    fn from_config_str_is_case_insensitive() {
        // Validates: Requirement 1.3
        assert_eq!(
            AutoIndentMode::from_config_str("SMART").unwrap(),
            AutoIndentMode::Smart
        );
        assert_eq!(
            AutoIndentMode::from_config_str("Maintain").unwrap(),
            AutoIndentMode::Maintain
        );
        assert_eq!(
            AutoIndentMode::from_config_str("NONE").unwrap(),
            AutoIndentMode::None
        );
    }

    #[test]
    fn from_config_str_trims_whitespace() {
        // Validates: Requirement 1.3
        assert_eq!(
            AutoIndentMode::from_config_str("  smart  ").unwrap(),
            AutoIndentMode::Smart
        );
    }

    #[test]
    fn from_config_str_rejects_unknown() {
        // Validates: Requirement 1.3 — unknown mode returns error
        let err = AutoIndentMode::from_config_str("unknown").unwrap_err();
        match err {
            AutoIndentError::InvalidMode { value } => assert_eq!(value, "unknown"),
            _ => panic!("expected InvalidMode error"),
        }
    }

    #[test]
    fn display_formats_correctly() {
        assert_eq!(format!("{}", AutoIndentMode::None), "none");
        assert_eq!(format!("{}", AutoIndentMode::Maintain), "maintain");
        assert_eq!(format!("{}", AutoIndentMode::Smart), "smart");
    }

    #[test]
    fn resolve_effective_mode_smart_when_patterns_available() {
        // Validates: Requirement 1.2 — Smart when language has patterns
        let mode = resolve_effective_mode(AutoIndentMode::Smart, true, None);
        assert_eq!(mode, AutoIndentMode::Smart);
    }

    #[test]
    fn resolve_effective_mode_maintain_when_no_patterns() {
        // Validates: Requirement 1.2 — Maintain when no language patterns
        let mode = resolve_effective_mode(AutoIndentMode::Smart, false, None);
        assert_eq!(mode, AutoIndentMode::Maintain);
    }

    #[test]
    fn resolve_effective_mode_respects_explicit_none() {
        // Validates: Requirement 1.3 — user can explicitly set None
        let mode = resolve_effective_mode(AutoIndentMode::None, true, None);
        assert_eq!(mode, AutoIndentMode::None);
    }

    #[test]
    fn resolve_effective_mode_respects_explicit_maintain() {
        // Validates: Requirement 1.3 — user can explicitly set Maintain
        let mode = resolve_effective_mode(AutoIndentMode::Maintain, true, None);
        assert_eq!(mode, AutoIndentMode::Maintain);
    }

    #[test]
    fn resolve_effective_mode_language_override_takes_precedence() {
        // Validates: Requirement 1.3 — per-language override
        let mode =
            resolve_effective_mode(AutoIndentMode::Smart, true, Some(AutoIndentMode::Maintain));
        assert_eq!(mode, AutoIndentMode::Maintain);
    }

    #[test]
    fn resolve_effective_mode_none_override_respected() {
        // Validates: Requirement 1.3
        let mode = resolve_effective_mode(AutoIndentMode::Smart, true, Some(AutoIndentMode::None));
        assert_eq!(mode, AutoIndentMode::None);
    }
}
