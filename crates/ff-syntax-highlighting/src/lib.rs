//! # ff-syntax-highlighting
//!
//! Lexical highlighting engine for FileForgeWorkbench. Assigns abstract style-slot
//! indices (u8, 0–255) to character ranges based on lexical analysis of document
//! content. The engine is GUI-independent — it produces style data consumed by the
//! theme system for visual attribute resolution, never referencing colours or
//! rendering APIs directly.
//!
//! ## Key capabilities
//!
//! - Trait-based lexer interface for language-specific tokenization
//! - Per-document style buffers parallel to text content
//! - Incremental re-highlighting from first modified line's state
//! - Demand-driven styling (`ensure_styled_to`) for viewport rendering
//! - Up to 9 keyword sets per language with efficient hash-based lookup
//! - Sub-style allocation for fine-grained token differentiation
//! - Fold-level assignment alongside styling for display-line-mapping
//! - Idle-time background styling coordination
//! - Property-based lexer configuration with hot-reload support

pub mod engine;
pub mod error;
pub mod fold;
pub mod hilite;
pub mod keywords;
pub mod lexer;
pub mod state;
pub mod style;
pub mod types;

// Public API re-exports
pub use engine::highlight_engine::HighlightEngine;
pub use engine::idle_styling::{IdleStylingConfig, IdleStylingResult};
pub use error::SyntaxHighlightError;
pub use fold::context::FoldContext;
pub use fold::store::FoldData;
pub use hilite::{
    HiliteLogicScanner, HiliteModes, HiliteOperand, HiliteParenMatcher, HiliteState,
    ParenMatchResult,
};
pub use keywords::word_list::WordList;
pub use lexer::registry::LexerRegistry;
pub use lexer::traits::Lexer;
pub use style::buffer::StyleBuffer;
pub use style::context::StyleContext;
pub use style::sub_styles::{SubStyleAllocator, SubStyleRange};
pub use types::{
    BytePosition, FoldFlags, FoldLevel, HighlightSpan, KeywordSetDescriptor, KeywordSetIndex,
    LexerState, LineNumber, PropertyDescriptor, PropertyType, StyleSlotIndex, SyntaxHighlighter,
};
