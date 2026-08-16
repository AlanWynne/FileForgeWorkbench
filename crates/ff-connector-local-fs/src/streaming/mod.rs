//! Streaming I/O — chunked async reader, atomic writer, and memory-mapped access.
//!
//! Addresses: Requirement 6 (Large File Support), Requirement 1 AC 4 (atomic write)

pub mod mmap;
pub mod reader;
pub mod writer;

pub use mmap::memory_map;
pub use reader::ChunkedReader;
pub use writer::AtomicWriter;

use crate::path::NativePath;
use ff_vfs::VfsError;

/// Manages streaming I/O operations with configurable chunk sizes and mmap support.
///
/// Addresses: Requirement 6, all acceptance criteria
pub struct StreamingManager {
    /// Default chunk size for reads in bytes.
    chunk_size: usize,
    /// Whether memory-mapped I/O is enabled.
    enable_mmap: bool,
}

impl StreamingManager {
    /// Construct a new `StreamingManager` with the given configuration.
    pub fn new(chunk_size: usize, enable_mmap: bool) -> Self {
        Self {
            chunk_size,
            enable_mmap,
        }
    }

    /// Create a `ChunkedReader` for streaming file reads.
    ///
    /// Validates: Requirement 6, criteria 1–2, 8
    pub async fn open_reader(
        &self,
        path: &NativePath,
        progress: Option<Box<dyn Fn(u64, u64) + Send>>,
    ) -> Result<ChunkedReader, VfsError> {
        ChunkedReader::open(path, self.chunk_size, progress).await
    }

    /// Create an `AtomicWriter` for safe file writes.
    ///
    /// Validates: Requirement 1, criterion 4
    pub async fn open_atomic_writer(&self, path: &NativePath) -> Result<AtomicWriter, VfsError> {
        AtomicWriter::new(path).await
    }

    /// Memory-map a file for random access reads.
    ///
    /// Falls back to returning an error if mmap is disabled or unavailable.
    ///
    /// Validates: Requirement 6, criteria 3–4, 7
    pub async fn memory_map(&self, path: &NativePath) -> Result<memmap2::Mmap, VfsError> {
        if !self.enable_mmap {
            return Err(VfsError::Io {
                uri: format!("vfs://local{}", path.to_string_lossy().replace('\\', "/")),
                operation: "memory_map".to_string(),
                source: std::io::Error::new(
                    std::io::ErrorKind::Unsupported,
                    "memory-mapped I/O is disabled by configuration",
                ),
            });
        }
        mmap::memory_map(path).await
    }

    /// Returns the configured chunk size.
    pub fn chunk_size(&self) -> usize {
        self.chunk_size
    }

    /// Returns whether mmap is enabled.
    pub fn is_mmap_enabled(&self) -> bool {
        self.enable_mmap
    }
}
