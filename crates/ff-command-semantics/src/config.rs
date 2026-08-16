//! Runtime configuration for the command semantics engine.
//!
//! Provides configurable behaviours for find scope, bounds handling,
//! case sensitivity, shift width, tag clearing, and invalid line command policy.
//! All keys are namespaced under `[commands]` in the TOML configuration file.

use crate::scope::ScopeFilter;

/// Maximum allowed shift width.
const MAX_SHIFT_WIDTH: u32 = 72;
/// Minimum allowed shift width.
const MIN_SHIFT_WIDTH: u32 = 1;

/// Policy for handling unrecognised line commands.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum InvalidLineCommandPolicy {
    /// Produce an error and abort the pipeline.
    Reject,
    /// Silently discard and continue.
    Ignore,
}

/// Runtime configuration for the command semantics engine.
///
/// # Configuration Keys
///
/// All keys are under the `[commands]` namespace:
/// - `commands.find_default_scope` — "visible" | "all" | "excluded"
/// - `commands.bounds_affect_find` — boolean
/// - `commands.case_sensitive_find` — boolean
/// - `commands.default_shift_width` — integer 1–72
/// - `commands.reset_clears_tags` — boolean
/// - `commands.invalid_line_command_policy` — "reject" | "ignore"
#[derive(Debug, Clone, PartialEq)]
pub struct CommandConfig {
    /// Default scope for FIND/CHANGE when no explicit scope given.
    pub find_default_scope: ScopeFilter,

    /// Whether column bounds restrict FIND/CHANGE search area.
    pub bounds_affect_find: bool,

    /// Whether FIND/CHANGE defaults to case-sensitive matching.
    pub case_sensitive_find: bool,

    /// Number of columns for > and < shift commands (1–72).
    pub default_shift_width: u32,

    /// Whether RESET clears line tags in addition to exclusion state.
    pub reset_clears_tags: bool,

    /// How unrecognised line commands are handled.
    pub invalid_line_command_policy: InvalidLineCommandPolicy,
}

impl Default for CommandConfig {
    fn default() -> Self {
        Self {
            find_default_scope: ScopeFilter::Visible,
            bounds_affect_find: true,
            case_sensitive_find: false,
            default_shift_width: 2,
            reset_clears_tags: false,
            invalid_line_command_policy: InvalidLineCommandPolicy::Reject,
        }
    }
}

impl CommandConfig {
    /// Validate and clamp shift_width to [1, 72].
    ///
    /// Returns the clamped value. Caller should log a WARN if clamping occurred.
    pub fn clamp_shift_width(value: i64) -> u32 {
        if value < MIN_SHIFT_WIDTH as i64 {
            MIN_SHIFT_WIDTH
        } else if value > MAX_SHIFT_WIDTH as i64 {
            MAX_SHIFT_WIDTH
        } else {
            value as u32
        }
    }

    /// Parse a find_default_scope string value, returning None for invalid values.
    pub fn parse_find_default_scope(value: &str) -> Option<ScopeFilter> {
        match value.to_lowercase().as_str() {
            "visible" => Some(ScopeFilter::Visible),
            "all" => Some(ScopeFilter::All),
            "excluded" => Some(ScopeFilter::Excluded),
            _ => None,
        }
    }

    /// Parse an invalid_line_command_policy string value, returning None for invalid values.
    pub fn parse_invalid_line_command_policy(value: &str) -> Option<InvalidLineCommandPolicy> {
        match value.to_lowercase().as_str() {
            "reject" => Some(InvalidLineCommandPolicy::Reject),
            "ignore" => Some(InvalidLineCommandPolicy::Ignore),
            _ => None,
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    // Validates: Requirement 6.1
    #[test]
    fn default_config_has_correct_defaults() {
        let config = CommandConfig::default();
        assert_eq!(config.find_default_scope, ScopeFilter::Visible);
        assert!(config.bounds_affect_find);
        assert!(!config.case_sensitive_find);
        assert_eq!(config.default_shift_width, 2);
        assert!(!config.reset_clears_tags);
        assert_eq!(
            config.invalid_line_command_policy,
            InvalidLineCommandPolicy::Reject
        );
    }

    // Validates: Requirement 6.6
    #[test]
    fn clamp_shift_width_within_range_unchanged() {
        assert_eq!(CommandConfig::clamp_shift_width(1), 1);
        assert_eq!(CommandConfig::clamp_shift_width(2), 2);
        assert_eq!(CommandConfig::clamp_shift_width(36), 36);
        assert_eq!(CommandConfig::clamp_shift_width(72), 72);
    }

    // Validates: Requirement 6.6
    #[test]
    fn clamp_shift_width_below_min_clamps_to_1() {
        assert_eq!(CommandConfig::clamp_shift_width(0), 1);
        assert_eq!(CommandConfig::clamp_shift_width(-10), 1);
        assert_eq!(CommandConfig::clamp_shift_width(-1000), 1);
    }

    // Validates: Requirement 6.6
    #[test]
    fn clamp_shift_width_above_max_clamps_to_72() {
        assert_eq!(CommandConfig::clamp_shift_width(73), 72);
        assert_eq!(CommandConfig::clamp_shift_width(100), 72);
        assert_eq!(CommandConfig::clamp_shift_width(1000), 72);
    }

    // Validates: Requirement 6.1
    #[test]
    fn parse_find_default_scope_valid_values() {
        assert_eq!(
            CommandConfig::parse_find_default_scope("visible"),
            Some(ScopeFilter::Visible)
        );
        assert_eq!(
            CommandConfig::parse_find_default_scope("all"),
            Some(ScopeFilter::All)
        );
        assert_eq!(
            CommandConfig::parse_find_default_scope("excluded"),
            Some(ScopeFilter::Excluded)
        );
        assert_eq!(
            CommandConfig::parse_find_default_scope("VISIBLE"),
            Some(ScopeFilter::Visible)
        );
    }

    // Validates: Requirement 6.2
    #[test]
    fn parse_find_default_scope_invalid_returns_none() {
        assert_eq!(CommandConfig::parse_find_default_scope("invalid"), None);
        assert_eq!(CommandConfig::parse_find_default_scope(""), None);
    }

    // Validates: Requirement 6.1
    #[test]
    fn parse_invalid_line_command_policy_valid_values() {
        assert_eq!(
            CommandConfig::parse_invalid_line_command_policy("reject"),
            Some(InvalidLineCommandPolicy::Reject)
        );
        assert_eq!(
            CommandConfig::parse_invalid_line_command_policy("ignore"),
            Some(InvalidLineCommandPolicy::Ignore)
        );
        assert_eq!(
            CommandConfig::parse_invalid_line_command_policy("REJECT"),
            Some(InvalidLineCommandPolicy::Reject)
        );
    }

    // Validates: Requirement 6.2
    #[test]
    fn parse_invalid_line_command_policy_invalid_returns_none() {
        assert_eq!(
            CommandConfig::parse_invalid_line_command_policy("warn"),
            None
        );
        assert_eq!(CommandConfig::parse_invalid_line_command_policy(""), None);
    }
}
