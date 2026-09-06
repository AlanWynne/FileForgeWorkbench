# Plugin Manager UI Requirements

## Introduction

This sub-project defines the Plugin Manager panel -- a dedicated UI for
listing, enabling, disabling, and configuring installed plugins. It is
accessed via POM option 8 (Plugins) and the `Plugins` menu.

The plugin architecture (`ff-plugin`) already defines the plugin trait,
lifecycle, and capability discovery. This sub-project adds only the UI
layer on top of the existing plugin registry.

## Glossary

| Term | Definition |
|------|-----------|
| Plugin | A crate implementing the `FileForgePlugin` trait registered with the plugin registry |
| Plugin registry | The runtime store of all loaded plugins, owned by `ff-plugin` |
| Plugin state | One of: Loaded, Active, Inactive, Failed |
| Plugin capability | A declared service a plugin provides (commands, viewers, language support, toolchain) |
| Plugin Manager panel | The egui panel rendered when POM option 8 is selected |

---

## Requirement 1: Plugin Manager Panel

**User Story:** As a workbench user, I want a Plugin Manager panel
accessible from POM option 8, so that I can see all installed plugins
and their current state at a glance.

**Source:** Gap analysis section 6.1 -- Plugin Manager UI (MISSING, High priority).

### Acceptance Criteria

1. WHEN the user selects POM option 8 or types `=8` or `PLUGINS` in the
   Command Field, THE workbench SHALL open a `PluginManagerPanel` tab.
2. THE Plugin Manager panel SHALL display a scrollable list of all
   plugins currently registered with the plugin registry.
3. FOR EACH plugin in the list, THE panel SHALL display: plugin name,
   version, description, current state (Active/Inactive/Failed), and
   the capabilities it provides.
4. WHEN a plugin is in the Failed state, THE panel SHALL display the
   failure reason alongside the plugin entry.
5. THE plugin list SHALL be sorted alphabetically by plugin name by
   default.
6. THE panel SHALL include a filter text field that narrows the list
   to plugins whose name or description contains the filter string.

---

## Requirement 2: Enable and Disable Plugins

**User Story:** As a workbench user, I want to enable and disable
individual plugins without restarting the workbench, so that I can
control which features are active.

**Source:** Gap analysis section 6.1 -- Plugin Manager UI (MISSING, High priority).

### Acceptance Criteria

1. WHEN a plugin is Active, THE panel SHALL display a `Disable` button
   for that plugin.
2. WHEN the user clicks `Disable`, THE workbench SHALL call the plugin's
   `deactivate()` lifecycle method and update the plugin state to
   Inactive.
3. WHEN a plugin is Inactive, THE panel SHALL display an `Enable` button
   for that plugin.
4. WHEN the user clicks `Enable`, THE workbench SHALL call the plugin's
   `activate()` lifecycle method and update the plugin state to Active.
5. WHEN a plugin fails to activate, THE workbench SHALL display the
   failure reason in the panel and set the plugin state to Failed --
   the workbench SHALL NOT crash.
6. THE enabled/disabled state of each plugin SHALL be persisted in the
   session configuration so that the state is restored on next launch.

---

## Requirement 3: Plugin Details View

**User Story:** As a workbench user, I want to see detailed information
about a plugin, so that I can understand what it does and how to
configure it.

**Source:** Gap analysis section 6.1 -- Plugin Manager UI (MISSING, High priority).

### Acceptance Criteria

1. WHEN the user selects a plugin in the list, THE panel SHALL display
   a detail area showing: full description, author, licence, homepage
   URL (if provided), list of capabilities, list of commands registered
   by the plugin, and list of configuration keys owned by the plugin.
2. WHEN a plugin provides configuration keys, THE detail area SHALL
   display each key with its current value and a link to the Settings
   panel filtered to that plugin's namespace.
3. WHEN a plugin provides commands, THE detail area SHALL display each
   command name and its bound keyboard shortcut (if any).

---

## Requirement 4: Session Persistence

**User Story:** As a workbench user, I want the Plugin Manager panel
to be restored when I reopen the workbench, so that my workflow is
not interrupted.

**Source:** Consistent with session persistence requirements across all panels.

### Acceptance Criteria

1. WHEN the workbench exits with a PluginManagerPanel tab open, THE
   session SHALL persist the tab so it is restored on next launch.
2. WHEN the workbench starts and restores a PluginManagerPanel tab,
   THE panel SHALL reload the current plugin registry state.
