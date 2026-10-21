//! Help content loading — discovers and loads `.help.md` files.
//!
//! Resolves the help content directory, discovers all `.help.md` files,
//! and delegates parsing to the content parser.

use std::path::{Path, PathBuf};

use crate::config::HelpConfig;
use crate::content_parser::ContentParser;
use crate::error::HelpError;
use crate::topic::HelpTopic;

/// Result of loading help content from a directory.
#[derive(Debug)]
pub struct ContentLoadResult {
    /// Successfully parsed topics.
    pub topics: Vec<HelpTopic>,
    /// Files that could not be parsed (path + error message).
    pub warnings: Vec<(PathBuf, String)>,
    /// Total number of files scanned.
    pub files_scanned: usize,
}

/// Loads help content from `.help.md` files on disk.
///
/// Resolves the content directory, discovers files, and parses them
/// into `HelpTopic` instances for registry population.
pub struct ContentLoader {
    /// Search paths for finding the help directory.
    search_paths: Vec<PathBuf>,
}

impl ContentLoader {
    /// Create a new content loader with the given search paths.
    pub fn new(search_paths: Vec<PathBuf>) -> Self {
        Self { search_paths }
    }

    /// Create a content loader from configuration.
    ///
    /// Uses the config's `directory` if set, otherwise adds default search paths.
    pub fn from_config(config: &HelpConfig) -> Self {
        let mut paths = Vec::new();

        if let Some(ref dir) = config.directory {
            paths.push(PathBuf::from(dir));
        }

        // Default search paths: binary directory, user data directory
        if let Ok(exe) = std::env::current_exe() {
            if let Some(exe_dir) = exe.parent() {
                paths.push(exe_dir.join("help"));
            }
        }

        if let Some(data_dir) = dirs::data_dir() {
            paths.push(data_dir.join("FileForgeWorkbench").join("help"));
        }

        Self::new(paths)
    }

    /// Resolve the help content directory using the search order.
    ///
    /// Returns the first directory that exists from the search paths.
    pub fn resolve_help_directory(&self) -> Option<PathBuf> {
        for path in &self.search_paths {
            if path.is_dir() {
                return Some(path.clone());
            }
        }
        None
    }

    /// Discover all `.help.md` files in the given directory, recursing into
    /// subdirectories.
    ///
    /// Shipped content is grouped into subfolders (`commands/`, `line-commands/`,
    /// `modes/`, `features/`) for maintainability, so discovery must walk the
    /// tree, not just the top level (CR-NR-097, design D1).
    pub fn discover_files(dir: &Path) -> Vec<PathBuf> {
        let mut files = Vec::new();
        Self::discover_into(dir, &mut files);
        files.sort();
        files
    }

    /// Recursive worker for [`discover_files`].
    fn discover_into(dir: &Path, files: &mut Vec<PathBuf>) {
        if let Ok(entries) = std::fs::read_dir(dir) {
            for entry in entries.flatten() {
                let path = entry.path();
                if path.is_dir() {
                    Self::discover_into(&path, files);
                } else if path.is_file() {
                    if let Some(name) = path.file_name().and_then(|n| n.to_str()) {
                        if name.ends_with(".help.md") {
                            files.push(path);
                        }
                    }
                }
            }
        }
    }

    /// Load all `.help.md` files from the resolved directory.
    ///
    /// # Errors
    ///
    /// Returns `HelpError::ContentDirectoryMissing` if no help directory is found.
    pub fn load_all(&self) -> Result<ContentLoadResult, HelpError> {
        let dir = self.resolve_help_directory().ok_or_else(|| {
            let paths: Vec<_> = self
                .search_paths
                .iter()
                .map(|p| p.display().to_string())
                .collect();
            HelpError::ContentDirectoryMissing {
                searched_paths: paths.join(", "),
            }
        })?;

        let files = Self::discover_files(&dir);
        let mut result = ContentLoadResult {
            topics: Vec::new(),
            warnings: Vec::new(),
            files_scanned: files.len(),
        };

        for file_path in &files {
            match std::fs::read_to_string(file_path) {
                Ok(content) => match ContentParser::parse_file(file_path, &content) {
                    Ok(topics) => result.topics.extend(topics),
                    Err(e) => {
                        result.warnings.push((file_path.clone(), e.to_string()));
                    }
                },
                Err(e) => {
                    result.warnings.push((file_path.clone(), e.to_string()));
                }
            }
        }

        Ok(result)
    }

