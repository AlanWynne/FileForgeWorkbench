//! Common newtypes and type aliases for the ff-select crate.

use std::collections::HashMap;

/// A mapping of field names to their extracted string values for a single record.
pub type FieldValues = HashMap<String, String>;

/// A mapping of field names to their data types.
pub type FieldTypes = HashMap<String, FieldDataType>;

/// The data type of a field in a Record_Structure.
///
/// Determines which comparison mode the evaluator uses.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
#[non_exhaustive]
pub enum FieldDataType {
    /// UTF-8 or ASCII string field.
    Str,
    /// Integer numeric field.
    Int,
    /// Floating-point numeric field.
    Float,
    /// COMP-3 packed-decimal field.
    Packed,
    /// Boolean field (compared as string).
    Bool,
    /// EBCDIC-encoded string field.
    Ebcdic,
}

/// Record fields for bulk evaluation.
#[derive(Debug, Clone)]
pub struct RecordFields {
    /// Field name to extracted value mapping.
    pub values: FieldValues,
}
