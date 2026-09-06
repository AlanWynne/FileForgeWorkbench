# Plugin Manager UI Design

## Overview

The Plugin Manager panel is a new `TabKind::PluginManager` tab in
`ff-desktop`. It reads from the existing `ff-plugin` plugin registry
and calls lifecycle methods on plugins. No new crate is required.

---

## Design Decisions

### 1. TabKind::PluginManager

A new variant is added to the `TabKind` enum in `tab_state.rs`. Routing:
- POM option 8 button click -> `handle_command("8")`
- `=8` command -> `handle_command("=8")`
- `PLUGINS` command -> `handle_command("PLUGINS")`

All three routes set the active tab to `TabKind::PluginManager` (or open
a new tab if none exists), following the same pattern as `FilesPanel` and
`SettingsPanel`.

### 2. PluginManagerPanelState

A new `plugin_manager_panel.rs` module in `ff-desktop/src/` holds:

```rust
pub struct PluginManagerPanelState {
    filter: String,
    selected_plugin: Option<String>,  // plugin name
}
```

The panel reads plugin data directly from `WorkbenchShell`'s plugin
registry reference on each frame (immediate-mode pattern -- no cached
copy needed).

### 3. Enable/Disable via PluginContext

The `ff-plugin` crate's `PluginRegistry` already exposes `activate(name)`
and `deactivate(name)` methods. The panel calls these directly. Persistence
of enabled/disabled state uses a new `plugins.disabled: Vec<String>` key
in the session TOML.

### 4. No Plugin Marketplace

Plugin installation from a remote registry is explicitly out of scope
(gap analysis: "future phase after Plugin Manager UI"). The panel only
manages already-loaded plugins.

### 5. Layout

Two-pane layout matching the Files Panel pattern:
- Left pane: scrollable plugin list with filter field at top
- Right pane: detail area for the selected plugin

---

## Module Layout

```
ff-desktop/src/
  plugin_manager_panel.rs   -- PluginManagerPanelState, render()
  shell/state.rs            -- add PluginManagerPanelState field
  shell/commands.rs         -- route "8", "=8", "PLUGINS"
  shell/render.rs           -- dispatch TabKind::PluginManager
  tab_state.rs              -- add PluginManager variant to TabKind
  session_manager.rs        -- persist/restore PluginManager tab + disabled list
```
