//! # ff-help — Context-Sensitive Help System for FileForgeWorkbench
//!
//! This crate is the **context-sensitive help subsystem** for FileForgeWorkbench.
//! It provides:
//!
//! - **F1-triggered context detection** — resolves the most relevant help topic
//!   based on the current editor state (command line, prefix area, mode)
//! - **Help Panel model** — dockable, non-modal panel state with breadcrumb,
//!   Table of Contents, and scroll management
//! - **Help Topic Registry** — thread-safe indexed store with O(1) lookup,
//!   aggregating topics from files, commands, and plugins
//! - **Help content loading** — parses `.help.md` Markdown files
//! - **Search** — case-insensitive keyword search with relevance ranking
//! - **Navigation** — back/forward history stack
//! - **Dynamic content** — function key display, Help Index generation
//! - **HELP command** — primary command routing (`HELP`, `HELP CHANGE`, `HELP OFF`, etc.)
//! - **Plugin bridge** — topic registration/deregistration during plugin lifecycle
//! - **Help menu** — menu item definitions for the Help menu bar entry
//!
//! ## Architecture
//!
//! ```text
//! Shell Layer (egui) → reads HelpPanelModel for rendering
//!       ↕
//! ff-help (this crate)
//!   • ContextDetector: focus + command + mode → TopicKey
//!   • HelpTopicRegistry: TopicKey → HelpTopic (thread-safe)
//!   • ContentLoader/Parser: .help.md → HelpTopic[]
//!   • NavigationStack: back/forward history
//!   • HelpSearch: keyword search + ranking
//!   • HelpPanelModel: panel state, breadcrumb, TOC
//!   • DynamicContentGenerator: function keys, index
//!   • Commands: HELP primary command routing
//!       ↕
//! Upstream: ff-command, ff-layout, ff-config, ff-plugin, ff-logging
//! ```
//!
//! ## Position in Architecture
//!
//! `ff-help` is a **Wave 9 (Desktop Integration)** crate. It depends on
//! Wave 2 platform crates (`ff-command`, `ff-layout`, `ff-config`, `ff-plugin`)
//! and the foundation (`ff-logging`).

// ─── Public Modules ─────────────────────────────────────────────────────────

/// Help system error types.
pub mod error;

/// `TopicKey` — typed identifier for help topics.
pub mod topic_key;

/// `HelpTopic` — a single unit of help content.
pub mod topic;

/// Help Topic Registry — thread-safe indexed store.
pub mod registry;

/// Context detection for F1 help activation.
pub mod context_detector;

/// Help content loading — file discovery and loading.
pub mod content_loader;

/// Help content parsing — `.help.md` file format parser.
pub mod content_parser;

/// Help Panel model — panel state, breadcrumb, TOC.
pub mod help_panel;

/// Navigation stack — back/forward history.
pub mod navigation;

/// Help search — keyword matching with relevance ranking.
pub mod search;

/// Plugin help registration bridge.
pub mod plugin_help;

/// Dynamic content generation — function keys, Help Index.
pub mod dynamic_content;

/// HELP command handler and F1 activation logic.
pub mod commands;

/// Help menu model — menu items and About dialog.
pub mod menu;

/// Help configuration — `[help]` TOML section typed access.
pub mod config;

// ─── Public API Re-exports ──────────────────────────────────────────────────

pub use commands::{resolve_help_argument, HelpAction};
pub use config::{HelpConfig, HelpPanelPosition};
pub use content_loader::{ContentLoadResult, ContentLoader};
pub use content_parser::ContentParser;
pub use context_detector::{ContextDetector, EditorContext, EditorMode};
pub use dynamic_content::{DynamicContentGenerator, FunctionKeyBinding, KeyMapAccess};
pub use error::HelpError;
pub use help_panel::{BreadcrumbEntry, HelpPanelModel, TocEntry};
pub use menu::{help_menu_items, AboutInfo, HelpMenuAction, HelpMenuItem};
pub use navigation::NavigationStack;
pub use plugin_help::HelpPluginBridge;
pub use registry::HelpTopicRegistry;
pub use search::{HelpSearch, MatchLocation, SearchResult};
pub use topic::{HelpTopic, TopicSource};
pub use topic_key::{TopicCategory, TopicKey};
