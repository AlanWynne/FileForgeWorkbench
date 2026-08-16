//! User profile management.
//!
//! Handles listing, activating, deactivating, and persisting named user
//! profiles. Each profile is a TOML file providing an overlay between the
//! User and Project layers.

use std::path::{Path, PathBuf};

use crate::error::ConfigError;
use crate::layer::ConfigLayer;
use crate::loader::{load_toml_file, LayerData};

// Re-export the log_warn macro usage via ff_logging dependency.
// The macro is invoked directly as `ff_logging::log_warn!`.

/// Metadata for a discovered user profile.
///
/// Each profile corresponds to a single TOML file in the profiles directory.
/// The profile name is derived from the filename (without the `.toml` extension).
///
/// Addresses: Requirement 4, criteria 1/7
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct UserProfile {
    /// Profile name (derived from filename without extension).
    pub name: String,
    /// Path to the profile's TOML file.
    pub path: PathBuf,
}

/// Manages user profile state: the profiles directory and currently active profile.
///
/// The `ProfileManager` is responsible for tracking which profile (if any) is
/// currently active and loading profile files as `LayerData` ready for insertion
/// into the merge pipeline at the `Profile` layer.
///
/// Addresses: Requirement 4, criteria 2/3
#[derive(Debug, Clone)]
pub struct ProfileManager {
    /// Path to the directory containing profile TOML files.
    profiles_dir: PathBuf,
    /// The name of the currently active profile, or `None` if no profile is active.
    active_profile: Option<String>,
}

impl ProfileManager {
    /// Create a new `ProfileManager` with the given profiles directory.
    ///
    /// No profile is active initially.
    pub fn new(profiles_dir: PathBuf) -> Self {
        Self {
            profiles_dir,
            active_profile: None,
        }
    }

    /// Returns the name of the currently active profile, or `None` if no profile is active.
    pub fn active_profile(&self) -> Option<&str> {
        self.active_profile.as_deref()
    }

    /// Returns a reference to the profiles directory path.
    pub fn profiles_dir(&self) -> &Path {
        &self.profiles_dir
    }

    /// Activate a named profile by loading its TOML file.
    ///
    /// Looks up `{profiles_dir}/{name}.toml`, parses it, and returns the loaded
    /// data as a `LayerData` with `layer: ConfigLayer::Profile`. The caller is
    /// responsible for inserting this into the merge pipeline and recomputing
    /// effective values.
    ///
    /// On success, the active profile name is updated internally.
    ///
    /// # Errors
    ///
    /// Returns `ConfigError::ProfileNotFound` if the profile file does not exist.
    /// Returns `ConfigError::ParseError` if the file contains invalid TOML.
    /// Returns `ConfigError::Io` for other I/O errors.
    pub fn set_active_profile(&mut self, name: &str) -> Result<LayerData, ConfigError> {
        let profile_path = self.profiles_dir.join(format!("{}.toml", name));

        if !profile_path.exists() {
            return Err(ConfigError::ProfileNotFound {
                name: name.to_string(),
            });
        }

        let values = load_toml_file(&profile_path)?;

        self.active_profile = Some(name.to_string());

        Ok(LayerData {
            layer: ConfigLayer::Profile,
            source_path: profile_path,
            values,
        })
    }

    /// Deactivate the currently active profile.
    ///
    /// Sets `active_profile` to `None`, signalling to the caller that the
    /// Profile layer should be removed from the merge pipeline and effective
    /// values should be recomputed. The actual recompute happens in the
    /// `ConfigHandle` orchestration layer.
    ///
    /// If no profile is currently active, this is a no-op.
    pub fn deactivate_profile(&mut self) {
        self.active_profile = None;
    }

    /// Persist the currently active profile selection to the user config file.
    ///
    /// Writes the active profile name into the `[_session].active_profile` key
    /// within the user configuration file. If no profile is active (deactivated),
    /// removes the `active_profile` key from `[_session]`.
    ///
    /// If the user config file does not exist, it is created with only the
    /// `[_session]` table. Existing content in the file is preserved.
    ///
    /// # Errors
    ///
    /// Returns `ConfigError::Io` if the file cannot be read or written.
    /// Returns `ConfigError::ParseError` if the existing file contains invalid TOML.
    ///
    /// Addresses: Requirement 4, criterion 5
    pub fn persist_active_profile(&self, user_config_path: &Path) -> Result<(), ConfigError> {
        // Read the existing file content, or start with an empty document
        let content = if user_config_path.exists() {
            std::fs::read_to_string(user_config_path).map_err(ConfigError::Io)?
        } else {
            String::new()
        };

        // Parse the existing content as a TOML table
        let mut doc: toml::Table =
            content
                .parse()
                .map_err(|e: toml::de::Error| ConfigError::ParseError {
                    path: user_config_path.to_path_buf(),
                    details: e.to_string(),
                })?;

        match &self.active_profile {
            Some(name) => {
                // Ensure the `_session` table exists and set `active_profile`
                let session = doc
                    .entry("_session")
                    .or_insert_with(|| toml::Value::Table(toml::Table::new()));

                if let toml::Value::Table(session_table) = session {
                    session_table.insert(
                        "active_profile".to_string(),
                        toml::Value::String(name.clone()),
                    );
                } else {
                    // `_session` exists but is not a table — replace it
                    *session = toml::Value::Table(toml::Table::new());
                    if let toml::Value::Table(session_table) = session {
                        session_table.insert(
                            "active_profile".to_string(),
                            toml::Value::String(name.clone()),
                        );
                    }
                }
            }
            None => {
                // Remove `active_profile` from `_session`
                if let Some(toml::Value::Table(session_table)) = doc.get_mut("_session") {
                    session_table.remove("active_profile");
                    // If `_session` is now empty, remove the table entirely
                    if session_table.is_empty() {
                        doc.remove("_session");
                    }
                }
            }
        }

        // Serialize and write back
        let serialized = toml::to_string(&doc).map_err(|e| ConfigError::ParseError {
            path: user_config_path.to_path_buf(),
            details: format!("serialization failed: {}", e),
        })?;

        // Ensure parent directory exists
        if let Some(parent) = user_config_path.parent() {
            if !parent.exists() {
                std::fs::create_dir_all(parent).map_err(ConfigError::Io)?;
            }
        }

        std::fs::write(user_config_path, serialized).map_err(ConfigError::Io)?;

        Ok(())
    }

