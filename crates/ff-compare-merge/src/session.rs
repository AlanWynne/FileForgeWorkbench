//! Diff engine: Myers and Patience algorithms operating on line slices.
//!
//! This module is re-exported from the crate root via the `session` module.
//! It is also used directly by tests.

use crate::options::{CompareOptions, DiffAlgorithm};
use crate::result::{DiffHunk, DiffResult, InlineChange};

/// The core comparison engine. Stateless — all configuration is passed per call.
pub struct DiffEngine;

impl DiffEngine {
    /// Compute the diff between two sequences of lines.
    pub fn diff(left: &[&str], right: &[&str], options: &CompareOptions) -> DiffResult {
        // Normalise lines according to options
        let left_norm: Vec<String> = left
            .iter()
            .map(|l| options.normalise(l).into_owned())
            .collect();
        let right_norm: Vec<String> = right
            .iter()
            .map(|l| options.normalise(l).into_owned())
            .collect();
        let left_refs: Vec<&str> = left_norm.iter().map(String::as_str).collect();
        let right_refs: Vec<String> = right_norm.to_vec();
        let right_refs: Vec<&str> = right_refs.iter().map(String::as_str).collect();

        let raw_hunks = match options.algorithm {
            DiffAlgorithm::Myers => myers_diff(&left_refs, &right_refs),
            DiffAlgorithm::Patience => patience_diff(&left_refs, &right_refs),
        };

        // Attach inline changes to Changed hunks, using original (non-normalised) lines
        let hunks = raw_hunks
            .into_iter()
            .map(|hunk| match hunk {
                DiffHunk::Changed {
                    left_start,
                    left_count,
                    right_start,
                    right_count,
                    ..
                } => {
                    let inline_changes = compute_inline_changes(
                        left,
                        right,
                        left_start,
                        left_count,
                        right_start,
                        right_count,
                    );
                    DiffHunk::Changed {
                        left_start,
                        left_count,
                        right_start,
                        right_count,
                        inline_changes,
                    }
                }
                other => other,
            })
            .collect();

        DiffResult::new(hunks)
    }

    /// Compute inline character-level changes for a pair of changed lines.
    pub fn inline_diff(left_line: &str, right_line: &str) -> Vec<InlineChange> {
        char_diff(left_line, right_line)
    }
}

// ─── Myers Diff ─────────────────────────────────────────────────────────────

/// Myers O(ND) diff algorithm. Returns a sequence of DiffHunks.
fn myers_diff(left: &[&str], right: &[&str]) -> Vec<DiffHunk> {
    if left.is_empty() && right.is_empty() {
        return vec![];
    }
    if left.is_empty() {
        return vec![DiffHunk::Added {
            right_start: 0,
            count: right.len(),
        }];
    }
    if right.is_empty() {
        return vec![DiffHunk::Removed {
            left_start: 0,
            count: left.len(),
        }];
    }
    if left == right {
        return vec![DiffHunk::Equal {
            left_start: 0,
            right_start: 0,
            count: left.len(),
        }];
    }

    let edit_script = myers_ses(left, right);
    build_hunks_from_ses(left, right, &edit_script)
}

/// Edit operation in the shortest edit script.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
enum EditOp {
    Equal,
    Insert,
    Delete,
}

/// Compute the shortest edit script using Myers algorithm.
fn myers_ses(left: &[&str], right: &[&str]) -> Vec<EditOp> {
    let n = left.len();
    let m = right.len();
    let max = n + m;

    if max == 0 {
        return vec![];
    }

    let offset = max as isize;
    let size = 2 * max + 1;
    let mut v: Vec<isize> = vec![0; size];
    let mut trace: Vec<Vec<isize>> = Vec::new();

    'outer: for d in 0..=(max as isize) {
        trace.push(v.clone());
        let mut k = -d;
        while k <= d {
            let idx = (k + offset) as usize;
            let mut x = if k == -d
                || (k != d && v[(k - 1 + offset) as usize] < v[(k + 1 + offset) as usize])
            {
                v[(k + 1 + offset) as usize]
            } else {
                v[(k - 1 + offset) as usize] + 1
            };
            let mut y = x - k;
            while x < n as isize && y < m as isize && left[x as usize] == right[y as usize] {
                x += 1;
                y += 1;
            }
            v[idx] = x;
            if x >= n as isize && y >= m as isize {
                break 'outer;
            }
            k += 2;
        }
    }

    // Backtrack to recover the edit script
    let mut ops = Vec::new();
    let mut x = n as isize;
    let mut y = m as isize;

    for d in (0..trace.len() as isize).rev() {
        let v_prev = &trace[d as usize];
        let k = x - y;
        let _unused = (k + offset) as usize;

        let prev_k = if k == -d
            || (k != d
                && v_prev.get((k - 1 + offset) as usize).copied().unwrap_or(-1)
                    < v_prev.get((k + 1 + offset) as usize).copied().unwrap_or(-1))
        {
            k + 1
        } else {
            k - 1
        };

        let prev_x = v_prev.get((prev_k + offset) as usize).copied().unwrap_or(0);
        let prev_y = prev_x - prev_k;

        // Diagonal moves (equal)
        while x > prev_x && y > prev_y {
            ops.push(EditOp::Equal);
            x -= 1;
            y -= 1;
        }

        if d > 0 {
            if x == prev_x {
                ops.push(EditOp::Insert);
                y -= 1;
            } else {
                ops.push(EditOp::Delete);
                x -= 1;
            }
        }
    }

    ops.reverse();
    ops
}

