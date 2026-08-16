//! `ShortcutBinding` — single chord or multi-key sequence binding.

use std::fmt;

use super::chord::KeyChord;

/// Timeout for multi-key sequence pending state (milliseconds).
pub const SEQUENCE_TIMEOUT_MS: u64 = 2000;

/// A shortcut binding — either a single chord or a multi-key sequence.
///
/// Single chords (e.g., Ctrl+S) resolve immediately.
/// Multi-key sequences (e.g., Ctrl+K followed by Ctrl+C) require two
/// successive chord inputs.
#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub enum ShortcutBinding {
    /// Single chord binding (e.g., Ctrl+S).
    Single(KeyChord),
    /// Multi-key sequence (e.g., Ctrl+K, Ctrl+C).
    Sequence(KeyChord, KeyChord),
}

impl fmt::Display for ShortcutBinding {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::Single(chord) => write!(f, "{chord}"),
            Self::Sequence(first, second) => write!(f, "{first} {second}"),
        }
    }
}
