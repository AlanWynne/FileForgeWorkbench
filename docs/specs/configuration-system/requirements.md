# Requirements Document

## Introduction

This feature specifies the configuration system for FileForgeWorkbench (`ff-config` crate). The configuration system is the **central settings management layer** for the entire workbench platform. It provides TOML-based configuration files, a layered override model with well-defined precedence, hot-reload without application restart, named user profiles, per-project overrides, EditorConfig integration, a typed access API with compile-time key definitions, plugin namespace scoping, and a runtime-queryable schema with validation.

The configuration system implements cross-cutting Requirement 5 from the project-master spec and is consumed by virtually every subsystem in the workbench: `ff-logging` reads log level settings, `ff-core` (platform-core) reads startup parameters, `ff-command` reads shortcut bindings, `ff-theme` reads appearance settings, `ff-plugin` reads plugin-specific namespaces, and `ff-vfs` reads provider settings. Plugins access configuration through their `PluginContext` using scoped, namespaced reads and writes.

The `ff-config` crate is a Wave 2 (Platform Architecture) component. It depends on `ff-logging` for diagnostics and is consumed by all higher-level crates including the GUI shell.

**Source references:**
- **WB** = Workbench Architecture Brief §8, Principle 3 (Configuration as Data)
- **FFE** = FileForgeEditor `startup-and-session` (config loading sequence, adapted)
- **FFW** = FileForgeWorkbench cross-cutting Requirement 5 (configuration namespace)

## Glossary

- **Configuration_System**: The `ff-config` crate responsible for loading, merging, validating, watching, and serving configuration values to all platform subsystems and plugins. [WB, FFW]
- **Configuration_File**: A TOML-format file containing settings organized into tables and key-value pairs. [WB]
- **Configuration_Layer**: A level in the layered override model. Each layer has a fixed priority. Higher-priority layers override lower-priority layers on a key-by-key basis. [WB, FFW]
- **Effective_Value**: The final resolved value for a configuration key after all layers have been merged. The effective value is the value from the highest-priority layer that defines the key. [FFW]
- **Provenance**: Metadata attached to an effective value indicating which Configuration_Layer provided it. [FFW]
- **Configuration_Schema**: A structured definition of all known configuration keys, including their type, default value, constraints, and description. [WB]
- **Hot_Reload**: The capability to detect changes to configuration files on disk and apply updated settings without restarting the application. [WB]
- **Reload_Callback**: A function registered by a subsystem to be invoked when configuration keys it depends on change during hot-reload. [WB]
- **User_Profile**: A named collection of settings that, when active, overlays the user layer with profile-specific overrides. [WB]
- **Plugin_Namespace**: A TOML table scoped to a specific plugin (e.g., `[plugins.my-plugin]`), isolating plugin settings from each other and from core settings. [WB, FFW]
- **EditorConfig**: An external per-file configuration standard (editorconfig.org) that specifies indent style, line endings, and other formatting properties per file path pattern. [SCI]
- **Settings_Key**: A dot-separated identifier for a configuration value (e.g., `"editor.tab_size"`, `"logging.level"`, `"theme.active"`). [WB]
- **Layer_Precedence**: The fixed ordering of configuration layers from lowest to highest priority: Defaults → System → User → Profile → Project → Workspace. [FFW]

## Requirements

### Requirement 1: TOML-Based Configuration Format

**User Story:** As a workbench user, I want all configuration stored in human-readable TOML files with a well-defined schema, so that I can view and edit settings with any text editor and understand the structure without special tooling.

**Source:** WB Architecture Brief §8, Principle 3. [WB, FFW]

#### Acceptance Criteria

1. THE Configuration_System SHALL use TOML as the primary configuration file format; all configuration files SHALL be valid TOML documents conforming to TOML v1.0 syntax.
2. THE Configuration_System SHALL store configuration files in well-defined locations per layer: system-wide (`/etc/ffworkbench/config.toml` on Linux, platform equivalent on Windows/macOS), user-level (`~/.config/ffworkbench/config.toml`), project-level (`.ffworkbench/config.toml` in the project root), and workspace-level (`config.toml` in the workspace root).
3. ALL settings SHALL be organized into TOML tables using a namespace hierarchy (e.g., `[logging]`, `[editor]`, `[theme]`, `[plugins]`, `[vfs]`), with dot-separated key paths within tables for sub-settings.
4. ALL configuration keys SHALL have typed schema definitions specifying the expected TOML value type (string, integer, float, boolean, array, table).
5. LANGUAGE profile configuration SHALL be stored in separate TOML files within a `languages/` subdirectory (e.g., `languages/rust.toml`, `languages/cobol.toml`), not in the main configuration file.
6. WHEN a configuration file contains syntax errors (invalid TOML), THE Configuration_System SHALL reject the entire file, retain all previously loaded values for that layer, emit a WARN-level log record identifying the file path and parse error location, and continue operating.

