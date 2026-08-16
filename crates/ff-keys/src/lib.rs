//! # ff-keys — Function Keys, Key Label Bar, RETRIEVE, and Command History
//!
//! This crate manages configurable function key maps (F1–F24), the Key Label Bar
//! display model, the RETRIEVE command for single-step history recall, and the
//! bounded deduplicated Command History ring with cross-session TOML persistence.
//!
//! ## Architecture
//!
//! ```text
//! ┌─────────────────────────────────────────────────────┐
//! │              Shell Layer (renders UI)                 │
//! ├─────────────────────────────────────────────────────┤
//! │  ff-keys (this crate)                                │
//! │  ├─ KeyMapResolver (global vs profile selection)     │
//! │  ├─ KeyLabelBarModel (display slots for GUI)         │
//! │  ├─ CommandHistory (bounded dedup ring)              │
//! │  ├─ RetrieveState (pointer cycling)                  │
//! │  └─ HistoryStore (TOML persistence)                  │
//! ├─────────────────────────────────────────────────────┤
//! │  ff-command │ ff-config │ ff-logging │ ff-core       │
//! └─────────────────────────────────────────────────────┘
//! ```
//!
//! ## Key Design Decisions
//!
//! - **Full-replacement key map model**: When a profile key map is active, the
//!   global key map is entirely inactive. Keys not in the profile map are unassigned.
//! - **GUI-independent**: All logic is GUI-free; the shell renders using our models.
//! - **Graceful degradation**: Missing or corrupt history files never prevent startup.

// ─── Public Modules ─────────────────────────────────────────────────────────

/// Function key enumeration with parsing and display support.
pub mod function_key;

/// Key map data structures (`KeyBinding`, `KeyMap`, TOML parsing).
pub mod key_map;

/// Key map resolver — global vs profile selection logic.
pub mod key_map_resolver;

/// Key Label Bar display model.
pub mod key_label_bar;

/// Command History — bounded, deduplicated, ordered ring.
pub mod command_history;

/// RETRIEVE command and Retrieve Pointer logic.
pub mod retrieve;

/// History Store — TOML persistence for Command History.
pub mod history_store;

/// Configuration accessors for the `[keys]` namespace.
pub mod config_keys;

/// Error types for the function-keys-and-history subsystem.
pub mod error;

// ─── Public API Re-exports ──────────────────────────────────────────────────

pub use command_history::{CommandHistory, HistoryEntry};
pub use config_keys::{
    KeysConfig, DEFAULT_EXCLUDED_COMMANDS, DEFAULT_HISTORY_FILE, DEFAULT_MAX_HISTORY_ENTRIES,
};
pub use error::KeysError;
pub use function_key::{FunctionKey, KeyModifier, ModifiedKey};
pub use history_store::HistoryStore;
pub use key_label_bar::{KeyLabelBarModel, KeyLabelSlot};
pub use key_map::{KeyBinding, KeyMap, KeyMapWarning};
pub use key_map_resolver::KeyMapResolver;
pub use retrieve::{RetrieveResult, RetrieveState};
