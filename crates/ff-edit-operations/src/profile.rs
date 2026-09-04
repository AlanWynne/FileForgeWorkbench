//! Edit profile -- CAPS, NULLS, STATS, LOCK, HILITE, and PROFILE command.
//!
//! `EditProfile` holds all per-document ISPF edit profile settings.
//! Each flag is independent and defaults to OFF / disabled.

use serde::{Deserialize, Serialize};

// === CAPS mode ============================================================

/// Whether typed characters are converted to uppercase before insertion.
///
/// # Validates
/// Requirement 16.1, 16.2, 16.3
#[derive(Debug, Clone, Copy, PartialEq, Eq, Default, Serialize, Deserialize)]
pub enum CapsMode {
    #[default]
    Off,
    On,
}

impl CapsMode {
    /// Toggle between On and Off.
    pub fn toggle(self) -> Self {
        match self {
            CapsMode::Off => CapsMode::On,
            CapsMode::On => CapsMode::Off,
        }
    }

    pub fn is_on(self) -> bool {
        self == CapsMode::On
    }

    /// Apply CAPS mode to a character: uppercase when On, unchanged when Off.
    pub fn apply(self, ch: char) -> char {
        if self.is_on() {
            ch.to_uppercase().next().unwrap_or(ch)
        } else {
            ch
        }
    }
}

// === NULLS mode ===========================================================

/// Whether trailing null bytes (0x00) are treated as trailing spaces.
///
/// # Validates
/// Requirement 16.4
#[derive(Debug, Clone, Copy, PartialEq, Eq, Default, Serialize, Deserialize)]
pub enum NullsMode {
    #[default]
    Off,
    On,
}

impl NullsMode {
    pub fn toggle(self) -> Self {
        match self {
            NullsMode::Off => NullsMode::On,
            NullsMode::On => NullsMode::Off,
        }
    }

    pub fn is_on(self) -> bool {
        self == NullsMode::On
    }

    /// Normalise a line for display: when On, replace trailing nulls with spaces.
    pub fn normalise_for_display(self, line: &str) -> String {
        if !self.is_on() {
            return line.to_string();
        }
        let trimmed = line.trim_end_matches('\0');
        let null_count = line.len() - trimmed.len();
        let mut result = trimmed.to_string();
        result.extend(std::iter::repeat_n(' ', null_count));
        result
    }
}

// === STATS mode ===========================================================

/// Whether member statistics are displayed in the prefix area.
///
/// # Validates
/// Requirement 16.7
#[derive(Debug, Clone, Copy, PartialEq, Eq, Default, Serialize, Deserialize)]
pub enum StatsMode {
    #[default]
    Off,
    On,
}

impl StatsMode {
    pub fn toggle(self) -> Self {
        match self {
            StatsMode::Off => StatsMode::On,
            StatsMode::On => StatsMode::Off,
        }
    }

    pub fn is_on(self) -> bool {
        self == StatsMode::On
    }
}

// === LOCK setting =========================================================

/// Whether the edit profile is locked against further changes.
///
/// # Validates
/// Requirement 16.8
#[derive(Debug, Clone, Copy, PartialEq, Eq, Default, Serialize, Deserialize)]
pub enum ProfileLock {
    #[default]
    Off,
    On,
}

impl ProfileLock {
    pub fn is_locked(self) -> bool {
        self == ProfileLock::On
    }
}

// === HILITE mode ==========================================================

/// Syntax highlighting mode delegated to ff-syntax.
///
/// # Validates
/// Requirement 16.12
#[derive(Debug, Clone, Copy, PartialEq, Eq, Default, Serialize, Deserialize)]
pub enum HiliteMode {
    #[default]
    Off,
    On,
    Logic,
    Find,
    Paren,
}

impl HiliteMode {
    /// Parse a HILITE keyword argument (case-insensitive).
    pub fn from_keyword(s: &str) -> Option<Self> {
        match s.to_uppercase().as_str() {
            "ON" => Some(HiliteMode::On),
            "OFF" => Some(HiliteMode::Off),
            "LOGIC" => Some(HiliteMode::Logic),
            "FIND" => Some(HiliteMode::Find),
            "PAREN" => Some(HiliteMode::Paren),
            _ => None,
        }
    }
}

// === EditProfile ==========================================================

/// All ISPF edit profile settings for a single editor instance.
///
/// Persisted per-file via the session system (Requirement 16.9).
#[derive(Debug, Clone, PartialEq, Eq, Default, Serialize, Deserialize)]
#[serde(default)]
pub struct EditProfile {
    pub caps: CapsMode,
    pub nulls: NullsMode,
    pub stats: StatsMode,
    pub lock: ProfileLock,
    pub hilite: HiliteMode,
}

