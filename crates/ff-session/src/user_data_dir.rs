//! User Data Directory initialisation -- platform-specific path resolution,
//! directory creation, subdirectory repair, and permission checking.
//!
//! Addresses: Requirement 3 (User Data Directory Initialisation)

use std::path::{Path, PathBuf};

use crate::SessionError;

/// Required subdirectories within the User Data Directory.
pub const REQUIRED_SUBDIRS: &[&str] = &["sessions", "recovery", "profiles", "plugins"];

/// The application directory name used within platform config paths.
const APP_DIR_NAME: &str = "ffworkbench";

/// Manages the User Data Directory -- location resolution, creation,
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

    /// Initialise the User Data Directory -- create it and all required
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

// === Application Profile (CR-NR-081, startup-and-session Requirement 22) =====

/// Process-global Active_Profile. `None` = the DEFAULT_PROFILE (the pre-CR-NR-081
/// User_Data_Dir location, no behaviour change). Set ONCE at startup, before any
/// subsystem resolves the User_Data_Dir (Requirement 22.5).
static ACTIVE_PROFILE: std::sync::RwLock<Option<String>> = std::sync::RwLock::new(None);

/// Set the Active_Profile for the process (Requirement 22.1).
///
/// `Some(name)` selects Application_Profile `name`; `None` (or an empty/blank
/// name) selects the DEFAULT_PROFILE. Call this ONCE during startup BEFORE any
/// `UserDataDir::resolve`/configuration init so no subsystem resolves the wrong
/// profile's data.
///
/// Validates: startup-and-session Requirement 22.1, 22.5, 22.6
pub fn set_active_profile(name: Option<&str>) {
    let normalised = name
        .map(str::trim)
        .filter(|s| !s.is_empty())
        .map(str::to_string);
    if let Ok(mut guard) = ACTIVE_PROFILE.write() {
        *guard = normalised;
    }
}

/// Return the Active_Profile name, or `None` for the DEFAULT_PROFILE.
///
/// Validates: startup-and-session Requirement 22.1
pub fn active_profile() -> Option<String> {
    ACTIVE_PROFILE.read().ok().and_then(|g| g.clone())
}

/// Slug a profile name for use as a directory component (Requirement 22.3):
/// lowercase, with every non-alphanumeric character replaced by `-`. Matches the
/// slugging used for menu / menu-bar names so naming is consistent across the
/// app.
///
/// Validates: startup-and-session Requirement 22.3
pub fn profile_slug(name: &str) -> String {
    name.trim()
        .to_lowercase()
        .chars()
        .map(|c| if c.is_ascii_alphanumeric() { c } else { '-' })
        .collect()
}

/// Resolve the platform-default User Data Directory path.
///
/// Uses the `dirs` crate for platform-aware directory resolution. WHEN an
/// Application_Profile is active (CR-NR-081, Requirement 22), the path is the
/// per-profile sub-directory `<base>/ffworkbench/profiles/<slug>/`; otherwise it
/// is `<base>/ffworkbench/` (the DEFAULT_PROFILE, unchanged). Every
/// `UserDataDir::resolve(None)` caller funnels through here, so the active
/// profile isolates all user data with no per-subsystem change.
///
/// Validates: startup-and-session Requirement 22.2, 22.3, 22.9
fn platform_default_path() -> Result<PathBuf, SessionError> {
    let base = dirs::config_dir().ok_or_else(|| SessionError::UserDataDirUnavailable {
        path: PathBuf::from("(unknown)"),
        reason: "cannot determine platform config directory".to_string(),
    })?;
    let app_base = base.join(APP_DIR_NAME);
    match active_profile() {
        Some(name) => Ok(app_base.join("profiles").join(profile_slug(&name))),
        None => Ok(app_base),
    }
}

/// Resolve the DEFAULT_PROFILE's User_Data_Dir base -- `<config>/ffworkbench` --
/// INDEPENDENT of the active profile.
///
/// Unlike `UserDataDir::resolve(None)` (which funnels through
/// `platform_default_path` and therefore returns the ACTIVE profile's
/// sub-directory), this always returns the profile-agnostic base. Used by
/// features that must address the default profile and enumerate all profiles
/// regardless of which profile the process is running under (e.g. targeted
/// RESET BARE, CR-NR-083).
///
/// Validates: startup-and-session Requirement 22.2, 22.3
pub fn default_base() -> Result<PathBuf, SessionError> {
    let base = dirs::config_dir().ok_or_else(|| SessionError::UserDataDirUnavailable {
        path: PathBuf::from("(unknown)"),
        reason: "cannot determine platform config directory".to_string(),
    })?;
    Ok(base.join(APP_DIR_NAME))
}

