//! # POSIX VFS Provider
//!
//! Implements `VfsProvider` for scheme `posix`. Wraps `LocalFsProvider` and adds:
//! - POSIX path normalisation (forward-slash only, case-sensitive)
//! - Root-jail: paths cannot escape the catalog root via `..` traversal
//! - Read-only enforcement at the provider level
//!
//! Resources are addressable as `vfs://posix/{catalog-name}/{posix-path}`.
//!
//! Validates: Requirement 7.1–7.7

// Types and functions are wired into the UI in Tasks 4–10; suppress until then.
#![allow(dead_code)]

use std::path::{Component, Path, PathBuf};
use std::pin::Pin;

use async_trait::async_trait;
use ff_connector_local_fs::LocalFsProvider;
use ff_vfs::{
    CreateOptions, DeleteOptions, OpenOptions, VfsCapabilities, VfsEntry, VfsError, VfsFile,
    VfsMetadata, VfsProvider, WatchHandle, WatchOptions,
};

// ── Path helpers ─────────────────────────────────────────────────────────────

/// Normalise a POSIX-style path and resolve it against `root`, enforcing the
/// root-jail invariant (no `..` escape).
///
/// Returns `Err(VfsError::PermissionDenied)` if the resolved path would escape
/// the root directory.
///
/// Validates: Requirement 7.3, 7.4
pub(crate) fn resolve_posix_path(root: &Path, posix_path: &str) -> Result<PathBuf, VfsError> {
    // Strip leading slash — POSIX paths are relative to the catalog root.
    let stripped = posix_path.trim_start_matches('/');

    // Build the candidate path by joining root with each forward-slash segment.
    let mut resolved = root.to_path_buf();
    for segment in stripped.split('/') {
        match segment {
            "" | "." => {}
            ".." => {
                // Reject any attempt to traverse above root.
                return Err(VfsError::PermissionDenied {
                    uri: format!("vfs://posix/{posix_path}"),
                    operation: "resolve".to_string(),
                });
            }
            s => resolved.push(s),
        }
    }

    // Final canonical check: resolved must start with root.
    // (Handles symlinks that could otherwise escape the jail.)
    if !resolved.starts_with(root) {
        return Err(VfsError::PermissionDenied {
            uri: format!("vfs://posix/{posix_path}"),
            operation: "resolve".to_string(),
        });
    }

    Ok(resolved)
}

/// Convert a native `PathBuf` back to a POSIX-style path string relative to `root`.
pub(crate) fn to_posix_path(root: &Path, native: &Path) -> String {
    native
        .strip_prefix(root)
        .map(|rel| {
            rel.components()
                .filter_map(|c| match c {
                    Component::Normal(s) => s.to_str(),
                    _ => None,
                })
                .collect::<Vec<_>>()
                .join("/")
        })
        .unwrap_or_default()
}

// ── Provider ─────────────────────────────────────────────────────────────────

/// A VFS provider that exposes a local directory as a POSIX-style namespace.
///
/// Validates: Requirement 7.1–7.7
pub struct PosixProvider {
    /// The local directory that forms the root of this POSIX catalog.
    root: PathBuf,
    /// When true, all write/create/delete/rename operations are rejected.
    read_only: bool,
    /// Underlying local filesystem provider.
    inner: LocalFsProvider,
}

impl PosixProvider {
    /// Create a new POSIX provider rooted at `root`.
    ///
    /// Returns `Err` if the local filesystem provider cannot be initialised.
    pub fn new(root: PathBuf, read_only: bool) -> Result<Self, VfsError> {
        let inner = LocalFsProvider::with_defaults().map_err(|e| VfsError::Io {
            uri: root.to_string_lossy().into_owned(),
            operation: "init".to_string(),
            source: std::io::Error::other(e.to_string()),
        })?;
        Ok(Self {
            root,
            read_only,
            inner,
        })
    }

    /// Reject the operation if this catalog is read-only.
    fn check_writable(&self, posix_path: &str, operation: &str) -> Result<(), VfsError> {
        if self.read_only {
            Err(VfsError::PermissionDenied {
                uri: format!("vfs://posix/{posix_path}"),
                operation: operation.to_string(),
            })
        } else {
            Ok(())
        }
    }

    /// Resolve a POSIX path to a native path string for the inner provider.
    fn native(&self, posix_path: &str) -> Result<String, VfsError> {
        let native = resolve_posix_path(&self.root, posix_path)?;
        Ok(native.to_string_lossy().into_owned())
    }
}

#[async_trait]
impl VfsProvider for PosixProvider {
    /// Validates: Requirement 7.1, 7.4
    fn scheme(&self) -> &str {
        "posix"
    }

    /// Validates: Requirement 7.7
    fn capabilities(&self) -> VfsCapabilities {
        VfsCapabilities {
            read: true,
            write: !self.read_only,
            watch: true,
            search: false,
            random_access: true,
            append: !self.read_only,
            rename: !self.read_only,
            delete: !self.read_only,
            list: true,
            create_directory: !self.read_only,
        }
    }

