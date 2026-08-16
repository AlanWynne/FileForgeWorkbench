//! # ff-file-tree — File Tree Panel Core Logic
//!
//! GUI-independent tree state model for the FileForgeWorkbench unified resource
//! explorer panel. Provides node hierarchy management, sort/filter engines,
//! keyboard navigation, and context menu construction.
//!
//! All VFS I/O and rendering are handled by the shell layer; this crate owns
//! only the data model and pure logic.

pub mod context_menu;
pub mod error;
pub mod filter;
pub mod keyboard;
pub mod node;
pub mod sort;
pub mod state;

pub use context_menu::{ContextAction, ContextMenuBuilder};
pub use error::FileTreeError;
pub use filter::{FilterEngine, FilterState};
pub use keyboard::{KeyboardHandler, TreeAction};
pub use node::{FileCategory, NodeId, NodeType, TreeNode, TreeNodeData};
pub use sort::{SortEngine, SortOrder};
pub use state::TreeState;