---

### Requirement 2: Layered Configuration Model

**User Story:** As a workbench user, I want configuration to cascade from system-wide defaults down through user preferences and project-specific overrides, so that I can customize behaviour at the appropriate scope without duplicating settings.

**Source:** WB Architecture Brief §8, FFW cross-cutting Requirement 5. [WB, FFW]

#### Acceptance Criteria

1. THE Configuration_System SHALL support the following layers in ascending priority order: Defaults (hardcoded in crate code) → System (global platform-wide file) → User (per-user file) → Profile (active named profile overlay) → Project (`.ffworkbench/config.toml` in project root) → Workspace (workspace root config).
2. WHEN multiple layers define the same configuration key, THE Configuration_System SHALL resolve the effective value by selecting the value from the highest-priority layer that defines the key (key-by-key merge, not file-level replacement).
3. THE query API SHALL return the effective value for any given Settings_Key along with its Provenance -- indicating which Configuration_Layer provided the value.
4. THE layer precedence SHALL be fixed as defined in criterion 1 and SHALL NOT be user-configurable or plugin-modifiable.
5. WHEN a configuration key is not defined in any layer, THE Configuration_System SHALL return the default value declared in the Configuration_Schema for that key.
6. IF a configuration key has no schema-defined default and is not defined in any layer, THEN THE Configuration_System SHALL return an error indicating the key is undefined.
7. THE Configuration_System SHALL merge table values recursively: if two layers both define a TOML table (e.g., `[editor]`), their keys are merged rather than the higher-priority table replacing the lower-priority table entirely.

---

### Requirement 3: Hot-Reload

**User Story:** As a workbench user, I want configuration changes I make to take effect immediately without restarting the application, so that I can iterate on settings quickly during a work session.

**Source:** WB Architecture Brief §8, FFW cross-cutting Requirement 5.4. [WB, FFW]

#### Acceptance Criteria

1. THE Configuration_System SHALL monitor all loaded configuration files for changes using an OS-native file watcher (e.g., inotify on Linux, ReadDirectoryChangesW on Windows, FSEvents on macOS).
2. WHEN a monitored configuration file is modified on disk, THE Configuration_System SHALL detect the change and re-read the file within 2 seconds of the modification event.
3. WHEN a configuration file is successfully re-read, THE Configuration_System SHALL re-merge all layers, recompute affected effective values, and notify registered Reload_Callbacks for all keys whose effective value changed.
4. SUBSYSTEMS SHALL register Reload_Callbacks for the configuration keys they depend on, and THE Configuration_System SHALL invoke these callbacks with the new effective values when those keys change.
5. THE Configuration_System SHALL apply all changes from a single file atomically: either all changed values from the re-read file are applied together, or none are applied (no partial state from a single file reload).
6. WHEN a re-read configuration file contains invalid TOML or fails schema validation, THE Configuration_System SHALL reject the reload, retain all previous effective values, emit a WARN-level log record identifying the file and the validation error, and NOT invoke any Reload_Callbacks.
7. THE Configuration_System SHALL debounce rapid successive file change events for the same file, coalescing multiple events within a 500-millisecond window into a single reload operation.

---

### Requirement 4: User Profiles

**User Story:** As a workbench user, I want to define multiple named configuration profiles (e.g., "mainframe", "web-dev", "database") and switch between them without restarting, so that I can quickly adapt the workbench to different working contexts.

**Source:** WB Architecture Brief §8. [WB]

#### Acceptance Criteria

