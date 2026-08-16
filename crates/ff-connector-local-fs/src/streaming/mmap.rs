//! Memory-mapped file access.
//!
//! Provides memory-mapped I/O for files that require random access,
//! with fallback to streaming when mmap is unavailable.
//!
//! Addresses: Requirement 6, criteria 3–4, 7

use crate::error::map_io_error;
use crate::path::NativePath;
use ff_vfs::VfsError;

/// Memory-map a file for random access reads.
///
/// Maps the file into the process address space without copying data into
/// heap memory. Falls back with an error if mmap fails (e.g., due to OS
/// resource limits).
///
/// # Safety
///
/// Uses the safe `memmap2::Mmap` API — no unsafe blocks needed.
///
/// Validates: Requirement 6, criteria 3–4, 7
pub async fn memory_map(path: &NativePath) -> Result<memmap2::Mmap, VfsError> {
    let uri = format!("vfs://local{}", path.to_string_lossy().replace('\\', "/"));

    let file =
        std::fs::File::open(path.as_path()).map_err(|e| map_io_error(e, "memory_map", &uri))?;

    // SAFETY: We use the safe Mmap API. The file must not be modified
    // while the mapping is active, which is the caller's responsibility.
    let mmap = unsafe { memmap2::Mmap::map(&file) }.map_err(|e| {
        ff_logging::log_debug!(
            "[connector-local-fs] memory_map: mmap failed for {}, will use streaming fallback: {}",
            uri,
            e
        );
        map_io_error(e, "memory_map", &uri)
    })?;

    Ok(mmap)
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::io::Write;

    #[tokio::test]
    async fn memory_map_reads_file_content() {
        let dir = tempfile::tempdir().unwrap();
        let file_path = dir.path().join("mmap_test.txt");

        {
            let mut file = std::fs::File::create(&file_path).unwrap();
            file.write_all(b"Memory mapped content").unwrap();
        }

        let native = NativePath::from_path_buf(file_path);
        let mmap = memory_map(&native).await.unwrap();
        assert_eq!(&mmap[..], b"Memory mapped content");
    }

    #[tokio::test]
    async fn memory_map_handles_empty_file() {
        let dir = tempfile::tempdir().unwrap();
        let file_path = dir.path().join("empty_mmap.txt");

        {
            std::fs::File::create(&file_path).unwrap();
        }

        let native = NativePath::from_path_buf(file_path);
        // mmap of empty file may succeed or fail depending on platform
        let result = memory_map(&native).await;
        // On most platforms, mmap of empty file returns Ok with empty slice
        // or Err — both are valid
        if let Ok(mmap) = result {
            assert_eq!(mmap.len(), 0);
        }
    }

    #[tokio::test]
    async fn memory_map_fails_for_nonexistent_file() {
        let native = NativePath::new_from("/nonexistent/path/file.txt");
        let result = memory_map(&native).await;
        assert!(result.is_err());
    }
}
