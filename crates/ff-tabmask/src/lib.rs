//! # ff-tabmask — Tab Stop Management and Insert Mask Templates
//!
//! This crate provides the **TABS** and **MASK** command subsystem for
//! FileForgeWorkbench. It manages:
//!
//! - An ordered list of distinct tab stop column positions per session
//! - TABS_Lines (non-editable rulers) showing active tab stop positions
//! - An insert mask template string per session
//! - MASK_Lines (editable template lines) for viewing/modifying the mask
//! - Tab key cursor advancement using configured stops
//! - Mask application to blank lines created by I/In line commands
//! - Default tab stops and mask from configuration and language definitions
//! - RESET interactions (clear display artifacts, preserve state)
//! - Shift-to-tab-stop computation for `>` / `<` line commands
//!
//! ## Architecture
//!
//! The crate is GUI-independent — all functionality is testable via the public API
//! without a running editor. Upstream crates are connected via trait interfaces
//! defined in [`traits`].
//!
//! ## Example
//!
//! ```rust
//! use ff_tabmask::tab_stops::TabStopList;
//! use ff_tabmask::state::{TabsState, TabStopSource, MaskState, TabsMaskState};
//!
//! // Create a session with custom tab stops
//! let stops = TabStopList::from_columns(vec![7, 12, 72]);
//! let tabs_state = TabsState::new(stops, TabStopSource::LanguageDefinition);
//! let state = TabsMaskState::new(tabs_state, MaskState::empty());
//! ```

// ─── Public Modules ─────────────────────────────────────────────────────────

/// Display artifact lifecycle and rendering (TABS_Line / MASK_Line).
pub mod artifacts;

/// Command handlers for TABS and MASK primary and line commands.
pub mod commands;

/// Defaults loader — configuration and language integration.
pub mod defaults;

/// Error types for the ff-tabmask crate.
pub mod error;

/// Insert mask template management.
pub mod mask;

/// Shift-to-tab-stop computation for `>` and `<` line commands.
pub mod shift;

/// Per-session state model for TABS and MASK features.
pub mod state;

/// Tab key cursor advancement logic.
pub mod tab_key;

/// Tab stop list management.
pub mod tab_stops;

/// Trait interfaces for upstream dependencies.
pub mod traits;

// ─── Public API Re-exports ──────────────────────────────────────────────────

pub use artifacts::{
    ArtifactKind, ArtifactMetadata, DisplayArtifactManager, EditorMode, UndoClassification,
};
pub use commands::reset_tabs::{execute_reset_tabs, handle_reset};
pub use commands::{
    execute_line_command, execute_mask_command, execute_tabs_command, MaskCommandResult,
    TabsCommandResult,
};
pub use defaults::{DefaultsLoader, MaskManager};
pub use error::TabsMaskError;
pub use mask::MaskLine;
pub use shift::{compute_shift_left, compute_shift_right, ShiftAction};
pub use state::{ArtifactPosition, MaskState, TabStopSource, TabsMaskState, TabsState};
pub use tab_key::{compute_tab_action, TabKeyAction};
pub use tab_stops::TabStopList;
pub use traits::{ConfigProvider, DocumentContext, LanguageDefinitionRef};
