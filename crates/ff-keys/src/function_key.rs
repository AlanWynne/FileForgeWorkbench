//! Function key enumeration and parsing.
//!
//! Defines the `FunctionKey` enum (F1–F24), the `KeyModifier` enum (None/Shift/Ctrl/Alt),
//! and the `ModifiedKey` struct combining both into one of 96 addressable key slots.

use std::fmt;
use std::str::FromStr;

use serde::{Deserialize, Serialize};

use crate::error::KeysError;

/// Represents a function key in the F1–F24 range.
///
/// F1 is reserved (context-help) but included for completeness in the enum.
/// Only F2–F24 are user-assignable.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, PartialOrd, Ord, Serialize, Deserialize)]
pub enum FunctionKey {
    /// F1 — reserved for context help.
    F1,
    F2,
    F3,
    F4,
    F5,
    F6,
    F7,
    F8,
    F9,
    F10,
    F11,
    F12,
    F13,
    F14,
    F15,
    F16,
    F17,
    F18,
    F19,
    F20,
    F21,
    F22,
    F23,
    F24,
}

impl FunctionKey {
    /// All function keys in order F1–F24.
    pub const ALL: [FunctionKey; 24] = [
        FunctionKey::F1,
        FunctionKey::F2,
        FunctionKey::F3,
        FunctionKey::F4,
        FunctionKey::F5,
        FunctionKey::F6,
        FunctionKey::F7,
        FunctionKey::F8,
        FunctionKey::F9,
        FunctionKey::F10,
        FunctionKey::F11,
        FunctionKey::F12,
        FunctionKey::F13,
        FunctionKey::F14,
        FunctionKey::F15,
        FunctionKey::F16,
        FunctionKey::F17,
        FunctionKey::F18,
        FunctionKey::F19,
        FunctionKey::F20,
        FunctionKey::F21,
        FunctionKey::F22,
        FunctionKey::F23,
        FunctionKey::F24,
    ];

    /// The minimum assignable function key (F2, since F1 is reserved for Help).
    pub const MIN_ASSIGNABLE: FunctionKey = FunctionKey::F2;

    /// The maximum function key.
    pub const MAX: FunctionKey = FunctionKey::F24;

    /// Parse a function key from a string like "F3", "F12", "f24".
    ///
    /// Accepts case-insensitive input. Returns `None` for out-of-range values
    /// or unparseable strings.
    pub fn parse(s: &str) -> Option<Self> {
        let s = s.trim();
        let upper = s.to_ascii_uppercase();
        if !upper.starts_with('F') {
            return None;
        }
        let num_str = &upper[1..];
        let num: u8 = num_str.parse().ok()?;
        Self::from_number(num)
    }

    /// Create a `FunctionKey` from its numeric value (1–24).
    pub fn from_number(n: u8) -> Option<Self> {
        match n {
            1 => Some(FunctionKey::F1),
            2 => Some(FunctionKey::F2),
            3 => Some(FunctionKey::F3),
            4 => Some(FunctionKey::F4),
            5 => Some(FunctionKey::F5),
            6 => Some(FunctionKey::F6),
            7 => Some(FunctionKey::F7),
            8 => Some(FunctionKey::F8),
            9 => Some(FunctionKey::F9),
            10 => Some(FunctionKey::F10),
            11 => Some(FunctionKey::F11),
            12 => Some(FunctionKey::F12),
            13 => Some(FunctionKey::F13),
            14 => Some(FunctionKey::F14),
            15 => Some(FunctionKey::F15),
            16 => Some(FunctionKey::F16),
            17 => Some(FunctionKey::F17),
            18 => Some(FunctionKey::F18),
            19 => Some(FunctionKey::F19),
            20 => Some(FunctionKey::F20),
            21 => Some(FunctionKey::F21),
            22 => Some(FunctionKey::F22),
            23 => Some(FunctionKey::F23),
            24 => Some(FunctionKey::F24),
            _ => None,
        }
    }