/// Build DiffHunks from a shortest edit script.
fn build_hunks_from_ses(_left: &[&str], _right: &[&str], ops: &[EditOp]) -> Vec<DiffHunk> {
    let mut hunks: Vec<DiffHunk> = Vec::new();
    let mut li = 0usize; // left index
    let mut ri = 0usize; // right index

    let mut i = 0;
    while i < ops.len() {
        match ops[i] {
            EditOp::Equal => {
                let start_l = li;
                let start_r = ri;
                let mut count = 0;
                while i < ops.len() && ops[i] == EditOp::Equal {
                    li += 1;
                    ri += 1;
                    count += 1;
                    i += 1;
                }
                hunks.push(DiffHunk::Equal {
                    left_start: start_l,
                    right_start: start_r,
                    count,
                });
            }
            EditOp::Delete => {
                let start_l = li;
                let start_r = ri;
                let mut del_count = 0;
                let mut ins_count = 0;
                while i < ops.len() && (ops[i] == EditOp::Delete || ops[i] == EditOp::Insert) {
                    match ops[i] {
                        EditOp::Delete => {
                            li += 1;
                            del_count += 1;
                        }
                        EditOp::Insert => {
                            ri += 1;
                            ins_count += 1;
                        }
                        _ => unreachable!(),
                    }
                    i += 1;
                }
                if del_count > 0 && ins_count > 0 {
                    hunks.push(DiffHunk::Changed {
                        left_start: start_l,
                        left_count: del_count,
                        right_start: start_r,
                        right_count: ins_count,
                        inline_changes: vec![],
                    });
                } else if del_count > 0 {
                    hunks.push(DiffHunk::Removed {
                        left_start: start_l,
                        count: del_count,
                    });
                } else {
                    hunks.push(DiffHunk::Added {
                        right_start: start_r,
                        count: ins_count,
                    });
                }
            }
            EditOp::Insert => {
                let start_l = li;
                let start_r = ri;
                let mut del_count = 0;
                let mut ins_count = 0;
                while i < ops.len() && (ops[i] == EditOp::Delete || ops[i] == EditOp::Insert) {
                    match ops[i] {
                        EditOp::Delete => {
                            li += 1;
                            del_count += 1;
                        }
                        EditOp::Insert => {
                            ri += 1;
                            ins_count += 1;
                        }
                        _ => unreachable!(),
                    }
                    i += 1;
                }
                if del_count > 0 && ins_count > 0 {
                    hunks.push(DiffHunk::Changed {
                        left_start: start_l,
                        left_count: del_count,
                        right_start: start_r,
                        right_count: ins_count,
                        inline_changes: vec![],
                    });
                } else if del_count > 0 {
                    hunks.push(DiffHunk::Removed {
                        left_start: start_l,
                        count: del_count,
                    });
                } else {
                    hunks.push(DiffHunk::Added {
                        right_start: start_r,
                        count: ins_count,
                    });
                }
            }
        }
    }

    hunks
}

// ─── Patience Diff ──────────────────────────────────────────────────────────

