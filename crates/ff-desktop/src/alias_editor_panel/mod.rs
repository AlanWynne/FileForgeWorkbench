//! Alias Editor Context (command-configurator Req 5, CR-NR-113).
//!
//! Modelled on the Menus Editor (`menus_editor_panel`): a pure render function
//! that returns an [`AliasEditorAction`], with all side effects (validation,
//! serialisation, file writes, live-resolver refresh) applied by the shell
//! command layer (`shell/alias_editor.rs`). The panel itself never touches the
//! filesystem.

mod filter;
mod host;
mod render;
mod rows;
mod state;

pub(crate) use filter::displayed_indices;
pub use host::alias_surface_column;
pub(crate) use host::{alias_snapshot_cells, ALIAS_COLUMNS};
pub use state::{AliasEditorAction, AliasEditorState, AliasRowEdit};
