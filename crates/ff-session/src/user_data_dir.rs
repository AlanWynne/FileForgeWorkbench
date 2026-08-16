//! User Data Directory initialisation — platform-specific path resolution,
//! directory creation, subdirectory repair, and permission checking.
//!
//! Addresses: Requirement 3 (User Data Directory Initialisation)

use std::path::{Path, PathBuf};

use crate::SessionError;

/// Required subdirectories within the User Data Directory.
pub const REQUIRED_SUBDIRS: &[&str] = &["sessions", "recovery", "profiles", "plugins"];

/// The application directory name used within platform config paths.
const APP_DIR_NAME: &str = "ffworkbench";

/// Manages the User Data Directory — location resolution, creation,
/// and subdirectory repair.
///
/// The User Data Directory is the platform-specific location for persistent
/// user data: session files, recovery files, profiles, and plugin data.
#[derive(Debug, Clone)]
pub struct UserDataDir {
    /// The resolved path to the User Data Directory.
    path: PathBuf,

    /// Whether the directory is available and writable.
    available: bool,
}

impl UserDataDir {
    /// Create a `UserDataDir` from a pre-resolved path without performing
    /// any filesystem operations. Used for testing and when the path is
    /// known to exist.
    pub fn from_path(path: PathBuf) -> Self {
        Self {
            path,
            available: false,
        }
    }

    /// Resolve the User Data Directory path.
    ///
    /// If `custom_path` is `Some`, use that path. Otherwise, use the
    /// platform default:
    /// - Linux: `~/.config/ffworkbench/`
    /// - macOS: `~/Library/Application Support/ffworkbench/`
    /// - Windows: `%APPDATA%\ffworkbench\`
    ///
    /// Addresses: Requirement 3 AC 3.4
    pub fn resolve(custom_path: Option<&Path>) -> Result<Self, SessionError> {
        let path = match custom_path {
            Some(p) => p.to_path_buf(),
            None => platform_default_path()?,
        };

        Ok(Self {
            path,
            available: false,
        })
    }

    /// Initialise the User Data Directory — create it and all required
    /// subdirectories if they don't exist.
    ///
    /// Performs incremental repair: creates missing subdirectories without
    /// affecting existing content.
    ///
    /// # Errors
    ///
    /// Returns `SessionError::UserDataDirUnavailable` if the directory
    /// cannot be created or is not writable.
    ///
    /// Addresses: Requirement 3 AC 3.1, 3.2, 3.3
    pub fn initialise(&mut self) -> Result<(), SessionError> {
        // Create the main directory if it doesn't exist
        if !self.path.exists() {
            std::fs::create_dir_all(&self.path).map_err(|e| {
                SessionError::UserDataDirUnavailable {
                    path: self.path.clone(),
                    reason: format!("cannot create directory: {e}"),
                }
            })?;
        }

        // Create required subdirectories (incremental repair)
        for subdir in REQUIRED_SUBDIRS {
            let subdir_path = self.path.join(subdir);
            if !subdir_path.exists() {
                std::fs::create_dir_all(&subdir_path).map_err(|e| {
                    SessionError::UserDataDirUnavailable {
                        path: subdir_path,
                        reason: format!("cannot create subdirectory: {e}"),
                    }
                })?;
            }
        }

        // Verify writability by attempting to create and remove a test file
        let test_file = self.path.join(".write_test");
        std::fs::write(&test_file, b"").map_err(|e| SessionError::UserDataDirUnavailable {
            path: self.path.clone(),
            reason: format!("directory is not writable: {e}"),
        })?;
        let _ = std::fs::remove_file(&test_file);

        self.available = true;
        Ok(())
    }

    /// Returns the resolved path to the User Data Directory.
    pub fn path(&self) -> &Path {
        &self.path
    }

    /// Returns whether the directory is available and writable.
    pub fn is_available(&self) -> bool {
        self.available
    }

    /// Returns the path to the `sessions/` subdirectory.
    pub fn sessions_dir(&self) -> PathBuf {
        self.path.join("sessions")
    }

    /// Returns the path to the `recovery/` subdirectory.
    pub fn recovery_dir(&self) -> PathBuf {
        self.path.join("recovery")
    }

    /// Returns the path to the `profiles/` subdirectory.
    pub fn profiles_dir(&self) -> PathBuf {
        self.path.join("profiles")
    }

    /// Returns the path to the `plugins/` subdirectory.
    pub fn plugins_dir(&self) -> PathBuf {
        self.path.join("plugins")
    }

    /// Returns the path to the session file (`session.toml`).
    pub fn session_file_path(&self) -> PathBuf {
        self.path.join("session.toml")
    }
}

