//! # ff-exclude-show-filter — Line Visibility Management Engine
//!
//! This crate implements the ISPF-style EXCLUDE/SHOW/RESET primary commands
//! and X/Xn/XX line commands for FileForgeWorkbench. It provides a
//! GUI-independent logical layer that drives per-line visibility state
//! through the `ff-display-line-mapping` subsystem.
//!
//! ## Key Properties
//!
//! - **Non-undoable**: All operations modify transient session state only
//! - **Flat exclusion**: No hierarchy or fold levels; distinct from code folding
//! - **Delegation**: Visibility storage lives in `display-line-mapping`
//! - **GUI-independent**: Pure logical layer with no rendering dependencies
//!
//! ## Quick Start
//!
//! ```rust
//! use ff_display_line_mapping::ContractionState;
//! use ff_exclude_show_filter::{ExclusionEngine, ExcludeArgs, ExcludeScope};
//! use ff_exclude_show_filter::DocumentAccess;
//!
//! // Create a simple document with 5 lines
//! struct MyDoc(Vec<String>);
//! impl DocumentAccess for MyDoc {
//!     fn line_content(&self, line: usize) -> Option<&str> {
//!         self.0.get(line).map(|s| s.as_str())
//!     }
//!     fn line_count(&self) -> usize { self.0.len() }
//!     fn is_tagged(&self, _line: usize) -> bool { false }
//! }
//!
//! let mapping = ContractionState::new(5);
//! let doc = MyDoc(vec![
//!     "hello world".into(),
//!     "foo bar".into(),
//!     "hello again".into(),
//!     "baz".into(),
//!     "world hello".into(),
//! ]);
//!
//! let mut engine = ExclusionEngine::new(mapping, doc);
//! let args = ExcludeArgs::Text {
//!     pattern: "hello".to_string(),
//!     scope: ExcludeScope::Visible,
//! };
//! let result = engine.execute_exclude(&args).unwrap();
//! assert_eq!(result.lines_affected, 3);
//! assert!(engine.is_excluded(0));
//! assert!(!engine.is_excluded(1));
//! ```

// ─── Public Modules ─────────────────────────────────────────────────────────

pub mod error;
pub mod exclusion_engine;
pub mod registration;
pub mod text_matcher;
pub mod types;

// ─── Public API Re-exports ──────────────────────────────────────────────────

pub use error::ExcludeFilterError;
pub use exclusion_engine::{DocumentAccess, ExclusionEngine, ExclusionListener};
pub use registration::{
    register_commands, ExcludeCommandHandler, ResetCommandHandler, ShowCommandHandler,
};
pub use text_matcher::TextMatcher;
pub use types::{
    ExcludeArgs, ExcludeResult, ExcludeScope, ExclusionBlock, ExclusionChanged, LineCommandExclude,
    ResetResult, ResetVariant, ShowArgs, ShowResult, TextMatchMode,
};
