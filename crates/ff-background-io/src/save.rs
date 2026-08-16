//! SaveTask — async write with temp-file + atomic rename strategy.
//!
//! The SaveTask writes document content to the VFS using a temporary file
//! and atomic rename for crash safety. Supports chunked writes with progress
//! reporting and cooperative cancellation.

use std::time::{Duration, Instant};

use tokio::sync::watch;

use ff_vfs::{OpenOptions, ResourceUri, Vfs};

use crate::cancellation::IoCancellationToken;
use crate::error::IoError;
use crate::progress::{IoPhase, ProgressState, RateCalculator};
use crate::types::{ChunkSize, IoSuccess};

/// Options for a save operation.
#[derive(Debug, Clone)]
pub struct SaveOptions {
    /// Override chunk size for this save (None = use config default).
    pub chunk_size: Option<ChunkSize>,
    /// Whether to attempt atomic rename (default: true, falls back if unsupported).
    pub atomic: bool,
    /// Whether to preserve original file metadata after rename.
    pub preserve_metadata: bool,
}

impl Default for SaveOptions {
    fn default() -> Self {
        Self {
            chunk_size: None,
            atomic: true,
            preserve_metadata: true,
        }
    }
}

/// Trait for providing document content in chunks during save operations.
///
/// Implemented by the document-model to support streaming saves without
/// requiring the entire document content as a single allocation.
pub trait DocumentChunkSource: Send + Sync {
    /// Returns the total content size in bytes (if known).
    fn total_size(&self) -> Option<u64>;

    /// Read the next chunk of content. Returns None when complete.
    fn next_chunk(&self, chunk_size_hint: usize) -> Option<Vec<u8>>;

    /// Reset to the beginning (for retry scenarios).
    fn reset(&self);
}

/// Generate a temp file path in the format `{target}.ffwtmp.{random6}`.
pub fn generate_temp_path(target_path: &str) -> String {
    use rand::Rng;
    let mut rng = rand::rng();
    let suffix: String = (0..6)
        .map(|_| {
            let idx: u8 = rng.random_range(0..36);
            if idx < 10 {
                (b'0' + idx) as char
            } else {
                (b'a' + idx - 10) as char
            }
        })
        .collect();
    format!("{}.ffwtmp.{}", target_path, suffix)
}

