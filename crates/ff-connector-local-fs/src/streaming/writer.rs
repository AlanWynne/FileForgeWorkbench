//! AtomicWriter — performs atomic writes via temp file + rename strategy.
//!
//! Falls back to direct write if atomic rename is not supported on the target filesystem.
//!
//! Addresses: Requirement 1 AC 4, Requirement 6 AC 5

use std::path::PathBuf;

use tokio::io::AsyncWriteExt;

use crate::error::map_io_error;
use crate::path::NativePath;
use ff_vfs::VfsError;

/// Performs atomic writes by writing to a temporary file then renaming.
///
/// The strategy:
/// 1. Write data to a temporary file in the same directory as the target
/// 2. Rename the temp file to the target (atomic on most filesystems)
/// 3. If rename fails (e.g., cross-device), fall back to direct write
///
/// Addresses: Requirement 1 AC 4
pub struct AtomicWriter {
    /// Target path for the final file.
    target_path: PathBuf,
    /// Temporary file path (same directory as target).
    temp_path: PathBuf,
}

impl AtomicWriter {
    /// Create a new `AtomicWriter` for the given target path.
    pub async fn new(target: &NativePath) -> Result<Self, VfsError> {
        let target_path = target.as_path().to_path_buf();
        let parent = target_path.parent().unwrap_or(target_path.as_path());

        // Ensure parent directory exists
        tokio::fs::create_dir_all(parent).await.map_err(|e| {
            map_io_error(
                e,
                "write",
                &format!("vfs://local{}", target.to_string_lossy().replace('\\', "/")),
            )
        })?;

        // Generate temp file name in the same directory
        let file_name = target_path
            .file_name()
            .unwrap_or_default()
            .to_string_lossy();
        let temp_name = format!(".{}.tmp", file_name);
        let temp_path = parent.join(&temp_name);

        Ok(Self {
            target_path,
            temp_path,
        })
    }

    /// Write data atomically to the target path.
    ///
    /// Writes to a temp file first, then renames. Falls back to direct write
    /// if rename fails.
    ///
    /// Validates: Requirement 1 AC 4, Requirement 6 AC 5
    pub async fn write_all(&self, data: &[u8]) -> Result<(), VfsError> {
        let uri = format!(
            "vfs://local{}",
            self.target_path.to_string_lossy().replace('\\', "/")
        );

        // Step 1: Write to temp file
        let mut file = tokio::fs::File::create(&self.temp_path)
            .await
            .map_err(|e| map_io_error(e, "write", &uri))?;

        file.write_all(data)
            .await
            .map_err(|e| map_io_error(e, "write", &uri))?;

        file.sync_all()
            .await
            .map_err(|e| map_io_error(e, "write", &uri))?;

        drop(file);

        // Step 2: Rename temp to target (atomic)
        match tokio::fs::rename(&self.temp_path, &self.target_path).await {
            Ok(()) => Ok(()),
            Err(_rename_err) => {
                // Fallback: direct write (non-atomic)
                ff_logging::log_warn!(
                    "[connector-local-fs] write: atomic rename failed, falling back to direct write for {}",
                    uri
                );

                // Read the temp file and write directly
                let content = tokio::fs::read(&self.temp_path)
                    .await
                    .map_err(|e| map_io_error(e, "write", &uri))?;

                tokio::fs::write(&self.target_path, &content)
                    .await
                    .map_err(|e| map_io_error(e, "write", &uri))?;

                // Clean up temp file
                let _ = tokio::fs::remove_file(&self.temp_path).await;

                Ok(())
            }
        }
    }

    /// Write data in chunks (for streaming writes).
    ///
    /// Validates: Requirement 6 AC 5
    pub async fn write_chunked(
        &self,
        chunks: impl IntoIterator<Item = &[u8]>,
    ) -> Result<(), VfsError> {
        let uri = format!(
            "vfs://local{}",
            self.target_path.to_string_lossy().replace('\\', "/")
        );

        let mut file = tokio::fs::File::create(&self.temp_path)
            .await
            .map_err(|e| map_io_error(e, "write", &uri))?;

        for chunk in chunks {
            file.write_all(chunk)
                .await
                .map_err(|e| map_io_error(e, "write", &uri))?;
        }

        file.sync_all()
            .await
            .map_err(|e| map_io_error(e, "write", &uri))?;

        drop(file);

        // Atomic rename
        match tokio::fs::rename(&self.temp_path, &self.target_path).await {
            Ok(()) => Ok(()),
            Err(_) => {
                ff_logging::log_warn!(
                    "[connector-local-fs] write: atomic rename failed for chunked write, falling back to direct write"
                );
                let content = tokio::fs::read(&self.temp_path)
                    .await
                    .map_err(|e| map_io_error(e, "write", &uri))?;
                tokio::fs::write(&self.target_path, &content)
                    .await
                    .map_err(|e| map_io_error(e, "write", &uri))?;
                let _ = tokio::fs::remove_file(&self.temp_path).await;
                Ok(())
            }
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[tokio::test]
    async fn atomic_writer_creates_file_with_content() {
        let dir = tempfile::tempdir().unwrap();
        let target = dir.path().join("output.txt");
        let native = NativePath::from_path_buf(target.clone());

        let writer = AtomicWriter::new(&native).await.unwrap();
        writer.write_all(b"Hello, atomic write!").await.unwrap();

        let content = tokio::fs::read_to_string(&target).await.unwrap();
        assert_eq!(content, "Hello, atomic write!");
    }

    #[tokio::test]
    async fn atomic_writer_overwrites_existing_file() {
        let dir = tempfile::tempdir().unwrap();
        let target = dir.path().join("existing.txt");
        tokio::fs::write(&target, b"original content")
            .await
            .unwrap();

        let native = NativePath::from_path_buf(target.clone());
        let writer = AtomicWriter::new(&native).await.unwrap();
        writer.write_all(b"new content").await.unwrap();

        let content = tokio::fs::read_to_string(&target).await.unwrap();
        assert_eq!(content, "new content");
    }

    #[tokio::test]
    async fn atomic_writer_creates_parent_directories() {
        let dir = tempfile::tempdir().unwrap();
        let target = dir.path().join("nested").join("dir").join("file.txt");
        let native = NativePath::from_path_buf(target.clone());

        let writer = AtomicWriter::new(&native).await.unwrap();
        writer.write_all(b"nested write").await.unwrap();

        let content = tokio::fs::read_to_string(&target).await.unwrap();
        assert_eq!(content, "nested write");
    }

    #[tokio::test]
    async fn atomic_writer_chunked_write() {
        let dir = tempfile::tempdir().unwrap();
        let target = dir.path().join("chunked.txt");
        let native = NativePath::from_path_buf(target.clone());

        let writer = AtomicWriter::new(&native).await.unwrap();
        let chunks: Vec<&[u8]> = vec![b"chunk1", b"chunk2", b"chunk3"];
        writer.write_chunked(chunks).await.unwrap();

        let content = tokio::fs::read_to_string(&target).await.unwrap();
        assert_eq!(content, "chunk1chunk2chunk3");
    }
}