1. THE Configuration_System SHALL support multiple named user profiles, where each profile is a named collection of settings stored in a separate TOML file (e.g., `~/.config/ffworkbench/profiles/mainframe.toml`).
2. WHEN a profile is active, THE Configuration_System SHALL treat the profile's settings as a layer with priority between the User layer and the Project layer (higher than User, lower than Project).
3. THE Configuration_System SHALL allow switching the active profile at runtime without application restart; WHEN the active profile changes, THE Configuration_System SHALL recompute all affected effective values and invoke Reload_Callbacks for changed keys.
4. AT MOST one profile SHALL be active at any time; activating a new profile SHALL automatically deactivate the previously active profile.
5. THE Configuration_System SHALL persist the currently active profile selection in the user-layer session state, so that the same profile is automatically activated on next application startup.
6. WHEN the active profile is set to a profile name that does not exist (file missing or unreadable), THE Configuration_System SHALL emit a WARN-level log record, deactivate the profile (falling back to no active profile), and continue operating with the remaining layers.
7. THE Configuration_System SHALL provide an API to list all available profiles by scanning the profiles directory.

---

### Requirement 5: Per-Project Overrides

**User Story:** As a team member working on a shared project, I want project-specific configuration stored in the project repository, so that all team members get consistent settings (indent style, line endings, language-specific options) without manual setup.

**Source:** WB Architecture Brief §8. [WB, FFW]

#### Acceptance Criteria

1. THE Configuration_System SHALL recognize a `.ffworkbench/config.toml` file in the project root directory as the project-layer configuration source.
2. WHEN a project is opened, THE Configuration_System SHALL automatically detect and load the project-layer configuration file if it exists, merging its settings into the layered model at the Project priority level.
3. THE project-layer settings SHALL override user-layer and profile-layer settings for all keys defined in the project configuration file (higher priority than User and Profile layers, lower priority than Workspace layer).
4. THE project-layer configuration file SHALL be suitable for version control: it SHALL contain only project-relevant settings (no user-specific paths, credentials, or machine-specific values).
5. WHEN the project-layer configuration file is modified on disk while the project is open, THE Configuration_System SHALL hot-reload the changes per the hot-reload requirements (Requirement 3).
6. WHEN a project is closed, THE Configuration_System SHALL unload the project-layer configuration and recompute effective values using the remaining layers, invoking Reload_Callbacks for any keys whose effective value changed.
7. IF the project-layer configuration file cannot be read (permission error, invalid TOML), THEN THE Configuration_System SHALL emit a WARN-level log record, skip the project layer for that project, and continue operating.

---

### Requirement 6: EditorConfig Support

**User Story:** As a developer working on projects that use EditorConfig, I want the workbench to respect `.editorconfig` files, so that my editor automatically applies the correct indent style, line endings, and whitespace settings for each file without manual configuration.

**Source:** EditorConfig specification (editorconfig.org). Cross-references `edit-operations`, `document-model`. [SCI]

#### Acceptance Criteria

1. THE Configuration_System SHALL read `.editorconfig` files conforming to the EditorConfig specification (editorconfig.org), parsing all standard properties.
2. THE Configuration_System SHALL apply the following EditorConfig properties when resolving editor settings for a specific file: `indent_style`, `indent_size`, `tab_width`, `end_of_line`, `charset`, `trim_trailing_whitespace`, and `insert_final_newline`.
3. WHEN resolving settings for a specific file, EditorConfig properties SHALL override the corresponding configuration-system settings with higher priority than all configuration layers (EditorConfig is the highest-priority source for the properties it defines, scoped to the specific file).
4. THE Configuration_System SHALL resolve EditorConfig properties using standard path traversal: starting from the file's directory, walk up parent directories reading `.editorconfig` files until a file with `root = true` is found or the filesystem root is reached.
5. WHEN multiple `.editorconfig` files apply to the same file path, THE Configuration_System SHALL merge their properties with closer (deeper) files taking priority over farther (shallower) files, per the EditorConfig specification.
6. IF an `.editorconfig` file contains syntax errors, THE Configuration_System SHALL skip that file, emit a WARN-level log record identifying the file and parse error, and continue resolution using remaining `.editorconfig` files in the path hierarchy.
7. EditorConfig resolution SHALL only apply to file-specific editor settings (indent, line endings, whitespace); it SHALL NOT override non-editor configuration keys (logging, theme, plugin settings, etc.).

---

### Requirement 7: Typed Access API

**User Story:** As a workbench developer, I want type-safe access to configuration values with compile-time key validation where possible, so that I catch typos and type mismatches at compile time rather than runtime, and get sensible defaults for missing or invalid values.

**Source:** WB Architecture Brief §8. [WB, FFW]

#### Acceptance Criteria

