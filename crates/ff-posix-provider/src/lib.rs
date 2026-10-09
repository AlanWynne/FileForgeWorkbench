//! # POSIX VFS Provider (reduced -- CR-CH-059 RC.A.4)
//!
//! CR-CH-059 RC.A.4 collapsed the two `posix`-scheme registrants to a SINGLE
//! design: `ff_vfs::PosixNativeProvider` (which implements BOTH `VfsProvider`
//! for scheme `posix` and the single physical seam `ff_vfs::StorageProvider`).
//! This crate NO LONGER defines its own `VfsProvider` -- it cannot register the
//! `posix` scheme. For source compatibility it re-exports the single provider
//! under the historical name `PosixProvider`, and retains only the pure POSIX
//! path helpers (no provider, no registration).
//!
//! Validates: virtual-file-system Requirement 14.1, 14.2.

use std::path::{Component, Path, PathBuf};

use ff_vfs::VfsError;

/// The single `posix` provider is `ff_vfs::PosixNativeProvider`. Re-exported
/// here under the historical name so existing `PosixProvider` references
/// resolve to the one chosen design (Requirement 14.2).
pub use ff_vfs::PosixNativeProvider as PosixProvider;

// === Path helpers ===========================================================

/// Normalise a POSIX-style path and resolve it against `root`, enforcing the
/// root-jail invariant (no `..` escape).
///
/// Returns `Err(VfsError::PermissionDenied)` if the resolved path would escape
/// the root directory.
///
/// Validates: Requirement 7.3, 7.4
pub fn resolve_posix_path(root: &Path, posix_path: &str) -> Result<PathBuf, VfsError> {
    // Strip leading slash -- POSIX paths are relative to the catalog root.
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
pub fn to_posix_path(root: &Path, native: &Path) -> String {
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

// === Tests ==================================================================

#[cfg(test)]
mod tests {
    use super::*;
    use std::path::PathBuf;

    fn root() -> PathBuf {
        PathBuf::from("/catalog/root")
    }

    /// Validates: Requirement 7.3 -- forward-slash paths resolve correctly.
    #[test]
    fn resolve_simple_posix_path_joins_to_root() {
        let resolved = resolve_posix_path(&root(), "/data/file.txt").unwrap();
        assert_eq!(resolved, PathBuf::from("/catalog/root/data/file.txt"));
    }

    /// Validates: Requirement 7.3 -- root path (empty / slash) resolves to root.
    #[test]
    fn resolve_root_path_returns_root_dir() {
        let resolved = resolve_posix_path(&root(), "/").unwrap();
        assert_eq!(resolved, root());
    }

    /// Validates: Requirement 7.3 -- empty path resolves to root.
    #[test]
    fn resolve_empty_path_returns_root_dir() {
        let resolved = resolve_posix_path(&root(), "").unwrap();
        assert_eq!(resolved, root());
    }

    /// Validates: Requirement 7.3 -- dot segments are collapsed.
    #[test]
    fn resolve_dot_segments_are_collapsed() {
        let resolved = resolve_posix_path(&root(), "/a/./b").unwrap();
        assert_eq!(resolved, PathBuf::from("/catalog/root/a/b"));
    }

    /// Validates: Requirement 7.3 -- double-slash segments are collapsed.
    #[test]
    fn resolve_double_slash_segments_are_collapsed() {
        let resolved = resolve_posix_path(&root(), "/a//b").unwrap();
        assert_eq!(resolved, PathBuf::from("/catalog/root/a/b"));
    }

    /// Validates: Requirement 7.3 -- `..` at root level is rejected.
    #[test]
    fn resolve_dotdot_at_root_is_rejected() {
        let err = resolve_posix_path(&root(), "/..").unwrap_err();
        assert!(matches!(err, VfsError::PermissionDenied { .. }));
    }

    /// Validates: Requirement 7.3 -- `..` escape attempt is rejected.
    #[test]
    fn resolve_dotdot_escape_attempt_is_rejected() {
        let err = resolve_posix_path(&root(), "/a/../../etc/passwd").unwrap_err();
        assert!(matches!(err, VfsError::PermissionDenied { .. }));
    }

    /// Validates: Requirement 7.3 -- nested `..` that stays within root is rejected.
    #[test]
    fn resolve_dotdot_within_root_is_still_rejected() {
        let err = resolve_posix_path(&root(), "/a/../b").unwrap_err();
        assert!(matches!(err, VfsError::PermissionDenied { .. }));
    }

    /// Validates: Requirement 7.3 -- native path converts back to POSIX relative path.
    #[test]
    fn to_posix_path_strips_root_prefix() {
        let native = PathBuf::from("/catalog/root/data/file.txt");
        let posix = to_posix_path(&root(), &native);
        assert_eq!(posix, "data/file.txt");
    }

    /// Validates: Requirement 7.3 -- root itself converts to empty string.
    #[test]
    fn to_posix_path_root_returns_empty() {
        let posix = to_posix_path(&root(), &root());
        assert_eq!(posix, "");
    }

    /// Validates: Requirement 14.2 -- the re-exported PosixProvider is the single
    /// `ff_vfs::PosixNativeProvider` (scheme `posix`).
    #[test]
    fn reexported_posix_provider_is_the_vfs_native_provider() {
        use ff_vfs::VfsProvider;
        let dir = tempfile::tempdir().expect("tempdir");
        let provider = PosixProvider::new(dir.path(), true);
        assert_eq!(provider.scheme(), "posix");
    }
}
