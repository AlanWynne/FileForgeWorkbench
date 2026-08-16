//! Working directory resolver for child processes.
//!
//! Resolves the current working directory based on configuration
//! (`project_root` or `file_directory` mode) with appropriate fallbacks.

use std::path::{Path, PathBuf};

use crate::config::WorkingDirectoryMode;

/// Resolves the working directory for spawned child processes.
///
/// Implements the fallback chain defined in Requirement 11:
/// - `project_root`: project root → home directory
/// - `file_directory`: file parent → project root → home directory
#[derive(Debug, Clone)]
pub struct WorkingDirResolver;

impl WorkingDirResolver {
    /// Resolves the working directory based on mode and context.
    ///
    /// # Arguments
    ///
    /// * `mode` - The configured working directory mode.
    /// * `project_root` - The project root path if a project is open.
    /// * `active_file_path` - The path of the active file if one is open.
    ///
    /// # Returns
    ///
    /// Always returns a valid path (falls back to home directory as last resort).
    pub fn resolve(
        mode: WorkingDirectoryMode,
        project_root: Option<&Path>,
        active_file_path: Option<&Path>,
    ) -> PathBuf {
        match mode {
            WorkingDirectoryMode::ProjectRoot => Self::resolve_project_root(project_root),
            WorkingDirectoryMode::FileDirectory => {
                Self::resolve_file_directory(active_file_path, project_root)
            }
        }
    }

    /// Resolves using `project_root` mode.
    ///
    /// Returns the project root if available, otherwise the home directory.
    fn resolve_project_root(project_root: Option<&Path>) -> PathBuf {
        if let Some(root) = project_root {
            if root.exists() {
                return root.to_path_buf();
            }
        }
        Self::home_directory()
    }

    /// Resolves using `file_directory` mode.
    ///
    /// Returns the active file's parent directory if available, otherwise
    /// falls back to project root, then home directory.
    fn resolve_file_directory(
        active_file_path: Option<&Path>,
        project_root: Option<&Path>,
    ) -> PathBuf {
        // Try active file's parent directory
        if let Some(file_path) = active_file_path {
            if let Some(parent) = file_path.parent() {
                if parent.exists() {
                    return parent.to_path_buf();
                }
            }
        }

        // Fallback to project root
        if let Some(root) = project_root {
            if root.exists() {
                return root.to_path_buf();
            }
        }

        // Final fallback to home directory
        Self::home_directory()
    }

    /// Returns the user's home directory.
    ///
    /// Falls back to the current directory if home cannot be determined.
    fn home_directory() -> PathBuf {
        dirs::home_dir().unwrap_or_else(|| PathBuf::from("."))
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use tempfile::TempDir;

    // Validates: Requirement 11.1
    #[test]
    fn project_root_mode_uses_project_root_when_available() {
        let temp = TempDir::new().unwrap();
        let project_root = temp.path();

        let result = WorkingDirResolver::resolve(
            WorkingDirectoryMode::ProjectRoot,
            Some(project_root),
            None,
        );

        assert_eq!(result, project_root);
    }

    // Validates: Requirement 11.2
    #[test]
    fn project_root_mode_falls_back_to_home_when_no_project() {
        let result = WorkingDirResolver::resolve(WorkingDirectoryMode::ProjectRoot, None, None);

        // Should be the home directory
        let home = dirs::home_dir().unwrap_or_else(|| PathBuf::from("."));
        assert_eq!(result, home);
    }

    // Validates: Requirement 11.3
    #[test]
    fn file_directory_mode_uses_file_parent_directory() {
        let temp = TempDir::new().unwrap();
        let file_path = temp.path().join("subdir").join("test.txt");
        std::fs::create_dir_all(file_path.parent().unwrap()).unwrap();

        let result = WorkingDirResolver::resolve(
            WorkingDirectoryMode::FileDirectory,
            None,
            Some(&file_path),
        );

        assert_eq!(result, file_path.parent().unwrap());
    }

    // Validates: Requirement 11.4
    #[test]
    fn file_directory_mode_falls_back_to_project_root_for_unsaved_buffer() {
        let temp = TempDir::new().unwrap();
        let project_root = temp.path();

        let result = WorkingDirResolver::resolve(
            WorkingDirectoryMode::FileDirectory,
            Some(project_root),
            None, // No active file (unsaved buffer)
        );

        assert_eq!(result, project_root);
    }

    // Validates: Requirement 11.4
    #[test]
    fn file_directory_mode_falls_back_to_home_when_nothing_available() {
        let result = WorkingDirResolver::resolve(WorkingDirectoryMode::FileDirectory, None, None);

        let home = dirs::home_dir().unwrap_or_else(|| PathBuf::from("."));
        assert_eq!(result, home);
    }

    // Validates: Requirement 11.1
    #[test]
    fn resolution_always_returns_non_empty_path() {
        let result = WorkingDirResolver::resolve(WorkingDirectoryMode::ProjectRoot, None, None);
        assert!(!result.as_os_str().is_empty());

        let result = WorkingDirResolver::resolve(WorkingDirectoryMode::FileDirectory, None, None);
        assert!(!result.as_os_str().is_empty());
    }
}