    /// The numeric value of this function key (F1=1, F2=2, ..., F24=24).
    pub fn number(&self) -> u8 {
        match self {
            FunctionKey::F1 => 1,
            FunctionKey::F2 => 2,
            FunctionKey::F3 => 3,
            FunctionKey::F4 => 4,
            FunctionKey::F5 => 5,
            FunctionKey::F6 => 6,
            FunctionKey::F7 => 7,
            FunctionKey::F8 => 8,
            FunctionKey::F9 => 9,
            FunctionKey::F10 => 10,
            FunctionKey::F11 => 11,
            FunctionKey::F12 => 12,
            FunctionKey::F13 => 13,
            FunctionKey::F14 => 14,
            FunctionKey::F15 => 15,
            FunctionKey::F16 => 16,
            FunctionKey::F17 => 17,
            FunctionKey::F18 => 18,
            FunctionKey::F19 => 19,
            FunctionKey::F20 => 20,
            FunctionKey::F21 => 21,
            FunctionKey::F22 => 22,
            FunctionKey::F23 => 23,
            FunctionKey::F24 => 24,
        }
    }

    /// Whether this key is in the assignable range (F2–F24).
    /// F1 is reserved for context-help.
    pub fn is_assignable(&self) -> bool {
        self.number() >= 2
    }

    /// The display name (e.g., "F3", "F12").
    pub fn display_name(&self) -> &'static str {
        match self {
            FunctionKey::F1 => "F1",
            FunctionKey::F2 => "F2",
            FunctionKey::F3 => "F3",
            FunctionKey::F4 => "F4",
            FunctionKey::F5 => "F5",
            FunctionKey::F6 => "F6",
            FunctionKey::F7 => "F7",
            FunctionKey::F8 => "F8",
            FunctionKey::F9 => "F9",
            FunctionKey::F10 => "F10",
            FunctionKey::F11 => "F11",
            FunctionKey::F12 => "F12",
            FunctionKey::F13 => "F13",
            FunctionKey::F14 => "F14",
            FunctionKey::F15 => "F15",
            FunctionKey::F16 => "F16",
            FunctionKey::F17 => "F17",
            FunctionKey::F18 => "F18",
            FunctionKey::F19 => "F19",
            FunctionKey::F20 => "F20",
            FunctionKey::F21 => "F21",
            FunctionKey::F22 => "F22",
            FunctionKey::F23 => "F23",
            FunctionKey::F24 => "F24",
        }
    }
}

impl fmt::Display for FunctionKey {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.write_str(self.display_name())
    }
}

impl FromStr for FunctionKey {
    type Err = KeysError;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        FunctionKey::parse(s).ok_or_else(|| KeysError::InvalidFunctionKey { key: s.to_string() })
    }
}

// ── KeyModifier ──────────────────────────────────────────────────────────────

/// The modifier applied to a function key.
///
/// Represents the four modifier variants for each of F1–F24,
/// giving 96 addressable key slots per key map.
///
/// Validates: Requirement 20.9, 20.12
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, PartialOrd, Ord, Serialize, Deserialize)]
pub enum KeyModifier {
    /// Plain function key (no modifier).
    None,
    /// Shift + function key.
    Shift,
    /// Ctrl + function key.
    Ctrl,
    /// Alt + function key.
    Alt,
}

impl KeyModifier {
    /// All four modifier variants in canonical order.
    pub const ALL: [KeyModifier; 4] = [
        KeyModifier::None,
        KeyModifier::Shift,
        KeyModifier::Ctrl,
        KeyModifier::Alt,
    ];

    /// The TOML key name prefix for this modifier.
    ///
    /// - `None`  → `"F"`  (e.g., `F3`)
    /// - `Shift` → `"SF"` (e.g., `SF3`)
    /// - `Ctrl`  → `"CF"` (e.g., `CF3`)
    /// - `Alt`   → `"AF"` (e.g., `AF3`)
    pub fn toml_prefix(self) -> &'static str {
        match self {
            KeyModifier::None => "F",
            KeyModifier::Shift => "SF",
            KeyModifier::Ctrl => "CF",
            KeyModifier::Alt => "AF",
        }
    }
}