1. THE Configuration_System SHALL provide typed getter methods for all supported TOML value types: `get_string(key) → Result<String>`, `get_int(key) → Result<i64>`, `get_float(key) → Result<f64>`, `get_bool(key) → Result<bool>`, `get_array(key) → Result<Vec<ConfigValue>>`, and `get_table(key) → Result<ConfigTable>`.
2. THE Configuration_System SHALL support compile-time key definitions using Rust `const` values (e.g., `const KEY_EDITOR_TAB_SIZE: &str = "editor.tab_size";`), allowing keys to be validated at compile time where the consumer uses the const definition rather than a string literal.
3. EACH configuration key in the schema SHALL have a declared default value of the appropriate type, which THE Configuration_System SHALL return when the key is not defined in any active layer.
4. EACH configuration key in the schema SHALL optionally have validation rules including: minimum value, maximum value, allowed enum values (set of valid strings or integers), and regex pattern (for string values).
5. WHEN a typed getter is called and the stored value's type does not match the requested type (e.g., `get_int` called on a string value), THE Configuration_System SHALL return the schema-defined default value for that key and emit a WARN-level log record indicating the type mismatch, the key, the expected type, and the actual type found.
6. WHEN a typed getter is called and the stored value fails validation rules (out of range, not in enum set, fails regex), THE Configuration_System SHALL return the schema-defined default value for that key and emit a WARN-level log record indicating the validation failure.
7. THE Configuration_System SHALL provide a generic getter `get(key) → Result<ConfigValue>` that returns the effective value without type coercion, for cases where the consumer handles multiple types dynamically.

---

### Requirement 8: Plugin Configuration Scoping

**User Story:** As a plugin developer, I want my plugin's configuration to be isolated in its own namespace, so that my settings cannot conflict with other plugins or core settings, and I can only read/write within my own scope.

**Source:** WB Architecture Brief §10, FFW cross-cutting Requirement 5.6. [WB, FFW]

#### Acceptance Criteria

1. EACH plugin SHALL have its configuration scoped under a dedicated TOML table: `[plugins.{plugin-name}]`, where `{plugin-name}` is the plugin's registered identifier (lowercase ASCII, hyphens, and digits only).
2. THE Configuration_System SHALL provide a scoped configuration handle to each plugin via `PluginContext`, restricting read and write access to the plugin's own namespace (`plugins.{plugin-name}.*`).
3. WHEN a plugin attempts to read or write a configuration key outside its namespace (e.g., a plugin named `"sql-viewer"` attempts to access `"editor.tab_size"` or `"plugins.other-plugin.setting"`), THE Configuration_System SHALL reject the operation and return an error indicating a namespace violation.
4. PLUGINS SHALL declare their default configuration values in their plugin manifest (metadata), and THE Configuration_System SHALL register these as the Defaults layer for the plugin's namespace during plugin initialization.
5. PLUGIN configuration keys SHALL participate in hot-reload: WHEN a configuration file is modified and contains changes to a plugin's namespace, THE Configuration_System SHALL invoke the plugin's registered Reload_Callbacks with the updated values.
6. WHEN a plugin is unloaded (shutdown lifecycle phase), THE Configuration_System SHALL deregister the plugin's Reload_Callbacks and remove the plugin's schema entries; previously persisted configuration values for the plugin SHALL be retained in configuration files but not actively served.
7. THE Configuration_System SHALL prevent plugins from registering configuration keys that collide with core settings namespaces (`logging`, `editor`, `theme`, `vfs`, `commands`, `layout`).

---

### Requirement 9: Configuration Schema and Validation

**User Story:** As a workbench developer, I want a formal schema for all configuration keys, so that the system can validate values at load time, generate settings UI automatically, and provide meaningful error messages for invalid configuration.

**Source:** WB Architecture Brief §8. [WB]

#### Acceptance Criteria

