//! Record codecs for mainframe dataset I/O.
//!
//! Codecs translate between raw binary storage bytes and logical record vectors.
//! They have no dependency on SQLite, the filesystem, or egui.
//!
//! # Codec selection by RECFM
//!
//! | RECFM    | Codec           |
//! |----------|-----------------|
//! | F, FB    | `FixedCodec`    |
//! | V, VB    | `VariableCodec` |
//! | U        | `BinaryCodec`   |
//! | import/export only | `TextCodec` |

mod binary;
mod fixed;
mod text;
mod variable;

pub use binary::BinaryCodec;
pub use fixed::FixedCodec;
pub use text::TextCodec;
pub use variable::VariableCodec;

/// Translates between raw storage bytes and logical records.
///
/// Implementations must be stateless and have no filesystem or database dependency.
///
/// # Errors
///
/// Returns `CodecError` on malformed input (bad RDW, wrong byte count, etc.).
pub trait RecordCodec {
    /// Encode a slice of logical records into raw storage bytes.
    fn encode(&self, records: &[Vec<u8>]) -> Result<Vec<u8>, CodecError>;

    /// Decode raw storage bytes into logical records.
    fn decode(&self, bytes: &[u8]) -> Result<Vec<Vec<u8>>, CodecError>;
}

/// Errors produced by record codecs.
///
/// Each variant carries dataset identity and record position for diagnostics.
/// Validates: Requirement 16.7
#[derive(Debug, thiserror::Error)]
#[non_exhaustive]
pub enum CodecError {
    /// Byte count is not a multiple of LRECL (fixed-length decode).
    #[error(
        "codec: fixed decode: {byte_count} bytes is not a multiple of LRECL {lrecl} \
         in dataset '{dataset}'"
    )]
    NotMultipleOfLrecl {
        byte_count: usize,
        lrecl: usize,
        dataset: String,
    },

    /// RDW length field is invalid (variable-length decode).
    #[error(
        "codec: variable decode: malformed RDW at record {record_index} \
         (byte offset {byte_offset}) in dataset '{dataset}': {reason}"
    )]
    MalformedRdw {
        record_index: usize,
        byte_offset: usize,
        dataset: String,
        reason: String,
    },

    /// Record data exceeds LRECL (fixed-length encode).
    #[error(
        "codec: fixed encode: record {record_index} length {record_len} \
         exceeds LRECL {lrecl} in dataset '{dataset}'"
    )]
    RecordTooLong {
        record_index: usize,
        record_len: usize,
        lrecl: usize,
        dataset: String,
    },
}
