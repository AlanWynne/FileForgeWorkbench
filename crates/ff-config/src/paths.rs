//! Platform-specific path resolution.
//!
//! Resolves the filesystem locations of configuration files for each layer
//! and platform (Linux, Windows, macOS): system config, user config, profiles
//! directory, languages directory, and project/workspace config.

use std::path::{Path, PathBuf};

/// Resolve the system-wide configuration file path.
///
/// Returns the platform-specific path for the system configuration file:
/// - Linux: `/etc/ffworkbench/config.toml`
/// - Windows: `%PROGRAMDATA%\FFWorkbench\config.toml`
/// - macOS: `/Library/Application Support/FFWorkbench/config.toml`
pub fn system_config_path() -> PathBuf {
    #[cfg(target_os = "linux")]
    {
        PathBuf::from("/etc/ffworkbench/config.toml")
    }

    #[cfg(target_os = "windows")]
    {
        std::env::var("PROGRAMDATA")
            .map(|p| PathBuf::from(p).join("FFWorkbench").join("config.toml"))
            .unwrap_or_else(|_| PathBuf::from("C:\\ProgramData\\FFWorkbench\\config.toml"))
    }

    #[cfg(target_os = "macos")]
    {
        PathBuf::from("/Library/Application Support/FFWorkbench/config.toml")
    }
}

/// Resolve the user configuration file path.
///
/// Uses the platform's standard config directory:
/// - Linux: `$XDG_CONFIG_HOME/ffworkbench/config.toml` (typically `~/.config/ffworkbench/config.toml`)
/// - Windows: `%APPDATA%\FFWorkbench\config.toml`
/// - macOS: `~/Library/Application Support/FFWorkbench/config.toml`
///
/// Returns `None` if the platform config directory cannot be determined.
pub fn user_config_path() -> Option<PathBuf> {
    dirs::config_dir().map(|d| d.join("ffworkbench").join("config.toml"))
}

/// Resolve the user profiles directory.
///
/// The profiles directory is a `profiles/` subdirectory within the user
/// configuration directory. Each profile is a separate TOML file within
/// this directory.
///
/// Returns `None` if the platform config directory cannot be determined.
pub fn user_profiles_dir() -> Option<PathBuf> {
    dirs::config_dir().map(|d| d.join("ffworkbench").join("profiles"))
}

/// Resolve the languages directory.
///
/// The languages directory is a `languages/` subdirectory within the user
/// configuration directory. Language-specific settings are stored here as
/// separate TOML files (e.g., `rust.toml`, `cobol.toml`).
///
/// Returns `None` if the platform config directory cannot be determined.
pub fn languages_dir() -> Option<PathBuf> {
    dirs::config_dir().map(|d| d.join("ffworkbench").join("languages"))
}

/// Resolve the project configuration file path.
///
/// The project configuration file is always located at
/// `.ffworkbench/config.toml` relative to the given project root directory.
pub fn project_config_path(project_root: &Path) -> PathBuf {
    project_root.join(".ffworkbench").join("config.toml")
}

#[cfg(test)]
mod tests {
    use super::*;

    // Validates: Requirement 1.2 — system config path is platform-appropriate
    #[test]
    fn system_config_path_ends_with_config_toml() {
        let path = system_config_path();
        assert!(
            path.ends_with("config.toml"),
            "System config path should end with config.toml, got: {}",
            path.display()
        );
    }

    #[cfg(target_os = "windows")]
    #[test]
    fn system_config_path_uses_programdata_on_windows() {
        // Validates: Requirement 1.2 — Windows uses %PROGRAMDATA%\FFWorkbench
        let path = system_config_path();
        let path_str = path.to_string_lossy();
        assert!(
            path_str.contains("FFWorkbench"),
            "Windows system path should contain FFWorkbench, got: {}",
            path_str
        );
    }

