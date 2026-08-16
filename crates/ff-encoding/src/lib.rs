//! # ff-encoding
//!
//! Encoding detection, conversion, and character-handling subsystem for FileForgeWorkbench.
//!
//! This crate provides:
//! - Encoding detection from raw byte streams (BOM-based and heuristic)
//! - Bidirectional encoding conversion (source → UTF-8 on load, UTF-8 → target on save)
//! - UTF-8 validation and repair
//! - Word-character classification (`CharClassify` and Unicode `CharacterCategoryMap`)
//! - Unicode case folding and conversion (fold, upper, lower)
//! - Grapheme cluster boundary detection (UAX #29)
//! - DBCS lead/trail byte logic for legacy East Asian encodings
//! - Word-part (sub-word) boundary detection for camelCase/snake_case navigation
//! - Encoding family classification and encoding registry

pub mod bom;
pub mod case_fold;
pub mod category_map;
pub mod classify;
pub mod convert;
pub mod dbcs;
pub mod detect;
pub mod encoding;
pub mod error;
pub mod grapheme;
pub mod registry;
pub mod utf8;
pub mod word_part;

// Public re-exports

// Encoding types and registry
pub use encoding::{Encoding, EncodingFamily, EncodingMetadata};
pub use registry::EncodingRegistry;

// Errors
pub use error::EncodingError;

// BOM
pub use bom::{bom_bytes, detect_bom, write_bom, BomEncoding, BomInfo};

// Detection
pub use detect::{
    detect_encoding, detect_encoding_with_fallback, DetectionConfidence, DetectionResult,
};

// Conversion
pub use convert::{
    convert_from_utf8, convert_to_utf8, ConversionIssue, ConversionResult, StreamDecoder,
    StreamEncoder, UnmappableAction,
};

// UTF-8 utilities
pub use utf8::{utf8_byte_length_from_lead, utf8_classify, utf8_fix_invalid, utf8_validate};

// Classification
pub use category_map::{CharacterCategory, CharacterCategoryMap};
pub use classify::{CharClassify, CharacterClass};

// Case folding
pub use case_fold::{CaseFolder, CaseMode, ICaseConverter};

// Grapheme clusters
pub use grapheme::{
    is_grapheme_boundary, next_grapheme_boundary, prev_grapheme_boundary, GraphemeIterator,
    GraphemeMode,
};

// DBCS
pub use dbcs::{
    dbcs_is_lead_byte, dbcs_is_trail_byte, is_dbcs_code_page, is_dbcs_valid_single_byte,
    safe_segment, DbcsCodePage, DbcsCodePageDef,
};

// Word-part navigation
pub use word_part::{is_word_part_separator, word_part_left, word_part_right};