impl fmt::Display for KeyModifier {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            KeyModifier::None => write!(f, ""),
            KeyModifier::Shift => write!(f, "Shift+"),
            KeyModifier::Ctrl => write!(f, "Ctrl+"),
            KeyModifier::Alt => write!(f, "Alt+"),
        }
    }
}

// ── ModifiedKey ──────────────────────────────────────────────────────────────

/// A function key combined with an optional modifier.
///
/// Represents one of 96 addressable key slots (4 modifiers × 24 keys).
/// Used as the key type in `KeyMap`.
///
/// TOML key name syntax:
/// - Plain:  `F1`–`F24`
/// - Shift:  `SF1`–`SF24`
/// - Ctrl:   `CF1`–`CF24`
/// - Alt:    `AF1`–`AF24`
///
/// Validates: Requirement 20.11, 20.12
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, PartialOrd, Ord, Serialize, Deserialize)]
pub struct ModifiedKey {
    /// The base function key.
    pub key: FunctionKey,
    /// The modifier applied.
    pub modifier: KeyModifier,
}

impl ModifiedKey {
    /// Create a plain (unmodified) `ModifiedKey`.
    pub fn plain(key: FunctionKey) -> Self {
        Self {
            key,
            modifier: KeyModifier::None,
        }
    }

    /// Create a Shift+Fn `ModifiedKey`.
    pub fn shift(key: FunctionKey) -> Self {
        Self {
            key,
            modifier: KeyModifier::Shift,
        }
    }

    /// Create a Ctrl+Fn `ModifiedKey`.
    pub fn ctrl(key: FunctionKey) -> Self {
        Self {
            key,
            modifier: KeyModifier::Ctrl,
        }
    }

    /// Create an Alt+Fn `ModifiedKey`.
    pub fn alt(key: FunctionKey) -> Self {
        Self {
            key,
            modifier: KeyModifier::Alt,
        }
    }

    /// All 96 `ModifiedKey` values in canonical order (None×24, Shift×24, Ctrl×24, Alt×24).
    ///
    /// Validates: Requirement 20.12
    pub const ALL: [ModifiedKey; 96] = {
        let mut arr = [ModifiedKey {
            key: FunctionKey::F1,
            modifier: KeyModifier::None,
        }; 96];
        let modifiers = [
            KeyModifier::None,
            KeyModifier::Shift,
            KeyModifier::Ctrl,
            KeyModifier::Alt,
        ];
        let keys = FunctionKey::ALL;
        let mut m = 0usize;
        while m < 4 {
            let mut k = 0usize;
            while k < 24 {
                arr[m * 24 + k] = ModifiedKey {
                    key: keys[k],
                    modifier: modifiers[m],
                };
                k += 1;
            }
            m += 1;
        }
        arr
    };

    /// Parse a `ModifiedKey` from a TOML key name string.
    ///
    /// Accepts (case-insensitive):
    /// - `F1`–`F24`   → `None` modifier
    /// - `SF1`–`SF24` → `Shift` modifier
    /// - `CF1`–`CF24` → `Ctrl` modifier
    /// - `AF1`–`AF24` → `Alt` modifier
    ///
    /// Returns `None` for any other input.
    ///
    /// Validates: Requirement 20.11
    pub fn parse(s: &str) -> Option<Self> {
        let upper = s.trim().to_ascii_uppercase();
        // Determine modifier prefix and strip it
        let (modifier, rest) = if let Some(r) = upper.strip_prefix("SF") {
            (KeyModifier::Shift, r)
        } else if let Some(r) = upper.strip_prefix("CF") {
            (KeyModifier::Ctrl, r)
        } else if let Some(r) = upper.strip_prefix("AF") {
            (KeyModifier::Alt, r)
        } else {
            (KeyModifier::None, upper.strip_prefix('F')?)
        };
        let num: u8 = rest.parse().ok()?;
        let key = FunctionKey::from_number(num)?;
        Some(Self { key, modifier })
    }

