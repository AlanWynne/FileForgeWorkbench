//! Global replace engine -- cross-file replacement via ff-file-ops.
//!
//! Addresses: Requirement 5

use ff_find_and_replace::indexer::MutableSliceIndexer;
use ff_find_and_replace::scope::AllLinesFilter;
use ff_find_and_replace::{ChangeOutcome, ChangeRequest, FindEngine, FindRequest, WordMatchMode};

use crate::error::GlobalSearchError;
use crate::result::FileMatches;
use crate::search::GlobalSearchRequest;

/// Summary of a completed replace operation.
///
/// Addresses: Requirement 5.4
#[derive(Debug, Clone)]
pub struct ReplaceSummary {
    /// Total number of replacements made.
    pub replacements: u64,
    /// Number of files modified.
    pub files_modified: u64,
}

/// Files that could not be replaced because they have unsaved editor changes.
///
/// Addresses: Requirement 5.6
#[derive(Debug, Clone)]
pub struct ConflictList {
    /// Paths of files with unsaved changes that were skipped.
    pub paths: Vec<String>,
}

/// Orchestrates cross-file replacement.
pub struct GlobalReplaceEngine;

impl GlobalReplaceEngine {
    /// Apply `replacement` to all matches in `file_matches`.
    ///
    /// Reads each file, applies substitutions, writes back via `std::fs::write`.
    /// Files listed in `unsaved_paths` are skipped and returned in `ConflictList`.
    ///
    /// Addresses: Requirement 5.3, 5.6
    pub fn replace_all(
        file_matches: &[FileMatches],
        request: &GlobalSearchRequest,
        replacement: &str,
        unsaved_paths: &[String],
    ) -> Result<(ReplaceSummary, ConflictList), GlobalSearchError> {
        let mut replacements: u64 = 0;
        let mut files_modified: u64 = 0;
        let mut conflict_paths: Vec<String> = Vec::new();

        for fm in file_matches {
            // Validates: Requirement 5.6 -- skip files with unsaved changes.
            if unsaved_paths.contains(&fm.file_path) {
                conflict_paths.push(fm.file_path.clone());
                continue;
            }

            let content = match std::fs::read_to_string(&fm.file_path) {
                Ok(s) => s,
                Err(_) => continue,
            };

            let word_match = if request.whole_word {
                WordMatchMode::WholeWord
            } else {
                WordMatchMode::None
            };

            let find_req = FindRequest {
                term: request.query.clone(),
                mode: request.mode,
                direction: ff_find_and_replace::SearchDirection::Next,
                scope: ff_find_and_replace::scope::ScopeModifier::All,
                case_sensitive: request.case_sensitive,
                word_match,
                column_range: None,
                cursor_position: ff_find_and_replace::types::BytePosition::ZERO,
            };
            let change_req = ChangeRequest::new(find_req, replacement);

            let mut indexer = MutableSliceIndexer::new(&content);
            let filter = AllLinesFilter;
            let mut engine = FindEngine::new();

            if let Ok(ChangeOutcome::Changed(r)) =
                engine.change_all(&change_req, &mut indexer, &filter, None)
            {
                let new_content = indexer.content_str().unwrap_or(&content).to_string();
                if std::fs::write(&fm.file_path, new_content.as_bytes()).is_ok() {
                    replacements += r.replacement_count;
                    files_modified += 1;
                }
            }
        }

        Ok((
            ReplaceSummary {
                replacements,
                files_modified,
            },
            ConflictList {
                paths: conflict_paths,
            },
        ))
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::result::SearchResult;
    use ff_find_and_replace::SearchMode;
    use tempfile::TempDir;

    fn make_file(dir: &TempDir, name: &str, content: &str) -> String {
        let path = dir.path().join(name);
        std::fs::write(&path, content).unwrap();
        path.to_string_lossy().into_owned()
    }

    fn req(query: &str) -> GlobalSearchRequest {
        GlobalSearchRequest {
            query: query.to_string(),
            mode: SearchMode::Literal,
            case_sensitive: true,
            whole_word: false,
            include_globs: Vec::new(),
            exclude_globs: Vec::new(),
            roots: Vec::new(),
        }
    }

    // Validates: Requirement 5.3 -- replace modifies file content
    #[test]
    fn replace_all_modifies_file_content() {
        let dir = TempDir::new().unwrap();
        let path = make_file(&dir, "a.txt", "hello world hello");
        let fm = FileMatches {
            file_path: path.clone(),
            matches: vec![SearchResult {
                line_number: 1,
                col_start: 0,
                col_end: 5,
                line_text: "hello world hello".to_string(),
            }],
        };
        let (summary, conflicts) =
            GlobalReplaceEngine::replace_all(&[fm], &req("hello"), "bye", &[]).unwrap();
        assert_eq!(summary.replacements, 2);
        assert_eq!(summary.files_modified, 1);
        assert!(conflicts.paths.is_empty());
        let result = std::fs::read_to_string(&path).unwrap();
        assert_eq!(result, "bye world bye");
    }

    // Validates: Requirement 5.6 -- unsaved-changes conflict detected
    #[test]
    fn replace_all_skips_unsaved_files() {
        let dir = TempDir::new().unwrap();
        let path = make_file(&dir, "b.txt", "foo bar");
        let fm = FileMatches {
            file_path: path.clone(),
            matches: vec![SearchResult {
                line_number: 1,
                col_start: 0,
                col_end: 3,
                line_text: "foo bar".to_string(),
            }],
        };
        let (summary, conflicts) =
            GlobalReplaceEngine::replace_all(&[fm], &req("foo"), "baz", &[path.clone()]).unwrap();
        assert_eq!(summary.files_modified, 0);
        assert_eq!(conflicts.paths, vec![path]);
        assert_eq!(
            std::fs::read_to_string(dir.path().join("b.txt")).unwrap(),
            "foo bar"
        );
    }
}
