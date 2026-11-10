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

<!-- CR-NR-083: targeted RESET BARE. The existing criteria 19.1-19.8 describe
     bare `RESET BARE`. The criteria below add the `<profilename> [<profilename>
     ...]` (one or more named profiles) and `ALL` targets. All forms resolve to a
     LIST of target profiles and reuse the SAME single confirmation dialog
     (criterion 19.2), which now names the profile(s) to be reset. -->

9. WHEN bare `RESET BARE` (no argument) is invoked, THE shell SHALL target ONLY
   the Application_Profile the running process is under (startup-and-session
   Requirement 22): the Active_Profile when one is active, else the
   DEFAULT_PROFILE. It SHALL archive and reset ONLY that profile's User_Data_Dir
   and SHALL leave every other profile's User_Data_Dir untouched. (This restates
   the existing single-profile behaviour so the argument forms below are
   unambiguous.)
10. THE shell SHALL accept `RESET BARE <profilename> [<profilename> ...]`, a
    whitespace-separated list of one or more Application_Profile names to reset.
    Each name's User_Data_Dir is resolved as
    `<config-base>/ffworkbench/profiles/<slug>/`. Profile-name matching SHALL use
    the same slug rule as startup-and-session Requirement 22.3 (lowercase;
    non-alphanumeric replaced by `-`), so `RESET BARE ISPF` and `RESET BARE ispf`
    target the same profile. Duplicate names in the list (after slugging) SHALL
    be collapsed to a single target.
11. WHEN a `RESET BARE <profilename> ...` list is confirmed, THE shell SHALL, for
    EACH named profile, archive (move, not delete) that profile's configuration
    into that profile's own `<User_Data_Dir>/config-archive/<timestamp>/`
    (criteria 19.4, 19.5, 19.8 apply per-profile; best-effort -- a failure on one
    profile does not abort the others) and SHALL leave every profile NOT in the
    list untouched. WHEN the list INCLUDES the Active_Profile, THE shell SHALL
    additionally reset the running in-memory state to the compiled baselines and
    reopen the Recovery_Baseline POM (criterion 19.6); otherwise the running
    process's in-memory state and open Workspaces SHALL be left unchanged (only
    the targeted profiles' on-disk configuration is archived).
12. WHEN a `RESET BARE <profilename> ...` list contains ANY name that does not
    exist (no `profiles/<slug>/` directory and it is not the DEFAULT_PROFILE),
    THE shell SHALL take no action, archive nothing, and report a non-blocking
    error naming the unknown profile(s). The confirmation dialog SHALL NOT be
    shown when any named profile is unknown (all-or-nothing: the operator fixes
    the name and retries).
13. THE shell SHALL accept `RESET BARE ALL`, which targets EVERY Application_
    Profile: the DEFAULT_PROFILE (`<config-base>/ffworkbench/`) AND every
    `profiles/<slug>/` directory discovered under the config base. WHEN
    confirmed, THE shell SHALL archive each targeted profile into that profile's
    own `config-archive/<timestamp>/` (best-effort per criterion 19.5; a failure
    on one profile does not abort the others) and THEN reset the running
    in-memory state to the compiled baselines and reopen the Recovery_Baseline
    POM (criterion 19.6), because the Active_Profile is always within the ALL
    set.
14. `ALL` SHALL be a reserved, case-insensitive keyword for criterion 19.13:
    `RESET BARE ALL`, `RESET BARE all`, and `RESET BARE All` all mean "every
    profile". A profile whose name slugs to `all` therefore CANNOT be targeted
    individually by name through this command; this is a documented limitation
    (such a profile is still reset as part of `RESET BARE ALL`). No other profile
    name is reserved. `ALL` is only a keyword when it is the SOLE argument; it is
    NOT mixed with other names in a subset list.
15. AFTER resolving the target list (bare, one-or-more named, or ALL) and BEFORE
    archiving anything, THE shell SHALL present the SAME single confirmation
    dialog as criterion 19.2, with its existing Confirm and Cancel buttons, the
    ONLY difference being that the dialog SHALL name the profile(s) that will be
    reset: the single profile name for one target, or the enumerated list of
    profile names (DEFAULT_PROFILE plus each named profile) for a subset or ALL.
    Confirm resets exactly the listed profiles (criteria 19.11, 19.13); Cancel
    leaves all profiles untouched (criterion 19.3 applies).

<!-- CR-CH-116: write-through config model. Reset bare now ARCHIVES the alias
     store too and REBUILDS the built-in menu files and the alias seed file from
     the Recovery_Source, rather than merely archiving the menu directory and
     resetting in-memory state (which it did under CR-CH-021). The rebuild uses
     the SAME Config_Rebuild_Helper as the First_Run_Write (Requirement 23.6). -->

16. (CR-CH-116) THE archive set (criterion 19.4) SHALL ADD the `aliases/`
    directory to the items moved into `config-archive/<timestamp>/` when present,
    so a user's alias overrides (e.g. `aliases/<locale>/FFCMD.toml`) are moved
    aside before the seed is rebuilt. (Under CR-CH-021 Reset bare did not touch
    aliases; this closes that gap.) Files that do not exist are skipped without
    error, consistent with criterion 19.4/19.5.