    /// The canonical TOML key name for this `ModifiedKey` (e.g., `"SF3"`, `"CF12"`, `"F7"`).
    ///
    /// Validates: Requirement 20.11
    pub fn toml_name(&self) -> String {
        format!("{}{}", self.modifier.toml_prefix(), self.key.number())
    }

    /// Whether this key uses the plain (no modifier) variant.
    pub fn is_plain(&self) -> bool {
        self.modifier == KeyModifier::None
    }
}

impl fmt::Display for ModifiedKey {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{}{}", self.modifier, self.key.display_name())
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    // ── FunctionKey tests ────────────────────────────────────────────────

    #[test]
    fn parse_valid_function_keys() {
        // Validates: Requirement 1.3 — F1–F24 range support
        assert_eq!(FunctionKey::parse("F1"), Some(FunctionKey::F1));
        assert_eq!(FunctionKey::parse("F12"), Some(FunctionKey::F12));
        assert_eq!(FunctionKey::parse("F24"), Some(FunctionKey::F24));
    }

    #[test]
    fn parse_case_insensitive() {
        // Validates: Requirement 1.3 — case-insensitive parsing
        assert_eq!(FunctionKey::parse("f3"), Some(FunctionKey::F3));
        assert_eq!(FunctionKey::parse("f24"), Some(FunctionKey::F24));
        assert_eq!(FunctionKey::parse("F10"), Some(FunctionKey::F10));
    }

    #[test]
    fn parse_invalid_function_keys_returns_none() {
        // Validates: Requirement 1.5 — reject keys outside F1–F24
        assert_eq!(FunctionKey::parse("F0"), None);
        assert_eq!(FunctionKey::parse("F25"), None);
        assert_eq!(FunctionKey::parse("F99"), None);
        assert_eq!(FunctionKey::parse("G3"), None);
        assert_eq!(FunctionKey::parse(""), None);
        assert_eq!(FunctionKey::parse("hello"), None);
        assert_eq!(FunctionKey::parse("F"), None);
        assert_eq!(FunctionKey::parse("Fa"), None);
    }

    #[test]
    fn from_str_valid() {
        // Validates: Requirement 1.3
        let key: FunctionKey = "F5".parse().unwrap();
        assert_eq!(key, FunctionKey::F5);
    }

    #[test]
    fn from_str_invalid_returns_error() {
        // Validates: Requirement 1.5
        let result: Result<FunctionKey, _> = "F0".parse();
        assert!(result.is_err());
    }

    #[test]
    fn display_format() {
        assert_eq!(FunctionKey::F1.to_string(), "F1");
        assert_eq!(FunctionKey::F12.to_string(), "F12");
        assert_eq!(FunctionKey::F24.to_string(), "F24");
    }

    #[test]
    fn number_round_trip() {
        for key in FunctionKey::ALL {
            let num = key.number();
            assert_eq!(FunctionKey::from_number(num), Some(key));
        }
    }

    #[test]
    fn is_assignable_excludes_f1() {
        assert!(!FunctionKey::F1.is_assignable());
        assert!(FunctionKey::F2.is_assignable());
        assert!(FunctionKey::F24.is_assignable());
    }

    // ── ModifiedKey tests ────────────────────────────────────────────────

    #[test]
    fn modified_key_plain_constructor() {
        // Validates: Requirement 20.12
        let mk = ModifiedKey::plain(FunctionKey::F3);
        assert_eq!(mk.key, FunctionKey::F3);
        assert_eq!(mk.modifier, KeyModifier::None);
        assert!(mk.is_plain());
    }

    #[test]
    fn modified_key_shift_constructor() {
        let mk = ModifiedKey::shift(FunctionKey::F3);
        assert_eq!(mk.modifier, KeyModifier::Shift);
        assert!(!mk.is_plain());
    }

    #[test]
    fn modified_key_ctrl_constructor() {
        let mk = ModifiedKey::ctrl(FunctionKey::F12);
        assert_eq!(mk.modifier, KeyModifier::Ctrl);
    }

    #[test]
    fn modified_key_alt_constructor() {
        let mk = ModifiedKey::alt(FunctionKey::F1);
        assert_eq!(mk.modifier, KeyModifier::Alt);
    }

