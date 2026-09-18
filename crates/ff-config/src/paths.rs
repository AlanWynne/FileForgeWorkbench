//! Platform-specific path resolution.
//!
//! Resolves the filesystem locations of configuration files for each layer
//! and platform (Linux, Windows, macOS): system config, user config, profiles
//! directory, languages directory, and project/workspace config.

use std::path::{Path, PathBuf};

// === Application Profile awareness (CR-NR-081, startup-and-session Req 22) ====

/// Process-global active Application_Profile for the CONFIG layer. `None` = the
/// DEFAULT_PROFILE (config at `<config>/ffworkbench/config.toml`, unchanged).
///
/// ff-config cannot depend on ff-session (layering), so it keeps its OWN
/// set-once profile mirror; the desktop shell sets BOTH `ff_session` and this
/// from the single startup arg-parse, BEFORE `init()` (Req 22.5).
static ACTIVE_PROFILE: std::sync::RwLock<Option<String>> = std::sync::RwLock::new(None);

/// Set the active Application_Profile for the config layer (CR-NR-081).
///
/// `Some(name)` isolates config under `profiles/<slug>/`; `None` (or an
/// empty/blank name) uses the default location. Call ONCE at startup BEFORE
/// `init()` so config resolves the right profile from the first read.
///
/// Validates: startup-and-session Requirement 22.3, 22.5
pub fn set_active_profile(name: Option<&str>) {
    let normalised = name
        .map(str::trim)
        .filter(|s| !s.is_empty())
        .map(str::to_string);
    if let Ok(mut guard) = ACTIVE_PROFILE.write() {
        *guard = normalised;
    }
}

/// Return the active config-layer profile name, or `None` for the default.
pub fn active_profile() -> Option<String> {
    ACTIVE_PROFILE.read().ok().and_then(|g| g.clone())
}

/// Slug a profile name for a directory component: lowercase, non-alphanumeric
/// replaced by `-`. Matches `ff_session::profile_slug` so both layers resolve
/// the SAME `profiles/<slug>/` directory for a given profile name.
fn profile_slug(name: &str) -> String {
    name.trim()
        .to_lowercase()
        .chars()
        .map(|c| if c.is_ascii_alphanumeric() { c } else { '-' })
        .collect()
}

/// Join the active profile's sub-directory onto the ffworkbench config base
/// when a profile is active, else return the base unchanged.
fn profile_aware_base(base: PathBuf) -> PathBuf {
    match active_profile() {
        Some(name) => base.join("profiles").join(profile_slug(&name)),
        None => base,
    }
}

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

/// Resolve the user configuration directory (without the filename).
///
/// Returns `None` if the platform config directory cannot be determined.
pub fn user_config_dir() -> Option<PathBuf> {
    // Profile-aware (CR-NR-081): `<config>/ffworkbench/profiles/<slug>` when a
    // profile is active, else `<config>/ffworkbench` (default, unchanged).
    dirs::config_dir().map(|d| profile_aware_base(d.join("ffworkbench")))
}

/// Resolve the user configuration file path.
///
/// Uses the platform's standard config directory:
/// - Linux: `$XDG_CONFIG_HOME/ffworkbench/config.toml` (typically `~/.config/ffworkbench/config.toml`)
/// - Windows: `%APPDATA%\FFWorkbench\config.toml`
/// - macOS: `~/Library/Application Support/FFWorkbench/config.toml`
///
/// Returns `None` if the platform config directory cannot be determined.
///
/// Test/override seam: if the `FFWB_USER_CONFIG_PATH` environment variable is
/// set, its value is used verbatim as the user config file path. This lets
/// tests (and embedding hosts) redirect user-config writes to a temporary file
/// instead of the real per-user config, so `set_user_value`-invoking tests do
/// not read/write the developer's actual config (test-isolation hazard B048).
/// When the variable is unset, production behaviour is unchanged.
pub fn user_config_path() -> Option<PathBuf> {
    // The test/host override always wins (test isolation, B048).
    if let Some(p) = std::env::var_os("FFWB_USER_CONFIG_PATH") {
        return Some(PathBuf::from(p));
    }
    // Profile-aware (CR-NR-081): `<config>/ffworkbench/profiles/<slug>/config.toml`
    // when a profile is active, else `<config>/ffworkbench/config.toml` (default,
    // unchanged), so each Application_Profile has its own isolated config.
    dirs::config_dir().map(|d| profile_aware_base(d.join("ffworkbench")).join("config.toml"))
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

    // Validates: startup-and-session Req 22.3 (CR-NR-081, B064) -- when a config
    // profile is active, the user config path is isolated under
    // `.../ffworkbench/profiles/<slug>/config.toml`; the default (None) is the
    // unchanged base path. Drives the process-global serially and restores it.
    // Guarded by the env override being unset so the test seam does not mask it.
    #[test]
    fn user_config_path_is_profile_aware() {
        // Skip if a host/test override is active (it always wins by design).
        if std::env::var_os("FFWB_USER_CONFIG_PATH").is_some() {
            return;
        }
        // Only meaningful when the platform config dir resolves.
        set_active_profile(None);
        let Some(default_path) = user_config_path() else {
            return;
        };
        // Default: <config>/ffworkbench/config.toml, NOT under profiles/.
        assert!(default_path.ends_with("config.toml"));
        assert!(
            !default_path.to_string_lossy().contains("profiles"),
            "default config must not be under profiles/, got: {}",
            default_path.display()
        );

        // Active profile 'ispf' -> <config>/ffworkbench/profiles/ispf/config.toml.
        set_active_profile(Some("ISPF"));
        let ispf_path = user_config_path().expect("path");
        let base = default_path.parent().unwrap(); // <config>/ffworkbench
        assert_eq!(
            ispf_path,
            base.join("profiles").join("ispf").join("config.toml")
        );

        // A different profile resolves to a distinct dir.
        set_active_profile(Some("rust"));
        let rust_path = user_config_path().expect("path");
        assert_ne!(rust_path, ispf_path);
        assert_eq!(
            rust_path,
            base.join("profiles").join("rust").join("config.toml")
        );

        // Restore the default so no other test sees a stray profile.
        set_active_profile(None);
        assert_eq!(user_config_path(), Some(default_path));
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