    /// Attempt to auto-activate the previously persisted profile on startup.
    ///
    /// Reads the persisted profile selection from the user config file and, if
    /// found, attempts to activate that profile. This is called during the
    /// initialization sequence to restore the previously active profile.
    ///
    /// # Returns
    ///
    /// - `Some(LayerData)` if a persisted profile was found and activated successfully.
    /// - `None` if no persisted profile exists, or if activation fails (missing file,
    ///   parse error). On failure the profile is deactivated and a warning is emitted.
    ///
    /// Addresses: Requirement 4, criterion 5 (auto-restore on startup)
    pub fn auto_activate(&mut self, user_config_path: &Path) -> Option<LayerData> {
        let name = Self::read_persisted_profile(user_config_path)?;

        match self.set_active_profile(&name) {
            Ok(layer_data) => Some(layer_data),
            Err(err) => {
                ff_logging::log_warn!(
                    "[config] profile: auto-activate failed for persisted profile '{}': {}",
                    name,
                    err
                );
                self.deactivate_profile();
                None
            }
        }
    }

    /// Attempt to activate a profile, handling missing/unreadable profiles gracefully.
    ///
    /// This method implements the full Requirement 4 AC 4.6 behavior:
    /// - Attempts to activate the named profile via [`set_active_profile`]
    /// - On success, returns `Ok(Some(LayerData))` with the loaded profile data
    /// - On `ProfileNotFound` or I/O error: emits a WARN-level log record,
    ///   deactivates the profile (falling back to no active profile), and
    ///   returns `Ok(None)` — allowing the system to continue operating
    /// - On parse error: same graceful handling (warn + deactivate + continue)
    ///
    /// This is the recommended entry point for `ConfigHandle` when switching
    /// profiles at runtime. The caller should:
    /// 1. Call `try_activate_profile(name)`
    /// 2. If `Ok(Some(data))` → insert profile layer into merge pipeline, recompute
    /// 3. If `Ok(None)` → remove profile layer from merge pipeline, recompute
    /// 4. Never needs to handle errors — all failures are gracefully absorbed
    ///
    /// # Returns
    ///
    /// - `Ok(Some(LayerData))` — profile activated successfully
    /// - `Ok(None)` — profile activation failed; WARN logged, profile deactivated,
    ///   system continues with no active profile
    ///
    /// Addresses: Requirement 4, criterion 6
    pub fn try_activate_profile(&mut self, name: &str) -> Result<Option<LayerData>, ConfigError> {
        match self.set_active_profile(name) {
            Ok(layer_data) => Ok(Some(layer_data)),
            Err(ref err @ ConfigError::ProfileNotFound { .. })
            | Err(ref err @ ConfigError::ParseError { .. })
            | Err(ref err @ ConfigError::Io(..)) => {
                ff_logging::log_warn!(
                    "[config] profile: profile '{}' not found or unreadable: {}",
                    name,
                    err
                );
                self.deactivate_profile();
                Ok(None)
            }
            Err(other) => Err(other),
        }
    }

    /// Read the persisted active profile name from the user config file.
    ///
    /// Looks for the `[_session].active_profile` key in the user configuration
    /// file. Returns `Some(name)` if found, `None` otherwise.
    ///
    /// On any error (missing file, parse error, wrong type), returns `None`
    /// silently — the caller should proceed without an active profile.
    ///
    /// Addresses: Requirement 4, criterion 5
    pub fn read_persisted_profile(user_config_path: &Path) -> Option<String> {
        let content = std::fs::read_to_string(user_config_path).ok()?;
        let doc: toml::Table = content.parse().ok()?;
        let session = doc.get("_session")?;
        if let toml::Value::Table(session_table) = session {
            if let Some(toml::Value::String(name)) = session_table.get("active_profile") {
                if !name.is_empty() {
                    return Some(name.clone());
                }
            }
        }
        None
    }
}

