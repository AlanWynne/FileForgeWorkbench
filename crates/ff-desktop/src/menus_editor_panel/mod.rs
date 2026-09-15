//! Menus Editor Context (menu-workspace Req 13, CR-NR-075).
//!
//! Mirrors the Theme editor (`theme_editor_panel`): a pure render function that
//! returns a [`MenusEditorAction`], with all side effects (validation,
//! serialisation, file writes) applied by the shell command layer
//! (`shell/menus_editor.rs`). The panel itself never touches the filesystem.

mod render;
mod state;

pub use render::render;
pub use state::{MenusEditorAction, MenusEditorState};
