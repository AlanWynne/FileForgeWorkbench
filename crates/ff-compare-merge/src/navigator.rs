//! DiffNavigator — tracks current navigation position within diff hunks.

/// Unique identifier for a CompareSession.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub struct SessionId(u64);

impl SessionId {
    /// Create a new SessionId from a raw value.
    pub fn new(id: u64) -> Self {
        Self(id)
    }
}

/// Tracks the current navigation position within diff hunks.
/// Supports next/previous with wrapping.
pub struct DiffNavigator {
    /// Index of the currently focused hunk (0-based, among non-Equal hunks).
    current_index: usize,
    /// Total number of difference hunks (non-Equal).
    total_hunks: usize,
    /// Whether the last navigation wrapped around.
    pub wrapped: bool,
}

impl DiffNavigator {
    /// Create a navigator for the given number of diff hunks.
    pub fn new(total_hunks: usize) -> Self {
        Self {
            current_index: 0,
            total_hunks,
            wrapped: false,
        }
    }

    /// Move to the next hunk. Wraps to first if at end.
    /// Returns true if wrap occurred.
    #[allow(clippy::should_implement_trait)]
    pub fn next(&mut self) -> bool {
        if self.total_hunks == 0 {
            return false;
        }
        let next = self.current_index + 1;
        if next >= self.total_hunks {
            self.current_index = 0;
            self.wrapped = true;
            true
        } else {
            self.current_index = next;
            self.wrapped = false;
            false
        }
    }

    /// Move to the previous hunk. Wraps to last if at beginning.
    /// Returns true if wrap occurred.
    pub fn prev(&mut self) -> bool {
        if self.total_hunks == 0 {
            return false;
        }
        if self.current_index == 0 {
            self.current_index = self.total_hunks - 1;
            self.wrapped = true;
            true
        } else {
            self.current_index -= 1;
            self.wrapped = false;
            false
        }
    }

    /// Get the current hunk index (0-based).
    pub fn current(&self) -> usize {
        self.current_index
    }

    /// Get display string "N of M" (1-based).
    pub fn display_position(&self) -> String {
        if self.total_hunks == 0 {
            return "No differences".to_string();
        }
        format!("Diff {} of {}", self.current_index + 1, self.total_hunks)
    }

    /// Set the current index directly.
    pub fn set_current(&mut self, index: usize) {
        if self.total_hunks > 0 {
            self.current_index = index.min(self.total_hunks - 1);
        }
    }

    /// Total number of hunks.
    pub fn total(&self) -> usize {
        self.total_hunks
    }
}

/// Identifies the source of content in a comparison.
#[derive(Debug, Clone)]
pub enum CompareSource {
    /// A VFS-addressable resource.
    Resource { uri: String, label: String },
    /// The saved version of a document.
    SavedVersion { uri: String, label: String },
    /// Clipboard content (ephemeral, no URI).
    Clipboard { label: String },
    /// A text selection from a document.
    Selection {
        document_label: String,
        line_range: String,
        label: String,
    },
}

impl CompareSource {
    /// Returns the display label for this source.
    pub fn label(&self) -> &str {
        match self {
            CompareSource::Resource { label, .. } => label,
            CompareSource::SavedVersion { label, .. } => label,
            CompareSource::Clipboard { label } => label,
            CompareSource::Selection { label, .. } => label,
        }
    }
}

/// The stateful context of an active comparison.
pub struct CompareSession {
    /// Unique session identifier.
    pub id: SessionId,
    /// Left resource source.
    pub left_source: CompareSource,
    /// Right resource source.
    pub right_source: CompareSource,
    /// Left resource content (lines).
    pub left_lines: Vec<String>,
    /// Right resource content (lines).
    pub right_lines: Vec<String>,
    /// Computed diff result.
    pub diff_result: crate::result::DiffResult,
    /// Current comparison options.
    pub options: crate::options::CompareOptions,
    /// Navigation state.
    pub navigator: DiffNavigator,
    /// Per-hunk resolution status (for merge sessions).
    pub hunk_resolutions: Vec<crate::merge::ConflictResolution>,
    /// Whether merge operations are allowed.
    pub merge_enabled: bool,
}

impl CompareSession {
    /// Create a new CompareSession from pre-loaded content.
    pub fn new(
        id: SessionId,
        left_source: CompareSource,
        right_source: CompareSource,
        left_lines: Vec<String>,
        right_lines: Vec<String>,
        options: crate::options::CompareOptions,
    ) -> Self {
        let left_refs: Vec<&str> = left_lines.iter().map(String::as_str).collect();
        let right_refs: Vec<&str> = right_lines.iter().map(String::as_str).collect();
        let diff_result = crate::session::DiffEngine::diff(&left_refs, &right_refs, &options);
        let total_diff_hunks = diff_result.statistics.hunks_count;
        let hunk_resolutions = vec![crate::merge::ConflictResolution::Unresolved; total_diff_hunks];
        Self {
            id,
            left_source,
            right_source,
            left_lines,
            right_lines,
            diff_result,
            options,
            navigator: DiffNavigator::new(total_diff_hunks),
            hunk_resolutions,
            merge_enabled: true,
        }
    }

