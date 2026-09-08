//! `CommandStore` -- load, save, validate, and hot-reload `commands.toml`.
//!
//! Validates: command-configurator Requirement 1, 4.1, 4.2, 4.6.

use std::path::{Path, PathBuf};
use std::time::SystemTime;

use ff_command::CommandTarget;

use super::CommandDefinition;

/// The loaded Command_Store plus load diagnostics.
///
/// Holds only the valid definitions; invalid entries are skipped and recorded
/// in `load_error` (Requirement 1.6). `last_modified` drives hot-reload.
#[derive(Debug, Clone, Default)]
pub struct CommandStore {
    /// Absolute path to `commands.toml`.
    pub file_path: PathBuf,
    /// Valid definitions, in file order.
    pub definitions: Vec<CommandDefinition>,
    /// Human-readable message when one or more entries were skipped/invalid.
    pub load_error: Option<String>,
    /// File modification time at last load, for hot-reload.
    pub last_modified: Option<SystemTime>,
}

/// The result of validating a single `CommandDefinition`.
///
/// Validates: command-configurator Requirement 4.1, 4.2, 4.6.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum ValidationError {
    /// The `id` field is empty.
    EmptyId,
    /// The `id` violates the Command_ID naming rule.
    InvalidId(String),
    /// The `label` field is empty.
    EmptyLabel,
    /// An External target has an empty `program`.
    EmptyProgram,
    /// The `id` shadows a reserved built-in command id.
    ReservedId(String),
}

impl std::fmt::Display for ValidationError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            ValidationError::EmptyId => write!(f, "command id must not be empty"),
            ValidationError::InvalidId(id) => write!(
                f,
                "command id '{id}' is invalid (use lowercase letters, digits, dots, underscores)"
            ),
            ValidationError::EmptyLabel => write!(f, "command label must not be empty"),
            ValidationError::EmptyProgram => {
                write!(f, "external command program must not be empty")
            }
            ValidationError::ReservedId(id) => {
                write!(
                    f,
                    "command id '{id}' conflicts with a reserved built-in command"
                )
            }
        }
    }
}

/// Validate a single definition against the Command_Store rules.
///
/// `is_reserved` reports whether an id shadows a reserved built-in command id
/// (Requirement 4.6); the caller supplies it (typically a registry lookup).
///
/// Validates: command-configurator Requirement 4.1, 4.2, 4.6.
pub fn validate_definition<F>(
    def: &CommandDefinition,
    is_reserved: F,
) -> Result<(), ValidationError>
where
    F: Fn(&str) -> bool,
{
    if def.id.is_empty() {
        return Err(ValidationError::EmptyId);
    }
    if !ff_command::CommandId::is_valid(&def.id) {
        return Err(ValidationError::InvalidId(def.id.clone()));
    }
    if is_reserved(&def.id) {
        return Err(ValidationError::ReservedId(def.id.clone()));
    }
    if def.label.trim().is_empty() {
        return Err(ValidationError::EmptyLabel);
    }
    // External target: program must be non-empty (Requirement 4.2).
    // (mode is a typed enum, so an invalid mode cannot be constructed here.)
    if let CommandTarget::External { program, .. } = &def.target {
        if program.trim().is_empty() {
            return Err(ValidationError::EmptyProgram);
        }
    }
    Ok(())
}

impl CommandStore {
    /// Create an empty store bound to `file_path` (no file read).
    pub fn new(file_path: impl Into<PathBuf>) -> Self {
        Self {
            file_path: file_path.into(),
            definitions: Vec::new(),
            load_error: None,
            last_modified: None,
        }
    }

    /// Load definitions from the store file at `path`.
    ///
    /// An absent file yields an empty store with no error (Requirement 1.5).
    /// Invalid TOML at the file level, or individual entries missing required
    /// fields, are tolerated per Requirement 1.6: valid entries are retained
    /// and a `load_error` summarises what was skipped. Duplicate ids keep the
    /// first occurrence (Requirement 1.4).
    ///
    /// Validates: command-configurator Requirement 1.1, 1.4, 1.5, 1.6.
    pub fn load(path: impl AsRef<Path>) -> Self {
        let file_path = path.as_ref().to_path_buf();
        let last_modified = std::fs::metadata(&file_path)
            .ok()
            .and_then(|m| m.modified().ok());

        let contents = match std::fs::read_to_string(&file_path) {
            Ok(c) => c,
            Err(_) => {
                // Absent/unreadable file -> empty store, no error.
                return Self {
                    file_path,
                    definitions: Vec::new(),
                    load_error: None,
                    last_modified: None,
                };
            }
        };

        let mut store = Self {
            file_path,
            definitions: Vec::new(),
            load_error: None,
            last_modified,
        };
        store.parse_into(&contents);
        store
    }