    /// Resolve the help content directory for a specific locale (localization
    /// CR-NR-103, Req 6.1, 6.2).
    ///
    /// For each base in the search paths, probe `base/<locale>` first, then
    /// `base` itself (the English fallback); return the first directory that
    /// exists. For locale `"en"` the `<locale>` sub-directory is not expected to
    /// exist, so this resolves the SAME base directory `resolve_help_directory`
    /// (and therefore `load_all`) would pick -- keeping the English baseline
    /// behaviour unchanged (Req 10.2).
    pub fn resolve_help_directory_for_locale(&self, locale: &str) -> Option<PathBuf> {
        for base in &self.search_paths {
            let localised = base.join(locale);
            if localised.is_dir() {
                return Some(localised);
            }
            if base.is_dir() {
                return Some(base.clone());
            }
        }
        None
    }

    /// Load all `.help.md` files for a specific locale (localization CR-NR-103,
    /// Req 6.1, 6.3).
    ///
    /// Resolves the directory with [`resolve_help_directory_for_locale`], then
    /// REUSES the existing `discover_files` + `ContentParser` pipeline unchanged.
    /// Topic_Key and cross-references come from the parsed content, not the path,
    /// so they stay unlocalised (Req 6.3). For locale `"en"` the resolved content
    /// is identical to `load_all` (Req 10.2).
    ///
    /// # Errors
    ///
    /// Returns `HelpError::ContentDirectoryMissing` if no help directory is found
    /// for the locale (neither a localised nor a base directory exists).
    pub fn load_all_for_locale(&self, locale: &str) -> Result<ContentLoadResult, HelpError> {
        let dir = self
            .resolve_help_directory_for_locale(locale)
            .ok_or_else(|| {
                let paths: Vec<_> = self
                    .search_paths
                    .iter()
                    .map(|p| p.display().to_string())
                    .collect();
                HelpError::ContentDirectoryMissing {
                    searched_paths: paths.join(", "),
                }
            })?;

        let files = Self::discover_files(&dir);
        let mut result = ContentLoadResult {
            topics: Vec::new(),
            warnings: Vec::new(),
            files_scanned: files.len(),
        };

        for file_path in &files {
            match std::fs::read_to_string(file_path) {
                Ok(content) => match ContentParser::parse_file(file_path, &content) {
                    Ok(topics) => result.topics.extend(topics),
                    Err(e) => {
                        result.warnings.push((file_path.clone(), e.to_string()));
                    }
                },
                Err(e) => {
                    result.warnings.push((file_path.clone(), e.to_string()));
                }
            }
        }

        Ok(result)
    }