1. THE Configuration_System SHALL maintain a Configuration_Schema containing a definition for every known configuration key, registered either by core subsystems at startup or by plugins during their initialization phase.
2. EACH schema entry SHALL define: the key path (dot-separated string), the value type (string, integer, float, boolean, array, table), a default value, and a human-readable description of the setting's purpose.
3. EACH schema entry SHALL optionally define constraint metadata: minimum value (for numeric types), maximum value (for numeric types), allowed enum values (for string or integer types), and a regex pattern (for string types).
4. WHEN configuration is loaded (at startup or during hot-reload), THE Configuration_System SHALL validate every loaded value against its schema entry; IF a value violates its schema constraints, THEN THE Configuration_System SHALL discard the invalid value, apply the schema-defined default, and emit a WARN-level log record identifying the key, the invalid value, and the constraint that was violated.
5. THE Configuration_Schema SHALL be queryable at runtime: subsystems and the settings UI SHALL be able to enumerate all registered keys, retrieve their types, defaults, constraints, and descriptions without loading or modifying configuration values.
6. WHEN a configuration file contains a key that has no schema entry (unknown key), THE Configuration_System SHALL ignore the unknown key, emit a DEBUG-level log record indicating the unrecognized key, and continue loading -- unknown keys SHALL NOT cause errors or warnings at WARN level.
7. THE Configuration_Schema SHALL support registration of new keys at runtime (by plugins during initialization), allowing the schema to grow dynamically as plugins are loaded.

---

### Requirement 15: Config View -- Interactive Configuration-Key Browser

> **Terminology (CR-CH-025 rename).** This requirement governs the CONFIG VIEW:
> the flat configuration-key browser/editor opened by the `CONFIG` command
> (Config Context, `TabKind::ConfigPanel`, `WorkspaceKind::Config`, tab title
> `[CONFIG]`). It is NOT the SETTINGS MENU (the data-driven `Menu_Workspace`
> backed by `menus/settings.toml`, opened by the menu name `SETTINGS`). Throughout
> this spec, "Settings" refers to that MENU; "Config"/"CONFIG" refers to this
> key browser.

**User Story:** As a workbench user, I want a graphical Config View that lets me view and
change all configuration values without editing TOML files manually, so that I can adjust
workbench behaviour quickly and safely from within the application.

**Source:** [WB] Configuration as Data; [ISPF-POM] POM option 0.

*(REVISED by CR-CH-025. The earlier Phase CW design -- a two-level Settings_Menu that acted as a
NAMESPACE SELECTOR whose options each opened a filtered `Settings_Namespace_View` -- is
SUPERSEDED. The Settings_Menu is now a small LAUNCHER (options `A` -> `CONFIG`, `T` -> `THEME`,
`M` -> `MENUS`, `R` -> `RESET BARE`; menu-workspace Requirement 12.3), reached by resolving the
menu name `SETTINGS` through the unified command-resolution chain (command-framework Requirement
8.3) -- NOT a hardcoded verb. The flat configuration-key browser, and namespace filtering, now
live on the first-class `CONFIG [<namespace>]` command (Requirement 20). cw-requirements Req 9/10
[the namespace-selector Settings_Menu and the per-option Settings_Namespace_View] are superseded
accordingly; see `docs/specs/menu-workspace/cw-requirements.md`.)*

#### Acceptance Criteria

1. WHEN the user selects the Settings option from the Home Context (Primary Option Menu), OR the
     menu name `SETTINGS` resolves through the command-resolution chain (command-framework
     Requirement 8.3 stage 3; e.g. typing `SETTINGS` or `=0`), THE shell SHALL open the
     Settings_Menu -- the Menu_Workspace backed by `menus/settings.toml` (or the compiled Settings
     Recovery_Baseline when no user file exists). (REVISED by CR-CH-025: `SETTINGS` is a Menu_Name,
     not a bespoke verb; the Settings_Menu is a launcher, not a namespace selector.)

2. THE flat configuration-key view (the widget that lists keys with their values, provenance, and
     edit controls) SHALL be opened by the `CONFIG` command (Requirement 20): bare `CONFIG` shows
     ALL keys unfiltered; `CONFIG <namespace>` shows only keys under that namespace prefix. (REVISED
     by CR-CH-025: this replaces the former Settings_Namespace_View opened per namespace option and
     the opaque `A` command; the Settings_Menu `A` option dispatches `CONFIG`.)

3. FOR each configuration key, THE Config View SHALL display:
     - The key's human-readable description (from the schema entry)
     - The current effective value
     - The provenance layer that provided the effective value (e.g., `Default`, `User`, `Project`)
     - An appropriate input widget based on the value type:
       - Boolean -> checkbox
       - Integer / Float with min/max -> slider; without constraints -> numeric text field
       - String with `allowed_values` -> drop-down selector
       - String without constraints -> single-line text field

4. WHEN the user changes a value in the Config View and confirms (presses Enter or moves
     focus away from the field), THE shell SHALL validate the new value against the schema
     constraints; IF valid, THE shell SHALL write the new value to the user-layer configuration
     file and update the effective value immediately (no restart required).