    /// Parse `contents` into `self.definitions`, tolerating bad entries.
    fn parse_into(&mut self, contents: &str) {
        let value: toml::Value = match contents.parse() {
            Ok(v) => v,
            Err(e) => {
                self.load_error = Some(format!("commands.toml is not valid TOML: {e}"));
                return;
            }
        };

        let entries = match value.get("command").and_then(|c| c.as_array()) {
            Some(arr) => arr,
            None => {
                // No [[command]] entries -- an empty but valid store.
                return;
            }
        };

        let mut skipped: Vec<String> = Vec::new();
        for (idx, entry) in entries.iter().enumerate() {
            match entry.clone().try_into::<CommandDefinition>() {
                Ok(def) => {
                    if self.definitions.iter().any(|d| d.id == def.id) {
                        // Requirement 1.4: keep first, skip duplicate.
                        skipped.push(format!("duplicate id '{}'", def.id));
                        continue;
                    }
                    self.definitions.push(def);
                }
                Err(e) => {
                    // Requirement 1.6: skip invalid entry, keep the rest.
                    skipped.push(format!("entry #{} ({e})", idx + 1));
                }
            }
        }

        if !skipped.is_empty() {
            self.load_error = Some(format!(
                "skipped {} entry/entries: {}",
                skipped.len(),
                skipped.join("; ")
            ));
        }
    }

