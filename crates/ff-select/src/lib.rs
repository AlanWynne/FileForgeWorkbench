//! # ff-select — Record Selection Criteria Engine
//!
//! This crate provides the **field-level record filtering engine** for
//! FileForgeWorkbench. When FileForge_Mode is active, users can define
//! criteria expressions that control which records are displayed in
//! Grid_Edit_Mode and Grid_Browse_Mode.
//!
//! ## Capabilities
//!
//! - Define criteria sets: ordered list of criterion rows with comparison operators
//! - Evaluate criteria against record field values (type-aware: string, numeric, packed-decimal)
//! - Combine criteria with logical AND/OR connectors and parenthesised grouping
//! - Glob-style wildcard matching in string comparisons
//! - Persist named criteria sets to `.criteria.json` files in a Criteria_Catalog
//! - Manage Criteria_Locations and Active_Criteria_Location via configuration
//! - Register CRITERIA command (SET/CLEAR/SHOW/SAVE) in the command framework
//! - Provide criteria scope integration with FIND/CHANGE operations
//! - Track filter state for status bar indicator rendering
//!
//! ## Architecture
//!
//! The crate is GUI-independent — all functionality is testable via the public
//! API without a running editor. UI rendering (Criteria_Panel, Criteria_Catalog_Dialog)
//! is handled by the shell-side `ff-desktop` crate.

// ─── Public Modules ─────────────────────────────────────────────────────────

/// Data model: CriteriaSet, Criterion, operators, connectors.
pub mod model;

/// Core evaluation logic: orchestrates comparison and logical combination.
pub mod evaluator;

/// Field-type-aware comparison engine (string, numeric, packed-decimal).
pub mod comparison;

/// Logical combination: AND/OR with parenthesised grouping and precedence.
pub mod logical;

/// Glob-style wildcard pattern matching.
pub mod wildcard;

/// JSON-based persistence for named criteria sets.
pub mod persistence;

/// Criteria_Location management (catalog path CRUD).
pub mod location;

/// CRITERIA command registration and argument parsing.
pub mod commands;

/// Active filter state tracking and status bar indicators.
pub mod filter_state;

/// Criteria scope integration for FIND/CHANGE operations.
pub mod scope;

/// Expression validation (groups, regex, types, fields).
pub mod validator;

/// Configuration loading from `[criteria]` TOML namespace.
pub mod config;

/// Structure association and auto-suggestion logic.
pub mod association;

/// Common newtypes and type aliases.
pub mod types;

/// Error types for the ff-select crate.
pub mod error;

// ─── Public API Re-exports ──────────────────────────────────────────────────

pub use config::CriteriaConfig;
pub use error::CriteriaError;
pub use evaluator::CriteriaEvaluator;
pub use filter_state::FilterState;
pub use logical::{LogicalCombiner, LogicalRow};
pub use model::{
    ComparisonMode, CriteriaConnector, CriteriaOperator, CriteriaResult, CriteriaSet, Criterion,
    RowResult,
};
pub use persistence::{CriteriaPersistence, CriteriaSetMetadata};
pub use scope::CriteriaScope;
pub use validator::{CriteriaValidator, ValidationIssue};
pub use wildcard::WildcardMatcher;