    /// Recompute the diff with updated options (without reloading content).
    pub fn recompute(&mut self, options: crate::options::CompareOptions) {
        let left_refs: Vec<&str> = self.left_lines.iter().map(String::as_str).collect();
        let right_refs: Vec<&str> = self.right_lines.iter().map(String::as_str).collect();
        self.diff_result = crate::session::DiffEngine::diff(&left_refs, &right_refs, &options);
        let total = self.diff_result.statistics.hunks_count;
        self.hunk_resolutions = vec![crate::merge::ConflictResolution::Unresolved; total];
        self.navigator = DiffNavigator::new(total);
        self.options = options;
    }

    /// Returns true if all diff hunks are resolved.
    pub fn all_resolved(&self) -> bool {
        self.hunk_resolutions
            .iter()
            .all(|r| *r != crate::merge::ConflictResolution::Unresolved)
    }

    /// Count of unresolved hunks.
    pub fn unresolved_count(&self) -> usize {
        self.hunk_resolutions
            .iter()
            .filter(|r| **r == crate::merge::ConflictResolution::Unresolved)
            .count()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn navigator_next_advances_index() {
        // Validates: Requirement 6.1 — next_diff advances to next hunk
        let mut nav = DiffNavigator::new(3);
        assert_eq!(nav.current(), 0);
        nav.next();
        assert_eq!(nav.current(), 1);
        nav.next();
        assert_eq!(nav.current(), 2);
    }

    #[test]
    fn navigator_next_wraps_to_first() {
        // Validates: Requirement 6.4 — next_diff wraps to first hunk
        let mut nav = DiffNavigator::new(3);
        nav.set_current(2);
        let wrapped = nav.next();
        assert!(wrapped);
        assert_eq!(nav.current(), 0);
    }

    #[test]
    fn navigator_prev_wraps_to_last() {
        // Validates: Requirement 6.6 — prev_diff wraps to last hunk
        let mut nav = DiffNavigator::new(3);
        let wrapped = nav.prev();
        assert!(wrapped);
        assert_eq!(nav.current(), 2);
    }

    #[test]
    fn navigator_prev_decrements_index() {
        // Validates: Requirement 6.5 — prev_diff moves to previous hunk
        let mut nav = DiffNavigator::new(3);
        nav.set_current(2);
        nav.prev();
        assert_eq!(nav.current(), 1);
    }

    #[test]
    fn navigator_full_cycle_visits_all_hunks() {
        // Validates: Property 8 — N next() calls from 0 visits all and wraps
        let n = 5;
        let mut nav = DiffNavigator::new(n);
        let mut visited = std::collections::HashSet::new();
        visited.insert(nav.current());
        for _ in 0..n {
            nav.next();
            visited.insert(nav.current());
        }
        assert_eq!(visited.len(), n);
        assert_eq!(nav.current(), 0);
    }

    #[test]
    fn navigator_display_position_one_based() {
        // Validates: Requirement 6.9 — "Diff N of M" display
        let mut nav = DiffNavigator::new(5);
        assert_eq!(nav.display_position(), "Diff 1 of 5");
        nav.next();
        assert_eq!(nav.display_position(), "Diff 2 of 5");
    }

    #[test]
    fn navigator_zero_hunks_no_panic() {
        // Validates: Requirement 6 — empty diff navigation
        let mut nav = DiffNavigator::new(0);
        assert!(!nav.next());
        assert!(!nav.prev());
        assert_eq!(nav.display_position(), "No differences");
    }

    #[test]
    fn session_new_computes_diff() {
        // Validates: Requirement 7 — session creation computes diff
        let left = vec!["a".to_string(), "b".to_string()];
        let right = vec!["a".to_string(), "x".to_string()];
        let session = CompareSession::new(
            SessionId::new(1),
            CompareSource::Resource {
                uri: "left".to_string(),
                label: "left".to_string(),
            },
            CompareSource::Resource {
                uri: "right".to_string(),
                label: "right".to_string(),
            },
            left,
            right,
            crate::options::CompareOptions::default(),
        );
        assert!(session.diff_result.statistics.hunks_count > 0);
    }

    #[test]
    fn session_recompute_updates_diff() {
        // Validates: Requirement 11.4 — option change triggers recompute
        let left = vec!["Hello".to_string()];
        let right = vec!["hello".to_string()];
        let mut session = CompareSession::new(
            SessionId::new(1),
            CompareSource::Clipboard {
                label: "left".to_string(),
            },
            CompareSource::Clipboard {
                label: "right".to_string(),
            },
            left,
            right,
            crate::options::CompareOptions::default(),
        );
        // Initially different (case-sensitive)
        assert_eq!(session.diff_result.statistics.hunks_count, 1);
        // Recompute with ignore_case
        let new_opts = crate::options::CompareOptions {
            ignore_case: true,
            ..Default::default()
        };
        session.recompute(new_opts);
        // Now equal
        assert_eq!(session.diff_result.statistics.hunks_count, 0);
    }
}