    #[cfg(target_os = "linux")]
    #[test]
    fn system_config_path_uses_etc_on_linux() {
        // Validates: Requirement 1.2 — Linux uses /etc/ffworkbench
        let path = system_config_path();
        assert_eq!(path, PathBuf::from("/etc/ffworkbench/config.toml"));
    }

    #[cfg(target_os = "macos")]
    #[test]
    fn system_config_path_uses_library_on_macos() {
        // Validates: Requirement 1.2 — macOS uses /Library/Application Support
        let path = system_config_path();
        assert_eq!(
            path,
            PathBuf::from("/Library/Application Support/FFWorkbench/config.toml")
        );
    }

    // Validates: Requirement 1.2 — user config path resolution
    #[test]
    fn user_config_path_returns_some_with_config_toml() {
        let path = user_config_path();
        // On CI or restricted environments, config_dir() may return None
        if let Some(p) = path {
            assert!(
                p.ends_with("config.toml"),
                "User config path should end with config.toml, got: {}",
                p.display()
            );
            let parent = p.parent().unwrap();
            assert!(
                parent.ends_with("ffworkbench"),
                "User config should be in ffworkbench dir, got: {}",
                parent.display()
            );
        }
    }

    // Validates: Requirement 4.1 — profiles directory is under user config dir
    #[test]
    fn user_profiles_dir_is_profiles_subdir_of_config_dir() {
        let profiles = user_profiles_dir();
        if let Some(p) = profiles {
            assert!(
                p.ends_with("profiles"),
                "Profiles dir should end with 'profiles', got: {}",
                p.display()
            );
            let parent = p.parent().unwrap();
            assert!(
                parent.ends_with("ffworkbench"),
                "Profiles dir parent should be ffworkbench, got: {}",
                parent.display()
            );
        }
    }

    // Validates: Requirement 1.5 — languages directory is under user config dir
    #[test]
    fn languages_dir_is_languages_subdir_of_config_dir() {
        let langs = languages_dir();
        if let Some(p) = langs {
            assert!(
                p.ends_with("languages"),
                "Languages dir should end with 'languages', got: {}",
                p.display()
            );
            let parent = p.parent().unwrap();
            assert!(
                parent.ends_with("ffworkbench"),
                "Languages dir parent should be ffworkbench, got: {}",
                parent.display()
            );
        }
    }

    // Validates: Requirement 5.1 — project config at .ffworkbench/config.toml
    #[test]
    fn project_config_path_joins_correctly() {
        let root = Path::new("/home/user/my-project");
        let path = project_config_path(root);
        assert_eq!(
            path,
            PathBuf::from("/home/user/my-project/.ffworkbench/config.toml")
        );
    }

    // Validates: Requirement 5.1 — project config with Windows-style paths
    #[test]
    fn project_config_path_works_with_various_roots() {
        let root = Path::new("C:\\Users\\dev\\project");
        let path = project_config_path(root);
        assert_eq!(
            path,
            PathBuf::from("C:\\Users\\dev\\project\\.ffworkbench\\config.toml")
        );
    }

    // Validates: Requirement 1.2 — user and profiles share the same base directory
    #[test]
    fn user_config_and_profiles_share_base_directory() {
        let user_path = user_config_path();
        let profiles_path = user_profiles_dir();

        if let (Some(user), Some(profiles)) = (user_path, profiles_path) {
            let user_parent = user.parent().unwrap();
            let profiles_parent = profiles.parent().unwrap();
            assert_eq!(
                user_parent, profiles_parent,
                "User config and profiles should share the same parent directory"
            );
        }
    }

    // Validates: Requirement 1.5 — user config and languages share base directory
    #[test]
    fn user_config_and_languages_share_base_directory() {
        let user_path = user_config_path();
        let langs_path = languages_dir();

        if let (Some(user), Some(langs)) = (user_path, langs_path) {
            let user_parent = user.parent().unwrap();
            let langs_parent = langs.parent().unwrap();
            assert_eq!(
                user_parent, langs_parent,
                "User config and languages should share the same parent directory"
            );
        }
    }
}
