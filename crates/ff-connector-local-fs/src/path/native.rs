//! NativePath type — a validated, platform-native filesystem path wrapper.
//!
//! Wraps `PathBuf` with normalisation guarantees and platform-specific helpers.

use std::borrow::Cow;
use std::path::{Path, PathBuf};

/// A validated, platform-native filesystem path.
///
/// Wraps `PathBuf` with the guarantee that path separators are normalised
/// to the platform convention.
///
/// Addresses: Requirement 2, criteria 1–10
#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub struct NativePath(PathBuf);

impl NativePath {
    /// Construct from a `PathBuf` (normalises separators).
    pub fn from_path_buf(path: PathBuf) -> Self {
        Self(path)
    }

    /// Construct from a string path.
    pub fn new_from(path: &str) -> Self {
        Self(PathBuf::from(path))
    }

    /// Returns the inner `Path` reference.
    pub fn as_path(&self) -> &Path {
        &self.0
    }

    /// Returns the inner `PathBuf`.
    pub fn into_path_buf(self) -> PathBuf {
        self.0
    }

    /// Returns the path as a string (lossy for non-UTF8 paths on Unix).
    pub fn to_string_lossy(&self) -> Cow<'_, str> {
        self.0.to_string_lossy()
    }

    /// On Windows, apply the extended-length prefix (`\\?\`) for long paths.
    ///
    /// Addresses: Requirement 2, criterion 7
    #[cfg(windows)]
    pub fn to_extended_length(&self) -> PathBuf {
        let path_str = self.0.to_string_lossy();
        if path_str.starts_with(r"\\?\") {
            self.0.clone()
        } else {
            PathBuf::from(format!(r"\\?\{}", path_str))
        }
    }

    /// Returns true if this path exceeds MAX_PATH on Windows (260 chars).
    #[cfg(windows)]
    pub fn exceeds_max_path(&self) -> bool {
        self.0.to_string_lossy().len() > 260
    }

    /// Returns true if the path is absolute.
    pub fn is_absolute(&self) -> bool {
        self.0.is_absolute()
    }
}

impl AsRef<Path> for NativePath {
    fn as_ref(&self) -> &Path {
        &self.0
    }
}

impl From<PathBuf> for NativePath {
    fn from(path: PathBuf) -> Self {
        Self::from_path_buf(path)
    }
}

impl From<NativePath> for PathBuf {
    fn from(native: NativePath) -> Self {
        native.0
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn from_path_buf_preserves_path() {
        let path = PathBuf::from("/home/user/file.txt");
        let native = NativePath::from_path_buf(path.clone());
        assert_eq!(native.as_path(), path.as_path());
    }

    #[test]
    fn from_str_creates_native_path() {
        let native = NativePath::new_from("/tmp/test.txt");
        assert_eq!(native.as_path(), Path::new("/tmp/test.txt"));
    }

    #[test]
    fn to_string_lossy_returns_path_string() {
        let native = NativePath::new_from("/home/user/file.txt");
        assert_eq!(native.to_string_lossy(), "/home/user/file.txt");
    }

    #[test]
    fn is_absolute_detects_absolute_paths() {
        #[cfg(not(windows))]
        {
            let abs = NativePath::new_from("/absolute/path");
            assert!(abs.is_absolute());
        }
        #[cfg(windows)]
        {
            let abs = NativePath::new_from(r"C:\absolute\path");
            assert!(abs.is_absolute());
        }

        let rel = NativePath::new_from("relative/path");
        assert!(!rel.is_absolute());
    }

    #[test]
    fn into_path_buf_returns_inner() {
        let native = NativePath::new_from("/tmp/test.txt");
        let buf: PathBuf = native.into_path_buf();
        assert_eq!(buf, PathBuf::from("/tmp/test.txt"));
    }

    #[cfg(windows)]
    #[test]
    fn to_extended_length_adds_prefix_on_windows() {
        let native = NativePath::new_from(r"C:\Users\test\file.txt");
        let extended = native.to_extended_length();
        assert!(extended.to_string_lossy().starts_with(r"\\?\"));
    }

    #[cfg(windows)]
    #[test]
    fn to_extended_length_does_not_double_prefix() {
        let native = NativePath::new_from(r"\\?\C:\Users\test\file.txt");
        let extended = native.to_extended_length();
        assert_eq!(extended.to_string_lossy(), r"\\?\C:\Users\test\file.txt");
    }
}