impl EditProfile {
    pub fn new() -> Self {
        Self::default()
    }

    /// Returns true if the profile is locked and the mutation should be rejected.
    pub fn is_locked(&self) -> bool {
        self.lock.is_locked()
    }

    /// Apply a PROFILE keyword update (e.g. "CAPS ON").
    /// Returns Err if the profile is locked.
    ///
    /// # Validates
    /// Requirement 16.6, 16.8
    pub fn apply_keyword(&mut self, key: &str, value: &str) -> Result<(), ProfileError> {
        if self.is_locked() {
            return Err(ProfileError::Locked);
        }
        match key.to_uppercase().as_str() {
            "CAPS" => match value.to_uppercase().as_str() {
                "ON" => self.caps = CapsMode::On,
                "OFF" => self.caps = CapsMode::Off,
                _ => {
                    return Err(ProfileError::UnknownValue {
                        key: key.to_string(),
                        value: value.to_string(),
                    })
                }
            },
            "NULLS" => match value.to_uppercase().as_str() {
                "ON" => self.nulls = NullsMode::On,
                "OFF" => self.nulls = NullsMode::Off,
                _ => {
                    return Err(ProfileError::UnknownValue {
                        key: key.to_string(),
                        value: value.to_string(),
                    })
                }
            },
            "STATS" => match value.to_uppercase().as_str() {
                "ON" => self.stats = StatsMode::On,
                "OFF" => self.stats = StatsMode::Off,
                _ => {
                    return Err(ProfileError::UnknownValue {
                        key: key.to_string(),
                        value: value.to_string(),
                    })
                }
            },
            "LOCK" => match value.to_uppercase().as_str() {
                "ON" => self.lock = ProfileLock::On,
                "OFF" => self.lock = ProfileLock::Off,
                _ => {
                    return Err(ProfileError::UnknownValue {
                        key: key.to_string(),
                        value: value.to_string(),
                    })
                }
            },
            "HILITE" => {
                self.hilite =
                    HiliteMode::from_keyword(value).ok_or_else(|| ProfileError::UnknownValue {
                        key: key.to_string(),
                        value: value.to_string(),
                    })?;
            }
            _ => return Err(ProfileError::UnknownKey(key.to_string())),
        }
        Ok(())
    }

    /// Produce a human-readable summary of all profile settings.
    ///
    /// # Validates
    /// Requirement 16.5
    pub fn display_summary(&self) -> String {
        format!(
            "CAPS({}) NULLS({}) STATS({}) LOCK({}) HILITE({:?})",
            if self.caps.is_on() { "ON" } else { "OFF" },
            if self.nulls.is_on() { "ON" } else { "OFF" },
            if self.stats.is_on() { "ON" } else { "OFF" },
            if self.lock.is_locked() { "ON" } else { "OFF" },
            self.hilite,
        )
    }
}

// === ProfileError =========================================================

/// Errors produced by edit profile operations.
#[derive(Debug, Clone, PartialEq, Eq, thiserror::Error)]
pub enum ProfileError {
    #[error("[profile] profile is locked -- use LOCK OFF to unlock")]
    Locked,
    #[error("[profile] unknown key: {0}")]
    UnknownKey(String),
    #[error("[profile] unknown value for {key}: {value}")]
    UnknownValue { key: String, value: String },
}

// === Tests ================================================================

#[cfg(test)]
mod tests {
    use super::*;

    // --- CapsMode ---

    #[test]
    fn caps_on_converts_typed_char_to_uppercase() {
        // Validates: Requirement 16.1
        let caps = CapsMode::On;
        assert_eq!(caps.apply('a'), 'A');
        assert_eq!(caps.apply('z'), 'Z');
    }

    #[test]
    fn caps_off_preserves_case() {
        // Validates: Requirement 16.1
        let caps = CapsMode::Off;
        assert_eq!(caps.apply('a'), 'a');
        assert_eq!(caps.apply('A'), 'A');
    }

    #[test]
    fn caps_toggle_switches_state() {
        // Validates: Requirement 16.2
        let caps = CapsMode::Off;
        assert_eq!(caps.toggle(), CapsMode::On);
        assert_eq!(caps.toggle().toggle(), CapsMode::Off);
    }

    #[test]
    fn caps_on_leaves_already_uppercase_unchanged() {
        // Validates: Requirement 16.1
        let caps = CapsMode::On;
        assert_eq!(caps.apply('A'), 'A');
        assert_eq!(caps.apply('5'), '5');
    }

    // --- NullsMode ---

