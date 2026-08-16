//! Built-in provider for file path completion.
//!
//! Queries the VFS abstraction layer for matching file and directory entries.
//! In this initial implementation, a mock VFS is used.

use crate::candidate::{CompletionCandidate, CompletionKind};
use crate::context::{CompletionContext, CompletionField};
use crate::error::CompletionError;
use crate::provider::CompletionProvider;

/// Provides file path completion candidates from the VFS.
///
/// Active when the cursor is in argument position and the command
/// expects a file path argument.
pub struct FilePathProvider {
    /// Mock file entries for testing. In production this would query
    /// the VFS asynchronously.
    mock_entries: Vec<FileEntry>,
}

/// A simplified file entry.
#[derive(Debug, Clone)]
struct FileEntry {
    name: String,
    path: String,
    is_directory: bool,
}

impl FilePathProvider {
    /// Creates a new file path provider with default mock entries.
    pub fn new() -> Self {
        Self {
            mock_entries: default_entries(),
        }
    }

    /// Creates a provider with custom mock entries for testing.
    pub fn with_entries(entries: Vec<(String, String, bool)>) -> Self {
        Self {
            mock_entries: entries
                .into_iter()
                .map(|(name, path, is_dir)| FileEntry {
                    name,
                    path,
                    is_directory: is_dir,
                })
                .collect(),
        }
    }
}

impl Default for FilePathProvider {
    fn default() -> Self {
        Self::new()
    }
}

impl CompletionProvider for FilePathProvider {
    fn id(&self) -> &str {
        "file_path"
    }

    fn is_applicable(&self, context: &CompletionContext) -> bool {
        context.field == CompletionField::PrimaryCommand
            && context.command_name.is_some()
            && context.argument_index == Some(0)
    }

    fn provide_candidates(
        &self,
        _context: &CompletionContext,
    ) -> Result<Vec<CompletionCandidate>, CompletionError> {
        let candidates = self
            .mock_entries
            .iter()
            .map(|entry| {
                let kind = if entry.is_directory {
                    CompletionKind::Directory
                } else {
                    CompletionKind::FilePath
                };
                let insert_text = if entry.is_directory {
                    format!("{}/", entry.path)
                } else {
                    entry.path.clone()
                };
                CompletionCandidate::new(entry.name.clone(), insert_text, kind)
                    .with_detail(entry.path.clone())
            })
            .collect();
        Ok(candidates)
    }
}

/// Default mock file entries for development.
fn default_entries() -> Vec<FileEntry> {
    vec![
        FileEntry {
            name: "src".to_string(),
            path: "/project/src".to_string(),
            is_directory: true,
        },
        FileEntry {
            name: "main.rs".to_string(),
            path: "/project/src/main.rs".to_string(),
            is_directory: false,
        },
        FileEntry {
            name: "lib.rs".to_string(),
            path: "/project/src/lib.rs".to_string(),
            is_directory: false,
        },
        FileEntry {
            name: "tests".to_string(),
            path: "/project/tests".to_string(),
            is_directory: true,
        },
        FileEntry {
            name: "Cargo.toml".to_string(),
            path: "/project/Cargo.toml".to_string(),
            is_directory: false,
        },
        FileEntry {
            name: "README.md".to_string(),
            path: "/project/README.md".to_string(),
            is_directory: false,
        },
    ]
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::context::CompletionContextBuilder;

    // Validates: Requirement 2.2 (file path completion in argument position)
    #[test]
    fn applicable_in_argument_position_with_command() {
        let provider = FilePathProvider::new();
        let ctx = CompletionContextBuilder::new()
            .field(CompletionField::PrimaryCommand)
            .command_name("EDIT")
            .argument_index(0)
            .prefix("/pro")
            .build();
        assert!(provider.is_applicable(&ctx));
    }

    #[test]
    fn not_applicable_in_command_position() {
        let provider = FilePathProvider::new();
        let ctx = CompletionContextBuilder::new()
            .field(CompletionField::PrimaryCommand)
            .prefix("ED")
            .build();
        assert!(!provider.is_applicable(&ctx));
    }

    // Validates: Requirement 2.2, 2.3 (directory candidates with trailing separator)
    #[test]
    fn directory_candidates_have_trailing_separator() {
        let provider = FilePathProvider::new();
        let ctx = CompletionContextBuilder::new()
            .field(CompletionField::PrimaryCommand)
            .command_name("EDIT")
            .argument_index(0)
            .prefix("/project/")
            .build();

        let candidates = provider.provide_candidates(&ctx).unwrap();
        let dirs: Vec<_> = candidates
            .iter()
            .filter(|c| c.kind == CompletionKind::Directory)
            .collect();
        assert!(!dirs.is_empty());
        for dir in &dirs {
            assert!(dir.insert_text.ends_with('/'));
        }
    }

    #[test]
    fn file_candidates_do_not_have_trailing_separator() {
        let provider = FilePathProvider::new();
        let ctx = CompletionContextBuilder::new()
            .field(CompletionField::PrimaryCommand)
            .command_name("EDIT")
            .argument_index(0)
            .build();

        let candidates = provider.provide_candidates(&ctx).unwrap();
        let files: Vec<_> = candidates
            .iter()
            .filter(|c| c.kind == CompletionKind::FilePath)
            .collect();
        assert!(!files.is_empty());
        for f in &files {
            assert!(!f.insert_text.ends_with('/'));
        }
    }
}
