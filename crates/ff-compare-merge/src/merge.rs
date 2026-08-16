//! Merge operations: two-way and three-way merge logic.

use crate::error::CompareError;
use crate::navigator::CompareSession;
use crate::result::DiffHunk;

/// The resolution status of a merge conflict or diff hunk.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ConflictResolution {
    /// Not yet resolved — requires user action.
    Unresolved,
    /// Resolved by accepting the left version.
    ResolvedLeft,
    /// Resolved by accepting the right version.
    ResolvedRight,
    /// Resolved by accepting both (left then right).
    ResolvedBoth,
    /// Resolved by custom user edit.
    ResolvedCustom,
}

/// A region where both left and right versions have modified the same lines
/// relative to the base, requiring manual resolution.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct MergeConflict {
    /// The base content for this region.
    pub base_lines: Vec<String>,
    /// The left version's content for this region.
    pub left_lines: Vec<String>,
    /// The right version's content for this region.
    pub right_lines: Vec<String>,
    /// Starting line in the merge result where this conflict appears.
    pub result_start: usize,
    /// Resolution status.
    pub status: ConflictResolution,
}

/// Classification of a region in a three-way merge.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum ThreeWayRegion {
    /// Same in all three versions — included in result automatically.
    Unchanged { lines: Vec<String> },
    /// Only left differs from base — auto-resolved to left.
    LeftOnlyChange { lines: Vec<String> },
    /// Only right differs from base — auto-resolved to right.
    RightOnlyChange { lines: Vec<String> },
    /// Both left and right differ from base — conflict requiring resolution.
    Conflict(MergeConflict),
}

/// Applies merge decisions to produce a merged output document.
pub struct MergeResolver;

impl MergeResolver {
    /// Accept the left version for the specified diff hunk index.
    pub fn accept_left(
        session: &CompareSession,
        hunk_index: usize,
    ) -> Result<Vec<String>, CompareError> {
        let diff_hunks: Vec<&DiffHunk> = session.diff_result.diff_hunks().collect();
        if hunk_index >= diff_hunks.len() {
            return Err(CompareError::HunkIndexOutOfRange {
                operation: "accept_left".to_string(),
                index: hunk_index,
                total: diff_hunks.len(),
            });
        }
        let hunk = diff_hunks[hunk_index];
        Ok(extract_left_lines(session, hunk))
    }

    /// Accept the right version for the specified diff hunk index.
    pub fn accept_right(
        session: &CompareSession,
        hunk_index: usize,
    ) -> Result<Vec<String>, CompareError> {
        let diff_hunks: Vec<&DiffHunk> = session.diff_result.diff_hunks().collect();
        if hunk_index >= diff_hunks.len() {
            return Err(CompareError::HunkIndexOutOfRange {
                operation: "accept_right".to_string(),
                index: hunk_index,
                total: diff_hunks.len(),
            });
        }
        let hunk = diff_hunks[hunk_index];
        Ok(extract_right_lines(session, hunk))
    }

    /// Accept both versions (left then right) for the specified hunk.
    pub fn accept_both(
        session: &CompareSession,
        hunk_index: usize,
    ) -> Result<Vec<String>, CompareError> {
        let diff_hunks: Vec<&DiffHunk> = session.diff_result.diff_hunks().collect();
        if hunk_index >= diff_hunks.len() {
            return Err(CompareError::HunkIndexOutOfRange {
                operation: "accept_both".to_string(),
                index: hunk_index,
                total: diff_hunks.len(),
            });
        }
        let hunk = diff_hunks[hunk_index];
        let mut result = extract_left_lines(session, hunk);
        result.extend(extract_right_lines(session, hunk));
        Ok(result)
    }

