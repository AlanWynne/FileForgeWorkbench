//! # ff-completion — Command Completion Engine for FileForgeWorkbench
//!
//! This crate implements the **auto-complete popup system** for the workbench.
//! It provides context-sensitive command name, argument, and line command completion
//! in the primary command field and prefix area.
//!
//! ## Architecture
//!
//! The crate is **GUI-independent** in its core logic — candidate generation,
//! filtering, ranking, and selection state operate without any GUI dependency.
//! Only the popup positioning model produces layout coordinates consumed by the
//! shell renderer.
//!
//! ```text
//! ┌────────────────────────────────────────────────────────────┐
//! │                     Shell (egui)                            │
//! │         Key events, popup rendering                        │
//! ├────────────────────────────────────────────────────────────┤
//! │              ff-completion (this crate)                     │
//! │   Engine · Providers · Matching · Positioning · Navigation │
//! ├────────────────────────────────────────────────────────────┤
//! │   ff-command · ff-config · ff-vfs · ff-lua-macro           │
//! └────────────────────────────────────────────────────────────┘
//! ```
//!
//! ## Usage
//!
//! ```rust,no_run
//! use ff_completion::{CompletionEngine, CompletionConfig, CompletionField};
//! use ff_completion::provider::create_default_registry;
//!
//! let config = CompletionConfig::default();
//! let registry = create_default_registry();
//! let mut engine = CompletionEngine::new(config, registry);
//!
//! // User types in command field
//! let action = engine.on_manual_trigger(CompletionField::PrimaryCommand, "FI", 2);
//! ```

// ─── Public Modules ─────────────────────────────────────────────────────────

/// Completion candidate — raw items from providers.
pub mod candidate;

/// Completion context — state snapshot at trigger time.
pub mod context;

/// Configuration types and validation.
pub mod config;

/// The completion engine — central orchestrator.
pub mod engine;

/// Error types for the completion subsystem.
pub mod error;

/// Filtered, ranked completion list.
pub mod list;

/// Prefix and fuzzy matching algorithms.
pub mod matching;

/// Selection state and keyboard navigation.
pub mod navigation;

/// Popup positioning model.
pub mod positioning;

/// Completion providers (built-in and trait definition).
pub mod provider;

// ─── Public API Re-exports ──────────────────────────────────────────────────

pub use candidate::{CompletionCandidate, CompletionKind};
pub use config::{CompletionConfig, MatchingMode, RawConfigValues, TriggerMode};
pub use context::{CompletionContext, CompletionContextBuilder, CompletionField};
pub use engine::{CompletionAction, CompletionEngine, NavigationAction};
pub use error::CompletionError;
pub use list::{CompletionItem, CompletionList};
pub use matching::{fuzzy_match, prefix_match, FuzzyMatchResult};
pub use navigation::SelectionState;
pub use positioning::{
    compute_popup_position, FieldRect, PopupAnchor, PopupBounds, PopupConfig, ViewportRect,
};
pub use provider::{CompletionProvider, ProviderId, ProviderRegistry};