    /// Generate a minimal built-in help page for when no content files exist.
    pub fn built_in_minimal_help(search_paths: &[PathBuf]) -> HelpTopic {
        let paths_list: Vec<_> = search_paths
            .iter()
            .map(|p| format!("- {}", p.display()))
            .collect();
        let body = format!(
            "# Help Content Not Installed\n\n\
             The help content files (`.help.md`) are not installed or could not be found.\n\n\
             ## Expected Locations\n\n\
             The help system searches for content in these locations:\n\n\
             {}\n\n\
             ## How to Install\n\n\
             Place `.help.md` files in one of the directories listed above.\n\
             See the FileForgeWorkbench documentation for help content authoring guidelines.\n",
            paths_list.join("\n")
        );

        HelpTopic::new(
            crate::topic_key::TopicKey::index(),
            "Help Content Not Installed".to_string(),
            body,
            crate::topic::TopicSource::FileBased {
                file_path: PathBuf::from("<built-in>"),
            },
        )
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::fs;
    use tempfile::TempDir;

    // Validates: Requirement 5.1 — Directory resolution
    #[test]
    fn resolve_help_directory_returns_first_existing() {
        let temp = TempDir::new().unwrap();
        let existing = temp.path().join("help");
        fs::create_dir_all(&existing).unwrap();

        let loader = ContentLoader::new(vec![PathBuf::from("/nonexistent/path"), existing.clone()]);

        assert_eq!(loader.resolve_help_directory(), Some(existing));
    }

    // Validates: Requirement 5.1 — No directory found
    #[test]
    fn resolve_help_directory_returns_none_when_no_match() {
        let loader = ContentLoader::new(vec![
            PathBuf::from("/nonexistent/path1"),
            PathBuf::from("/nonexistent/path2"),
        ]);
        assert_eq!(loader.resolve_help_directory(), None);
    }

    // Validates: Requirement 5.1 — File discovery
    #[test]
    fn discover_files_finds_help_md_files() {
        let temp = TempDir::new().unwrap();
        fs::write(temp.path().join("commands.help.md"), "content").unwrap();
        fs::write(temp.path().join("modes.help.md"), "content").unwrap();
        fs::write(temp.path().join("readme.md"), "not a help file").unwrap();

        let files = ContentLoader::discover_files(temp.path());
        assert_eq!(files.len(), 2);
        assert!(files[0]
            .file_name()
            .unwrap()
            .to_str()
            .unwrap()
            .ends_with(".help.md"));
    }

    // Validates: Requirement 5.6 — Built-in minimal help when no content
    #[test]
    fn built_in_minimal_help_mentions_expected_locations() {
        let paths = vec![PathBuf::from("/app/help"), PathBuf::from("/user/data/help")];
        let topic = ContentLoader::built_in_minimal_help(&paths);
        assert!(topic.body().contains("/app/help"));
        assert!(topic.body().contains("/user/data/help"));
        assert!(topic.body().contains("not installed"));
    }

    // Validates: Requirement 17.1 (CR-NR-097) — discovery recurses into subdirs
    #[test]
    fn discover_files_recurses_into_subdirectories() {
        let temp = TempDir::new().unwrap();
        fs::write(temp.path().join("index.help.md"), "content").unwrap();
        let sub = temp.path().join("commands");
        fs::create_dir_all(&sub).unwrap();
        fs::write(sub.join("find.help.md"), "content").unwrap();
        let deeper = sub.join("nested");
        fs::create_dir_all(&deeper).unwrap();
        fs::write(deeper.join("more.help.md"), "content").unwrap();

        let files = ContentLoader::discover_files(temp.path());
        assert_eq!(files.len(), 3, "must find files in nested subdirectories");
    }

    // Validates: Requirement 6.1 -- a present per-locale directory is chosen.
    #[test]
    fn resolve_help_directory_for_locale_picks_localised_dir_when_present() {
        let temp = TempDir::new().unwrap();
        let base = temp.path().join("help");
        let localised = base.join("fr");
        fs::create_dir_all(&localised).unwrap();

        let loader = ContentLoader::new(vec![base]);
        assert_eq!(
            loader.resolve_help_directory_for_locale("fr"),
            Some(localised)
        );
    }

    // Validates: Requirement 6.2 -- absent per-locale directory falls back to the
    // English base directory.
    #[test]
    fn resolve_help_directory_for_locale_falls_back_to_english_when_absent() {
        let temp = TempDir::new().unwrap();
        let base = temp.path().join("help");
        fs::create_dir_all(&base).unwrap();

        let loader = ContentLoader::new(vec![base.clone()]);
        assert_eq!(loader.resolve_help_directory_for_locale("fr"), Some(base));
    }

    // Validates: Requirement 10.2 -- locale "en" resolves the SAME base directory
    // that load_all / resolve_help_directory picks (English baseline unchanged).
    #[test]
    fn resolve_help_directory_for_locale_en_resolves_base_directory_unchanged() {
        let temp = TempDir::new().unwrap();
        let base = temp.path().join("help");
        fs::create_dir_all(&base).unwrap();

        let loader = ContentLoader::new(vec![base.clone()]);
        assert_eq!(loader.resolve_help_directory_for_locale("en"), Some(base));
        assert_eq!(
            loader.resolve_help_directory_for_locale("en"),
            loader.resolve_help_directory()
        );
    }

    // Validates: Requirement 6.1 -- load_all_for_locale loads the localised
    // directory's files; Topic discovery reuses the existing pipeline.
    #[test]
    fn load_all_for_locale_loads_localised_directory_files() {
        let temp = TempDir::new().unwrap();
        let base = temp.path().join("help");
        let localised = base.join("fr");
        fs::create_dir_all(&localised).unwrap();
        fs::write(base.join("index.help.md"), "en content").unwrap();
        fs::write(localised.join("index.help.md"), "fr content").unwrap();

        let loader = ContentLoader::new(vec![base]);
        let result = loader.load_all_for_locale("fr").unwrap();
        assert_eq!(result.files_scanned, 1);
    }

    // Validates: Requirement 6.2 -- load_all_for_locale falls back to the English
    // base directory when the locale directory is absent.
    #[test]
    fn load_all_for_locale_falls_back_to_english_base() {
        let temp = TempDir::new().unwrap();
        let base = temp.path().join("help");
        fs::create_dir_all(&base).unwrap();
        fs::write(base.join("index.help.md"), "en content").unwrap();

        let loader = ContentLoader::new(vec![base]);
        let result = loader.load_all_for_locale("fr").unwrap();
        assert_eq!(result.files_scanned, 1);
    }

    // Validates: Requirement 5.1 — load_all returns error when directory missing
    #[test]
    fn load_all_returns_error_when_no_directory() {
        let loader = ContentLoader::new(vec![PathBuf::from("/nonexistent")]);
        let result = loader.load_all();
        assert!(result.is_err());
        match result.unwrap_err() {
            HelpError::ContentDirectoryMissing { searched_paths } => {
                assert!(searched_paths.contains("nonexistent"));
            }
            _ => panic!("Expected ContentDirectoryMissing error"),
        }
    }
}