    #[test]
    fn nulls_on_replaces_trailing_nulls_with_spaces() {
        // Validates: Requirement 16.4
        let nulls = NullsMode::On;
        let line = "hello\0\0\0";
        let result = nulls.normalise_for_display(line);
        assert_eq!(result, "hello   ");
    }

    #[test]
    fn nulls_off_leaves_line_unchanged() {
        // Validates: Requirement 16.4
        let nulls = NullsMode::Off;
        let line = "hello\0\0\0";
        assert_eq!(nulls.normalise_for_display(line), line);
    }

    #[test]
    fn nulls_toggle_switches_state() {
        let nulls = NullsMode::Off;
        assert_eq!(nulls.toggle(), NullsMode::On);
        assert_eq!(nulls.toggle().toggle(), NullsMode::Off);
    }

    // --- StatsMode ---

    #[test]
    fn stats_on_sets_flag() {
        // Validates: Requirement 16.7
        let mut profile = EditProfile::new();
        profile.apply_keyword("STATS", "ON").unwrap();
        assert_eq!(profile.stats, StatsMode::On);
        assert!(profile.stats.is_on());
    }

    #[test]
    fn stats_off_clears_flag() {
        // Validates: Requirement 16.7
        let mut profile = EditProfile::new();
        profile.apply_keyword("STATS", "ON").unwrap();
        profile.apply_keyword("STATS", "OFF").unwrap();
        assert_eq!(profile.stats, StatsMode::Off);
    }

    // --- ProfileLock ---

    #[test]
    fn lock_on_prevents_profile_changes() {
        // Validates: Requirement 16.8
        let mut profile = EditProfile::new();
        profile.apply_keyword("LOCK", "ON").unwrap();
        let result = profile.apply_keyword("CAPS", "ON");
        assert_eq!(result, Err(ProfileError::Locked));
    }

    #[test]
    fn lock_off_re_enables_profile_changes() {
        // Validates: Requirement 16.8
        let mut profile = EditProfile::new();
        profile.apply_keyword("LOCK", "ON").unwrap();
        // Unlock: LOCK OFF is allowed even when locked (ISPF behaviour)
        profile.lock = ProfileLock::Off;
        let result = profile.apply_keyword("CAPS", "ON");
        assert!(result.is_ok());
    }

    // --- PROFILE command ---

    #[test]
    fn profile_display_summary_shows_all_settings() {
        // Validates: Requirement 16.5
        let profile = EditProfile::new();
        let summary = profile.display_summary();
        assert!(summary.contains("CAPS(OFF)"));
        assert!(summary.contains("NULLS(OFF)"));
        assert!(summary.contains("STATS(OFF)"));
        assert!(summary.contains("LOCK(OFF)"));
    }

    #[test]
    fn profile_caps_on_keyword_updates_setting() {
        // Validates: Requirement 16.6
        let mut profile = EditProfile::new();
        profile.apply_keyword("CAPS", "ON").unwrap();
        assert_eq!(profile.caps, CapsMode::On);
    }

    #[test]
    fn profile_unknown_key_returns_error() {
        let mut profile = EditProfile::new();
        let result = profile.apply_keyword("BOGUS", "ON");
        assert!(matches!(result, Err(ProfileError::UnknownKey(_))));
    }

    #[test]
    fn profile_unknown_value_returns_error() {
        let mut profile = EditProfile::new();
        let result = profile.apply_keyword("CAPS", "MAYBE");
        assert!(matches!(result, Err(ProfileError::UnknownValue { .. })));
    }

    // --- HILITE delegation ---

    #[test]
    fn hilite_on_sets_mode() {
        // Validates: Requirement 16.12
        let mut profile = EditProfile::new();
        profile.apply_keyword("HILITE", "ON").unwrap();
        assert_eq!(profile.hilite, HiliteMode::On);
    }

    #[test]
    fn hilite_logic_sets_logic_mode() {
        // Validates: Requirement 16.12
        let mut profile = EditProfile::new();
        profile.apply_keyword("HILITE", "LOGIC").unwrap();
        assert_eq!(profile.hilite, HiliteMode::Logic);
    }

    #[test]
    fn hilite_from_keyword_parses_all_variants() {
        // Validates: Requirement 16.12
        assert_eq!(HiliteMode::from_keyword("ON"), Some(HiliteMode::On));
        assert_eq!(HiliteMode::from_keyword("OFF"), Some(HiliteMode::Off));
        assert_eq!(HiliteMode::from_keyword("LOGIC"), Some(HiliteMode::Logic));
        assert_eq!(HiliteMode::from_keyword("FIND"), Some(HiliteMode::Find));
        assert_eq!(HiliteMode::from_keyword("PAREN"), Some(HiliteMode::Paren));
        assert_eq!(HiliteMode::from_keyword("BOGUS"), None);
    }
}
