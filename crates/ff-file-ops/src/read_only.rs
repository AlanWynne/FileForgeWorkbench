//! Read-only detection and enforcement for documents.
//!
//! Evaluates read-only status from VFS metadata, configuration patterns,
//! and provider capabilities. Supports manual user toggle override.

/// The read-only status of a document, indicating the source of the restriction.
///
/// Addresses: Requirement 8, criteria 1–7
#[derive(Debug, Clone, PartialEq, Eq)]
#[non_exhaustive]
pub enum ReadOnlyStatus {
    /// Document is writable.
    Writable,
    /// Read-only due to VFS provider reporting non-writable.
    VfsRestricted,
    /// Read-only due to configuration pattern match.
    ConfigRestricted,
    /// Read-only due to provider not supporting write capability.
    ProviderLacksWrite,
    /// Manually toggled read-only by user.
    UserToggled,
}

impl ReadOnlyStatus {
    /// Whether the document is effectively read-only.
    pub fn is_read_only(&self) -> bool {
        !matches!(self, Self::Writable)
    }
}

/// Toggle the read-only status of a document.
///
/// If currently writable, sets to `UserToggled`.
/// If currently read-only (any source), sets to `Writable`.
///
/// Addresses: Requirement 8 AC 8.5
pub fn toggle_read_only(current: &ReadOnlyStatus) -> ReadOnlyStatus {
    if current.is_read_only() {
        ReadOnlyStatus::Writable
    } else {
        ReadOnlyStatus::UserToggled
    }
}

/// Check if a file path matches a read-only glob pattern from configuration.
///
/// Addresses: Requirement 8 AC 8.4
pub fn matches_read_only_pattern(path: &str, pattern: &str) -> bool {
    if pattern.is_empty() {
        return false;
    }
    // Simple glob matching: support * and ? characters
    glob_match(pattern, path)
}

/// Simple glob pattern matching (supports `*` and `?`).
fn glob_match(pattern: &str, text: &str) -> bool {
    let pat: Vec<char> = pattern.chars().collect();
    let txt: Vec<char> = text.chars().collect();
    glob_match_impl(&pat, &txt, 0, 0)
}

fn glob_match_impl(pat: &[char], txt: &[char], pi: usize, ti: usize) -> bool {
    if pi == pat.len() {
        return ti == txt.len();
    }

    if pat[pi] == '*' {
        // Try matching zero or more characters
        for skip in 0..=(txt.len() - ti) {
            if glob_match_impl(pat, txt, pi + 1, ti + skip) {
                return true;
            }
        }
        return false;
    }

    if ti == txt.len() {
        return false;
    }

    if pat[pi] == '?' || pat[pi] == txt[ti] {
        return glob_match_impl(pat, txt, pi + 1, ti + 1);
    }

    false
}

/// Determine the display indicator for read-only status.
///
/// Addresses: Requirement 8 AC 8.3
pub fn read_only_indicator(status: &ReadOnlyStatus) -> Option<&'static str> {
    if status.is_read_only() {
        Some("[RO]")
    } else {
        None
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn writable_is_not_read_only() {
        assert!(!ReadOnlyStatus::Writable.is_read_only());
    }

    #[test]
    fn all_non_writable_variants_are_read_only() {
        assert!(ReadOnlyStatus::VfsRestricted.is_read_only());
        assert!(ReadOnlyStatus::ConfigRestricted.is_read_only());
        assert!(ReadOnlyStatus::ProviderLacksWrite.is_read_only());
        assert!(ReadOnlyStatus::UserToggled.is_read_only());
    }

    // Validates: Requirement 8 AC 8.5 — toggle from writable to read-only
    #[test]
    fn toggle_writable_to_read_only() {
        let result = toggle_read_only(&ReadOnlyStatus::Writable);
        assert_eq!(result, ReadOnlyStatus::UserToggled);
    }

    // Validates: Requirement 8 AC 8.5 — toggle from read-only to writable
    #[test]
    fn toggle_read_only_to_writable() {
        assert_eq!(
            toggle_read_only(&ReadOnlyStatus::VfsRestricted),
            ReadOnlyStatus::Writable
        );
        assert_eq!(
            toggle_read_only(&ReadOnlyStatus::ConfigRestricted),
            ReadOnlyStatus::Writable
        );
        assert_eq!(
            toggle_read_only(&ReadOnlyStatus::UserToggled),
            ReadOnlyStatus::Writable
        );
    }

    // Validates: Requirement 8 AC 8.4 — config pattern matching
    #[test]
    fn matches_pattern_with_wildcard() {
        assert!(matches_read_only_pattern("/logs/app.log", "*.log"));
        assert!(matches_read_only_pattern("/data/backup.bak", "*.bak"));
    }

    #[test]
    fn matches_pattern_with_path_wildcard() {
        assert!(matches_read_only_pattern(
            "/readonly/file.txt",
            "/readonly/*"
        ));
    }

    #[test]
    fn no_match_for_different_extension() {
        assert!(!matches_read_only_pattern("/docs/readme.md", "*.log"));
    }

    #[test]
    fn empty_pattern_matches_nothing() {
        assert!(!matches_read_only_pattern("/any/file.txt", ""));
    }

    #[test]
    fn question_mark_matches_single_char() {
        assert!(matches_read_only_pattern("/file1.txt", "/file?.txt"));
        assert!(!matches_read_only_pattern("/file12.txt", "/file?.txt"));
    }

    // Validates: Requirement 8 AC 8.3 — visual indicator
    #[test]
    fn read_only_indicator_shows_for_read_only() {
        assert_eq!(
            read_only_indicator(&ReadOnlyStatus::VfsRestricted),
            Some("[RO]")
        );
        assert_eq!(
            read_only_indicator(&ReadOnlyStatus::UserToggled),
            Some("[RO]")
        );
    }

    #[test]
    fn read_only_indicator_none_for_writable() {
        assert_eq!(read_only_indicator(&ReadOnlyStatus::Writable), None);
    }
}
