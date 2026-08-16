//! DiffExporter — generates unified diff format output from a DiffResult.

use crate::result::{DiffHunk, DiffResult};

/// Generates unified diff format output from a DiffResult.
pub struct DiffExporter;

impl DiffExporter {
    /// Export the diff as a unified diff format string.
    pub fn export(
        left_path: &str,
        right_path: &str,
        left_lines: &[&str],
        right_lines: &[&str],
        diff_result: &DiffResult,
        context_lines: usize,
        options_comment: Option<&str>,
    ) -> String {
        let mut output = String::new();

        if let Some(comment) = options_comment {
            output.push_str(&format!("# Options: {}\n", comment));
        }

        output.push_str(&format!("--- {}\n", left_path));
        output.push_str(&format!("+++ {}\n", right_path));

        let diff_hunks: Vec<&DiffHunk> = diff_result.diff_hunks().collect();
        for hunk in diff_hunks {
            let (l_start, l_count, r_start, r_count, lines) =
                render_hunk(hunk, left_lines, right_lines, context_lines);
            output.push_str(&format!(
                "@@ -{},{} +{},{} @@\n",
                l_start + 1,
                l_count,
                r_start + 1,
                r_count
            ));
            output.push_str(&lines);
        }

        output
    }
}

fn render_hunk(
    hunk: &DiffHunk,
    left_lines: &[&str],
    right_lines: &[&str],
    context: usize,
) -> (usize, usize, usize, usize, String) {
    let mut out = String::new();
    let mut l_start = 0usize;
    let mut r_start = 0usize;
    let mut l_count = 0usize;
    let mut r_count = 0usize;

    match hunk {
        DiffHunk::Removed { left_start, count } => {
            let ctx_start = left_start.saturating_sub(context);
            l_start = ctx_start;
            r_start = ctx_start;
            for i in ctx_start..*left_start {
                if let Some(line) = left_lines.get(i) {
                    out.push_str(&format!(" {}\n", line));
                    l_count += 1;
                    r_count += 1;
                }
            }
            for i in 0..*count {
                if let Some(line) = left_lines.get(left_start + i) {
                    out.push_str(&format!("-{}\n", line));
                    l_count += 1;
                }
            }
            for i in 0..context {
                if let Some(line) = left_lines.get(left_start + count + i) {
                    out.push_str(&format!(" {}\n", line));
                    l_count += 1;
                    r_count += 1;
                }
            }
        }
        DiffHunk::Added { right_start, count } => {
            let ctx_start = right_start.saturating_sub(context);
            l_start = ctx_start;
            r_start = ctx_start;
            for i in ctx_start..*right_start {
                if let Some(line) = right_lines.get(i) {
                    out.push_str(&format!(" {}\n", line));
                    l_count += 1;
                    r_count += 1;
                }
            }
            for i in 0..*count {
                if let Some(line) = right_lines.get(right_start + i) {
                    out.push_str(&format!("+{}\n", line));
                    r_count += 1;
                }
            }
            for i in 0..context {
                if let Some(line) = right_lines.get(right_start + count + i) {
                    out.push_str(&format!(" {}\n", line));
                    l_count += 1;
                    r_count += 1;
                }
            }
        }
        DiffHunk::Changed {
            left_start,
            left_count,
            right_start,
            right_count,
            ..
        } => {
            let ctx_start = left_start.saturating_sub(context);
            l_start = ctx_start;
            r_start = right_start.saturating_sub(context);
            for i in ctx_start..*left_start {
                if let Some(line) = left_lines.get(i) {
                    out.push_str(&format!(" {}\n", line));
                    l_count += 1;
                    r_count += 1;
                }
            }
            for i in 0..*left_count {
                if let Some(line) = left_lines.get(left_start + i) {
                    out.push_str(&format!("-{}\n", line));
                    l_count += 1;
                }
            }
            for i in 0..*right_count {
                if let Some(line) = right_lines.get(right_start + i) {
                    out.push_str(&format!("+{}\n", line));
                    r_count += 1;
                }
            }
            for i in 0..context {
                if let Some(line) = left_lines.get(left_start + left_count + i) {
                    out.push_str(&format!(" {}\n", line));
                    l_count += 1;
                    r_count += 1;
                }
            }
        }
        DiffHunk::Equal { .. } => {}
    }

    (l_start, l_count, r_start, r_count, out)
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::options::CompareOptions;
    use crate::session::DiffEngine;

    #[test]
    fn export_includes_headers() {
        // Validates: Requirement 17.2 — unified diff headers
        let left = vec!["a", "b", "c"];
        let right = vec!["a", "x", "c"];
        let result = DiffEngine::diff(&left, &right, &CompareOptions::default());
        let output = DiffExporter::export("left.txt", "right.txt", &left, &right, &result, 3, None);
        assert!(output.contains("--- left.txt"));
        assert!(output.contains("+++ right.txt"));
    }

    #[test]
    fn export_includes_hunk_header() {
        // Validates: Requirement 17.3 — @@ range header
        let left = vec!["a", "b", "c"];
        let right = vec!["a", "x", "c"];
        let result = DiffEngine::diff(&left, &right, &CompareOptions::default());
        let output = DiffExporter::export("left.txt", "right.txt", &left, &right, &result, 0, None);
        assert!(output.contains("@@"));
    }

    #[test]
    fn export_includes_removed_lines_with_minus() {
        // Validates: Requirement 17.3 — removed lines prefixed with -
        let left = vec!["a", "b", "c"];
        let right = vec!["a", "c"];
        let result = DiffEngine::diff(&left, &right, &CompareOptions::default());
        let output = DiffExporter::export("left.txt", "right.txt", &left, &right, &result, 0, None);
        assert!(output.contains("-b"));
    }

    #[test]
    fn export_includes_added_lines_with_plus() {
        // Validates: Requirement 17.3 — added lines prefixed with +
        let left = vec!["a", "c"];
        let right = vec!["a", "b", "c"];
        let result = DiffEngine::diff(&left, &right, &CompareOptions::default());
        let output = DiffExporter::export("left.txt", "right.txt", &left, &right, &result, 0, None);
        assert!(output.contains("+b"));
    }

    #[test]
    fn export_with_options_comment() {
        // Validates: Requirement 17.7 — options comment header
        let left = vec!["a"];
        let right = vec!["b"];
        let result = DiffEngine::diff(&left, &right, &CompareOptions::default());
        let output = DiffExporter::export(
            "left.txt",
            "right.txt",
            &left,
            &right,
            &result,
            0,
            Some("ignore_whitespace=leading_trailing"),
        );
        assert!(output.contains("# Options:"));
        assert!(output.contains("ignore_whitespace"));
    }

    #[test]
    fn export_identical_files_produces_no_hunks() {
        // Validates: Requirement 17 — no hunks for identical files
        let lines = vec!["a", "b", "c"];
        let result = DiffEngine::diff(&lines, &lines, &CompareOptions::default());
        let output =
            DiffExporter::export("left.txt", "right.txt", &lines, &lines, &result, 3, None);
        assert!(!output.contains("@@"));
    }
}
