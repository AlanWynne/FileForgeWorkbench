//! Keys Editor Context (function-keys-and-history Requirement 22, CR-CH-029).
//!
//! Mirrors the Menus editor (`menus_editor_panel`): a pure render that returns a
//! [`KeysEditorAction`], with all side effects (load a kind's key list, save to
//! `keymaps/<kind>.toml`, reset) applied by the shell. The panel itself never
//! touches the filesystem. Replaces the retired modal `KeyConfigDialog`.

mod render;
mod state;

pub use state::{rows_to_map, KeysEditorAction, KeysEditorState, KIND_NAMES};