    /// Build the complete merge result from resolved hunks.
    pub fn build_result(session: &CompareSession) -> Result<Vec<String>, CompareError> {
        let unresolved = session.unresolved_count();
        if unresolved > 0 {
            return Err(CompareError::UnresolvedConflicts { count: unresolved });
        }
        // Build result by walking hunks in order
        let mut result = Vec::new();
        let mut diff_idx = 0;
        for hunk in &session.diff_result.hunks {
            match hunk {
                DiffHunk::Equal {
                    left_start, count, ..
                } => {
                    for i in 0..*count {
                        if let Some(line) = session.left_lines.get(left_start + i) {
                            result.push(line.clone());
                        }
                    }
                }
                _ => {
                    let resolution = session
                        .hunk_resolutions
                        .get(diff_idx)
                        .copied()
                        .unwrap_or(ConflictResolution::ResolvedLeft);
                    match resolution {
                        ConflictResolution::ResolvedLeft => {
                            result.extend(extract_left_lines(session, hunk));
                        }
                        ConflictResolution::ResolvedRight => {
                            result.extend(extract_right_lines(session, hunk));
                        }
                        ConflictResolution::ResolvedBoth => {
                            result.extend(extract_left_lines(session, hunk));
                            result.extend(extract_right_lines(session, hunk));
                        }
                        ConflictResolution::ResolvedCustom | ConflictResolution::Unresolved => {
                            result.extend(extract_left_lines(session, hunk));
                        }
                    }
                    diff_idx += 1;
                }
            }
        }
        Ok(result)
    }
}

fn extract_left_lines(session: &CompareSession, hunk: &DiffHunk) -> Vec<String> {
    match hunk {
        DiffHunk::Equal {
            left_start, count, ..
        } => session.left_lines[*left_start..*left_start + count].to_vec(),
        DiffHunk::Removed { left_start, count } => {
            session.left_lines[*left_start..*left_start + count].to_vec()
        }
        DiffHunk::Changed {
            left_start,
            left_count,
            ..
        } => session.left_lines[*left_start..*left_start + left_count].to_vec(),
        DiffHunk::Added { .. } => vec![],
    }
}

fn extract_right_lines(session: &CompareSession, hunk: &DiffHunk) -> Vec<String> {
    match hunk {
        DiffHunk::Equal {
            right_start, count, ..
        } => session.right_lines[*right_start..*right_start + count].to_vec(),
        DiffHunk::Added { right_start, count } => {
            session.right_lines[*right_start..*right_start + count].to_vec()
        }
        DiffHunk::Changed {
            right_start,
            right_count,
            ..
        } => session.right_lines[*right_start..*right_start + right_count].to_vec(),
        DiffHunk::Removed { .. } => vec![],
    }
}

/// Three-way merge engine.
pub struct ThreeWayMerge;

impl ThreeWayMerge {
    /// Perform a three-way merge computation.
    pub fn merge(
        base: &[&str],
        left: &[&str],
        right: &[&str],
        options: &crate::options::CompareOptions,
    ) -> Vec<ThreeWayRegion> {
        use crate::session::DiffEngine;

        let base_to_left = DiffEngine::diff(base, left, options);
        let base_to_right = DiffEngine::diff(base, right, options);

        // Build a line-by-line classification of the base
        let base_len = base.len();
        // For each base line: is it changed in left? changed in right?
        let mut left_changed = vec![false; base_len.max(1)];
        let mut right_changed = vec![false; base_len.max(1)];

        for hunk in &base_to_left.hunks {
            match hunk {
                DiffHunk::Removed { left_start, count }
                | DiffHunk::Changed {
                    left_start,
                    left_count: count,
                    ..
                } => {
                    for i in 0..*count {
                        if left_start + i < left_changed.len() {
                            left_changed[left_start + i] = true;
                        }
                    }
                }
                _ => {}
            }
        }

        for hunk in &base_to_right.hunks {
            match hunk {
                DiffHunk::Removed { left_start, count }
                | DiffHunk::Changed {
                    left_start,
                    left_count: count,
                    ..
                } => {
                    for i in 0..*count {
                        if left_start + i < right_changed.len() {
                            right_changed[left_start + i] = true;
                        }
                    }
                }
                _ => {}
            }
        }

        // Build regions by walking base lines
        let mut regions = Vec::new();
        let mut i = 0;

        while i < base_len {
            let lc = left_changed[i];
            let rc = right_changed[i];

            match (lc, rc) {
                (false, false) => {
                    // Unchanged region
                    let start = i;
                    while i < base_len && !left_changed[i] && !right_changed[i] {
                        i += 1;
                    }
                    regions.push(ThreeWayRegion::Unchanged {
                        lines: base[start..i].iter().map(|s| s.to_string()).collect(),
                    });
                }
                (true, false) => {
                    // Left-only change
                    let start = i;
                    while i < base_len && left_changed[i] && !right_changed[i] {
                        i += 1;
                    }
                    // Find corresponding left lines
                    let left_lines = find_replacement_lines(&base_to_left, left, start, i - start);
                    regions.push(ThreeWayRegion::LeftOnlyChange { lines: left_lines });
                }
                (false, true) => {
                    // Right-only change
                    let start = i;
                    while i < base_len && !left_changed[i] && right_changed[i] {
                        i += 1;
                    }
                    let right_lines =
                        find_replacement_lines(&base_to_right, right, start, i - start);
                    regions.push(ThreeWayRegion::RightOnlyChange { lines: right_lines });
                }
                (true, true) => {
                    // Conflict
                    let start = i;
                    while i < base_len && left_changed[i] && right_changed[i] {
                        i += 1;
                    }
                    let base_lines: Vec<String> =
                        base[start..i].iter().map(|s| s.to_string()).collect();
                    let left_lines = find_replacement_lines(&base_to_left, left, start, i - start);
                    let right_lines =
                        find_replacement_lines(&base_to_right, right, start, i - start);

                    // If left and right made the same change, it's not a conflict
                    if left_lines == right_lines {
                        regions.push(ThreeWayRegion::LeftOnlyChange { lines: left_lines });
                    } else {
                        regions.push(ThreeWayRegion::Conflict(MergeConflict {
                            base_lines,
                            left_lines,
                            right_lines,
                            result_start: 0,
                            status: ConflictResolution::Unresolved,
                        }));
                    }
                }
            }
        }

        // Handle additions at the end (lines added in left or right beyond base)
        // These are handled by the diff hunks for Added type
        for hunk in &base_to_left.hunks {
            if let DiffHunk::Added { right_start, count } = hunk {
                if *right_start >= base_len {
                    let lines: Vec<String> = left[*right_start..*right_start + count]
                        .iter()
                        .map(|s| s.to_string())
                        .collect();
                    regions.push(ThreeWayRegion::LeftOnlyChange { lines });
                }
            }
        }

        regions
    }

