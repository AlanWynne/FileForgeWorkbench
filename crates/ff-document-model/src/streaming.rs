//! Streaming file loading from the VFS.
//!
//! Provides `StreamingFileReader` that reads from VFS in configurable chunks,
//! feeding the GapBuffer and SparseLineIndex incrementally.

use std::pin::Pin;

use tokio::io::AsyncRead;
use tokio::io::AsyncReadExt;
use tokio_util::sync::CancellationToken;

use ff_vfs::{ResourceUri, Vfs};

use crate::error::DocumentError;
use crate::gap_buffer::GapBuffer;
use crate::line_end::LineEndMode;
use crate::sparse_line_index::SparseLineIndex;
use crate::types::LoadingProgress;

/// Default chunk size for streaming reads (64 KB).
const DEFAULT_CHUNK_SIZE: usize = 65536;

/// Async chunked file reader that loads content from the VFS.
#[derive(Debug)]
pub struct StreamingFileReader {
    /// Chunk size in bytes.
    chunk_size: usize,
    /// Cancellation token for cooperative shutdown.
    cancel_token: CancellationToken,
}

impl StreamingFileReader {
    /// Create a reader with the specified chunk size.
    pub fn new(chunk_size: usize) -> Self {
        Self {
            chunk_size: chunk_size.max(1024),
            cancel_token: CancellationToken::new(),
        }
    }

    /// Create with default chunk size (64 KB).
    pub fn with_defaults() -> Self {
        Self::new(DEFAULT_CHUNK_SIZE)
    }

    /// Load a file from the VFS into the provided buffer and sparse index.
    pub async fn load(
        &self,
        vfs: &Vfs,
        uri: &ResourceUri,
        buffer: &mut GapBuffer,
        sparse_index: &mut SparseLineIndex,
        mode: LineEndMode,
    ) -> Result<LoadingProgress, DocumentError> {
        let mut stream: Pin<Box<dyn AsyncRead + Send>> =
            vfs.read_stream(uri)
                .await
                .map_err(|e| DocumentError::VfsIo {
                    operation: "read_stream".to_string(),
                    uri: uri.to_string(),
                    source: e,
                })?;

        let mut bytes_loaded: u64 = 0;
        let mut chunk_buf = vec![0u8; self.chunk_size];

        loop {
            // Check for cancellation
            if self.cancel_token.is_cancelled() {
                return Err(DocumentError::LoadCancelled { bytes_loaded });
            }

            let n = match stream.read(&mut chunk_buf).await {
                Ok(0) => break, // EOF
                Ok(n) => n,
                Err(e) => {
                    return Ok(LoadingProgress::Failed {
                        reason: e.to_string(),
                        bytes_loaded,
                    });
                }
            };

            let chunk = &chunk_buf[..n];

            // Append to gap buffer
            buffer.insert(buffer.length(), chunk);

            // Update sparse index
            sparse_index.process_chunk(chunk, mode);

            bytes_loaded += n as u64;
        }

        Ok(LoadingProgress::Complete {
            total_bytes: bytes_loaded,
            total_lines: sparse_index.lines_counted() + 1,
        })
    }

    /// Cancel an in-progress load.
    pub fn cancel(&self) {
        self.cancel_token.cancel();
    }

    /// Check if cancellation has been requested.
    pub fn is_cancelled(&self) -> bool {
        self.cancel_token.is_cancelled()
    }

    /// Get a clone of the cancellation token.
    pub fn cancel_token(&self) -> CancellationToken {
        self.cancel_token.clone()
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::pin::Pin;
    use std::task::{Context, Poll};
    use tokio::io::ReadBuf;

    /// A mock AsyncRead that yields data in configured chunks.
    struct MockAsyncReader {
        data: Vec<u8>,
        position: usize,
        chunk_size: usize,
    }

    impl MockAsyncReader {
        fn new(data: Vec<u8>, chunk_size: usize) -> Self {
            Self {
                data,
                position: 0,
                chunk_size,
            }
        }
    }

    impl AsyncRead for MockAsyncReader {
        fn poll_read(
            mut self: Pin<&mut Self>,
            _cx: &mut Context<'_>,
            buf: &mut ReadBuf<'_>,
        ) -> Poll<std::io::Result<()>> {
            let remaining = &self.data[self.position..];
            if remaining.is_empty() {
                return Poll::Ready(Ok(()));
            }
            let to_read = remaining.len().min(self.chunk_size).min(buf.remaining());
            buf.put_slice(&remaining[..to_read]);
            self.position += to_read;
            Poll::Ready(Ok(()))
        }
    }

    // Note: Full streaming integration tests require a mock VFS provider.
    // Unit tests here verify the reader logic with simple scenarios.

    #[test]
    fn reader_defaults() {
        let reader = StreamingFileReader::with_defaults();
        assert_eq!(reader.chunk_size, DEFAULT_CHUNK_SIZE);
        assert!(!reader.is_cancelled());
    }

    #[test]
    fn cancel_sets_flag() {
        let reader = StreamingFileReader::with_defaults();
        assert!(!reader.is_cancelled());
        reader.cancel();
        assert!(reader.is_cancelled());
    }
}