17. (CR-CH-116) AFTER archiving (criterion 19.4) and for each profile whose
    on-disk configuration is being reset, THE shell SHALL REBUILD, from the
    Recovery_Source via the shared Config_Rebuild_Helper (Requirement 23.6): the
    built-in menu files `menus/pom.toml`, `menus/settings.toml`, `menus/scrm.toml`
    (each a Complete_Config_File, Requirement 23.4) AND the FFCMD alias seed file
    `aliases/<locale>/FFCMD.toml` containing the single seed alias `X -> EXIT`
    (command-configurator Requirement 8). (Under CR-CH-021 Reset bare rebuilt no
    files; the barebones state came from the code-only fallback.) All rebuild
    writes SHALL honour the directory seams of Requirement 23.5.
18. (CR-CH-116) WHEN the reset targets the running (Active) profile, THE in-memory
    reset SHALL continue to reopen the Recovery_Baseline POM (criterion 19.6); with
    the files now rebuilt on disk, the reopened menus and the reloaded alias store
    SHALL read from the just-written files (Requirement 23.1), so the live state
    after Reset bare matches the on-disk files exactly.
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

---

### Requirement 21: Config View Keyboard Tree Navigation

**User Story:** As a workbench user, I want to navigate the Config View with the keyboard the
same way I navigate the File Navigator tree -- arrow keys to move between namespace groups and
keys, and to expand/collapse groups -- so that the configuration browser behaves like a standard
tree control and I do not have to reach for the mouse.

**Source:** [B069]; [CR-CH-039]; owner request ("The configuration workspace is a node tree, and
should be navigable as a node tree very much like the file navigator node tree works ... bear in
mind the expected behaviour of these kinds of trees and build to match"). Behavioural model: the
File Explorer tree keyboard reducer (`explorer_view.rs`, file-tree-panel Requirement 20 / 24) and
the standard desktop tree-control convention.

**Glossary:**
- **Config_Tree** -- the Config View rendered as a node tree: a level of NAMESPACE GROUP nodes
  (each an expandable branch, e.g. `editor`, `logging`, `theme`) whose children are the KEY nodes
  under that namespace (leaves, e.g. `editor.tab_size`).
- **Tree_Cursor** -- the single keyboard-focused node in the Config_Tree (distinct from egui
  widget focus and from the multi-select used elsewhere; the Config_Tree is single-selection).
- **Visible_Rows** -- the ordered list of nodes currently drawn: every namespace group node, plus
  the key nodes of each EXPANDED group, in display order (group, then its keys, then the next
  group), after the search/filter (Requirement 15.7) is applied.

#### Acceptance Criteria

1. THE Config View SHALL present its content as a Config_Tree: a top level of expandable namespace
   group nodes, each containing the configuration KEY nodes under that namespace prefix, ordered
   the same way the existing grouped view orders them (namespaces sorted, keys sorted within a
   group). The existing per-key widgets (value, provenance, Reset-to-Default), the search/filter,
   and the Source File indicator (Requirement 15.3-15.8) are UNCHANGED; this requirement adds
   keyboard navigation over that structure.

2. THE Config View SHALL maintain a single Tree_Cursor identifying the keyboard-focused node.
   WHEN the Config View is entered by Tab from the command field (the first interior stop is the
   Filter field, per B058 / Requirement 20), pressing Down (or Tab past the Filter field into the
   tree, consistent with the File Navigator) SHALL establish the Tree_Cursor on the first
   Visible_Row if none is set.

3. WHEN the Config_Tree has the keyboard and the Tree_Cursor is set, THE Config View SHALL respond
   to Down Arrow by moving the Tree_Cursor to the NEXT Visible_Row and to Up Arrow by moving it to
   the PREVIOUS Visible_Row, clamped at the first and last Visible_Row (no wrap), matching the File
   Navigator tree.

4. WHEN the Tree_Cursor is on a COLLAPSED namespace group node, THE Config View SHALL respond to
   Right Arrow by EXPANDING that group (its key nodes become visible). WHEN the Tree_Cursor is on
   an EXPANDED namespace group node, Right Arrow SHALL move the Tree_Cursor to that group's FIRST
   key node (standard tree convention).

5. WHEN the Tree_Cursor is on an EXPANDED namespace group node, THE Config View SHALL respond to
   Left Arrow by COLLAPSING that group. WHEN the Tree_Cursor is on a key node (a leaf) OR a
   collapsed group node, Left Arrow SHALL move the Tree_Cursor to that node's PARENT namespace
   group node (a key's Left goes to its group; a collapsed group's Left is a no-op as it has no
   parent level), matching the File Navigator tree.

6. WHEN the Tree_Cursor is on a namespace group node, THE Config View SHALL respond to Enter by
   TOGGLING that group's expand/collapse state. WHEN the Tree_Cursor is on a key node, Enter SHALL
   move egui keyboard focus to that key's value-editing widget (checkbox / slider / combo / text
   field) so the value can be edited, WITHOUT committing any change by itself.

7. WHEN the Config_Tree has the keyboard and the Tree_Cursor is set, THE Config View SHALL respond
   to Home by moving the Tree_Cursor to the FIRST Visible_Row and to End by moving it to the LAST
   Visible_Row.