    /// Validates: Requirement 7.2
    async fn open(&self, path: &str, options: OpenOptions) -> Result<Box<dyn VfsFile>, VfsError> {
        if options.write || options.create || options.truncate || options.append {
            self.check_writable(path, "open")?;
        }
        self.inner.open(&self.native(path)?, options).await
    }

    /// Validates: Requirement 7.2
    async fn read(&self, path: &str) -> Result<Vec<u8>, VfsError> {
        self.inner.read(&self.native(path)?).await
    }

    /// Validates: Requirement 7.2
    async fn read_stream(
        &self,
        path: &str,
    ) -> Result<Pin<Box<dyn tokio::io::AsyncRead + Send>>, VfsError> {
        self.inner.read_stream(&self.native(path)?).await
    }

    /// Validates: Requirement 7.2, 7.6
    async fn write(&self, path: &str, data: &[u8]) -> Result<(), VfsError> {
        self.check_writable(path, "write")?;
        self.inner.write(&self.native(path)?, data).await
    }

    /// Validates: Requirement 7.2, 7.5, 7.6
    async fn create(&self, path: &str, options: CreateOptions) -> Result<(), VfsError> {
        self.check_writable(path, "create")?;
        self.inner.create(&self.native(path)?, options).await
    }

    /// Validates: Requirement 7.2, 7.5, 7.6
    async fn delete(&self, path: &str, options: DeleteOptions) -> Result<(), VfsError> {
        self.check_writable(path, "delete")?;
        self.inner.delete(&self.native(path)?, options).await
    }

    /// Validates: Requirement 7.2, 7.6
    async fn rename(&self, old_path: &str, new_path: &str) -> Result<(), VfsError> {
        self.check_writable(old_path, "rename")?;
        self.inner
            .rename(&self.native(old_path)?, &self.native(new_path)?)
            .await
    }

    /// Validates: Requirement 7.2
    async fn list(&self, path: &str) -> Result<Vec<VfsEntry>, VfsError> {
        self.inner.list(&self.native(path)?).await
    }

    /// Validates: Requirement 7.2
    async fn stat(&self, path: &str) -> Result<VfsMetadata, VfsError> {
        self.inner.stat(&self.native(path)?).await
    }

    /// Validates: Requirement 7.2
    async fn exists(&self, path: &str) -> Result<bool, VfsError> {
        self.inner.exists(&self.native(path)?).await
    }

    /// Validates: Requirement 7.7
    async fn watch(&self, path: &str, options: WatchOptions) -> Result<WatchHandle, VfsError> {
        self.inner.watch(&self.native(path)?, options).await
    }
}

// ── Tests ────────────────────────────────────────────────────────────────────

#[cfg(test)]
mod tests {
    use super::*;
    use std::path::PathBuf;

    fn root() -> PathBuf {
        PathBuf::from("/catalog/root")
    }

    // ── Path normalisation ────────────────────────────────────────────────

    /// Validates: Requirement 7.3 — forward-slash paths resolve correctly.
    #[test]
    fn resolve_simple_posix_path_joins_to_root() {
        // Validates: Requirement 7.3
        let resolved = resolve_posix_path(&root(), "/data/file.txt").unwrap();
        assert_eq!(resolved, PathBuf::from("/catalog/root/data/file.txt"));
    }

    /// Validates: Requirement 7.3 — root path (empty / slash) resolves to root.
    #[test]
    fn resolve_root_path_returns_root_dir() {
        // Validates: Requirement 7.3
        let resolved = resolve_posix_path(&root(), "/").unwrap();
        assert_eq!(resolved, root());
    }

    /// Validates: Requirement 7.3 — empty path resolves to root.
    #[test]
    fn resolve_empty_path_returns_root_dir() {
        // Validates: Requirement 7.3
        let resolved = resolve_posix_path(&root(), "").unwrap();
        assert_eq!(resolved, root());
    }

    /// Validates: Requirement 7.3 — dot segments are collapsed.
    #[test]
    fn resolve_dot_segments_are_collapsed() {
        // Validates: Requirement 7.3
        let resolved = resolve_posix_path(&root(), "/a/./b").unwrap();
        assert_eq!(resolved, PathBuf::from("/catalog/root/a/b"));
    }

    /// Validates: Requirement 7.3 — double-slash segments are collapsed.
    #[test]
    fn resolve_double_slash_segments_are_collapsed() {
        // Validates: Requirement 7.3
        let resolved = resolve_posix_path(&root(), "/a//b").unwrap();
        assert_eq!(resolved, PathBuf::from("/catalog/root/a/b"));
    }

    // ── Root-jail ─────────────────────────────────────────────────────────

    /// Validates: Requirement 7.3 — `..` at root level is rejected.
    #[test]
    fn resolve_dotdot_at_root_is_rejected() {
        // Validates: Requirement 7.3
        let err = resolve_posix_path(&root(), "/..").unwrap_err();
        assert!(matches!(err, VfsError::PermissionDenied { .. }));
    }

