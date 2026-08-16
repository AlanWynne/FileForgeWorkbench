//! # ff-menu — Menu Bar, Context Menus, Status Bar, and Command Field
//!
//! This crate provides the complete menu and status bar system for
//! FileForgeWorkbench. It bridges the command framework (`ff-command`)
//! and configuration system (`ff-config`) to deliver:
//!
//! - A standard hierarchical **Menu Bar** (File, Edit, Search, View, Tools, Help)
//! - **Context menus** for editor areas, tab headers, panels, and file tree nodes
//! - A configurable multi-segment **Status Bar** at the bottom of the window
//! - The **Primary Command Field** ("Command ===>") for ISPF-style command entry
//! - **Plugin extensibility** for menu items and status bar segments
//! - **Recent Files** management with persistence across sessions
//!
//! ## Architecture
//!
//! All menu items route through `ff-command` dispatch — no menu action directly
//! mutates application state. The status bar renders real-time editor state via
//! configurable segments. Plugins contribute menu items and status segments
//! through well-defined extension points.
//!
//! ## Position in Architecture
//!
//! `ff-menu` is a **Wave 6 (UI and Rendering)** crate. It depends on Wave 2
//! platform crates (`ff-command`, `ff-config`, `ff-core`, `ff-plugin`) and is
//! consumed by the GUI shell (`ff-desktop`).

// ─── Public Modules ─────────────────────────────────────────────────────────

/// Menu bar model, builder, and rendering.
pub mod menu_bar;

/// Individual menu item types and bindings.
pub mod menu_item;

/// Menu model data structures (Menu, MenuEntry, SubMenu).
pub mod menu_model;

/// Context menu registry and types.
pub mod context_menu;

/// Status bar manager, segments, and layout.
pub mod status_bar;

/// Status bar segment types and alignment.
pub mod status_segment;

/// Primary command field controller and history.
pub mod command_field;

/// Recent files list management and persistence.
pub mod recent_files;

/// Keyboard navigation state machine for menus.
pub mod keyboard_nav;

/// Plugin extensibility — menu and status bar contributions.
pub mod extensibility;

/// Error types for the ff-menu crate.
pub mod error;

// ─── Public API Re-exports ──────────────────────────────────────────────────

pub use command_field::{CommandFieldController, CommandFieldState, SubmitResult};
pub use context_menu::{ContextMenuRegistry, ContextType};
pub use error::MenuError;
pub use extensibility::{MenuContribution, MenuContributionRegistry, MenuInsertPosition};
pub use keyboard_nav::MenuNavState;
pub use menu_bar::MenuBar;
pub use menu_item::{MenuCommandBinding, MenuItem, ToggleBinding};
pub use menu_model::{Menu, MenuEntry};
pub use recent_files::{RecentFileEntry, RecentFilesManager};
pub use status_bar::StatusBar;
pub use status_segment::{SegmentAlignment, StatusSegment, StatusSegmentProvider};