    #[test]
    fn modified_key_all_has_96_entries() {
        // Validates: Requirement 20.12 — 4 modifiers × 24 keys = 96
        assert_eq!(ModifiedKey::ALL.len(), 96);
    }

    #[test]
    fn modified_key_all_entries_are_unique() {
        // Validates: Requirement 20.12 — all 96 slots are distinct
        use std::collections::HashSet;
        let set: HashSet<ModifiedKey> = ModifiedKey::ALL.iter().copied().collect();
        assert_eq!(set.len(), 96);
    }

    #[test]
    fn modified_key_parse_plain() {
        // Validates: Requirement 20.11 — F1–F24 parse as None modifier
        assert_eq!(
            ModifiedKey::parse("F3"),
            Some(ModifiedKey::plain(FunctionKey::F3))
        );
        assert_eq!(
            ModifiedKey::parse("F24"),
            Some(ModifiedKey::plain(FunctionKey::F24))
        );
        assert_eq!(
            ModifiedKey::parse("f1"),
            Some(ModifiedKey::plain(FunctionKey::F1))
        );
    }

    #[test]
    fn modified_key_parse_shift() {
        // Validates: Requirement 20.11 — SF1–SF24 parse as Shift modifier
        assert_eq!(
            ModifiedKey::parse("SF3"),
            Some(ModifiedKey::shift(FunctionKey::F3))
        );
        assert_eq!(
            ModifiedKey::parse("sf12"),
            Some(ModifiedKey::shift(FunctionKey::F12))
        );
    }

    #[test]
    fn modified_key_parse_ctrl() {
        // Validates: Requirement 20.11 — CF1–CF24 parse as Ctrl modifier
        assert_eq!(
            ModifiedKey::parse("CF7"),
            Some(ModifiedKey::ctrl(FunctionKey::F7))
        );
    }

    #[test]
    fn modified_key_parse_alt() {
        // Validates: Requirement 20.11 — AF1–AF24 parse as Alt modifier
        assert_eq!(
            ModifiedKey::parse("AF12"),
            Some(ModifiedKey::alt(FunctionKey::F12))
        );
    }

    #[test]
    fn modified_key_parse_invalid() {
        // Validates: Requirement 20.11 — invalid strings return None
        assert_eq!(ModifiedKey::parse("G3"), None);
        assert_eq!(ModifiedKey::parse("SF0"), None);
        assert_eq!(ModifiedKey::parse("CF25"), None);
        assert_eq!(ModifiedKey::parse(""), None);
        assert_eq!(ModifiedKey::parse("XF3"), None);
    }

    #[test]
    fn modified_key_toml_name_round_trip() {
        // Validates: Requirement 20.11 — toml_name() produces parseable string
        for mk in ModifiedKey::ALL {
            let name = mk.toml_name();
            let parsed = ModifiedKey::parse(&name);
            assert_eq!(
                parsed,
                Some(mk),
                "round-trip failed for {name}: got {parsed:?}"
            );
        }
    }

    #[test]
    fn modified_key_toml_name_format() {
        // Validates: Requirement 20.11 — canonical TOML names
        assert_eq!(ModifiedKey::plain(FunctionKey::F3).toml_name(), "F3");
        assert_eq!(ModifiedKey::shift(FunctionKey::F3).toml_name(), "SF3");
        assert_eq!(ModifiedKey::ctrl(FunctionKey::F12).toml_name(), "CF12");
        assert_eq!(ModifiedKey::alt(FunctionKey::F1).toml_name(), "AF1");
    }

    #[test]
    fn modified_key_display() {
        assert_eq!(ModifiedKey::plain(FunctionKey::F3).to_string(), "F3");
        assert_eq!(ModifiedKey::shift(FunctionKey::F3).to_string(), "Shift+F3");
        assert_eq!(ModifiedKey::ctrl(FunctionKey::F12).to_string(), "Ctrl+F12");
        assert_eq!(ModifiedKey::alt(FunctionKey::F1).to_string(), "Alt+F1");
    }
}