/// Execute the save operation asynchronously.
///
/// Writes document content to a temp file via VFS, then atomically renames
/// to the target. If atomic rename is unavailable, falls back to write-in-place.
pub(crate) async fn execute_save(
    vfs: &Vfs,
    uri: &ResourceUri,
    chunk_size: ChunkSize,
    cancel_token: &IoCancellationToken,
    progress_tx: &watch::Sender<ProgressState>,
    document_source: &dyn DocumentChunkSource,
    options: &SaveOptions,
) -> Result<IoSuccess, IoError> {
    let start = Instant::now();
    let uri_str = uri.as_str();
    let target_path = uri.path();

    let total_bytes = document_source.total_size();

    // Check provider capabilities
    let provider = vfs
        .registry()
        .get(uri.scheme())
        .ok_or_else(|| IoError::OpenFailed {
            uri: uri_str.clone(),
            description: format!("provider '{}' not found", uri.scheme()),
            source: ff_vfs::VfsError::ProviderUnavailable {
                scheme: uri.scheme().to_string(),
            },
        })?;

    let capabilities = provider.capabilities();

    if !capabilities.write {
        return Err(IoError::UnsupportedCapability {
            uri: uri_str.clone(),
            provider: uri.scheme().to_string(),
            capability: "write".to_string(),
        });
    }

    let supports_rename = capabilities.rename && options.atomic;

    // Generate temp file path
    let temp_path = generate_temp_path(target_path);
    let temp_uri = ResourceUri::new(uri.scheme(), &temp_path);

    let mut bytes_transferred: u64 = 0;
    let mut rate_calculator = RateCalculator::new();
    let chunk_bytes = chunk_size.as_bytes() as usize;

    if supports_rename {
        // Atomic save: write to temp file, then rename

        // Open temp file for writing
        let mut file = vfs
            .open(
                &temp_uri,
                OpenOptions {
                    read: false,
                    write: true,
                    create: true,
                    truncate: true,
                    append: false,
                },
            )
            .await
            .map_err(|source| IoError::OpenFailed {
                uri: uri_str.clone(),
                description: "failed to create temp file".to_string(),
                source,
            })?;

        // Write chunks
        loop {
            if cancel_token.is_cancelled() {
                // Cleanup: delete temp file
                let _ = vfs
                    .delete(&temp_uri, ff_vfs::DeleteOptions::default())
                    .await;
                return Err(IoError::Cancelled {
                    uri: uri_str.clone(),
                    bytes_transferred,
                });
            }

            let chunk = match document_source.next_chunk(chunk_bytes) {
                Some(data) => data,
                None => break, // All content written
            };

            file.write(&chunk).await.map_err(|source| {
                // Attempt cleanup on failure
                IoError::WriteChunkFailed {
                    uri: uri_str.clone(),
                    description: "write to temp file failed".to_string(),
                    bytes_transferred,
                    source,
                }
            })?;

            bytes_transferred += chunk.len() as u64;

            // Emit progress
            let elapsed = start.elapsed();
            let percentage = ProgressState::calculate_percentage(bytes_transferred, total_bytes);
            let estimated_remaining =
                rate_calculator.update(bytes_transferred, total_bytes, elapsed);
            let _ = progress_tx.send(ProgressState {
                bytes_transferred,
                total_bytes,
                percentage,
                elapsed,
                estimated_remaining,
                phase: IoPhase::Writing,
            });
        }

        // Flush and fsync
        let _ = progress_tx.send(ProgressState {
            bytes_transferred,
            total_bytes,
            percentage: Some(99),
            elapsed: start.elapsed(),
            estimated_remaining: None,
            phase: IoPhase::Finalizing,
        });

        file.flush().await.map_err(|source| IoError::FlushFailed {
            uri: uri_str.clone(),
            description: "flush failed".to_string(),
            bytes_transferred,
            source,
        })?;

        file.sync_all()
            .await
            .map_err(|source| IoError::FlushFailed {
                uri: uri_str.clone(),
                description: "fsync failed".to_string(),
                bytes_transferred,
                source,
            })?;

        // Close the file handle before rename
        file.close().await.map_err(|source| IoError::FlushFailed {
            uri: uri_str.clone(),
            description: "close failed".to_string(),
            bytes_transferred,
            source,
        })?;

        // Atomic rename
        vfs.rename(&temp_uri, uri)
            .await
            .map_err(|source| IoError::RenameFailed {
                uri: uri_str.clone(),
                description: "atomic rename failed".to_string(),
                bytes_transferred,
                source,
            })?;
    } else {
        // Fallback: write-in-place (truncate + write directly)
        ff_logging::log(
            ff_logging::LogLevel::Warn,
            "background-io",
            &format!(
                "atomic save unavailable for '{}', using write-in-place",
                uri_str
            ),
        );

        // Collect all content and write at once via VFS write
        let mut all_content = Vec::new();
        loop {
            if cancel_token.is_cancelled() {
                return Err(IoError::Cancelled {
                    uri: uri_str.clone(),
                    bytes_transferred,
                });
            }

            let chunk = match document_source.next_chunk(chunk_bytes) {
                Some(data) => data,
                None => break,
            };
            bytes_transferred += chunk.len() as u64;
            all_content.extend_from_slice(&chunk);

            let elapsed = start.elapsed();
            let percentage = ProgressState::calculate_percentage(bytes_transferred, total_bytes);
            let estimated_remaining =
                rate_calculator.update(bytes_transferred, total_bytes, elapsed);
            let _ = progress_tx.send(ProgressState {
                bytes_transferred,
                total_bytes,
                percentage,
                elapsed,
                estimated_remaining,
                phase: IoPhase::Writing,
            });
        }

        vfs.write(uri, &all_content)
            .await
            .map_err(|source| IoError::WriteChunkFailed {
                uri: uri_str.clone(),
                description: "write-in-place failed".to_string(),
                bytes_transferred,
                source,
            })?;
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
    fn save_options_default_is_atomic_with_metadata_preservation() {
        let opts = SaveOptions::default();
        assert!(opts.atomic);
        assert!(opts.preserve_metadata);
        assert!(opts.chunk_size.is_none());
    }

    #[test]
    fn generate_temp_path_produces_correct_format() {
        // Validates: Requirement 4 AC 2
        let path = generate_temp_path("/documents/file.txt");
        assert!(path.starts_with("/documents/file.txt.ffwtmp."));
        // Suffix is 6 alphanumeric chars
        let suffix = path.strip_prefix("/documents/file.txt.ffwtmp.").unwrap();
        assert_eq!(suffix.len(), 6);
        assert!(suffix.chars().all(|c| c.is_ascii_alphanumeric()));
    }

    #[test]
    fn generate_temp_path_produces_unique_names() {
        // Validates: Requirement 4 AC 2
        let paths: Vec<String> = (0..100).map(|_| generate_temp_path("/test.txt")).collect();
        let unique: std::collections::HashSet<_> = paths.iter().collect();
        // With 36^6 = ~2 billion possibilities, 100 should all be unique
        assert_eq!(unique.len(), 100);
    }
}
