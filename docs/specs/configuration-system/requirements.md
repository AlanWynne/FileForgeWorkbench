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
3. THE query API SHALL return the effective value for any given Settings_Key along with its Provenance — indicating which Configuration_Layer provided the value.
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
6. WHEN a configuration file contains a key that has no schema entry (unknown key), THE Configuration_System SHALL ignore the unknown key, emit a DEBUG-level log record indicating the unrecognized key, and continue loading — unknown keys SHALL NOT cause errors or warnings at WARN level.
7. THE Configuration_Schema SHALL support registration of new keys at runtime (by plugins during initialization), allowing the schema to grow dynamically as plugins are loaded.

---

### Requirement 15: Settings Panel — Interactive Configuration Dialog

**User Story:** As a workbench user, I want a graphical Settings panel that lets me view and
change all configuration values without editing TOML files manually, so that I can adjust
workbench behaviour quickly and safely from within the application.

**Source:** [WB] Configuration as Data; [ISPF-POM] POM option 0.

#### Acceptance Criteria

15.1 WHEN the user selects option `0` from the Primary Option Menu, OR types `0` or `SETTINGS`
     or `=0` in any `Command ===>` field, THE shell SHALL open the Settings panel as a new tab
     with title `[SETTINGS]` and tab kind `SettingsPanel`.

15.2 THE Settings panel SHALL display all configuration keys registered in the `ff-config`
     schema, grouped by namespace (e.g., `Editor`, `Logging`, `Theme`, `Catalogs`, `VFS`),
     with each group rendered as a collapsible section.

15.3 FOR each configuration key, THE Settings panel SHALL display:
     - The key's human-readable description (from the schema entry)
     - The current effective value
     - The provenance layer that provided the effective value (e.g., `Default`, `User`, `Project`)
     - An appropriate input widget based on the value type:
       - Boolean → checkbox
       - Integer / Float with min/max → slider; without constraints → numeric text field
       - String with `allowed_values` → drop-down selector
       - String without constraints → single-line text field

15.4 WHEN the user changes a value in the Settings panel and confirms (presses Enter or moves
     focus away from the field), THE shell SHALL validate the new value against the schema
     constraints; IF valid, THE shell SHALL write the new value to the user-layer configuration
     file and update the effective value immediately (no restart required).

15.5 WHEN a value fails schema validation (out of range, not in allowed set, fails regex),
     THE Settings panel SHALL display an inline error message adjacent to the field and SHALL
     NOT persist the invalid value.

15.6 THE Settings panel SHALL display a `Reset to Default` button beside each key that has
     been overridden above the Defaults layer; WHEN clicked, THE shell SHALL remove the
     user-layer override for that key, restoring the schema default.

15.7 THE Settings panel SHALL include a search/filter input at the top; WHEN the user types
     in the filter, THE panel SHALL show only keys whose key path or description contains the
     filter text (case-insensitive substring match).

15.8 THE Settings panel SHALL display a read-only `Source File` indicator showing the path of
     the user-layer configuration file being edited.

15.9 THE `[SETTINGS]` tab SHALL persist in the session and be restored on next launch as a
     `SettingsPanel` tab kind.

15.10 WHEN the user presses `PF3` / `F3` or types `END` in the Settings panel command field,
      THE shell SHALL return the tab to the Primary Option Menu view.

15.11 WHEN the user clicks `Settings` in the POM option list (option 0 button), THE shell
      SHALL navigate to the Settings panel using the same routing as typing `0` in the command
      field.