5. WHEN a value fails schema validation (out of range, not in allowed set, fails regex),
     THE Config View SHALL display an inline error message adjacent to the field and SHALL
     NOT persist the invalid value.

6. THE Config View SHALL display a `Reset to Default` button beside each key that has
     been overridden above the Defaults layer; WHEN clicked, THE shell SHALL remove the
     user-layer override for that key, restoring the schema default.

7. THE Config View SHALL include a search/filter input at the top; WHEN the user types
     in the filter, THE Config View SHALL show only keys whose key path or description contains the
     filter text (case-insensitive substring match).

8. THE Config View SHALL display a read-only `Source File` indicator showing the path of
     the user-layer configuration file being edited.

9. THE `[CONFIG]` Workspace SHALL persist in the session and be restored on next launch as a
     `ConfigPanel` tab kind, with its namespace filter (if any) reapplied.
     *(CR-CH-012 / CR-CH-025: session persistence uses Workspace_Descriptors, not a closed "tab
     kind" enumeration. The Config View persists as a `CustomWorkspace { workspace_kind = config,
     params = { namespace } }` Workspace_Descriptor and is restored with that namespace filter
     applied -- see startup-and-session Requirement 21.3.)*

10. END/F3 navigation SHALL follow the per-tab Navigation_Stack (CR-CH-022): pressing `F3` or
      typing `END` pops one level. (REVISED by CR-CH-025: with the Settings_Namespace_View removed,
      the concrete effect is -- from a `CONFIG` flat view opened via the Settings_Menu `A` option,
      END returns to the Settings_Menu; from the Settings_Menu, END returns to the Home Context
      (POM). The stack, not a bespoke Settings return rule, governs this.)

11. WHEN the user clicks `Settings` in the POM option list (option 0 button), THE shell
      SHALL navigate to the Settings_Menu (the Menu_Workspace, per criterion 1) using the same
      routing as typing `0` in the command field. (The flat Config View is reached from there via
      the `A` -> `CONFIG` option, or directly by the `CONFIG` command.)

---

### Requirement 16: Configuration Audit Logging

**User Story:** As an enterprise administrator, I want every configuration change to be recorded
with a timestamp, the key changed, the old and new values, the actor, and the layer, so that I
can audit who changed what and when for compliance and troubleshooting purposes.

**Source:** Phase CQ enterprise features roadmap.

#### Acceptance Criteria

1. WHEN any configuration key's effective value changes (via set_user_value, profile switch,
   project load/unload, or hot-reload), THE Configuration_System SHALL append an AuditEntry to
   the audit log containing: ISO-8601 timestamp, key path, old effective value, new effective
   value, the ConfigLayer that caused the change, and an actor string (defaults to "user").
2. THE Configuration_System SHALL persist the audit log to a rolling file at
   `<user-config-dir>/audit.log` using a line-delimited TOML or JSON format, with a maximum
   of 10,000 entries before the oldest entries are discarded (ring-buffer semantics).
3. THE Configuration_System SHALL expose a `query_audit_log(filter: AuditFilter) -> Vec<AuditEntry>`
   API on ConfigHandle, supporting filtering by key prefix, layer, time range, and actor.
4. WHEN the audit log file cannot be written (permission error, disk full), THE Configuration_System
   SHALL emit a WARN-level log record and continue operating -- audit log write failures SHALL NOT
   prevent configuration changes from taking effect.
5. THE AuditEntry SHALL be a public type with fields: timestamp (SystemTime), key (String),
   old_value (Option<ConfigValue>), new_value (Option<ConfigValue>), layer (ConfigLayer),
   actor (String).
6. THE Configuration_System SHALL provide a `clear_audit_log()` method on ConfigHandle that
   truncates the in-memory and on-disk audit log.

---

### Requirement 17: Settings Export and Import

**User Story:** As a workbench user, I want to export my current configuration to a portable
TOML file and import a previously exported file to restore settings, so that I can back up my
preferences, share them with colleagues, or migrate to a new machine.

**Source:** Phase CQ enterprise features roadmap.

#### Acceptance Criteria

1. THE Configuration_System SHALL provide an `export_settings(scope: ExportScope, path: &Path)
   -> Result<(), ConfigError>` method on ConfigHandle that writes a TOML file containing all
   effective values for the specified scope.
2. THE ExportScope SHALL be an enum with variants: `AllLayers` (all effective values),
   `UserLayer` (only user-layer overrides), `ProjectLayer` (only project-layer overrides).
3. THE exported TOML file SHALL include a `[_export_meta]` header table containing: export
   timestamp, FFWB version string, and the ExportScope used.
4. THE Configuration_System SHALL provide an `import_settings(path: &Path, target_layer:
   ImportTarget) -> Result<ImportSummary, ConfigError>` method on ConfigHandle that reads an
   exported TOML file and merges its values into the specified target layer.
5. THE ImportTarget SHALL be an enum with variants: `UserLayer`, `ProjectLayer`.
6. WHEN importing, THE Configuration_System SHALL validate each imported value against the
   schema; invalid values SHALL be skipped and reported in the ImportSummary rather than
   causing the entire import to fail.
7. THE ImportSummary SHALL be a public struct with fields: imported_count (usize),
   skipped_count (usize), skipped_keys (Vec<String>).
8. WHEN the import file cannot be read or contains invalid TOML, THE Configuration_System
   SHALL return ConfigError::Io or ConfigError::ParseError respectively and make no changes.
9. AFTER a successful import, THE Configuration_System SHALL trigger a hot-reload cycle so
   that all registered callbacks are notified of changed keys.

---

### Requirement 18: Locked Configuration Keys

**User Story:** As an enterprise administrator, I want to mark specific configuration keys as
locked in the system layer so that users, profiles, and projects cannot override them, ensuring
consistent policy enforcement across all workbench instances.

**Source:** Phase CQ enterprise features roadmap.

#### Acceptance Criteria

1. THE system-layer configuration file SHALL support a `[_locked]` table containing a list of
   key paths that are locked: `locked_keys = ["editor.tab_size", "logging.level"]`.
2. WHEN a key is listed in `[_locked].locked_keys`, THE Configuration_System SHALL treat the
   system-layer value for that key as the effective value regardless of any higher-priority
   layer definitions -- locked keys are immune to override by User, Profile, Project, and
   Workspace layers.
3. WHEN `set_user_value()` is called for a locked key, THE Configuration_System SHALL return
   `ConfigError::KeyLocked { key }` and make no change.
4. WHEN a configuration file is loaded and contains a value for a locked key at a layer above
   System, THE Configuration_System SHALL silently ignore that value (the system-layer value
   wins) and emit a DEBUG-level log record identifying the key and the layer that attempted
   to override it.
5. THE Configuration_System SHALL expose an `is_locked(key: &str) -> bool` method on
   ConfigHandle so that the Config View and other consumers can check lock status before
   attempting writes.
6. THE Config View SHALL display a lock indicator (padlock icon or "LOCKED" badge) beside
   any key that is locked, and SHALL disable the value widget and Reset to Default button for
   locked keys.
7. THE ConfigError enum SHALL gain a `KeyLocked { key: String }` variant with message:
   `"[config] lock: key '{key}' is locked by system policy and cannot be modified"`.
8. WHEN the system-layer configuration file is hot-reloaded and the locked_keys list changes,
   THE Configuration_System SHALL recompute effective values for all newly locked or unlocked
   keys and invoke reload callbacks for any keys whose effective value changed as a result.

---

### Requirement 19: Reset to Barebones (RESET BARE)

**User Story:** As an operator whose configuration has become broken, cluttered,
or unrecoverable, I want a single command to archive my current configuration and
restart the workbench in a clean barebones state, so that I can rebuild from a
known-good baseline while keeping my old configuration safe for recovery.

**Source:** [CR-CH-021]

#### Acceptance Criteria

1. THE shell SHALL provide a `RESET BARE` command, registered with the command
   framework (Command_ID `"config.reset_bare"`) so it is dispatchable from the
   command line, menu options, keyboard bindings, and macros.
2. WHEN `RESET BARE` is invoked, THE shell SHALL FIRST present a modal
   confirmation dialog describing the action (archive current configuration and
   reopen in barebones mode) with explicit Confirm and Cancel actions. No files
   SHALL be moved or changed until the user confirms.
3. WHEN the user cancels the confirmation, THE shell SHALL take no action and
   leave all configuration and the current Workspaces unchanged.
4. WHEN the user confirms, THE shell SHALL ARCHIVE (move, not delete) the current
   configuration into `<User_Data_Dir>/config-archive/<timestamp>/`, where
   `<timestamp>` is a filesystem-safe UTC timestamp. The archive SHALL include,
   when present: the `menus/` directory, the `themes/` directory, the `keymaps/`
   directory (per-context key-map override files, function-keys-and-history
   Requirement 14, CR-CH-027), `session.toml`, the active `config.toml` (user
   layer), and the catalog registry file. Files that do not exist are skipped
   without error.
5. THE archive operation SHALL be non-destructive: after a successful `RESET
   BARE`, every archived file SHALL exist under `config-archive/<timestamp>/` and
   SHALL NOT remain at its original location. WHEN a move fails for any single
   item, THE shell SHALL report the failure in a non-blocking notice and continue
   with the remaining items (best-effort), leaving already-archived items in the
   archive.
6. AFTER a confirmed `RESET BARE`, THE workbench SHALL reset its in-memory
   configuration, menus, theme, and catalog state to the compiled baselines and
   reopen the Home Context showing the Recovery_Baseline POM (menu-workspace
   Requirement 12), WITHOUT requiring the user to relaunch the process. The
   default Home catalog SHALL be re-created (CR-NR-004 behaviour) so Files is
   immediately usable.
7. THE Settings_Menu SHALL expose an affordance (the `R` -> `RESET BARE` menu
   option) that dispatches the `RESET BARE` command through the same command
   path (command parity); the affordance SHALL NOT bypass the
   command or its confirmation dialog.
8. THE archived configuration SHALL remain readable so the operator can manually
   restore files from `config-archive/<timestamp>/` back into `<User_Data_Dir>/`;
   `RESET BARE` SHALL NOT delete or prune previous archive directories.
---

### Requirement 20: The CONFIG Command

**User Story:** As a workbench user, I want a single `CONFIG` command that opens the
configuration-key browser -- optionally scoped to one namespace -- so that I can view and edit
settings without the Settings menu being a special case and without an opaque one-letter command.

**Source:** [CR-CH-025]; owner request ("i like the config command idea to do the 'Settings A'
that we currently have"). Supersedes cw-requirements Req 9/10 (the Settings_Menu namespace
selector and per-option Settings_Namespace_View).

#### Acceptance Criteria

1. THE shell SHALL provide a `CONFIG` command, registered with the command framework
   (Command_ID `"config.open"`) so it is dispatchable from the command line, menu options,
   keyboard bindings, and macros, and resolvable through the unified command-resolution chain as a
   built-in command (command-framework Requirement 8.3 stage 2).
2. WHEN the user types bare `CONFIG` (no argument) in any `Command ===>` field, THE shell SHALL
   open the Config View (flat configuration-key browser) showing ALL keys, unfiltered (the
   behaviour previously reached by the opaque `A` command). It transforms the
   current tab in place and pushes the per-tab Navigation_Stack (CR-CH-022) so END returns to the
   previous Context.
3. WHEN the user types `CONFIG <namespace>` (e.g. `CONFIG editor`), THE shell SHALL open the flat
   configuration-key view with the filter pre-populated to that namespace prefix (e.g. `editor.`),
   so only keys under that namespace are shown on open. The namespace token SHALL be matched
   case-insensitively.
4. WHEN `<namespace>` does not match any configuration key prefix, THE shell SHALL still open the
   flat view (with the filter applied, showing no rows) rather than erroring, so the user can clear
   or edit the filter -- consistent with Requirement 15.7 (the filter is user-editable in place).
5. THE `CONFIG` view SHALL be the SAME flat configuration-key widget defined by Requirement 15
   (key, description, effective value, provenance, type-appropriate input, Reset-to-Default,
   validation, Source File indicator, search/filter). `CONFIG` only chooses the initial filter; it
   does not introduce a second settings surface.
6. THE Settings_Menu `A` option (menu-workspace Requirement 12.3) SHALL carry `command = "CONFIG"`,
   so selecting `A` on the Settings menu dispatches `CONFIG` through the same code path as typing
   it (command parity). There SHALL be NO remaining `A`-named command and NO hardcoded
   `SETTINGS <namespace>` intercept; namespace filtering is reached only via `CONFIG <namespace>`.
7. THE Config View SHALL persist in the session and be restored on next launch as a
   `CustomWorkspace { workspace_kind = config, params = { namespace } }` Workspace_Descriptor
   (startup-and-session Requirement 21.3), with the namespace filter (if any) reapplied -- the same
   persistence contract Requirement 15.9 defines for the Config Workspace.
