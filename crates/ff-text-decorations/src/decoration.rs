//! Single-indicator decoration wrapper.
//!
//! A `Decoration` wraps `RunStyles<u32>` for one indicator number
//! within a document.

// This module is a thin re-export; the actual storage lives in RunStyles.
// The DecorationList manages decoration instances directly using RunStyles<u32>.