/// Resolve the platform-default User Data Directory path.
///
/// Uses the `dirs` crate for platform-aware directory resolution.
fn platform_default_path() -> Result<PathBuf, SessionError> {
    let base = dirs::config_dir().ok_or_else(|| SessionError::UserDataDirUnavailable {
        path: PathBuf::from("(unknown)"),
        reason: "cannot determine platform config directory".to_string(),
    })?;
    Ok(base.join(APP_DIR_NAME))
}

#[cfg(test)]
mod tests {
    use super::*;
    use tempfile::TempDir;

    #[test]
    fn resolve_with_custom_path_uses_provided_path() {
        // Validates: Requirement 3 AC 3.4
        let custom = Path::new("/custom/data/dir");
        let udd = UserDataDir::resolve(Some(custom)).unwrap();
        assert_eq!(udd.path(), custom);
    }

    #[test]
    fn resolve_without_custom_path_uses_platform_default() {
        // Validates: Requirement 3 AC 3.4
        let udd = UserDataDir::resolve(None).unwrap();
        let path_str = udd.path().to_string_lossy();
        assert!(path_str.contains(APP_DIR_NAME));
    }

    #[test]
    fn initialise_creates_directory_and_subdirs() {
        // Validates: Requirement 3 AC 3.1
        let tmp = TempDir::new().unwrap();
        let data_dir = tmp.path().join("ffworkbench_test");

        let mut udd = UserDataDir::from_path(data_dir.clone());
        udd.initialise().unwrap();

        assert!(data_dir.exists());
        assert!(data_dir.join("sessions").exists());
        assert!(data_dir.join("recovery").exists());
        assert!(data_dir.join("profiles").exists());
        assert!(data_dir.join("plugins").exists());
        assert!(udd.is_available());
    }

    #[test]
    fn initialise_creates_missing_subdirs_without_affecting_existing() {
        // Validates: Requirement 3 AC 3.2
        let tmp = TempDir::new().unwrap();
        let data_dir = tmp.path().join("ffworkbench_test");

        // Pre-create the main dir and one subdir with content
        std::fs::create_dir_all(data_dir.join("sessions")).unwrap();
        std::fs::write(data_dir.join("sessions/existing.toml"), "content").unwrap();

        let mut udd = UserDataDir::from_path(data_dir.clone());
        udd.initialise().unwrap();

        // Existing content preserved
        assert!(data_dir.join("sessions/existing.toml").exists());
        let content = std::fs::read_to_string(data_dir.join("sessions/existing.toml")).unwrap();
        assert_eq!(content, "content");

        // Missing subdirs created
        assert!(data_dir.join("recovery").exists());
        assert!(data_dir.join("profiles").exists());
        assert!(data_dir.join("plugins").exists());
    }

    #[test]
    fn initialise_on_existing_complete_directory_is_idempotent() {
        // Validates: Requirement 3 AC 3.2
        let tmp = TempDir::new().unwrap();
        let data_dir = tmp.path().join("ffworkbench_test");

        let mut udd = UserDataDir::from_path(data_dir.clone());
        udd.initialise().unwrap();

        // Second initialisation should succeed without error
        let mut udd2 = UserDataDir::from_path(data_dir);
        udd2.initialise().unwrap();
        assert!(udd2.is_available());
    }

    #[test]
    fn subdirectory_accessors_return_correct_paths() {
        let tmp = TempDir::new().unwrap();
        let data_dir = tmp.path().join("test_dir");
        let udd = UserDataDir::from_path(data_dir.clone());

        assert_eq!(udd.sessions_dir(), data_dir.join("sessions"));
        assert_eq!(udd.recovery_dir(), data_dir.join("recovery"));
        assert_eq!(udd.profiles_dir(), data_dir.join("profiles"));
        assert_eq!(udd.plugins_dir(), data_dir.join("plugins"));
        assert_eq!(udd.session_file_path(), data_dir.join("session.toml"));
    }

    #[test]
    fn from_path_creates_unavailable_user_data_dir() {
        let udd = UserDataDir::from_path(PathBuf::from("/nonexistent"));
        assert!(!udd.is_available());
    }

    #[test]
    fn required_subdirs_contains_expected_entries() {
        assert!(REQUIRED_SUBDIRS.contains(&"sessions"));
        assert!(REQUIRED_SUBDIRS.contains(&"recovery"));
        assert!(REQUIRED_SUBDIRS.contains(&"profiles"));
        assert!(REQUIRED_SUBDIRS.contains(&"plugins"));
        assert_eq!(REQUIRED_SUBDIRS.len(), 4);
    }
}