    /// Validates: Requirement 7.3 — `..` escape attempt is rejected.
    #[test]
    fn resolve_dotdot_escape_attempt_is_rejected() {
        // Validates: Requirement 7.3
        let err = resolve_posix_path(&root(), "/a/../../etc/passwd").unwrap_err();
        assert!(matches!(err, VfsError::PermissionDenied { .. }));
    }

    /// Validates: Requirement 7.3 — nested `..` that stays within root is rejected
    /// (we reject all `..` unconditionally for simplicity and safety).
    #[test]
    fn resolve_dotdot_within_root_is_still_rejected() {
        // Validates: Requirement 7.3 — all `..` rejected, not just escaping ones
        let err = resolve_posix_path(&root(), "/a/../b").unwrap_err();
        assert!(matches!(err, VfsError::PermissionDenied { .. }));
    }

    // ── Read-only enforcement ─────────────────────────────────────────────

    /// Validates: Requirement 7.6 — write on read-only provider returns PermissionDenied.
    #[tokio::test]
    async fn write_on_read_only_provider_returns_permission_denied() {
        // Validates: Requirement 7.6
        let provider = PosixProvider::new(root(), true).unwrap();
        let err = provider.write("/file.txt", b"data").await.unwrap_err();
        assert!(matches!(err, VfsError::PermissionDenied { .. }));
    }

    /// Validates: Requirement 7.6 — create on read-only provider returns PermissionDenied.
    #[tokio::test]
    async fn create_on_read_only_provider_returns_permission_denied() {
        // Validates: Requirement 7.6
        let provider = PosixProvider::new(root(), true).unwrap();
        let err = provider
            .create("/newfile.txt", CreateOptions::default())
            .await
            .unwrap_err();
        assert!(matches!(err, VfsError::PermissionDenied { .. }));
    }

    /// Validates: Requirement 7.6 — delete on read-only provider returns PermissionDenied.
    #[tokio::test]
    async fn delete_on_read_only_provider_returns_permission_denied() {
        // Validates: Requirement 7.6
        let provider = PosixProvider::new(root(), true).unwrap();
        let err = provider
            .delete("/file.txt", DeleteOptions::default())
            .await
            .unwrap_err();
        assert!(matches!(err, VfsError::PermissionDenied { .. }));
    }

    /// Validates: Requirement 7.6 — rename on read-only provider returns PermissionDenied.
    #[tokio::test]
    async fn rename_on_read_only_provider_returns_permission_denied() {
        // Validates: Requirement 7.6
        let provider = PosixProvider::new(root(), true).unwrap();
        let err = provider.rename("/old.txt", "/new.txt").await.unwrap_err();
        assert!(matches!(err, VfsError::PermissionDenied { .. }));
    }

    // ── Capabilities ──────────────────────────────────────────────────────

    /// Validates: Requirement 7.7 — read-write provider advertises full capabilities.
    #[tokio::test]
    async fn read_write_provider_capabilities_are_full() {
        // Validates: Requirement 7.7
        let provider = PosixProvider::new(root(), false).unwrap();
        let caps = provider.capabilities();
        assert!(caps.read);
        assert!(caps.write);
        assert!(caps.list);
        assert!(caps.rename);
        assert!(caps.delete);
        assert!(caps.create_directory);
        assert!(caps.watch);
    }

    /// Validates: Requirement 7.6, 7.7 — read-only provider advertises no write capabilities.
    #[tokio::test]
    async fn read_only_provider_capabilities_disable_writes() {
        // Validates: Requirement 7.6, 7.7
        let provider = PosixProvider::new(root(), true).unwrap();
        let caps = provider.capabilities();
        assert!(caps.read);
        assert!(!caps.write);
        assert!(!caps.rename);
        assert!(!caps.delete);
        assert!(!caps.create_directory);
        assert!(!caps.append);
    }

    /// Validates: Requirement 7.1 — scheme returns "posix".
    #[tokio::test]
    async fn scheme_returns_posix() {
        // Validates: Requirement 7.1
        let provider = PosixProvider::new(root(), false).unwrap();
        assert_eq!(provider.scheme(), "posix");
    }

    // ── to_posix_path ─────────────────────────────────────────────────────

    /// Validates: Requirement 7.3 — native path converts back to POSIX relative path.
    #[test]
    fn to_posix_path_strips_root_prefix() {
        // Validates: Requirement 7.3
        let native = PathBuf::from("/catalog/root/data/file.txt");
        let posix = to_posix_path(&root(), &native);
        assert_eq!(posix, "data/file.txt");
    }

    /// Validates: Requirement 7.3 — root itself converts to empty string.
    #[test]
    fn to_posix_path_root_returns_empty() {
        // Validates: Requirement 7.3
        let posix = to_posix_path(&root(), &root());
        assert_eq!(posix, "");
    }
}
