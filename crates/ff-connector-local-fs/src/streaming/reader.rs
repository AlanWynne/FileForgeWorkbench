//! ChunkedReader — async reader that yields file content in configurable chunks.
//!
//! Implements `AsyncRead` for standard async stream consumption.
//!
//! Addresses: Requirement 6, criteria 1–2, 8

use std::io;
use std::pin::Pin;
use std::task::{Context, Poll};

use tokio::fs::File;
use tokio::io::{AsyncRead, ReadBuf};

use crate::error::map_io_error;
use crate::path::NativePath;
use ff_vfs::VfsError;

/// An async reader that yields file content in configurable chunks.
///
/// Implements `AsyncRead` for standard async stream consumption and
/// supports progress callbacks for UI progress reporting.
///
/// Addresses: Requirement 6, criteria 1–2, 8
pub struct ChunkedReader {
    /// The underlying Tokio file handle.
    file: File,
    /// Chunk size in bytes (used for progress granularity guidance).
    chunk_size: usize,
    /// Total file size for progress calculation.
    total_size: u64,
    /// Bytes read so far.
    bytes_read: u64,
    /// Optional progress callback (bytes_read, total_size).
    progress_callback: Option<Box<dyn Fn(u64, u64) + Send>>,
}

impl ChunkedReader {
    /// Open a file for chunked reading.
    ///
    /// # Arguments
    ///
    /// * `path` - The native path to the file.
    /// * `chunk_size` - The chunk size in bytes.
    /// * `progress` - Optional progress callback.
    pub async fn open(
        path: &NativePath,
        chunk_size: usize,
        progress: Option<Box<dyn Fn(u64, u64) + Send>>,
    ) -> Result<Self, VfsError> {
        let uri = format!("vfs://local{}", path.to_string_lossy().replace('\\', "/"));

        let file = File::open(path.as_path())
            .await
            .map_err(|e| map_io_error(e, "read_stream", &uri))?;

        let metadata = file
            .metadata()
            .await
            .map_err(|e| map_io_error(e, "read_stream", &uri))?;
        let total_size = metadata.len();

        Ok(Self {
            file,
            chunk_size,
            total_size,
            bytes_read: 0,
            progress_callback: progress,
        })
    }

    /// Returns the total file size.
    pub fn total_size(&self) -> u64 {
        self.total_size
    }

    /// Returns the number of bytes read so far.
    pub fn bytes_read(&self) -> u64 {
        self.bytes_read
    }

    /// Returns the configured chunk size.
    pub fn chunk_size(&self) -> usize {
        self.chunk_size
    }
}

impl AsyncRead for ChunkedReader {
    fn poll_read(
        mut self: Pin<&mut Self>,
        cx: &mut Context<'_>,
        buf: &mut ReadBuf<'_>,
    ) -> Poll<io::Result<()>> {
        let before = buf.filled().len();
        let result = Pin::new(&mut self.file).poll_read(cx, buf);

        if let Poll::Ready(Ok(())) = &result {
            let after = buf.filled().len();
            let read_this_call = (after - before) as u64;
            self.bytes_read += read_this_call;

            if let Some(ref callback) = self.progress_callback {
                callback(self.bytes_read, self.total_size);
            }
        }

        result
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::sync::atomic::{AtomicU64, Ordering};
    use std::sync::Arc;
    use tokio::io::AsyncReadExt;

    #[tokio::test]
    async fn chunked_reader_reads_entire_file() {
        let dir = tempfile::tempdir().unwrap();
        let file_path = dir.path().join("test.txt");
        let content = b"Hello, World! This is test content for chunked reading.";
        tokio::fs::write(&file_path, content).await.unwrap();

        let native = NativePath::from_path_buf(file_path);
        let mut reader = ChunkedReader::open(&native, 16, None).await.unwrap();

        let mut buf = Vec::new();
        reader.read_to_end(&mut buf).await.unwrap();
        assert_eq!(buf, content);
    }

    #[tokio::test]
    async fn chunked_reader_reports_progress() {
        let dir = tempfile::tempdir().unwrap();
        let file_path = dir.path().join("progress_test.txt");
        let content = vec![0u8; 1024];
        tokio::fs::write(&file_path, &content).await.unwrap();

        let last_reported = Arc::new(AtomicU64::new(0));
        let last_reported_clone = Arc::clone(&last_reported);

        let progress: Box<dyn Fn(u64, u64) + Send> = Box::new(move |bytes_read, _total| {
            last_reported_clone.store(bytes_read, Ordering::SeqCst);
        });

        let native = NativePath::from_path_buf(file_path);
        let mut reader = ChunkedReader::open(&native, 256, Some(progress))
            .await
            .unwrap();

        let mut buf = Vec::new();
        reader.read_to_end(&mut buf).await.unwrap();

        assert_eq!(last_reported.load(Ordering::SeqCst), 1024);
        assert_eq!(reader.total_size(), 1024);
    }

    #[tokio::test]
    async fn chunked_reader_handles_empty_file() {
        let dir = tempfile::tempdir().unwrap();
        let file_path = dir.path().join("empty.txt");
        tokio::fs::write(&file_path, b"").await.unwrap();

        let native = NativePath::from_path_buf(file_path);
        let mut reader = ChunkedReader::open(&native, 64, None).await.unwrap();

        let mut buf = Vec::new();
        reader.read_to_end(&mut buf).await.unwrap();
        assert!(buf.is_empty());
        assert_eq!(reader.total_size(), 0);
    }
}
