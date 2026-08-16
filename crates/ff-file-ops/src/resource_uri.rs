//! ResourceUri helpers and extension methods for file operations.
//!
//! The canonical `ResourceUri` type is defined in `ff-vfs`. This module
//! provides file-ops-specific helper functions for URI manipulation.

use ff_vfs::ResourceUri;

/// Extract the filename portion from a resource URI path.
///
/// Returns the last path component, or the full path if no separator is found.
pub fn filename_from_uri(uri: &ResourceUri) -> &str {
    let path = uri.path();
    path.rsplit('/').next().unwrap_or(path)
}

/// Generate a backup URI by appending a suffix to the filename.
///
/// Addresses: Requirement 7 AC 7.4
pub fn backup_uri_alongside(uri: &ResourceUri, suffix: &str) -> ResourceUri {
    let new_path = format!("{}{}", uri.path(), suffix);
    ResourceUri::new(uri.scheme(), new_path)
}

/// Generate a temp file URI in the same directory as the target.
///
/// Addresses: Requirement 7 AC 7.1
pub fn temp_uri_for(uri: &ResourceUri) -> ResourceUri {
    let new_path = format!("{}.tmp", uri.path());
    ResourceUri::new(uri.scheme(), new_path)
}

/// Extract the parent directory path from a URI.
///
/// Returns everything up to and including the last `/` separator.
pub fn parent_path(uri: &ResourceUri) -> &str {
    let path = uri.path();
    match path.rfind('/') {
        Some(idx) => &path[..=idx],
        None => "/",
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn filename_from_uri_extracts_last_component() {
        let uri = ResourceUri::new("local", "/home/user/document.txt");
        assert_eq!(filename_from_uri(&uri), "document.txt");
    }

    #[test]
    fn filename_from_uri_handles_root_file() {
        let uri = ResourceUri::new("local", "/file.txt");
        assert_eq!(filename_from_uri(&uri), "file.txt");
    }

    #[test]
    fn backup_uri_alongside_appends_suffix() {
        let uri = ResourceUri::new("local", "/docs/readme.md");
        let backup = backup_uri_alongside(&uri, ".bak");
        assert_eq!(backup.path(), "/docs/readme.md.bak");
        assert_eq!(backup.scheme(), "local");
    }

    #[test]
    fn temp_uri_for_appends_tmp_suffix() {
        let uri = ResourceUri::new("local", "/docs/readme.md");
        let temp = temp_uri_for(&uri);
        assert_eq!(temp.path(), "/docs/readme.md.tmp");
        assert_eq!(temp.scheme(), "local");
    }

    #[test]
    fn parent_path_extracts_directory() {
        let uri = ResourceUri::new("local", "/home/user/file.txt");
        assert_eq!(parent_path(&uri), "/home/user/");
    }

    #[test]
    fn parent_path_handles_root_file() {
        let uri = ResourceUri::new("local", "/file.txt");
        assert_eq!(parent_path(&uri), "/");
    }
}
