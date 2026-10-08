//! RecordFormat: the framing of the universal editable unit (a RECORD).
//!
//! The owning Command Environment supplies the `RecordFormat` (CR-CH-058). The
//! native CE supplies `Delimited`, generalising the existing `LineEndMode`;
//! mainframe CEs supply `Fixed`/`Variable` under a later V-stream gate. F1
//! exercises `Delimited` end-to-end; `Fixed`/`Variable` variants exist for
//! AC 11.1 and are NOT flattened to delimiter-terminated bytes (AC 11.5), but
//! their framing is only minimally supported in F1 (mainframe open/save is out
//! of scope of this gate).

use crate::line_end::{self, LineEndMode};

/// The delimiter terminator style for a `Delimited` record format.
///
/// `Mixed` means records may use differing terminators within one document
/// (the editor recognises CR, LF, and CRLF per the current `LineEndMode`).
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum DelimiterTerminator {
    /// CRLF (0x0D 0x0A) line endings.
    Crlf,
    /// LF (0x0A) line endings.
    Lf,
    /// CR (0x0D) line endings.
    Cr,
    /// Mixed terminators within one document.
    Mixed,
}

impl DelimiterTerminator {
    /// The exact delimiter bytes this terminator re-emits on save.
    ///
    /// `Mixed` has no single canonical delimiter; it returns LF as a fallback,
    /// but Mixed documents preserve each record's original terminator through
    /// the Immutable_Original_Index byte lengths rather than this helper.
    pub fn bytes(self) -> &'static [u8] {
        match self {
            DelimiterTerminator::Crlf => b"\r\n",
            DelimiterTerminator::Lf => b"\n",
            DelimiterTerminator::Cr => b"\r",
            DelimiterTerminator::Mixed => b"\n",
        }
    }
}

/// The framing of the editable unit (a RECORD) for a document.
///
/// `Delimited` is the native format and generalises `LineEndMode`; a Delimited
/// record IS a line (AC 11.3). `Fixed`/`Variable` are the mainframe formats and
/// are NOT flattened to delimiters (AC 11.5).
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum RecordFormat {
    /// Delimiter-terminated records (native, generalises LineEndMode).
    Delimited {
        /// The terminator style.
        terminator: DelimiterTerminator,
    },
    /// Fixed-length records (mainframe FB), position-terminated.
    Fixed {
        /// Logical record length in bytes.
        lrecl: u32,
    },
    /// Variable-length records (mainframe VB), length-prefixed.
    Variable {
        /// Maximum logical record length in bytes.
        max_lrecl: u32,
        /// Whether records carry a Record Descriptor Word prefix.
        rdw: bool,
    },
}

impl RecordFormat {
    /// The native Delimited format for the given `LineEndMode`.
    ///
    /// This is how the native CE generalises `LineEndMode` into a
    /// `RecordFormat` (AC 11.2). The terminator defaults to `Mixed` because
    /// `LineEndMode::Default` recognises CR, LF, and CRLF uniformly; the exact
    /// per-record terminator is preserved by the index byte lengths.
    pub fn native(_mode: LineEndMode) -> Self {
        RecordFormat::Delimited {
            terminator: DelimiterTerminator::Mixed,
        }
    }

    /// Returns `true` for the Delimited (native) format.
    pub fn is_delimited(self) -> bool {
        matches!(self, RecordFormat::Delimited { .. })
    }
}

impl Default for RecordFormat {
    fn default() -> Self {
        RecordFormat::native(LineEndMode::Default)
    }
}

/// A single framed record boundary within a byte slice.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct RecordBoundary {
    /// Byte offset of the record's first byte.
    pub start: u64,
    /// Total byte length of the record INCLUDING its terminator (Delimited) or
    /// slot (Fixed). This is what the index stores so re-emit is byte-identical.
    pub byte_length: u32,
}

/// Frame a byte slice into record boundaries per the given format.
///
/// For `Delimited` this reproduces the exact boundaries of the existing line
/// scan (CR, LF, CRLF, and Unicode terminators via `LineEndMode`), so a
/// Delimited record is byte-for-byte a line (AC 11.3, 11.4). The returned
/// boundaries' `byte_length` INCLUDE the terminator bytes so that re-emitting
/// each record reproduces the original bytes exactly (AC 12.12).
///
/// For `Fixed` the slice is split into `lrecl`-sized slots arithmetically.
/// `Variable` is unsupported in F1 and returns an empty vector (mainframe
/// open/save is out of F1 scope).
pub fn frame_records(bytes: &[u8], format: RecordFormat, mode: LineEndMode) -> Vec<RecordBoundary> {
    match format {
        RecordFormat::Delimited { .. } => frame_delimited(bytes, mode),
        RecordFormat::Fixed { lrecl } => frame_fixed(bytes, lrecl),
        RecordFormat::Variable { .. } => Vec::new(),
    }
}

/// Frame delimiter-terminated records. One record per terminator-terminated
/// span; a trailing span with no terminator is its own record; an empty slice
/// is a single zero-length record (the empty-document shape, AC 4.8).
fn frame_delimited(bytes: &[u8], mode: LineEndMode) -> Vec<RecordBoundary> {
    let mut boundaries = Vec::new();
    let mut record_start = 0usize;
    let mut i = 0usize;
    while i < bytes.len() {
        let le_len = line_end::line_ending_length_at(bytes, i, mode);
        if le_len > 0 {
            let end = i + le_len;
            boundaries.push(RecordBoundary {
                start: record_start as u64,
                byte_length: (end - record_start) as u32,
            });
            record_start = end;
            i = end;
        } else {
            i += 1;
        }
    }
    // Trailing record: the bytes after the final terminator (or the whole slice
    // if there was no terminator). Always present, even when empty, so the
    // record count matches the line count (line_index pushes a final line start).
    boundaries.push(RecordBoundary {
        start: record_start as u64,
        byte_length: (bytes.len() - record_start) as u32,
    });
    boundaries
}

/// Frame fixed-length records arithmetically. A trailing partial slot (short
/// final record) is kept as its own boundary.
fn frame_fixed(bytes: &[u8], lrecl: u32) -> Vec<RecordBoundary> {
    if lrecl == 0 {
        return vec![RecordBoundary {
            start: 0,
            byte_length: 0,
        }];
    }
    let mut boundaries = Vec::new();
    let lrecl_usize = lrecl as usize;
    let mut offset = 0usize;
    while offset < bytes.len() {
        let remaining = bytes.len() - offset;
        let len = remaining.min(lrecl_usize);
        boundaries.push(RecordBoundary {
            start: offset as u64,
            byte_length: len as u32,
        });
        offset += len;
    }
    if boundaries.is_empty() {
        boundaries.push(RecordBoundary {
            start: 0,
            byte_length: 0,
        });
    }
    boundaries
}

/// The exact delimiter bytes to re-emit for a terminator (re-baseline helper).
pub fn reemit_terminator(terminator: DelimiterTerminator) -> &'static [u8] {
    terminator.bytes()
}

#[cfg(test)]
#[path = "record_format_tests.rs"]
mod tests;
