//! ChunkRenderer — subdivides visible portions of long lines into render chunks.
//!
//! Adapted from Scintilla's `BreakFinder` with `lengthStartSubdivision = 300`.

use crate::types::{ChunkRange, RenderChunkSize};

/// Subdivides the visible portion of a long line into render chunks
/// of manageable length for efficient text drawing and hit-testing.
pub struct ChunkRenderer {
    /// Maximum characters per render chunk.
    chunk_size: RenderChunkSize,
}

impl ChunkRenderer {
    /// Create a ChunkRenderer with the given maximum chunk size.
    pub fn new(chunk_size: RenderChunkSize) -> Self {
        Self { chunk_size }
    }

    /// Subdivide a character range into render chunks.
    ///
    /// Returns a Vec of ChunkRange values, each ≤ chunk_size characters.
    /// The union of all chunks equals the original range (complete partition).
    pub fn subdivide(&self, range: ChunkRange) -> Vec<ChunkRange> {
        if range.is_empty() {
            return vec![];
        }

        let chunk_size = self.chunk_size.0 as u64;
        let mut chunks = Vec::new();
        let mut start = range.start.0;

        while start < range.end.0 {
            let end = (start + chunk_size).min(range.end.0);
            chunks.push(ChunkRange::new(start, end));
            start = end;
        }

        chunks
    }

    /// Determine the render chunk containing a given character offset.
    pub fn chunk_containing(
        &self,
        range: ChunkRange,
        offset: crate::types::CharOffset,
    ) -> ChunkRange {
        let chunk_size = self.chunk_size.0 as u64;
        let rel = offset.0.saturating_sub(range.start.0);
        let chunk_start = range.start.0 + (rel / chunk_size) * chunk_size;
        let chunk_end = (chunk_start + chunk_size).min(range.end.0);
        ChunkRange::new(chunk_start, chunk_end)
    }
}

impl Default for ChunkRenderer {
    fn default() -> Self {
        Self::new(RenderChunkSize::default())
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::types::CharOffset;

    #[test]
    fn subdivide_exact_multiple() {
        // Validates: Requirement 1 AC 6 — Property 8: Render Chunk Partition Completeness
        let renderer = ChunkRenderer::new(RenderChunkSize::new(300));
        let range = ChunkRange::new(0, 900);
        let chunks = renderer.subdivide(range);
        assert_eq!(chunks.len(), 3);
        assert_eq!(chunks[0], ChunkRange::new(0, 300));
        assert_eq!(chunks[1], ChunkRange::new(300, 600));
        assert_eq!(chunks[2], ChunkRange::new(600, 900));
    }

    #[test]
    fn subdivide_with_remainder() {
        // Validates: Requirement 1 AC 6 — Property 8
        let renderer = ChunkRenderer::new(RenderChunkSize::new(300));
        let range = ChunkRange::new(0, 700);
        let chunks = renderer.subdivide(range);
        assert_eq!(chunks.len(), 3);
        assert_eq!(chunks[2], ChunkRange::new(600, 700));
    }

    #[test]
    fn subdivide_empty_range() {
        let renderer = ChunkRenderer::default();
        let chunks = renderer.subdivide(ChunkRange::new(5, 5));
        assert!(chunks.is_empty());
    }

    #[test]
    fn subdivide_partition_completeness() {
        // Validates: Property 8 — union of chunks equals original range
        let renderer = ChunkRenderer::new(RenderChunkSize::new(100));
        let range = ChunkRange::new(50, 450);
        let chunks = renderer.subdivide(range);

        // First chunk starts at range start
        assert_eq!(chunks.first().unwrap().start.0, 50);
        // Last chunk ends at range end
        assert_eq!(chunks.last().unwrap().end.0, 450);
        // No gaps
        for i in 1..chunks.len() {
            assert_eq!(chunks[i].start.0, chunks[i - 1].end.0);
        }
        // All chunks within size limit
        for chunk in &chunks {
            assert!(chunk.len() <= 100);
        }
    }

    #[test]
    fn chunk_containing_finds_correct_chunk() {
        let renderer = ChunkRenderer::new(RenderChunkSize::new(300));
        let range = ChunkRange::new(0, 900);
        let chunk = renderer.chunk_containing(range, CharOffset(350));
        assert_eq!(chunk, ChunkRange::new(300, 600));
    }

    #[test]
    fn single_chunk_when_range_smaller_than_chunk_size() {
        let renderer = ChunkRenderer::new(RenderChunkSize::new(300));
        let range = ChunkRange::new(0, 100);
        let chunks = renderer.subdivide(range);
        assert_eq!(chunks.len(), 1);
        assert_eq!(chunks[0], ChunkRange::new(0, 100));
    }
}
