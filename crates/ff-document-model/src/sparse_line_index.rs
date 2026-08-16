//! Sparse line index for incremental background indexing during streaming loads.
//!
//! Records one checkpoint per N lines (default 1000), enabling partial
//! line lookups before the full index is available.

use crate::gap_buffer::GapBuffer;
use crate::line_end::{self, LineEndMode};
use crate::line_index::LineIndex;
use crate::types::{BytePosition, LineNumber};

/// Default checkpoint interval (one entry per N lines).
const DEFAULT_CHECKPOINT_INTERVAL: u64 = 1000;

/// Incremental checkpoint index built during streaming file loading.
#[derive(Debug, Clone)]
pub struct SparseLineIndex {
    /// Checkpoint entries: (line_number, byte_position).
    checkpoints: Vec<(u64, u64)>,
    /// Lines per checkpoint.
    checkpoint_interval: u64,
    /// Total lines seen so far.
    total_lines: u64,
    /// Total bytes processed.
    bytes_processed: u64,
    /// Byte offset of the last incomplete line ending check (for cross-chunk CRLF).
    pending_cr: bool,
}

impl SparseLineIndex {
    /// Create a new sparse index with the given checkpoint interval.
    pub fn new(checkpoint_interval: u64) -> Self {
        Self {
            checkpoints: vec![(0, 0)], // Line 0 always at position 0
            checkpoint_interval: checkpoint_interval.max(1),
            total_lines: 0,
            bytes_processed: 0,
            pending_cr: false,
        }
    }

    /// Create with default checkpoint interval.
    pub fn with_default_interval() -> Self {
        Self::new(DEFAULT_CHECKPOINT_INTERVAL)
    }

    /// Process a chunk of bytes, recording checkpoints as line endings are found.
    pub fn process_chunk(&mut self, chunk: &[u8], mode: LineEndMode) {
        let chunk_start_offset = self.bytes_processed;
        let mut i = 0;

        // Handle pending CR from previous chunk
        if self.pending_cr && !chunk.is_empty() {
            if chunk[0] == 0x0A {
                // CRLF across chunk boundary - line was already counted for CR
                i = 1;
            }
            self.pending_cr = false;
        }

        while i < chunk.len() {
            let le_len = line_end::line_ending_length_at(chunk, i, mode);
            if le_len > 0 {
                self.total_lines += 1;
                let line_start_pos = chunk_start_offset + (i as u64) + (le_len as u64);

                // Record checkpoint if at interval
                if self.total_lines.is_multiple_of(self.checkpoint_interval) {
                    self.checkpoints.push((self.total_lines, line_start_pos));
                }

                i += le_len;
            } else {
                i += 1;
            }
        }

        // Check if chunk ends with CR (potential CRLF split)
        if !chunk.is_empty() && chunk[chunk.len() - 1] == 0x0D {
            self.pending_cr = true;
        }

        self.bytes_processed += chunk.len() as u64;
    }

    /// Finalize into a complete LineIndex by rescanning the full buffer.
    pub fn finalize(self, buffer: &mut GapBuffer, mode: LineEndMode) -> LineIndex {
        let mut index = LineIndex::new();
        index.rebuild_from_buffer(buffer, mode);
        index
    }

    /// Query an approximate line number for a byte position using checkpoints.
    pub fn approximate_line(&self, position: BytePosition) -> Option<LineNumber> {
        if self.checkpoints.is_empty() {
            return None;
        }
        // Find the last checkpoint with byte_position <= position
        let pos = position.0;
        let idx = self.checkpoints.partition_point(|&(_, bp)| bp <= pos);
        if idx == 0 {
            Some(LineNumber(0))
        } else {
            let (line_num, _) = self.checkpoints[idx - 1];
            Some(LineNumber(line_num))
        }
    }

    /// Total lines counted so far.
    pub fn lines_counted(&self) -> u64 {
        self.total_lines
    }

    /// Total bytes processed.
    pub fn bytes_processed(&self) -> u64 {
        self.bytes_processed
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn new_sparse_index_starts_empty() {
        let idx = SparseLineIndex::new(100);
        assert_eq!(idx.lines_counted(), 0);
        assert_eq!(idx.bytes_processed(), 0);
    }

    #[test]
    fn process_chunk_counts_lines() {
        let mut idx = SparseLineIndex::new(2);
        idx.process_chunk(b"line1\nline2\nline3\n", LineEndMode::Default);
        assert_eq!(idx.lines_counted(), 3);
    }

    #[test]
    fn checkpoints_at_interval() {
        let mut idx = SparseLineIndex::new(2);
        idx.process_chunk(b"a\nb\nc\nd\ne\n", LineEndMode::Default);
        // 5 lines total. Checkpoints at lines 2 and 4.
        assert_eq!(idx.lines_counted(), 5);
        assert!(idx.checkpoints.len() >= 3); // (0,0) + checkpoint at line 2 + checkpoint at line 4
    }

    #[test]
    fn crlf_across_chunk_boundary() {
        let mut idx = SparseLineIndex::new(100);
        idx.process_chunk(b"hello\r", LineEndMode::Default);
        idx.process_chunk(b"\nworld\n", LineEndMode::Default);
        // "hello\r\n" = 1 line ending, "world\n" = 1 line ending = 2 total
        assert_eq!(idx.lines_counted(), 2);
    }

    #[test]
    fn approximate_line_lookup() {
        let mut idx = SparseLineIndex::new(2);
        // Each line is 6 bytes: "lineN\n"
        idx.process_chunk(b"line1\nline2\nline3\nline4\n", LineEndMode::Default);
        // Lines: 0@0, 1@6, 2@12, 3@18 (all start positions)
        // Checkpoints at lines 2 and 4

        let approx = idx.approximate_line(BytePosition(7));
        assert!(approx.is_some());
    }

    #[test]
    fn finalize_produces_correct_index() {
        let mut buf = GapBuffer::new(256);
        buf.insert(0, b"abc\ndef\nghi");

        let mut sparse = SparseLineIndex::new(2);
        sparse.process_chunk(b"abc\ndef\nghi", LineEndMode::Default);

        let index = sparse.finalize(&mut buf, LineEndMode::Default);
        assert_eq!(index.line_count(), 3);
    }
}