/// Patience diff: anchors on unique matching lines, fills between with Myers.
fn patience_diff(left: &[&str], right: &[&str]) -> Vec<DiffHunk> {
    if left.is_empty() && right.is_empty() {
        return vec![];
    }
    if left.is_empty() {
        return vec![DiffHunk::Added {
            right_start: 0,
            count: right.len(),
        }];
    }
    if right.is_empty() {
        return vec![DiffHunk::Removed {
            left_start: 0,
            count: left.len(),
        }];
    }
    if left == right {
        return vec![DiffHunk::Equal {
            left_start: 0,
            right_start: 0,
            count: left.len(),
        }];
    }

    // Find unique lines that appear exactly once in both left and right
    let anchors = find_patience_anchors(left, right);

    if anchors.is_empty() {
        // No unique anchors — fall back to Myers
        return myers_diff(left, right);
    }

    // Build hunks by filling between anchors with Myers
    let mut hunks = Vec::new();
    let mut prev_l = 0usize;
    let mut prev_r = 0usize;

    for (al, ar) in &anchors {
        let al = *al;
        let ar = *ar;

        // Fill the gap before this anchor
        if al > prev_l || ar > prev_r {
            let gap_l = &left[prev_l..al];
            let gap_r = &right[prev_r..ar];
            let gap_hunks = myers_diff(gap_l, gap_r);
            for h in gap_hunks {
                hunks.push(offset_hunk(h, prev_l, prev_r));
            }
        }

        // The anchor itself is an Equal hunk
        hunks.push(DiffHunk::Equal {
            left_start: al,
            right_start: ar,
            count: 1,
        });
        prev_l = al + 1;
        prev_r = ar + 1;
    }

    // Fill the tail after the last anchor
    if prev_l < left.len() || prev_r < right.len() {
        let tail_l = &left[prev_l..];
        let tail_r = &right[prev_r..];
        let tail_hunks = myers_diff(tail_l, tail_r);
        for h in tail_hunks {
            hunks.push(offset_hunk(h, prev_l, prev_r));
        }
    }

    merge_adjacent_equal(hunks)
}

/// Find unique-line anchors for patience diff.
fn find_patience_anchors(left: &[&str], right: &[&str]) -> Vec<(usize, usize)> {
    use std::collections::HashMap;

    // Count occurrences in left
    let mut left_count: HashMap<&str, usize> = HashMap::new();
    let mut left_pos: HashMap<&str, usize> = HashMap::new();
    for (i, &line) in left.iter().enumerate() {
        *left_count.entry(line).or_insert(0) += 1;
        left_pos.insert(line, i);
    }

    // Count occurrences in right
    let mut right_count: HashMap<&str, usize> = HashMap::new();
    let mut right_pos: HashMap<&str, usize> = HashMap::new();
    for (i, &line) in right.iter().enumerate() {
        *right_count.entry(line).or_insert(0) += 1;
        right_pos.insert(line, i);
    }

    // Unique lines appearing exactly once in both
    let mut anchors: Vec<(usize, usize)> = left_count
        .iter()
        .filter(|(&line, &lc)| lc == 1 && right_count.get(line).copied() == Some(1))
        .map(|(&line, _)| (left_pos[line], right_pos[line]))
        .collect();

    // Sort by left position, then verify right positions are increasing (LCS of anchors)
    anchors.sort_by_key(|&(l, _)| l);
    // Keep only the longest increasing subsequence by right position
    lis_by_right(&anchors)
}

/// Longest increasing subsequence of anchors by right position.
fn lis_by_right(anchors: &[(usize, usize)]) -> Vec<(usize, usize)> {
    if anchors.is_empty() {
        return vec![];
    }
    let mut tails: Vec<usize> = Vec::new(); // right positions of LIS tails
    let mut prev: Vec<Option<usize>> = vec![None; anchors.len()];
    let mut indices: Vec<usize> = Vec::new(); // index into anchors for each tail

    for (i, &(_, r)) in anchors.iter().enumerate() {
        let pos = tails.partition_point(|&t| t < r);
        if pos == tails.len() {
            tails.push(r);
            indices.push(i);
        } else {
            tails[pos] = r;
            indices[pos] = i;
        }
        prev[i] = if pos > 0 {
            Some(indices[pos - 1])
        } else {
            None
        };
    }

    // Reconstruct
    let mut result = Vec::new();
    let mut cur = Some(*indices.last().unwrap());
    while let Some(idx) = cur {
        result.push(anchors[idx]);
        cur = prev[idx];
    }
    result.reverse();
    result
}

fn offset_hunk(hunk: DiffHunk, left_offset: usize, right_offset: usize) -> DiffHunk {
    match hunk {
        DiffHunk::Equal {
            left_start,
            right_start,
            count,
        } => DiffHunk::Equal {
            left_start: left_start + left_offset,
            right_start: right_start + right_offset,
            count,
        },
        DiffHunk::Added { right_start, count } => DiffHunk::Added {
            right_start: right_start + right_offset,
            count,
        },
        DiffHunk::Removed { left_start, count } => DiffHunk::Removed {
            left_start: left_start + left_offset,
            count,
        },
        DiffHunk::Changed {
            left_start,
            left_count,
            right_start,
            right_count,
            inline_changes,
        } => DiffHunk::Changed {
            left_start: left_start + left_offset,
            left_count,
            right_start: right_start + right_offset,
            right_count,
            inline_changes,
        },
    }
}