8. THE node holding the Tree_Cursor SHALL be rendered with a visible selection indication
   (highlight / outline) distinct from egui hover, so the keyboard position is always apparent
   (matching the File Navigator's focused-node highlight).

9. Arrow-key tree navigation SHALL be active ONLY when keyboard focus is in the Config_Tree, NOT
   while the Filter field (or a key's editing widget) holds focus -- so typing in the Filter field
   or a text value is never hijacked by navigation. Pressing Escape while a key's editing widget
   holds focus (or Tab out of it) SHALL return control to the Config_Tree with the Tree_Cursor
   retained.

10. WHEN the search/filter (Requirement 15.7) changes the set of Visible_Rows, THE Config View
    SHALL keep the Tree_Cursor on its node if still visible, otherwise move it to the nearest
    remaining Visible_Row (or clear it when none remain), so navigation never points at a hidden
    node.

11. THE keyboard tree-navigation behaviour (Visible_Rows ordering, Up/Down/Left/Right/Enter/Home/
    End transitions on the Config_Tree) SHALL be implemented as a PURE, unit-testable reducer over
    an explicit tree + cursor model (mirroring `explorer_view::reduce_key`), so the transitions are
    covered by automated tests independent of rendering; the rendered focus/selection behaviour
    SHALL additionally have an `egui_kittest` test per the GUI Behaviour Testing rule.

---

### Requirement 22: Config View -- Demarcated Tree-View Re-skin

> **REVISED (config-tree-collapsing branch).** The original wording mandated
> literal box-drawing child connectors (U+2500-U+257F) and a SINGLE GLOBAL
> aligned value column. Commit 128d8d2 then moved to **egui-native collapsible
> `[<Group>]` headers** with **PER-GROUP aligned value columns**.
>
> **RE-REVISED (config-tree-fix branch, owner follow-up -- supersedes the
> 128d8d2 per-group decision).** The owner reviewed 128d8d2 and asked for two
> concrete fixes:
> 1. A DUPLICATE-RENDER defect was reported (the list appearing to draw twice --
>    a tree then a `Settings` heading + `Setting / Value / State` table). The
>    panel MUST render the config list EXACTLY ONCE (AC 22.1a below). (On audit
>    the stale table path had already been removed by 128d8d2; the single render
>    path is now asserted by a regression test so the duplicate cannot return.)
> 2. The layout MUST match the owner's mock: group headers FLUSH-LEFT, each
>    `[<Group>] : expanded|collapsed` showing the live collapse state in text
>    (the egui-native triangle is DROPPED); child rows INDENTED exactly one level
>    under their group; and a SINGLE GLOBAL `:`/value column aligned across EVERY
>    visible row of the panel -- the `Filter` and `Unsaved edits` header rows, the
>    group rows, and the child rows all share one `:` column (NOT per-group). No
>    `. . .` dot-leaders anywhere. This SUPERSEDES the per-group alignment and the
>    native-triangle header of 128d8d2.
>
> Click-to-collapse is preserved: the flush-left header is a clickable control
> that toggles the existing per-namespace collapsed state. Each changed criterion
> is annotated "(RE-REVISED ...)" below; AC 22.10 (first-Tab focus) and all
> preservation criteria (22.2, 22.7, 22.8, 22.9, 22.11) are retained unchanged in
> intent.

**User Story:** As a workbench user, I want the Configuration Context to be
clearly demarcated into a header area and a tree body, with egui-native
collapsible bracketed group headers, and the values within each group lined up in
a vertical column, so that the configuration browser reads like a standard,
well-structured tree control instead of an undifferentiated list.

**Source:** owner request ("The configuration editor is not clear and well
demarcated between areas ... create a tree view ... where branches can be
expanded and collapsed very much like they can currently but clearly represented
... the values for each node should be aligned vertically"). The grouping
mechanism is explicitly to be preserved ("that mechanism should just continue,
it is just the representation that I want changed"). The follow-up owner request
on this branch adopts egui's CollapsingHeader best practice in place of the
hand-drawn box-drawing connectors ("Surely egui has some existing framework to
support this kind of thing ... Let egui handle indentation and expansion state").

> **Scope note.** This is a VISUAL/representation change ONLY. It REFINES how the
> Config View from Requirements 15, 20, and 21 is drawn; it does NOT replace
> them. The grouping mechanism, expand/collapse behaviour, config persistence,
> SAVE/END/RETURN save semantics (Cross-Cutting Save Convention below), the
> `WorkspaceContext` focus contract, and the keyboard tree navigation
> (Requirement 21) are all preserved unchanged -- only the way the existing
> `Visible_Rows` model is drawn changes.

**Glossary (reuses Requirement 21 terms):** Config_Tree, Tree_Cursor,
Visible_Rows as defined in Requirement 21. Additional term:
- **Value_Column** -- (RE-REVISED: now a SINGLE GLOBAL column across the whole
  panel) the vertical column at which the `:` separator and the value/state of
  EVERY currently-visible row align: the `Filter` and `Unsaved edits` header
  rows, every group-header row, and every child key row. Its position is the max
  character width over ALL currently-visible LEFT-LABELS (a group's left-label is
  its bracketed name `[<Group>]`; a child's left-label is the one-level indent +
  its full key; a header row's left-label is its caption), plus one pad cell.
  Because the indentation is baked into the monospace left-label string (not an
  egui container indent), a single cross-row column is well-defined. This
  SUPERSEDES the per-group column of 128d8d2.

#### Functional intent

- **Single render, no duplicate.** The panel renders the config list EXACTLY
  ONCE. There is no second copy drawn as a `Settings` heading + a
  `Setting / Value / State` table beneath the tree. (RE-REVISED: duplicate-render
  defect fix.)
- **Demarcated header area.** A header area (the `Command ===>` line owned by the
  shell, a `Filter` row, and an `Unsaved edits : <yes|no>` row) separated from
  the tree body by a horizontal rule, with the `Filter` and `Unsaved edits` rows
  aligned to the same GLOBAL `:` column as the tree.
- **Flush-left group headers labelled with collapse state.** Each of the existing
  namespace groups renders FLUSH-LEFT as `[<DisplayName>] : expanded|collapsed`
  (the 8 confirmed groups: `[Accessibility]`, `[Catalogs]`, `[Editor]`,
  `[Logging]`, `[Menu]`, `[Theme]`, `[Ui]`, `[Vfs]`), in the existing sorted
  order; the state word reflects the live per-namespace collapse state. The
  egui-native triangle is DROPPED; the header is a clickable control that toggles
  the same collapsed state. (RE-REVISED: supersedes the native-triangle header of
  128d8d2.)
- **One-level-indented child rows, no box-drawing glyphs.** When a group is
  expanded, its child key rows render INDENTED exactly one level under the
  flush-left group header. The indent is part of the monospace left-label. No
  box-drawing connector glyphs (U+2500-U+257F).
- **Single GLOBAL vertically aligned value column.** The `:` separator and the
  value/state of EVERY visible row (header rows, group rows, child rows) line up
  in ONE column across the WHOLE panel, computed from the max width over all
  currently-visible left-labels. No `. . .` dot-leaders. (RE-REVISED: global, not
  per-group.)
- **Preserve everything else.** Grouping mechanism, expand/collapse, per-key edit
  controls, the SELECTABLE key name, the VISIBLE + SELECTABLE key description
  (Requirement 14.2), the focus contract, and the save convention are unchanged.

#### Acceptance Criteria

22.1. THE Config View SHALL render a header area, separated from the tree body by
a horizontal rule, that shows the `Command ===>` line, a `Filter` row, and an
`Unsaved edits : <yes|no>` row; the two areas SHALL be clearly demarcated.

22.1a. (RE-REVISED -- duplicate-render defect) THE Config View SHALL render the
config list EXACTLY ONCE. It SHALL NOT draw a second copy of the list (e.g. a
`Settings` heading followed by a `Setting / Value / State` table beneath the
tree). Each schema key's row SHALL appear exactly once in the panel. (Enforced by
a full-shell regression test asserting a known key's row resolves to a unique
widget; the stale table path was already removed in 128d8d2.)

22.2. WHEN the working-copy buffer is non-empty, THE `Unsaved edits` row SHALL
show `yes`; WHEN it is empty, it SHALL show `no` -- i.e. the indicator SHALL
equal `ConfigPanelState.is_dirty()`.

22.3. (RE-REVISED) THE Config View SHALL render each top-level namespace group as
a FLUSH-LEFT clickable header row labelled `[<DisplayName>] : expanded|collapsed`
(using the existing display-name capitalisation), in the existing sorted group
order from `visible_rows`. The state word SHALL reflect the live per-namespace
collapse state (`expanded` when not collapsed, `collapsed` when collapsed). The
egui-native triangle expander is DROPPED; the header carries no `v`/`>` arrow and
no key count. (Supersedes the native-triangle `[<DisplayName>]` header of
128d8d2.)

22.4. (RE-REVISED) WHEN a group is EXPANDED, THE Config View SHALL render its
child key nodes INDENTED exactly one level under the flush-left group header. The
one-level indent SHALL be part of the child row's monospace left-label (not an
egui container indent). THE Config View SHALL NOT paint box-drawing connector
glyphs (U+2500-U+257F) and SHALL NOT use `. . .` dot-leaders.

22.5. (RE-REVISED) THE Config View SHALL align the `:` separator and the
value/state of EVERY currently-visible row into a SINGLE GLOBAL vertical
Value_Column spanning the whole panel: the `Filter` header row, the
`Unsaved edits` header row, every group-header row, and every child key row. The
column position SHALL be the max character width over ALL currently-visible
left-labels (bracketed group names, indented child keys, and the header
captions), plus one pad cell; each left-label SHALL be right-padded in monospace
to that width, then ` : `, then the value/state. (Supersedes the per-group column
of 128d8d2 -- the owner explicitly requested GLOBAL alignment.)

22.6. (RE-REVISED) WHEN the set of currently-visible rows changes (any group is
expanded or collapsed, or the filter changes which keys match), THE Config View
SHALL recompute the GLOBAL Value_Column position from the new set of visible
left-labels (a collapsed group contributes only its header row, not its
children).

22.7. THE grouping mechanism (namespace derivation and group membership via
`namespace_of` + `visible_rows`) and the 8 confirmed groups (Accessibility,
Catalogs, Editor, Logging, Menu, Theme, Ui, Vfs) SHALL be unchanged by this
rendering redesign.

22.8. Expand/collapse SHALL continue to toggle the visibility of a group's child
key nodes using the existing per-namespace `collapsed` state, via BOTH the
existing mouse trigger (click the group header) AND the existing keyboard
triggers (Right/Left/Enter through `reduce_config_key`).

22.9. Each visible key node SHALL continue to expose its existing value widget,
provenance/LOCKED badge, staged-edit indicator, and Reset-to-Default control with
unchanged behaviour, AND its existing selectable key name and visible selectable
key description (Requirement 14.2) SHALL remain selectable and visible (NOT moved
to hover/tooltip, NOT downgraded to inert text).

22.10. THE Config View's `WorkspaceContext::render` SHALL return
`InteriorFocus::single(filter_field_id())` and be dispatched via
`render_workspace_context`; the first Tab from the command field SHALL focus the
Filter field with no phantom stop (covered by the full-shell first-Tab
egui_kittest test, extended/kept green).

22.11. THE SAVE / END / RETURN semantics SHALL be unchanged (SAVE commits+stays,
END commits+exits, RETURN discards+exits); no autosave SHALL be introduced.

22.12. (RE-REVISED) THE GLOBAL Value_Column position SHALL be produced by a pure,
egui-independent function over the whole panel's currently-visible left-labels
(max-width -> column), unit-tested for: the empty set, a single label, the
widest-label case over the whole panel (header + group + child labels together),
and the collapse case (a hidden group's children are excluded). The
left-label builders (flush-left bracketed group name; one-level-indented child
key) and the padding helper SHALL also be pure and unit-tested.

#### Out of scope (Requirement 22)

- Changing namespace derivation or group membership (grouping mechanism frozen).
- Changing config persistence, layering, validation, or the SAVE/END/RETURN flush.
- Changing the Requirement 21 keyboard navigation transition table -- only its
  on-screen representation is re-skinned.
- New config keys, groups, or editing widgets; any change to `menus/settings.toml`
  or the `CONFIG` command resolution (Requirement 20).
- Multi-level nesting below the two-level (group -> key) model; the tree stays two
  deep (depth 0 flush-left group, depth 1 one-level-indented key). No box-drawing
  connectors or dot-leaders are drawn at any level (AC 22.4).

---

## Cross-Cutting: Save Convention (project-master Requirement 11)

The Config / Settings Workspace Context follows the ISPF-style explicit-commit
convention (`.kiro/steering/save-convention.md`, project-master Requirement 11),
the SAME as every other editable Context. This governs the Config PANEL'S
edit-commit behaviour ONLY; it does NOT change `ff-config`'s layered-store
semantics (layers, precedence, hot-reload engine of Requirements 1-14), which
are unchanged.

1. ON entry to the Config Context, the panel operates on a WORKING-COPY BUFFER
   held in `ConfigPanelState`; the persisted store (the `ConfigHandle` user
   layer) is the source of the displayed values and is untouched by editing.
   (Req 11.1)
2. A bare widget edit (checkbox/slider/text/combo change, or a Reset click)
   stages into the working buffer ONLY -- it does NOT call `set_user_value` /
   `remove_user_value`. The former autosave-on-edit behaviour is removed.
   (Req 11.2)
3. SAVE flushes every staged edit to the backing store in one commit and leaves
   the Context open (buffer == persisted afterwards). (Req 11.3)
4. END flushes then exits; on a flush failure the Context stays open and reports
   the error. (Req 11.4)
5. RETURN discards the working buffer without writing the store and exits,
   leaving the store exactly as it was on entry. (Req 11.5)
6. The in-panel Save affordance enqueues the SAVE command, so it gates
   persistence through the SAME single dispatch seam as the typed/keyed SAVE
   verb -- no divergent save path. (Req 11.7, 11.8)

---

### Requirement 23: Write-Through Config Model for Built-in Menus and the Command Alias Seed (CR-CH-116)

> **FRAMEWORK CHANGE (owner-confirmed 2026-11-04).** This requirement SUPERSEDES
> the CODE-ONLY model of CR-CH-021 (framework-conformance.md mechanism 4) for the
> built-in menus (POM, Settings, SCRM) and the command alias seed. Under
> CR-CH-021 the compiled `DEFAULT_*_TOML` constants were the authoritative runtime
> source and were NEVER written to disk; a user file only OVERRODE them when
> present. Under this requirement the compiled defaults become the RECOVERY source
> only: the application WRITES complete config files on first run and RUNS FROM the
> files thereafter. The owner expressly confirmed the reversal (decision
> 2026-11-04). The affected "never materialised" criteria elsewhere are revised in
> lock-step: menu-workspace Requirement 4.1, 11.1, 12.1 and screen-snapshot-scrm
> Requirement 22.8.

**User Story:** As an operator, I want every built-in menu and command alias to
exist as a visible, editable TOML file that the application actually runs from, so
that nothing is hidden in compiled constants, while a corrupted or deleted file
can always be rebuilt from a known-good baseline in code.

**Source:** [CR-CH-116] owner decision (`.agents/tasks/config-model-change/decision.md`);
supersedes [CR-CH-021]. Investigation `.agents/tasks/config-model-change/investigation.md`.

#### Glossary additions

- **Recovery_Source**: The compiled Rust defaults (`DEFAULT_POM_TOML`,
  `DEFAULT_SETTINGS_TOML`, `DEFAULT_SCRM_MENU_TOML`, and the FFCMD alias seed) kept
  in code SOLELY to rebuild config files. The Recovery_Source is consumed on
  exactly two paths -- First_Run write and Reset-bare rebuild -- plus a last-ditch
  graceful fallback when an expected file is missing or unparseable at runtime. It
  is NO LONGER the normal runtime source.
- **First_Run_Write**: The one-time, per-file write of a complete default config
  file when that file is absent, performed during startup so the subsequent load
  reads the just-written file.
- **Config_Rebuild_Helper**: The single helper that serialises the Recovery_Source
  into complete config files through the configured directory seams. First_Run_Write
  and Reset-bare rebuild (Requirement 19) BOTH call it, so they cannot diverge.
- **Complete_Config_File**: A serialised config file in which EVERY loader-defaulted
  flag is written EXPLICITLY (for a menu: `show_calendar`, `group_separator`,
  `group_headers`, and each option's `enabled`), so the file fully determines
  behaviour with no reliance on a silent loader default.

#### Acceptance Criteria

1. THE workbench SHALL treat the compiled `DEFAULT_POM_TOML`, `DEFAULT_SETTINGS_TOML`,
   `DEFAULT_SCRM_MENU_TOML`, and the FFCMD alias seed as the Recovery_Source ONLY;
   at startup and during running the authoritative source of a built-in menu and of
   the command alias seed SHALL be the on-disk config file, not the compiled
   constant.
2. WHEN the workbench starts AND a built-in menu file (`menus/pom.toml`,
   `menus/settings.toml`, `menus/scrm.toml`) is ABSENT, THE workbench SHALL perform
   a First_Run_Write of that file from the Recovery_Source BEFORE loading it, so the
   subsequent load reads the just-written file. First_Run detection SHALL be
   PER-FILE: an absent file is written even when its siblings already exist (a hole
   is never left).
3. WHEN the workbench starts AND the FFCMD alias seed file for the active locale
   (`aliases/<locale>/FFCMD.toml`, Requirement from command-configurator Requirement 8)
   is ABSENT, THE workbench SHALL perform a First_Run_Write of it from the
   Recovery_Source (the single seed alias `X -> EXIT`) BEFORE the resolver seeds, so
   the resolver reads the just-written file.
4. EACH file written by a First_Run_Write SHALL be a Complete_Config_File: every
   loader-defaulted flag SHALL be emitted explicitly. In particular the Settings
   file SHALL contain `show_calendar = false` explicitly and the POM file SHALL
   state its `show_calendar` explicitly, so the reliance on the loader's silent
   `show_calendar = true` default (menu-workspace loader) is removed for written
   files. The menu serialiser SHALL support a COMPLETE mode (or the writer SHALL
   post-fill defaults before serialising) so a plain write of a Recovery_Source menu
   emits every flag.
5. ALL First_Run_Write and Reset-bare rebuild writes SHALL go through the SAME
   directory seams the loaders read (`menus_dir()` honouring `dir_overrides.menus`,
   `scrm_dir()` honouring `dir_overrides.scrm`, and the alias base honouring
   `dir_overrides.user_aliases` / the `FFWB_USER_ALIASES_DIR` env seam), so the
   shell test suite can continue to isolate all writes to a temporary directory. No
   write SHALL bypass these seams.
6. First_Run_Write and Reset-bare rebuild (Requirement 19) SHALL share ONE
   Config_Rebuild_Helper, so the content a fresh install gets and the content a
   Reset-bare restores are produced by the same code path and cannot diverge.
7. WHEN an expected built-in menu or alias file is present but fails to parse at
   runtime (invalid TOML, missing required field, or over the option limit), THE
   workbench SHALL fall back to the corresponding Recovery_Source for that render
   AND surface a non-blocking notice identifying the file and the error (menu
   fallback, menu-workspace Requirement 12.5), WITHOUT overwriting the user's file.
   A parse failure SHALL NOT be treated as "absent" and SHALL NOT trigger a
   First_Run_Write over the broken file; recovery of a broken file is the operator's
   explicit Reset-bare action.
8. THE workbench SHALL NOT perform a First_Run_Write while the application is in
   Degraded_Mode due to a User_Data_Dir write failure (startup-and-session
   Requirement 3.3/3.5); in that case it SHALL fall back to the Recovery_Source for
   the session (graceful degradation) and SHALL NOT loop attempting to write.
9. EACH Complete_Config_File written SHALL carry a config VERSION STAMP field (e.g.
   a top-level `config_version` integer) recording the Recovery_Source schema
   version it was written from, so a future UPGRADE command (deferred, see below)
   can detect a file older than the current default. The workbench SHALL NOT act on
   the version stamp in this change: a file whose version is older than the current
   default SHALL still load and run as-is (files remain authoritative). The stamp
   LEAVES ROOM for a future merge/upgrade; it does not implement one.
10. THE deferred UPGRADE behaviour (merging newly-added default options/aliases
    into an existing authoritative user file on a software update) SHALL NOT be
    implemented by this change. It is recorded as a follow-up only (an UPGRADE
    command that rebuilds the Reset-bare defaults and merges the existing user
    configuration into them). Until then, a user who already has a config file does
    NOT automatically receive a newly-added default option/alias -- an accepted
    consequence of the write-through model.

#### Out of scope (Requirement 23)

- Implementing the UPGRADE/merge command (criterion 10 -- follow-up only).
- Changing the `ff-config` layered-store engine (Requirements 1-14): `config.toml`
  already materialises on first write via `set_user_value` and is unchanged by this
  requirement. This requirement governs the built-in MENU files and the ALIAS seed
  file only.
- Writing themes, keymaps, or workspace-kinds on first run (those remain on their
  current model; this change is scoped to menus + the alias seed).
- Any change to the X / END / RETURN / EXIT command model (that is menu-workspace
  Requirement 14 and navigation-commands Requirement 21, revised separately under
  the same CR).

---

### Requirement 24: Expanded FFCMD alias seed set -- five seed aliases (CR-NR-120)

> **BEHAVIOUR CHANGE on CR-CH-116 (owner-approved 2026-11-06).** EXTENDS the
> write-through alias seed of Requirement 23.3 and command-configurator
> Requirement 8. Today the bare/first-run seed and the Reset-bare rebuild emit
> exactly ONE genuine seed alias, `X -> EXIT`. This requirement grows the seed set
> to exactly FIVE aliases so the SCRM shortcuts (`SS`, `CS`, `CT`, `CR`) survive a
> Reset bare instead of disappearing. The reported defect: after Reset bare the
> owner's `SS` alias vanished because `SS` was never part of the rebuilt baseline.
> NO new mechanism is introduced: the five rows are defined in the SAME single
> Rust Recovery_Source function that defines `X -> EXIT` today, written through to
> the SAME `aliases/<locale>/FFCMD.toml` file, and rebuilt by the SAME
> Config_Rebuild_Helper (Requirement 23.6). Nothing is hidden.

**User Story:** As an operator who relies on the SCRM shortcuts, I want `SS`, `CS`,
`CT`, and `CR` to be part of the rebuilt baseline, so that a Reset bare (or a first
run) restores them exactly as it restores `X -> EXIT`, instead of silently dropping
them.

**Source:** CR-NR-120 (owner, 2026-11-06, "Proceed with the alias seed set"):
expand the bare/first-run seed set from one alias to five. Owner-approved seed set
(carried verbatim): `X -> EXIT`, `SS -> SNAPSHOT`, `CS -> CAPTURE START`,
`CT -> CAPTURE STOP`, `CR -> CAPTURE REPLAY`. Cross-references: configuration-system
Requirement 23 (write-through config model, First_Run_Write, Reset-bare rebuild),
command-configurator Requirement 8 (the `X -> EXIT` seed this extends) and
Requirement 9 (CR-NR-119 display-time description inheritance), screen-snapshot-scrm
Requirement 7-10/24 (the SNAPSHOT / CAPTURE verbs these aliases target).

#### Acceptance Criteria

1. WHEN a bare FFCMD alias config is built for the active locale -- on First_Run_Write
   (Requirement 23.3) OR on a Reset-bare rebuild (Requirement 19.17) -- THE workbench
   SHALL seed `aliases/<locale>/FFCMD.toml` with EXACTLY these five FFCMD seed
   aliases, each with an EMPTY stored description, and NO other genuine (non-identity)
   seed aliases:
   - `X` -> `EXIT`
   - `SS` -> `SNAPSHOT`
   - `CS` -> the CAPTURE START shortcut (representation per criterion 24.5)
   - `CT` -> the CAPTURE STOP shortcut (representation per criterion 24.5)
   - `CR` -> the CAPTURE REPLAY shortcut (representation per criterion 24.5)
   [CR-NR-120]

2. THE five seed aliases SHALL be defined in ONE Rust Recovery_Source -- the SAME
   function that defines the `X -> EXIT` seed row today (the Config_Rebuild_Helper's
   alias-seed builder, Requirement 23.6 / command-configurator Requirement 8) -- so
   the First_Run_Write and the Reset-bare rebuild emit the identical set from a
   single definition and cannot diverge. No seed alias SHALL be defined anywhere
   else; nothing is hidden. [CR-NR-120]

3. EACH seed alias's STORED description SHALL be the EMPTY string; the five rows
   SHALL carry no description text on disk. The on-disk `FFCMD.toml` row shape SHALL
   be UNCHANGED from Requirement 23 / command-configurator Requirement 8 (surface,
   canonical, environment `ffcmd`, optional empty description, optional parameter
   template), so an existing reader parses the expanded file without schema change.
   [CR-NR-120]

4. EACH seed alias's DISPLAYED one-line description SHALL be INHERITED at display
   time from its target command's `cmd:<ROOT>` help topic through the CR-NR-119
   `short_description(cmd:<ROOT>)` lookup (command-configurator Requirement 9.1/9.5,
   context-help Requirement 22.2), keyed by the UPPERCASED canonical root the row
   stores. The seed SHALL persist NO description literal (criterion 24.3); inheritance
   is display-only and SHALL NOT alter the SAVE / on-disk shape (Requirement 9.2).
   WHEN no matching `cmd:<ROOT>` topic is loaded, the display SHALL fall back to EMPTY
   (Requirement 9.4), never failing. [CR-NR-120]

5. THE stored canonical VALUE of each seed alias SHALL be a VALID FFCMD canonical
   root (`EnvironmentRegistry::contains_root("FFCMD", <canonical>)` TRUE), so the
   seed rows PASS the alias load/save validation (command-configurator Requirement
   5.4 / 6.6, the `contains_root` root-validity rule) and are never skipped with a
   WARN. For the two-word CAPTURE shortcuts (`CS`, `CT`, `CR`) THE representation
   SHALL be: canonical root `CAPTURE` (a valid FFCMD root) WITH the subcommand carried
   in the row's parameter template (`command_template` = `START` / `STOP` / `REPLAY`),
   so the typed surface expands through the EXISTING `rewrite_verb_to_root` pre-pass
   to `CAPTURE START` / `CAPTURE STOP` / `CAPTURE REPLAY` and dispatches through the
   EXISTING single front door; and the inherited description (criterion 24.4) resolves
   from the EXISTING `cmd:CAPTURE` topic (so `CS`, `CT`, `CR` all display the single
   `cmd:CAPTURE` one-liner -- the accepted tradeoff of this representation). This is
   the DECIDED representation (Option A, owner-approved 2026-11-06); see the OWNER
   DECISION note below. [CR-NR-120]

6. THE five seed aliases SHALL each appear as an editable/removable user-override row
   in the Alias Editor's FFCMD view exactly like the `X -> EXIT` seed
   (command-configurator Requirement 8.6): the write-through model exposes them; they
   are not hidden logic. Removing one is the user's prerogative; a subsequent Reset
   bare restores the full five-alias set (criterion 24.1). [CR-NR-120]

7. THE resolver SHALL load the five seed aliases through the EXISTING User_Alias_Store
   overlay path routed into the FFCMD AliasTable (command-configurator Requirement
   5.7 / 8.7), introducing NO new resolution mechanism, dispatcher, navigation stack,
   or persistence format (framework-conformance mechanisms 1 and 4). [CR-NR-120]

> **OWNER DECISION -- two-word CAPTURE targets (`CS` / `CT` / `CR`).** The brief's
> confirmed decision stored the canonical VALUE as the full two-word string (e.g.
> `CAPTURE START`). Investigation found this contradicts two EXISTING framework
> facts, so the gate surfaces it rather than silently implementing a framework
> change:
>   (i) `EnvironmentRegistry::contains_root("FFCMD", ...)` is an EXACT full-string
>       match and the FFCMD root set contains `CAPTURE` (single word), NOT
>       `CAPTURE START`. A row whose canonical is `CAPTURE START` would be REJECTED
>       by the alias load/save validation (command-configurator Req 5.4/6.6) and
>       skipped with a WARN at load, so the seed would not survive.
>   (ii) The CR-NR-119 help topics authored for SCRM are `cmd:SNAPSHOT` and
>       `cmd:CAPTURE` (single). There is NO `cmd:CAPTURE START` / `cmd:CAPTURE STOP`
>       / `cmd:CAPTURE REPLAY` topic, so `short_description(cmd:CAPTURE START)`
>       returns None and the inherited description would be EMPTY, not the promised
>       one-liner.
> **DECISION: Option A, owner-approved 2026-11-06.** Criterion 24.5 implements
> **Option A (framework-conforming, no framework change):** canonical root
> `CAPTURE` + parameter template `START` / `STOP` / `REPLAY`; valid root, inherits
> from `cmd:CAPTURE`, no new mechanism, no new help topic. The accepted tradeoff:
> `CS`, `CT`, `CR` all display the single `cmd:CAPTURE` one-liner, and the editor
> shows canonical `CAPTURE` with the subcommand in the template column.
> **Option B (REJECTED at this gate):** store the canonical as the two-word
> `CAPTURE START` etc. -- would have REQUIRED a confirmed FRAMEWORK CHANGE (relax
> `contains_root` to accept compound/first-word roots, rippling into the no-shadow
> and root-validity rules) PLUS authoring three new `cmd:CAPTURE START/STOP/REPLAY`
> help topics under CR-NR-119. The owner chose A; B is recorded here only as the
> considered alternative and is NOT implemented.

#### Out of scope (Requirement 24)

- Turning any verb other than `X`, `SS`, `CS`, `CT`, `CR` into a seed alias.
- Changing the SNAPSHOT / CAPTURE command semantics (screen-snapshot-scrm) or the
  X / END / RETURN / EXIT model (menu-workspace Req 14, navigation-commands Req 21).
- Persisting any description literal on a seed row (descriptions are inherited at
  display time only, criterion 24.4).
- Authoring new `cmd:<ROOT>` help topics (would have been Option B only; Option A
  was chosen, so no new help topic is authored).