    /// Serialise the current definitions to a TOML string.
    ///
    /// Validates: command-configurator Requirement 1.1, 1.2.
    pub fn to_toml(&self) -> Result<String, toml::ser::Error> {
        #[derive(serde::Serialize)]
        struct Doc<'a> {
            command: &'a [CommandDefinition],
        }
        toml::to_string_pretty(&Doc {
            command: &self.definitions,
        })
    }

    /// Write the full store back to `file_path`, creating the parent directory
    /// (e.g. `commands/`) if it does not exist (Requirement 1.5, 2.4, 2.5).
    ///
    /// # Errors
    /// Returns an error if the directory cannot be created, the definitions
    /// cannot be serialised, or the file cannot be written.
    pub fn save(&mut self) -> Result<(), String> {
        if let Some(parent) = self.file_path.parent() {
            std::fs::create_dir_all(parent)
                .map_err(|e| format!("cannot create {}: {e}", parent.display()))?;
        }
        let toml = self
            .to_toml()
            .map_err(|e| format!("cannot serialise commands: {e}"))?;
        std::fs::write(&self.file_path, toml)
            .map_err(|e| format!("cannot write {}: {e}", self.file_path.display()))?;
        self.last_modified = std::fs::metadata(&self.file_path)
            .ok()
            .and_then(|m| m.modified().ok());
        Ok(())
    }

    /// Poll for on-disk changes and reload if the mtime changed.
    ///
    /// Mirrors `MenuWorkspaceState::poll_reload` (menu-workspace Requirement
    /// 4.3, 4.4) -- no separate file watcher.
    ///
    /// Validates: command-configurator Requirement 1.7.
    pub fn poll_reload(&mut self) {
        if let Ok(meta) = std::fs::metadata(&self.file_path) {
            if let Ok(modified) = meta.modified() {
                if Some(modified) != self.last_modified {
                    let reloaded = Self::load(&self.file_path);
                    self.definitions = reloaded.definitions;
                    self.load_error = reloaded.load_error;
                    self.last_modified = reloaded.last_modified;
                }
            }
        }
    }

    /// Find a definition by id.
    pub fn find(&self, id: &str) -> Option<&CommandDefinition> {
        self.definitions.iter().find(|d| d.id == id)
    }

    /// Add a definition after validating it. Rejects a duplicate id.
    ///
    /// Validates: command-configurator Requirement 2.3, 2.4, 4.1.
    pub fn add<F>(&mut self, def: CommandDefinition, is_reserved: F) -> Result<(), String>
    where
        F: Fn(&str) -> bool,
    {
        validate_definition(&def, is_reserved).map_err(|e| e.to_string())?;
        if self.find(&def.id).is_some() {
            return Err(format!("command id '{}' already exists", def.id));
        }
        self.definitions.push(def);
        Ok(())
    }

    /// Remove a definition by id. Returns true if one was removed.
    ///
    /// Validates: command-configurator Requirement 2.5.
    pub fn remove(&mut self, id: &str) -> bool {
        let before = self.definitions.len();
        self.definitions.retain(|d| d.id != id);
        self.definitions.len() != before
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use ff_command::{ExternalMode, TargetParams};
    use std::io::Write;
    use tempfile::TempDir;

    fn func_def(id: &str) -> CommandDefinition {
        CommandDefinition {
            id: id.to_string(),
            label: format!("Label for {id}"),
            description: None,
            category: "user".to_string(),
            target: CommandTarget::Function {
                command_id: "file.save".to_string(),
                params: TargetParams::new(),
            },
        }
    }

    fn write_store(dir: &TempDir, body: &str) -> PathBuf {
        let p = dir.path().join("commands.toml");
        let mut f = std::fs::File::create(&p).expect("create");
        f.write_all(body.as_bytes()).expect("write");
        p
    }

    // Validates: Requirement 1.5 -- absent file yields empty store, no error.
    #[test]
    fn absent_file_yields_empty_store() {
        let dir = TempDir::new().unwrap();
        let store = CommandStore::load(dir.path().join("commands.toml"));
        assert!(store.definitions.is_empty());
        assert!(store.load_error.is_none());
    }

    // Validates: Requirement 1.1, 1.2 -- save then load round-trips.
    #[test]
    fn save_then_load_round_trips_a_definition() {
        let dir = TempDir::new().unwrap();
        let path = dir.path().join("commands").join("commands.toml");
        let mut store = CommandStore::new(&path);
        store.definitions.push(func_def("build.release"));
        store.save().expect("save");

        let loaded = CommandStore::load(&path);
        assert_eq!(loaded.definitions.len(), 1);
        assert_eq!(loaded.definitions[0].id, "build.release");
        assert!(loaded.load_error.is_none());
    }

    // Validates: Requirement 1.5 -- save creates the commands/ directory.
    #[test]
    fn save_creates_parent_directory() {
        let dir = TempDir::new().unwrap();
        let path = dir.path().join("commands").join("commands.toml");
        let mut store = CommandStore::new(&path);
        store.definitions.push(func_def("a.b"));
        store.save().expect("save");
        assert!(path.exists());
    }

    // Validates: Requirement 1.4 -- duplicate id keeps first, records skip.
    #[test]
    fn duplicate_id_keeps_first_and_records_skip() {
        let dir = TempDir::new().unwrap();
        let body = r#"
[[command]]
id = "dup"
label = "First"
[command.target]
kind = "function"
command_id = "file.save"

[[command]]
id = "dup"
label = "Second"
[command.target]
kind = "function"
command_id = "edit.undo"
"#;
        let path = write_store(&dir, body);
        let store = CommandStore::load(&path);
        assert_eq!(store.definitions.len(), 1);
        assert_eq!(store.definitions[0].label, "First");
        assert!(store.load_error.as_ref().unwrap().contains("duplicate"));
    }

    // Validates: Requirement 1.6 -- invalid entry skipped, valid retained.
    #[test]
    fn invalid_entry_skipped_valid_retained() {
        let dir = TempDir::new().unwrap();
        let body = r#"
[[command]]
id = "good"
label = "Good"
[command.target]
kind = "function"
command_id = "file.save"

[[command]]
id = "bad"
# missing label and target -> entry fails to deserialise
"#;
        let path = write_store(&dir, body);
        let store = CommandStore::load(&path);
        assert_eq!(store.definitions.len(), 1);
        assert_eq!(store.definitions[0].id, "good");
        assert!(store.load_error.is_some());
    }

    // Validates: Requirement 1.6 -- malformed TOML surfaces a load error.
    #[test]
    fn malformed_toml_surfaces_load_error() {
        let dir = TempDir::new().unwrap();
        let path = write_store(&dir, "this is [ not valid toml");
        let store = CommandStore::load(&path);
        assert!(store.definitions.is_empty());
        assert!(store.load_error.is_some());
    }

    // Validates: Requirement 1.1 -- an External definition round-trips with mode.
    #[test]
    fn external_definition_round_trips() {
        let dir = TempDir::new().unwrap();
        let path = dir.path().join("commands.toml");
        let mut store = CommandStore::new(&path);
        store.definitions.push(CommandDefinition {
            id: "build.release".to_string(),
            label: "Build Release".to_string(),
            description: Some("build it".to_string()),
            category: "build".to_string(),
            target: CommandTarget::External {
                program: "pwsh".to_string(),
                args: vec!["-File".to_string(), "build.ps1".to_string()],
                working_dir: Some("${workspace_root}".to_string()),
                mode: ExternalMode::Captured,
            },
        });
        store.save().expect("save");
        let loaded = CommandStore::load(&path);
        assert_eq!(loaded.definitions.len(), 1);
        assert_eq!(loaded.definitions[0].target, store.definitions[0].target);
    }

    // Validates: Requirement 1.7 -- poll_reload picks up an on-disk change.
    #[test]
    fn poll_reload_picks_up_disk_change() {
        let dir = TempDir::new().unwrap();
        let path = dir.path().join("commands.toml");
        // Start empty (file absent).
        let mut store = CommandStore::load(&path);
        assert!(store.definitions.is_empty());

        // Write a definition to disk out of band, with a distinct mtime.
        let mut disk = CommandStore::new(&path);
        disk.definitions.push(func_def("added.later"));
        disk.save().expect("save");
        // Force the mtime to differ from the (None) baseline is unnecessary --
        // last_modified is None so any present file triggers reload.

        store.poll_reload();
        assert_eq!(store.definitions.len(), 1);
        assert_eq!(store.definitions[0].id, "added.later");
    }

    // Validates: Requirement 4.1 -- empty id / label rejected.
    #[test]
    fn validate_rejects_empty_id_and_label() {
        let mut d = func_def("");
        assert_eq!(
            validate_definition(&d, |_| false),
            Err(ValidationError::EmptyId)
        );
        d.id = "ok.id".to_string();
        d.label = "   ".to_string();
        assert_eq!(
            validate_definition(&d, |_| false),
            Err(ValidationError::EmptyLabel)
        );
    }

    // Validates: Requirement 4.1 -- invalid id rejected.
    #[test]
    fn validate_rejects_invalid_id() {
        let d = func_def("Not Valid Id");
        assert!(matches!(
            validate_definition(&d, |_| false),
            Err(ValidationError::InvalidId(_))
        ));
    }

    // Validates: Requirement 4.2 -- external with empty program rejected.
    #[test]
    fn validate_rejects_empty_external_program() {
        let d = CommandDefinition {
            id: "run.thing".to_string(),
            label: "Run".to_string(),
            description: None,
            category: "user".to_string(),
            target: CommandTarget::External {
                program: "  ".to_string(),
                args: vec![],
                working_dir: None,
                mode: ExternalMode::Detached,
            },
        };
        assert_eq!(
            validate_definition(&d, |_| false),
            Err(ValidationError::EmptyProgram)
        );
    }

    // Validates: Requirement 4.6 -- reserved id rejected.
    #[test]
    fn validate_rejects_reserved_id() {
        let d = func_def("file.save");
        assert_eq!(
            validate_definition(&d, |id| id == "file.save"),
            Err(ValidationError::ReservedId("file.save".to_string()))
        );
    }

    // Validates: Requirement 2.3, 2.4 -- add validates and rejects duplicates.
    #[test]
    fn add_validates_and_rejects_duplicate() {
        let dir = TempDir::new().unwrap();
        let mut store = CommandStore::new(dir.path().join("commands.toml"));
        store.add(func_def("a.b"), |_| false).expect("add ok");
        assert_eq!(store.definitions.len(), 1);
        assert!(store.add(func_def("a.b"), |_| false).is_err());
        assert!(store.add(func_def(""), |_| false).is_err());
    }

    // Validates: Requirement 2.5 -- remove deletes by id.
    #[test]
    fn remove_deletes_by_id() {
        let dir = TempDir::new().unwrap();
        let mut store = CommandStore::new(dir.path().join("commands.toml"));
        store.add(func_def("a.b"), |_| false).unwrap();
        assert!(store.remove("a.b"));
        assert!(store.definitions.is_empty());
        assert!(!store.remove("missing"));
    }
}