/// Scan the given profiles directory for `.toml` files and return discovered profiles.
///
/// Each `.toml` file in the directory is treated as a profile. The profile name
/// is the file stem (filename without the `.toml` extension). For example,
/// `mainframe.toml` produces a profile with name `"mainframe"`.
///
/// Results are sorted alphabetically by name for deterministic ordering.
///
/// If the directory does not exist or cannot be read, returns an empty `Vec`
/// (no error is raised).
///
/// # Examples
///
/// ```no_run
/// use std::path::Path;
/// use ff_config::profile::discover_profiles;
///
/// let profiles = discover_profiles(Path::new("/home/user/.config/ffworkbench/profiles"));
/// for p in &profiles {
///     println!("Found profile: {} at {}", p.name, p.path.display());
/// }
/// ```
pub fn discover_profiles(profiles_dir: &Path) -> Vec<UserProfile> {
    let entries = match std::fs::read_dir(profiles_dir) {
        Ok(entries) => entries,
        Err(_) => return Vec::new(),
    };

    let mut profiles: Vec<UserProfile> = entries
        .filter_map(|entry| entry.ok())
        .filter_map(|entry| {
            let path = entry.path();
            if path.is_file() && path.extension().and_then(|e| e.to_str()) == Some("toml") {
                let name = path.file_stem()?.to_str()?.to_owned();
                Some(UserProfile { name, path })
            } else {
                None
            }
        })
        .collect();

    profiles.sort_by(|a, b| a.name.cmp(&b.name));
    profiles
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::layer::ConfigLayer;
    use crate::value::ConfigValue;
    use std::fs;
    use tempfile::TempDir;

    // Validates: Requirement 4.7 — list all available profiles by scanning the profiles directory
    #[test]
    fn discover_profiles_finds_toml_files_in_directory() {
        let dir = TempDir::new().unwrap();
        fs::write(dir.path().join("mainframe.toml"), "# mainframe profile").unwrap();
        fs::write(dir.path().join("web-dev.toml"), "# web-dev profile").unwrap();

        let profiles = discover_profiles(dir.path());

        assert_eq!(profiles.len(), 2);
        assert_eq!(profiles[0].name, "mainframe");
        assert_eq!(profiles[1].name, "web-dev");
    }

    // Validates: Requirement 4.1 — profile name derived from filename without extension
    #[test]
    fn discover_profiles_extracts_name_from_filename_stem() {
        let dir = TempDir::new().unwrap();
        fs::write(dir.path().join("database.toml"), "").unwrap();

        let profiles = discover_profiles(dir.path());

        assert_eq!(profiles.len(), 1);
        assert_eq!(profiles[0].name, "database");
        assert_eq!(profiles[0].path, dir.path().join("database.toml"));
    }

    // Validates: Requirement 4.7 — only .toml files are considered profiles
    #[test]
    fn discover_profiles_ignores_non_toml_files() {
        let dir = TempDir::new().unwrap();
        fs::write(dir.path().join("notes.txt"), "not a profile").unwrap();
        fs::write(dir.path().join("data.json"), "{}").unwrap();
        fs::write(dir.path().join("valid.toml"), "# profile").unwrap();

        let profiles = discover_profiles(dir.path());

        assert_eq!(profiles.len(), 1);
        assert_eq!(profiles[0].name, "valid");
    }

    // Validates: Requirement 4.7 — subdirectories are not treated as profiles
    #[test]
    fn discover_profiles_ignores_subdirectories() {
        let dir = TempDir::new().unwrap();
        fs::create_dir(dir.path().join("subdir.toml")).unwrap();
        fs::write(dir.path().join("real.toml"), "").unwrap();

        let profiles = discover_profiles(dir.path());

        assert_eq!(profiles.len(), 1);
        assert_eq!(profiles[0].name, "real");
    }

    // Validates: Requirement 4.7 — non-existent directory returns empty vec (no error)
    #[test]
    fn discover_profiles_returns_empty_vec_for_nonexistent_directory() {
        let profiles = discover_profiles(Path::new("/nonexistent/path/that/does/not/exist"));

        assert!(profiles.is_empty());
    }

    // Validates: Requirement 4.7 — results sorted alphabetically by name
    #[test]
    fn discover_profiles_returns_results_sorted_alphabetically() {
        let dir = TempDir::new().unwrap();
        fs::write(dir.path().join("zebra.toml"), "").unwrap();
        fs::write(dir.path().join("alpha.toml"), "").unwrap();
        fs::write(dir.path().join("middle.toml"), "").unwrap();

        let profiles = discover_profiles(dir.path());

        assert_eq!(profiles.len(), 3);
        assert_eq!(profiles[0].name, "alpha");
        assert_eq!(profiles[1].name, "middle");
        assert_eq!(profiles[2].name, "zebra");
    }

    // Validates: Requirement 4.7 — empty directory returns empty vec
    #[test]
    fn discover_profiles_returns_empty_vec_for_empty_directory() {
        let dir = TempDir::new().unwrap();

        let profiles = discover_profiles(dir.path());

        assert!(profiles.is_empty());
    }

    // ========================================================================
    // 14.2 — ProfileManager: set_active_profile and active_profile
    // ========================================================================

    // Validates: Requirement 4.2 — ProfileManager starts with no active profile
    #[test]
    fn profile_manager_starts_with_no_active_profile() {
        let dir = TempDir::new().unwrap();
        let manager = ProfileManager::new(dir.path().to_path_buf());

        assert_eq!(manager.active_profile(), None);
    }

    // Validates: Requirement 4.2 — set_active_profile loads profile and returns LayerData at Profile layer
    #[test]
    fn set_active_profile_loads_valid_profile_and_returns_layer_data() {
        let dir = TempDir::new().unwrap();
        fs::write(
            dir.path().join("mainframe.toml"),
            "[editor]\ntab_size = 8\n",
        )
        .unwrap();

        let mut manager = ProfileManager::new(dir.path().to_path_buf());
        let layer_data = manager.set_active_profile("mainframe").unwrap();

        assert_eq!(layer_data.layer, ConfigLayer::Profile);
        assert_eq!(layer_data.source_path, dir.path().join("mainframe.toml"));

        // Verify the values were loaded
        if let Some(ConfigValue::Table(editor)) = layer_data.values.get("editor") {
            assert_eq!(editor.get("tab_size"), Some(&ConfigValue::Integer(8)));
        } else {
            panic!("Expected editor table in loaded profile");
        }
    }

    // Validates: Requirement 4.2 — active_profile returns the name after activation
    #[test]
    fn active_profile_returns_name_after_successful_activation() {
        let dir = TempDir::new().unwrap();
        fs::write(dir.path().join("web-dev.toml"), "[editor]\ntab_size = 2\n").unwrap();

        let mut manager = ProfileManager::new(dir.path().to_path_buf());
        manager.set_active_profile("web-dev").unwrap();

        assert_eq!(manager.active_profile(), Some("web-dev"));
    }

    // Validates: Requirement 4.6 — ProfileNotFound error for missing profile
    #[test]
    fn set_active_profile_returns_profile_not_found_for_missing_file() {
        let dir = TempDir::new().unwrap();
        let mut manager = ProfileManager::new(dir.path().to_path_buf());

        let result = manager.set_active_profile("nonexistent");

        assert!(result.is_err());
        match result.unwrap_err() {
            ConfigError::ProfileNotFound { name } => {
                assert_eq!(name, "nonexistent");
            }
            other => panic!("Expected ProfileNotFound, got: {:?}", other),
        }

        // Active profile should remain None after failure
        assert_eq!(manager.active_profile(), None);
    }

    // Validates: Requirement 4.2 — ParseError for invalid TOML profile
    #[test]
    fn set_active_profile_returns_parse_error_for_invalid_toml() {
        let dir = TempDir::new().unwrap();
        fs::write(dir.path().join("broken.toml"), "this is not [valid toml [[").unwrap();

        let mut manager = ProfileManager::new(dir.path().to_path_buf());
        let result = manager.set_active_profile("broken");

        assert!(result.is_err());
        match result.unwrap_err() {
            ConfigError::ParseError { path, .. } => {
                assert_eq!(path, dir.path().join("broken.toml"));
            }
            other => panic!("Expected ParseError, got: {:?}", other),
        }

        // Active profile should remain None after failure
        assert_eq!(manager.active_profile(), None);
    }

    // Validates: Requirement 4.3 — switching profiles updates active_profile name
    #[test]
    fn set_active_profile_switches_from_one_profile_to_another() {
        let dir = TempDir::new().unwrap();
        fs::write(dir.path().join("alpha.toml"), "[editor]\ntab_size = 2\n").unwrap();
        fs::write(dir.path().join("beta.toml"), "[editor]\ntab_size = 4\n").unwrap();

        let mut manager = ProfileManager::new(dir.path().to_path_buf());

        manager.set_active_profile("alpha").unwrap();
        assert_eq!(manager.active_profile(), Some("alpha"));

        let layer_data = manager.set_active_profile("beta").unwrap();
        assert_eq!(manager.active_profile(), Some("beta"));

        // Verify the new profile's values are returned
        if let Some(ConfigValue::Table(editor)) = layer_data.values.get("editor") {
            assert_eq!(editor.get("tab_size"), Some(&ConfigValue::Integer(4)));
        } else {
            panic!("Expected editor table in beta profile");
        }
    }

    // Validates: Requirement 4.2 — source_path in LayerData points to the profile file
    #[test]
    fn set_active_profile_layer_data_has_correct_source_path() {
        let dir = TempDir::new().unwrap();
        fs::write(
            dir.path().join("database.toml"),
            "[logging]\nlevel = \"debug\"\n",
        )
        .unwrap();

        let mut manager = ProfileManager::new(dir.path().to_path_buf());
        let layer_data = manager.set_active_profile("database").unwrap();

        assert_eq!(layer_data.source_path, dir.path().join("database.toml"));
    }

    // Validates: Requirement 4.2 — profiles_dir getter returns the configured directory
    #[test]
    fn profiles_dir_returns_configured_directory() {
        let dir = TempDir::new().unwrap();
        let manager = ProfileManager::new(dir.path().to_path_buf());

        assert_eq!(manager.profiles_dir(), dir.path());
    }

    // ========================================================================
    // 14.3 — ProfileManager: deactivate_profile
    // ========================================================================

    // Validates: Requirement 4.3 — deactivation after activation returns None for active_profile
    #[test]
    fn deactivate_profile_after_activation_returns_none() {
        let dir = TempDir::new().unwrap();
        fs::write(dir.path().join("web-dev.toml"), "[editor]\ntab_size = 2\n").unwrap();

        let mut manager = ProfileManager::new(dir.path().to_path_buf());
        manager.set_active_profile("web-dev").unwrap();
        assert_eq!(manager.active_profile(), Some("web-dev"));

        manager.deactivate_profile();
        assert_eq!(manager.active_profile(), None);
    }

    // Validates: Requirement 4.3 — deactivation when already inactive is a no-op (no error)
    #[test]
    fn deactivate_profile_when_already_inactive_is_noop() {
        let dir = TempDir::new().unwrap();
        let mut manager = ProfileManager::new(dir.path().to_path_buf());

        // Already inactive — should not panic or error
        assert_eq!(manager.active_profile(), None);
        manager.deactivate_profile();
        assert_eq!(manager.active_profile(), None);
    }

    // Validates: Requirement 4.4 — at most one profile active at any time; activating a new
    // profile automatically deactivates the previous one
    #[test]
    fn single_activation_invariant_only_one_profile_active_at_any_time() {
        let dir = TempDir::new().unwrap();
        fs::write(dir.path().join("alpha.toml"), "[editor]\ntab_size = 2\n").unwrap();
        fs::write(dir.path().join("beta.toml"), "[editor]\ntab_size = 4\n").unwrap();
        fs::write(dir.path().join("gamma.toml"), "[editor]\ntab_size = 8\n").unwrap();

        let mut manager = ProfileManager::new(dir.path().to_path_buf());

        // Initially no profile is active
        assert_eq!(manager.active_profile(), None);

        // Activate alpha
        manager.set_active_profile("alpha").unwrap();
        assert_eq!(manager.active_profile(), Some("alpha"));

        // Activate beta — alpha is automatically deactivated
        manager.set_active_profile("beta").unwrap();
        assert_eq!(manager.active_profile(), Some("beta"));
        // Only beta is active, not alpha
        assert_ne!(manager.active_profile(), Some("alpha"));

        // Activate gamma — beta is automatically deactivated
        let layer_data = manager.set_active_profile("gamma").unwrap();
        assert_eq!(manager.active_profile(), Some("gamma"));
        // Only gamma is active, not alpha or beta
        assert_ne!(manager.active_profile(), Some("alpha"));
        assert_ne!(manager.active_profile(), Some("beta"));

        // Verify returned LayerData is from the latest activation only
        assert_eq!(layer_data.layer, ConfigLayer::Profile);
        if let Some(ConfigValue::Table(editor)) = layer_data.values.get("editor") {
            assert_eq!(editor.get("tab_size"), Some(&ConfigValue::Integer(8)));
        } else {
            panic!("Expected editor table in gamma profile");
        }

        // Deactivate — no profile active
        manager.deactivate_profile();
        assert_eq!(manager.active_profile(), None);
    }

    // Validates: Requirement 4.2 — empty profile file produces empty values
    #[test]
    fn set_active_profile_loads_empty_profile_successfully() {
        let dir = TempDir::new().unwrap();
        fs::write(dir.path().join("empty.toml"), "").unwrap();

        let mut manager = ProfileManager::new(dir.path().to_path_buf());
        let layer_data = manager.set_active_profile("empty").unwrap();

        assert_eq!(layer_data.layer, ConfigLayer::Profile);
        assert!(layer_data.values.is_empty());
        assert_eq!(manager.active_profile(), Some("empty"));
    }

    // ========================================================================
    // 14.5 — ProfileManager: active profile persistence
    // ========================================================================

    // Validates: Requirement 4.5 — persist_active_profile writes to [_session].active_profile
    #[test]
    fn persist_active_profile_writes_session_table_to_user_config() {
        let dir = TempDir::new().unwrap();
        let profiles_dir = dir.path().join("profiles");
        fs::create_dir(&profiles_dir).unwrap();
        fs::write(
            profiles_dir.join("mainframe.toml"),
            "[editor]\ntab_size = 8\n",
        )
        .unwrap();

        let user_config = dir.path().join("config.toml");

        let mut manager = ProfileManager::new(profiles_dir);
        manager.set_active_profile("mainframe").unwrap();
        manager.persist_active_profile(&user_config).unwrap();

        // Verify the file was created and contains the expected content
        let content = fs::read_to_string(&user_config).unwrap();
        let doc: toml::Table = content.parse().unwrap();
        let session = doc.get("_session").unwrap();
        if let toml::Value::Table(session_table) = session {
            assert_eq!(
                session_table.get("active_profile"),
                Some(&toml::Value::String("mainframe".to_string()))
            );
        } else {
            panic!("Expected _session to be a table");
        }
    }

    // Validates: Requirement 4.5 — persist_active_profile preserves existing config content
    #[test]
    fn persist_active_profile_preserves_existing_file_content() {
        let dir = TempDir::new().unwrap();
        let profiles_dir = dir.path().join("profiles");
        fs::create_dir(&profiles_dir).unwrap();
        fs::write(
            profiles_dir.join("web-dev.toml"),
            "[editor]\ntab_size = 2\n",
        )
        .unwrap();

        let user_config = dir.path().join("config.toml");
        // Write existing config content
        fs::write(
            &user_config,
            "[editor]\ntab_size = 4\n\n[logging]\nlevel = \"info\"\n",
        )
        .unwrap();

        let mut manager = ProfileManager::new(profiles_dir);
        manager.set_active_profile("web-dev").unwrap();
        manager.persist_active_profile(&user_config).unwrap();

        // Verify existing content is preserved
        let content = fs::read_to_string(&user_config).unwrap();
        let doc: toml::Table = content.parse().unwrap();

        // Original content still present
        if let Some(toml::Value::Table(editor)) = doc.get("editor") {
            assert_eq!(editor.get("tab_size"), Some(&toml::Value::Integer(4)));
        } else {
            panic!("Expected editor table to be preserved");
        }

        if let Some(toml::Value::Table(logging)) = doc.get("logging") {
            assert_eq!(
                logging.get("level"),
                Some(&toml::Value::String("info".to_string()))
            );
        } else {
            panic!("Expected logging table to be preserved");
        }

        // Session table added
        if let Some(toml::Value::Table(session)) = doc.get("_session") {
            assert_eq!(
                session.get("active_profile"),
                Some(&toml::Value::String("web-dev".to_string()))
            );
        } else {
            panic!("Expected _session table to be added");
        }
    }

    // Validates: Requirement 4.5 — deactivation removes active_profile key from _session
    #[test]
    fn persist_active_profile_removes_key_when_deactivated() {
        let dir = TempDir::new().unwrap();
        let profiles_dir = dir.path().join("profiles");
        fs::create_dir(&profiles_dir).unwrap();
        fs::write(profiles_dir.join("alpha.toml"), "[editor]\ntab_size = 2\n").unwrap();

        let user_config = dir.path().join("config.toml");

        let mut manager = ProfileManager::new(profiles_dir);

        // Activate and persist
        manager.set_active_profile("alpha").unwrap();
        manager.persist_active_profile(&user_config).unwrap();

        // Verify it was written
        let content = fs::read_to_string(&user_config).unwrap();
        assert!(content.contains("active_profile"));

        // Deactivate and persist again
        manager.deactivate_profile();
        manager.persist_active_profile(&user_config).unwrap();

        // Verify active_profile key is removed and _session table is cleaned up
        let content = fs::read_to_string(&user_config).unwrap();
        let doc: toml::Table = content.parse().unwrap();
        assert!(
            doc.get("_session").is_none(),
            "Empty _session table should be removed"
        );
    }

    // Validates: Requirement 4.5 — persist creates file if it doesn't exist
    #[test]
    fn persist_active_profile_creates_file_if_missing() {
        let dir = TempDir::new().unwrap();
        let profiles_dir = dir.path().join("profiles");
        fs::create_dir(&profiles_dir).unwrap();
        fs::write(
            profiles_dir.join("database.toml"),
            "[logging]\nlevel = \"debug\"\n",
        )
        .unwrap();

        let user_config = dir.path().join("subdir").join("config.toml");
        // File and parent directory do not exist yet

        let mut manager = ProfileManager::new(profiles_dir);
        manager.set_active_profile("database").unwrap();
        manager.persist_active_profile(&user_config).unwrap();

        assert!(user_config.exists());
        let content = fs::read_to_string(&user_config).unwrap();
        let doc: toml::Table = content.parse().unwrap();
        if let Some(toml::Value::Table(session)) = doc.get("_session") {
            assert_eq!(
                session.get("active_profile"),
                Some(&toml::Value::String("database".to_string()))
            );
        } else {
            panic!("Expected _session table in newly created file");
        }
    }

    // Validates: Requirement 4.5 — read_persisted_profile returns profile name from file
    #[test]
    fn read_persisted_profile_returns_stored_name() {
        let dir = TempDir::new().unwrap();
        let user_config = dir.path().join("config.toml");
        fs::write(
            &user_config,
            "[editor]\ntab_size = 4\n\n[_session]\nactive_profile = \"mainframe\"\n",
        )
        .unwrap();

        let result = ProfileManager::read_persisted_profile(&user_config);
        assert_eq!(result, Some("mainframe".to_string()));
    }

    // Validates: Requirement 4.5 — read_persisted_profile returns None for missing file
    #[test]
    fn read_persisted_profile_returns_none_for_missing_file() {
        let result = ProfileManager::read_persisted_profile(Path::new("/nonexistent/config.toml"));
        assert_eq!(result, None);
    }

    // Validates: Requirement 4.5 — read_persisted_profile returns None when no _session table
    #[test]
    fn read_persisted_profile_returns_none_when_no_session_table() {
        let dir = TempDir::new().unwrap();
        let user_config = dir.path().join("config.toml");
        fs::write(&user_config, "[editor]\ntab_size = 4\n").unwrap();

        let result = ProfileManager::read_persisted_profile(&user_config);
        assert_eq!(result, None);
    }

    // Validates: Requirement 4.5 — read_persisted_profile returns None for invalid TOML
    #[test]
    fn read_persisted_profile_returns_none_for_invalid_toml() {
        let dir = TempDir::new().unwrap();
        let user_config = dir.path().join("config.toml");
        fs::write(&user_config, "this is not [valid toml [[").unwrap();

        let result = ProfileManager::read_persisted_profile(&user_config);
        assert_eq!(result, None);
    }

    // Validates: Requirement 4.5 — read_persisted_profile returns None if active_profile is not a string
    #[test]
    fn read_persisted_profile_returns_none_for_non_string_active_profile() {
        let dir = TempDir::new().unwrap();
        let user_config = dir.path().join("config.toml");
        fs::write(&user_config, "[_session]\nactive_profile = 42\n").unwrap();

        let result = ProfileManager::read_persisted_profile(&user_config);
        assert_eq!(result, None);
    }

    // Validates: Requirement 4.5 — read_persisted_profile returns None for empty string
    #[test]
    fn read_persisted_profile_returns_none_for_empty_string_value() {
        let dir = TempDir::new().unwrap();
        let user_config = dir.path().join("config.toml");
        fs::write(&user_config, "[_session]\nactive_profile = \"\"\n").unwrap();

        let result = ProfileManager::read_persisted_profile(&user_config);
        assert_eq!(result, None);
    }

    // Validates: Requirement 4.5 — round-trip: persist then read back
    #[test]
    fn persist_and_read_active_profile_round_trip() {
        let dir = TempDir::new().unwrap();
        let profiles_dir = dir.path().join("profiles");
        fs::create_dir(&profiles_dir).unwrap();
        fs::write(
            profiles_dir.join("web-dev.toml"),
            "[editor]\ntab_size = 2\n",
        )
        .unwrap();

        let user_config = dir.path().join("config.toml");

        let mut manager = ProfileManager::new(profiles_dir);
        manager.set_active_profile("web-dev").unwrap();
        manager.persist_active_profile(&user_config).unwrap();

        let read_back = ProfileManager::read_persisted_profile(&user_config);
        assert_eq!(read_back, Some("web-dev".to_string()));
    }

    // Validates: Requirement 4.5 — round-trip with deactivation
    #[test]
    fn persist_and_read_deactivated_profile_round_trip() {
        let dir = TempDir::new().unwrap();
        let profiles_dir = dir.path().join("profiles");
        fs::create_dir(&profiles_dir).unwrap();
        fs::write(profiles_dir.join("alpha.toml"), "[editor]\ntab_size = 2\n").unwrap();

        let user_config = dir.path().join("config.toml");

        let mut manager = ProfileManager::new(profiles_dir);

        // Activate, persist, then deactivate and persist again
        manager.set_active_profile("alpha").unwrap();
        manager.persist_active_profile(&user_config).unwrap();
        assert_eq!(
            ProfileManager::read_persisted_profile(&user_config),
            Some("alpha".to_string())
        );

        manager.deactivate_profile();
        manager.persist_active_profile(&user_config).unwrap();
        assert_eq!(ProfileManager::read_persisted_profile(&user_config), None);
    }

    // ========================================================================
    // 14.6 — ProfileManager: auto_activate on startup
    // ========================================================================

    // Validates: Requirement 4.5 — auto_activate succeeds when persisted profile exists and is valid
    #[test]
    fn auto_activate_succeeds_when_persisted_profile_exists_and_is_valid() {
        let dir = TempDir::new().unwrap();
        let profiles_dir = dir.path().join("profiles");
        fs::create_dir(&profiles_dir).unwrap();
        fs::write(
            profiles_dir.join("mainframe.toml"),
            "[editor]\ntab_size = 8\n",
        )
        .unwrap();

        let user_config = dir.path().join("config.toml");
        fs::write(&user_config, "[_session]\nactive_profile = \"mainframe\"\n").unwrap();

        let mut manager = ProfileManager::new(profiles_dir);
        let result = manager.auto_activate(&user_config);

        assert!(result.is_some());
        let layer_data = result.unwrap();
        assert_eq!(layer_data.layer, ConfigLayer::Profile);
        assert_eq!(manager.active_profile(), Some("mainframe"));

        // Verify the loaded values
        if let Some(ConfigValue::Table(editor)) = layer_data.values.get("editor") {
            assert_eq!(editor.get("tab_size"), Some(&ConfigValue::Integer(8)));
        } else {
            panic!("Expected editor table in loaded profile");
        }
    }

    // Validates: Requirement 4.5 — auto_activate returns None when no persisted profile
    #[test]
    fn auto_activate_returns_none_when_no_persisted_profile() {
        let dir = TempDir::new().unwrap();
        let profiles_dir = dir.path().join("profiles");
        fs::create_dir(&profiles_dir).unwrap();

        let user_config = dir.path().join("config.toml");
        // Config file exists but has no _session.active_profile
        fs::write(&user_config, "[editor]\ntab_size = 4\n").unwrap();

        let mut manager = ProfileManager::new(profiles_dir);
        let result = manager.auto_activate(&user_config);

        assert!(result.is_none());
        assert_eq!(manager.active_profile(), None);
    }

    // Validates: Requirement 4.5 — auto_activate returns None and deactivates when persisted profile file is missing
    #[test]
    fn auto_activate_returns_none_and_deactivates_when_profile_file_missing() {
        let dir = TempDir::new().unwrap();
        let profiles_dir = dir.path().join("profiles");
        fs::create_dir(&profiles_dir).unwrap();
        // Note: no "ghost.toml" file exists in profiles_dir

        let user_config = dir.path().join("config.toml");
        // Persisted profile references a profile that doesn't exist on disk
        fs::write(&user_config, "[_session]\nactive_profile = \"ghost\"\n").unwrap();

        let mut manager = ProfileManager::new(profiles_dir);
        let result = manager.auto_activate(&user_config);

        assert!(result.is_none());
        // Profile should be deactivated after failure
        assert_eq!(manager.active_profile(), None);
    }

    // ========================================================================
    // 14.7 — Missing profile handling (AC 4.6)
    // ========================================================================

    // Validates: Requirement 4.6 — try_activate_profile returns Ok(None) and deactivates
    // when profile file does not exist
    #[test]
    fn try_activate_profile_returns_none_and_deactivates_when_file_missing() {
        let dir = TempDir::new().unwrap();
        let profiles_dir = dir.path().join("profiles");
        fs::create_dir(&profiles_dir).unwrap();
        // No profile file exists

        let mut manager = ProfileManager::new(profiles_dir);
        let result = manager.try_activate_profile("nonexistent");

        // Should return Ok(None) — graceful handling, not an error
        assert!(result.is_ok());
        assert!(result.unwrap().is_none());

        // Profile should be deactivated (no active profile)
        assert_eq!(manager.active_profile(), None);
    }

    // Validates: Requirement 4.6 — try_activate_profile returns Ok(None) and deactivates
    // when profile file contains invalid TOML
    #[test]
    fn try_activate_profile_returns_none_and_deactivates_when_file_has_invalid_toml() {
        let dir = TempDir::new().unwrap();
        let profiles_dir = dir.path().join("profiles");
        fs::create_dir(&profiles_dir).unwrap();
        fs::write(
            profiles_dir.join("broken.toml"),
            "this is not [valid toml [[",
        )
        .unwrap();

        let mut manager = ProfileManager::new(profiles_dir);
        let result = manager.try_activate_profile("broken");

        // Should return Ok(None) — graceful handling
        assert!(result.is_ok());
        assert!(result.unwrap().is_none());

        // Profile should be deactivated
        assert_eq!(manager.active_profile(), None);
    }

    // Validates: Requirement 4.6 — try_activate_profile succeeds for valid profile
    #[test]
    fn try_activate_profile_returns_layer_data_for_valid_profile() {
        let dir = TempDir::new().unwrap();
        let profiles_dir = dir.path().join("profiles");
        fs::create_dir(&profiles_dir).unwrap();
        fs::write(
            profiles_dir.join("mainframe.toml"),
            "[editor]\ntab_size = 8\n",
        )
        .unwrap();

        let mut manager = ProfileManager::new(profiles_dir);
        let result = manager.try_activate_profile("mainframe");

        assert!(result.is_ok());
        let layer_data = result.unwrap();
        assert!(layer_data.is_some());

        let data = layer_data.unwrap();
        assert_eq!(data.layer, ConfigLayer::Profile);
        assert_eq!(manager.active_profile(), Some("mainframe"));
    }

    // Validates: Requirement 4.6 — try_activate_profile deactivates previously active
    // profile when new profile is not found
    #[test]
    fn try_activate_profile_deactivates_previous_profile_on_failure() {
        let dir = TempDir::new().unwrap();
        let profiles_dir = dir.path().join("profiles");
        fs::create_dir(&profiles_dir).unwrap();
        fs::write(profiles_dir.join("alpha.toml"), "[editor]\ntab_size = 2\n").unwrap();
        // No "ghost.toml" exists

        let mut manager = ProfileManager::new(profiles_dir);

        // First activate a valid profile
        manager.set_active_profile("alpha").unwrap();
        assert_eq!(manager.active_profile(), Some("alpha"));

        // Now try to activate a missing profile
        let result = manager.try_activate_profile("ghost");

        // Should succeed gracefully
        assert!(result.is_ok());
        assert!(result.unwrap().is_none());

        // Profile should be fully deactivated — no fallback to "alpha"
        assert_eq!(manager.active_profile(), None);
    }

    // Validates: Requirement 4.6 — system continues operating after missing profile
    // (verify that subsequent valid activation still works)
    #[test]
    fn try_activate_profile_system_continues_operating_after_missing_profile() {
        let dir = TempDir::new().unwrap();
        let profiles_dir = dir.path().join("profiles");
        fs::create_dir(&profiles_dir).unwrap();
        fs::write(profiles_dir.join("valid.toml"), "[editor]\ntab_size = 4\n").unwrap();
        // No "missing.toml" exists

        let mut manager = ProfileManager::new(profiles_dir);

        // Try a missing profile — should handle gracefully
        let result = manager.try_activate_profile("missing");
        assert!(result.is_ok());
        assert!(result.unwrap().is_none());
        assert_eq!(manager.active_profile(), None);

        // Now try a valid profile — should succeed normally
        let result = manager.try_activate_profile("valid");
        assert!(result.is_ok());
        let layer_data = result.unwrap().unwrap();
        assert_eq!(layer_data.layer, ConfigLayer::Profile);
        assert_eq!(manager.active_profile(), Some("valid"));
    }
}