fn merge_adjacent_equal(hunks: Vec<DiffHunk>) -> Vec<DiffHunk> {
    let mut result: Vec<DiffHunk> = Vec::new();
    for hunk in hunks {
        match (&mut result.last_mut(), &hunk) {
            (
                Some(DiffHunk::Equal {
                    count: prev_count, ..
                }),
                DiffHunk::Equal {
                    count: new_count, ..
                },
            ) => {
                *prev_count += new_count;
            }
            _ => result.push(hunk),
        }
    }
    result
}

// ─── Inline Change Detection ─────────────────────────────────────────────────

fn compute_inline_changes(
    left: &[&str],
    right: &[&str],
    left_start: usize,
    left_count: usize,
    right_start: usize,
    right_count: usize,
) -> Vec<InlineChange> {
    // Only compute inline changes for 1:1 changed line pairs
    if left_count != 1 || right_count != 1 {
        return vec![];
    }
    let l = left.get(left_start).copied().unwrap_or("");
    let r = right.get(right_start).copied().unwrap_or("");
    char_diff(l, r)
}

/// Character-level diff between two strings using Myers on bytes.
fn char_diff(left: &str, right: &str) -> Vec<InlineChange> {
    let lb: Vec<u8> = left.bytes().collect();
    let rb: Vec<u8> = right.bytes().collect();

    if lb == rb {
        return vec![];
    }

    // Find common prefix
    let prefix = lb.iter().zip(rb.iter()).take_while(|(a, b)| a == b).count();
    // Find common suffix (after prefix)
    let suffix = lb[prefix..]
        .iter()
        .rev()
        .zip(rb[prefix..].iter().rev())
        .take_while(|(a, b)| a == b)
        .count();

    let l_end = lb.len() - suffix;
    let r_end = rb.len() - suffix;

    if prefix >= l_end && prefix >= r_end {
        return vec![];
    }

    vec![InlineChange {
        left_range: prefix..l_end,
        right_range: prefix..r_end,
    }]
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::options::CompareOptions;
    use crate::result::DiffHunk;

    fn opts() -> CompareOptions {
        CompareOptions::default()
    }

    #[test]
    fn identical_inputs_produce_single_equal_hunk() {
        // Validates: Requirement 2.4 — identical inputs → single Equal hunk
        let lines = vec!["a", "b", "c"];
        let result = DiffEngine::diff(&lines, &lines, &opts());
        assert_eq!(result.hunks.len(), 1);
        assert!(result.hunks[0].is_equal());
        assert_eq!(result.statistics.hunks_count, 0);
    }

    #[test]
    fn empty_left_produces_single_added_hunk() {
        // Validates: Requirement 2.5 — empty left → single Added hunk
        let right = vec!["a", "b"];
        let result = DiffEngine::diff(&[], &right, &opts());
        assert_eq!(result.hunks.len(), 1);
        assert!(matches!(result.hunks[0], DiffHunk::Added { count: 2, .. }));
    }

    #[test]
    fn empty_right_produces_single_removed_hunk() {
        // Validates: Requirement 2.5 — empty right → single Removed hunk
        let left = vec!["a", "b"];
        let result = DiffEngine::diff(&left, &[], &opts());
        assert_eq!(result.hunks.len(), 1);
        assert!(matches!(
            result.hunks[0],
            DiffHunk::Removed { count: 2, .. }
        ));
    }

    #[test]
    fn both_empty_produces_no_hunks() {
        // Validates: Requirement 2.4 — both empty → no hunks
        let result = DiffEngine::diff(&[], &[], &opts());
        assert!(result.hunks.is_empty());
    }

    #[test]
    fn diff_covers_all_lines() {
        // Validates: Property 1 — all lines covered
        let left = vec!["a", "b", "c", "d"];
        let right = vec!["a", "x", "c", "y", "z"];
        let result = DiffEngine::diff(&left, &right, &opts());
        let left_covered: usize = result.hunks.iter().map(|h| h.left_line_count()).sum();
        let right_covered: usize = result.hunks.iter().map(|h| h.right_line_count()).sum();
        assert_eq!(left_covered, left.len());
        assert_eq!(right_covered, right.len());
    }

    #[test]
    fn diff_is_deterministic() {
        // Validates: Requirement 2.9 — deterministic output
        let left = vec!["a", "b", "c"];
        let right = vec!["a", "x", "c"];
        let r1 = DiffEngine::diff(&left, &right, &opts());
        let r2 = DiffEngine::diff(&left, &right, &opts());
        assert_eq!(r1, r2);
    }

    #[test]
    fn ignore_whitespace_all_treats_whitespace_lines_as_equal() {
        // Validates: Requirement 2.6 — ignore_whitespace All
        let left = vec!["hello world"];
        let right = vec!["hello   world"];
        let opts = CompareOptions {
            whitespace_mode: crate::options::WhitespaceMode::All,
            ..Default::default()
        };
        let result = DiffEngine::diff(&left, &right, &opts);
        assert_eq!(result.hunks.len(), 1);
        assert!(result.hunks[0].is_equal());
    }

    #[test]
    fn ignore_case_treats_case_variants_as_equal() {
        // Validates: Requirement 2.7 — ignore_case
        let left = vec!["Hello World"];
        let right = vec!["hello world"];
        let opts = CompareOptions {
            ignore_case: true,
            ..Default::default()
        };
        let result = DiffEngine::diff(&left, &right, &opts);
        assert_eq!(result.hunks.len(), 1);
        assert!(result.hunks[0].is_equal());
    }

    #[test]
    fn inline_diff_detects_changed_region() {
        // Validates: Requirement 2.8 — inline change detection
        let changes = DiffEngine::inline_diff("hello world", "hello rust");
        assert!(!changes.is_empty());
        // The changed region should cover "world" vs "rust"
        let change = &changes[0];
        assert!(change.left_range.start >= 6);
    }

    #[test]
    fn inline_diff_identical_lines_no_changes() {
        // Validates: Requirement 2.8 — no inline changes for identical lines
        let changes = DiffEngine::inline_diff("same line", "same line");
        assert!(changes.is_empty());
    }

    #[test]
    fn patience_diff_identical_inputs() {
        // Validates: Requirement 2.1a — patience handles identical inputs
        let lines = vec!["a", "b", "c"];
        let opts = CompareOptions {
            algorithm: crate::options::DiffAlgorithm::Patience,
            ..Default::default()
        };
        let result = DiffEngine::diff(&lines, &lines, &opts);
        assert_eq!(result.hunks.len(), 1);
        assert!(result.hunks[0].is_equal());
    }

    #[test]
    fn patience_diff_covers_all_lines() {
        // Validates: Property 1 — patience covers all lines
        let left = vec![
            "fn foo() {",
            "    let x = 1;",
            "}",
            "fn bar() {",
            "    let y = 2;",
            "}",
        ];
        let right = vec![
            "fn foo() {",
            "    let x = 42;",
            "}",
            "fn baz() {",
            "    let z = 3;",
            "}",
        ];
        let opts = CompareOptions {
            algorithm: crate::options::DiffAlgorithm::Patience,
            ..Default::default()
        };
        let result = DiffEngine::diff(&left, &right, &opts);
        let left_covered: usize = result.hunks.iter().map(|h| h.left_line_count()).sum();
        let right_covered: usize = result.hunks.iter().map(|h| h.right_line_count()).sum();
        assert_eq!(left_covered, left.len());
        assert_eq!(right_covered, right.len());
    }

    #[test]
    fn hunk_positions_are_monotonically_increasing() {
        // Validates: Property 14 — hunk positions are ordered
        let left = vec!["a", "b", "c", "d", "e"];
        let right = vec!["a", "x", "c", "y", "e"];
        let result = DiffEngine::diff(&left, &right, &opts());
        let mut prev_l_end = 0usize;
        let mut prev_r_end = 0usize;
        for hunk in &result.hunks {
            assert!(hunk.left_start() >= prev_l_end || hunk.left_line_count() == 0);
            assert!(hunk.right_start() >= prev_r_end || hunk.right_line_count() == 0);
            prev_l_end = hunk.left_start() + hunk.left_line_count();
            prev_r_end = hunk.right_start() + hunk.right_line_count();
        }
    }

    #[test]
    fn statistics_consistent_with_hunks() {
        // Validates: Property 7 — statistics match hunk data
        let left = vec!["a", "b", "c", "d"];
        let right = vec!["a", "x", "c", "y", "z"];
        let result = DiffEngine::diff(&left, &right, &opts());
        let computed = crate::result::DiffStatistics::from_hunks(&result.hunks);
        assert_eq!(result.statistics, computed);
    }
}
