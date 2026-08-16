//! LoadTask — async streaming read via VFS with chunk delivery and progress.
//!
//! The LoadTask reads a resource from the VFS in configurable chunks and delivers
//! each chunk to a callback as it arrives. It supports large-file streaming mode,
//! cooperative cancellation, and progress reporting.

use std::sync::Arc;
use std::time::{Duration, Instant};

use tokio::io::AsyncReadExt;
use tokio::sync::watch;

use ff_vfs::{ResourceUri, Vfs, VfsError};

use crate::cancellation::IoCancellationToken;
use crate::error::IoError;
use crate::progress::{IoPhase, ProgressState, RateCalculator};
use crate::types::{ChunkSize, IoSuccess, LargeFileThreshold};

/// Options for a load operation.
#[derive(Debug, Clone, Default)]
pub struct LoadOptions {
    /// Override chunk size for this load (None = use config default).
    pub chunk_size: Option<ChunkSize>,
    /// Override large-file threshold for this load.
    pub large_file_threshold: Option<LargeFileThreshold>,
}

/// Callback type for receiving loaded chunks.
pub type ChunkCallback = Arc<dyn Fn(&[u8]) + Send + Sync>;

/// Execute the load operation asynchronously.
///
/// Reads a resource from the VFS in chunks, delivering each chunk to the callback
/// and reporting progress via the watch channel. Checks for cancellation before
/// each chunk read.
pub(crate) async fn execute_load(
    vfs: &Vfs,
    uri: &ResourceUri,
    chunk_size: ChunkSize,
    large_file_threshold: LargeFileThreshold,
    cancel_token: &IoCancellationToken,
    progress_tx: &watch::Sender<ProgressState>,
    chunk_callback: &ChunkCallback,
) -> Result<IoSuccess, IoError> {
    let start = Instant::now();
    let uri_str = uri.as_str();

    // Query file size via stat (may fail for streaming-only providers)
    let total_bytes = match vfs.stat(uri).await {
        Ok(metadata) => metadata.size,
        Err(_) => None,
    };

    let is_large_file = total_bytes
        .map(|size| size >= large_file_threshold.as_bytes())
        .unwrap_or(false);

    // Get read stream from VFS
    let mut stream = vfs
        .read_stream(uri)
        .await
        .map_err(|source| IoError::OpenFailed {
            uri: uri_str.clone(),
            description: "failed to open read stream".to_string(),
            source,
        })?;

    let chunk_bytes = chunk_size.as_bytes() as usize;
    let mut buffer = vec![0u8; chunk_bytes];
    let mut bytes_transferred: u64 = 0;
    let mut rate_calculator = RateCalculator::new();
    let mut last_progress_emit = Instant::now();
    let progress_throttle = if is_large_file {
        Duration::from_millis(50)
    } else {
        Duration::ZERO
    };

    loop {
        // Check cancellation before each chunk read
        if cancel_token.is_cancelled() {
            return Err(IoError::Cancelled {
                uri: uri_str.clone(),
                bytes_transferred,
            });
        }

        // Read a chunk
        let bytes_read = stream
            .read(&mut buffer)
            .await
            .map_err(|e| IoError::ReadChunkFailed {
                uri: uri_str.clone(),
                description: format!("read error: {}", e),
                bytes_transferred,
                source: VfsError::Io {
                    uri: uri_str.clone(),
                    operation: "read_stream".to_string(),
                    source: e,
                },
            })?;

        if bytes_read == 0 {
            break; // EOF
        }

        // Deliver chunk to consumer
        chunk_callback(&buffer[..bytes_read]);
        bytes_transferred += bytes_read as u64;

        // Emit progress (with throttling for large files)
        let elapsed = start.elapsed();
        let now = Instant::now();
        if now.duration_since(last_progress_emit) >= progress_throttle {
            let percentage = ProgressState::calculate_percentage(bytes_transferred, total_bytes);
            let estimated_remaining =
                rate_calculator.update(bytes_transferred, total_bytes, elapsed);

            let _ = progress_tx.send(ProgressState {
                bytes_transferred,
                total_bytes,
                percentage,
                elapsed,
                estimated_remaining,
                phase: IoPhase::Reading,
            });
            last_progress_emit = now;
        }
    }

    // Emit final progress
    let elapsed = start.elapsed();
    let _ = progress_tx.send(ProgressState {
        bytes_transferred,
        total_bytes: total_bytes.or(Some(bytes_transferred)),
        percentage: Some(100),
        elapsed,
        estimated_remaining: Some(Duration::ZERO),
        phase: IoPhase::Complete,
    });

    Ok(IoSuccess {
        bytes_transferred,
        elapsed,
        uri: uri.clone(),
    })
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn load_options_default_has_none_overrides() {
        let opts = LoadOptions::default();
        assert!(opts.chunk_size.is_none());
        assert!(opts.large_file_threshold.is_none());
    }
}