/// Resolve the profiles root -- `<config>/ffworkbench/profiles` -- INDEPENDENT of
/// the active profile. Each named Application_Profile is a `<slug>/` child of
/// this directory (CR-NR-081, Requirement 22.3).
///
/// Validates: startup-and-session Requirement 22.3
pub fn profiles_root() -> Result<PathBuf, SessionError> {
    Ok(default_base()?.join("profiles"))
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

    // Validates: startup-and-session Req 22.3 (CR-NR-081) -- profile_slug rule.
    #[test]
    fn profile_slug_lowercases_and_replaces_non_alphanumerics() {
        assert_eq!(profile_slug("ispf"), "ispf");
        assert_eq!(profile_slug("Rust"), "rust");
        assert_eq!(profile_slug("My Profile"), "my-profile");
        assert_eq!(profile_slug("ISPF_3270"), "ispf-3270");
        assert_eq!(profile_slug("  spaced  "), "spaced");
    }

    // Validates: startup-and-session Req 22.1, 22.2, 22.3, 22.9 (CR-NR-081) --
    // the active profile redirects the resolved User_Data_Dir to a per-profile
    // sub-directory; the DEFAULT_PROFILE (None) is the base, unchanged; distinct
    // profiles resolve to distinct dirs. A SINGLE serialized test drives the
    // process-global so it does not race other tests; it always restores None.
    #[test]
    fn active_profile_redirects_resolved_user_data_dir() {
        // Default (None): base path, unchanged (Req 22.2).
        set_active_profile(None);
        let default_path = UserDataDir::resolve(None).unwrap().path().to_path_buf();
        assert!(
            default_path.ends_with(APP_DIR_NAME),
            "default = <base>/ffworkbench"
        );
        assert!(
            !default_path.to_string_lossy().contains("profiles"),
            "default profile must NOT be under a profiles/ sub-directory"
        );

        // Active profile 'ispf' -> <base>/ffworkbench/profiles/ispf (Req 22.1, 22.3).
        set_active_profile(Some("ispf"));
        let ispf_path = UserDataDir::resolve(None).unwrap().path().to_path_buf();
        assert_eq!(ispf_path, default_path.join("profiles").join("ispf"));
        assert_eq!(active_profile().as_deref(), Some("ispf"));

        // A different profile resolves to a DISTINCT dir (Req 22.9).
        set_active_profile(Some("Rust"));
        let rust_path = UserDataDir::resolve(None).unwrap().path().to_path_buf();
        assert_eq!(rust_path, default_path.join("profiles").join("rust"));
        assert_ne!(rust_path, ispf_path);

        // Empty/blank name is treated as the DEFAULT_PROFILE (Req 22.6).
        set_active_profile(Some("   "));
        assert_eq!(active_profile(), None);
        let blank_path = UserDataDir::resolve(None).unwrap().path().to_path_buf();
        assert_eq!(blank_path, default_path);

        // Always restore the default so no other test sees a stray profile.
        set_active_profile(None);
        assert_eq!(active_profile(), None);
    }

    // Validates: startup-and-session Req 22.2, 22.3 (CR-NR-083) -- default_base
    // and profiles_root are INDEPENDENT of the active profile: they return the
    // profile-agnostic base and profiles root regardless of which profile is
    // active. Serialized with the profile global; always restores None.
    #[test]
    fn default_base_and_profiles_root_ignore_active_profile() {
        set_active_profile(None);
        let base_default = default_base().expect("base");
        let root_default = profiles_root().expect("root");
        assert!(base_default.ends_with(APP_DIR_NAME));
        assert_eq!(root_default, base_default.join("profiles"));
        assert!(
            !base_default.to_string_lossy().contains("profiles"),
            "default_base must be the profile-agnostic base"
        );

        // Even with an active profile, the base/root are UNCHANGED (unlike
        // UserDataDir::resolve(None), which would return the profile sub-dir).
        set_active_profile(Some("ispf"));
        assert_eq!(default_base().expect("base"), base_default);
        assert_eq!(profiles_root().expect("root"), root_default);
        // The active profile's own dir is a child of the profiles root.
        let active = UserDataDir::resolve(None).unwrap().path().to_path_buf();
        assert_eq!(active, root_default.join("ispf"));

        set_active_profile(None);
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