    /// Build the auto-resolved result, leaving conflicts as markers.
    pub fn build_auto_result(regions: &[ThreeWayRegion]) -> (Vec<String>, Vec<MergeConflict>) {
        let mut result = Vec::new();
        let mut conflicts = Vec::new();

        for region in regions {
            match region {
                ThreeWayRegion::Unchanged { lines } => result.extend(lines.clone()),
                ThreeWayRegion::LeftOnlyChange { lines } => result.extend(lines.clone()),
                ThreeWayRegion::RightOnlyChange { lines } => result.extend(lines.clone()),
                ThreeWayRegion::Conflict(conflict) => {
                    let mut c = conflict.clone();
                    c.result_start = result.len();
                    conflicts.push(c);
                    // Leave a placeholder in the result
                    result.push("<<<<<<< LEFT".to_string());
                    result.extend(conflict.left_lines.clone());
                    result.push("=======".to_string());
                    result.extend(conflict.right_lines.clone());
                    result.push(">>>>>>> RIGHT".to_string());
                }
            }
        }

        (result, conflicts)
    }
}

fn find_replacement_lines(
    diff: &crate::result::DiffResult,
    target: &[&str],
    base_start: usize,
    _base_count: usize,
) -> Vec<String> {
    // Find the right-side lines corresponding to the changed base region
    for hunk in &diff.hunks {
        match hunk {
            DiffHunk::Changed {
                left_start,
                right_start,
                right_count,
                ..
            } if *left_start == base_start => {
                return target[*right_start..*right_start + right_count]
                    .iter()
                    .map(|s| s.to_string())
                    .collect();
            }
            DiffHunk::Added { right_start, count } if *right_start >= base_start => {
                return target[*right_start..*right_start + count]
                    .iter()
                    .map(|s| s.to_string())
                    .collect();
            }
            _ => {}
        }
    }
    vec![]
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::navigator::{CompareSession, CompareSource, SessionId};
    use crate::options::CompareOptions;

    fn make_session(left: &[&str], right: &[&str]) -> CompareSession {
        CompareSession::new(
            SessionId::new(1),
            CompareSource::Clipboard {
                label: "left".to_string(),
            },
            CompareSource::Clipboard {
                label: "right".to_string(),
            },
            left.iter().map(|s| s.to_string()).collect(),
            right.iter().map(|s| s.to_string()).collect(),
            CompareOptions::default(),
        )
    }

    #[test]
    fn accept_left_returns_left_content() {
        // Validates: Requirement 7.2 — accept_left returns left version
        let session = make_session(&["a", "b"], &["a", "x"]);
        let result = MergeResolver::accept_left(&session, 0).expect("accept_left");
        assert_eq!(result, vec!["b".to_string()]);
    }

    #[test]
    fn accept_right_returns_right_content() {
        // Validates: Requirement 7.3 — accept_right returns right version
        let session = make_session(&["a", "b"], &["a", "x"]);
        let result = MergeResolver::accept_right(&session, 0).expect("accept_right");
        assert_eq!(result, vec!["x".to_string()]);
    }

    #[test]
    fn accept_both_concatenates_left_then_right() {
        // Validates: Requirement 7.4 — accept_both concatenates both
        let session = make_session(&["a", "b"], &["a", "x"]);
        let result = MergeResolver::accept_both(&session, 0).expect("accept_both");
        assert_eq!(result, vec!["b".to_string(), "x".to_string()]);
    }

    #[test]
    fn accept_left_out_of_range_returns_error() {
        // Validates: Requirement 7 — error on invalid hunk index
        let session = make_session(&["a"], &["a"]);
        let result = MergeResolver::accept_left(&session, 99);
        assert!(result.is_err());
    }

    #[test]
    fn build_result_fails_with_unresolved_hunks() {
        // Validates: Requirement 7.9 — unresolved conflicts prevent build
        let session = make_session(&["a", "b"], &["a", "x"]);
        let result = MergeResolver::build_result(&session);
        assert!(matches!(
            result,
            Err(CompareError::UnresolvedConflicts { .. })
        ));
    }

    #[test]
    fn build_result_succeeds_when_all_resolved() {
        // Validates: Requirement 7.9 — all resolved → build succeeds
        let mut session = make_session(&["a", "b"], &["a", "x"]);
        session.hunk_resolutions[0] = ConflictResolution::ResolvedLeft;
        let result = MergeResolver::build_result(&session).expect("build_result");
        assert!(!result.is_empty());
    }

    #[test]
    fn three_way_unchanged_region_auto_resolved() {
        // Validates: Requirement 8.4 — unchanged regions auto-resolved
        let base = vec!["a", "b", "c"];
        let left = vec!["a", "b", "c"];
        let right = vec!["a", "b", "c"];
        let regions = ThreeWayMerge::merge(&base, &left, &right, &CompareOptions::default());
        assert!(regions
            .iter()
            .all(|r| matches!(r, ThreeWayRegion::Unchanged { .. })));
    }

    #[test]
    fn three_way_left_only_change_auto_resolved() {
        // Validates: Requirement 8.5 — left-only changes auto-resolved to left
        let base = vec!["a", "b", "c"];
        let left = vec!["a", "X", "c"];
        let right = vec!["a", "b", "c"];
        let regions = ThreeWayMerge::merge(&base, &left, &right, &CompareOptions::default());
        let has_left_only = regions
            .iter()
            .any(|r| matches!(r, ThreeWayRegion::LeftOnlyChange { .. }));
        assert!(has_left_only);
        let has_conflict = regions
            .iter()
            .any(|r| matches!(r, ThreeWayRegion::Conflict(_)));
        assert!(!has_conflict);
    }

    #[test]
    fn three_way_identical_changes_not_conflict() {
        // Validates: Requirement 8 Property 11 — identical changes are not conflicts
        let base = vec!["a", "b", "c"];
        let left = vec!["a", "X", "c"];
        let right = vec!["a", "X", "c"];
        let regions = ThreeWayMerge::merge(&base, &left, &right, &CompareOptions::default());
        let has_conflict = regions
            .iter()
            .any(|r| matches!(r, ThreeWayRegion::Conflict(_)));
        assert!(!has_conflict);
    }

    #[test]
    fn build_auto_result_includes_conflict_markers() {
        // Validates: Requirement 8.8 — conflict markers in auto result
        let conflict = MergeConflict {
            base_lines: vec!["b".to_string()],
            left_lines: vec!["X".to_string()],
            right_lines: vec!["Y".to_string()],
            result_start: 0,
            status: ConflictResolution::Unresolved,
        };
        let regions = vec![ThreeWayRegion::Conflict(conflict)];
        let (result, conflicts) = ThreeWayMerge::build_auto_result(&regions);
        assert_eq!(conflicts.len(), 1);
        assert!(result.iter().any(|l| l.contains("LEFT")));
        assert!(result.iter().any(|l| l.contains("RIGHT")));
    }
}
