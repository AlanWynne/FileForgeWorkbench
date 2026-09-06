# Tasks -- Plugin Manager UI

## Overview

New `TabKind::PluginManager` panel in `ff-desktop`. Reads from the
existing `ff-plugin` registry. No new crate required.

---

## Task 1. TabKind and routing (Req 1.1)

- [x] 1.1 Add `PluginManager` variant to `TabKind` enum in `tab_state.rs`
        - Satisfies: Req 1.1
- [x] 1.2 Add routing in `shell/commands.rs`: `"8"`, `"=8"`, `"PLUGINS"` ->
        open or focus `TabKind::PluginManager`
        - Satisfies: Req 1.1
- [x] 1.3 Add dispatch in `shell/render.rs` `render_central_panel()`:
        `TabKind::PluginManager` -> `plugin_manager_panel::render()`
        - Satisfies: Req 1.1
- [x] 1.4 Wire POM option 8 button to `handle_command("8")`
        - Satisfies: Req 1.1
- [x] 1.5 Write unit tests: `plugin_manager_tab_kind_exists`,
        `option_8_routes_to_plugin_manager`, `plugins_command_routes_to_plugin_manager`
        - Satisfies: Req 1.1

## Task 2. Plugin list rendering (Req 1.2-1.6)

- [x] 2.1 Create `ff-desktop/src/plugin_manager_panel.rs` with
        `PluginManagerPanelState { filter, selected_plugin }` and `render()`
        - Satisfies: Req 1.2
- [x] 2.2 Implement scrollable plugin list: iterate registry, render name,
        version, state badge (Active=green, Inactive=grey, Failed=red),
        capability tags
        - Satisfies: Req 1.2, 1.3, 1.4
- [x] 2.3 Implement filter text field at top of list; filter by name or
        description substring (case-insensitive)
        - Satisfies: Req 1.6
- [x] 2.4 Sort plugin list alphabetically by name
        - Satisfies: Req 1.5
- [x] 2.5 Write unit tests: `plugin_list_sorted_alphabetically`,
        `filter_narrows_plugin_list`, `shutdown_plugin_state_label_is_shutdown`
        - Satisfies: Req 1.4, 1.5, 1.6

## Task 3. Enable/Disable buttons (Req 2)

- [ ] 3.1 Render `Disable` button for Active plugins; on click call
        `registry.deactivate(name)` and update state
        - Satisfies: Req 2.1, 2.2
- [ ] 3.2 Render `Enable` button for Inactive plugins; on click call
        `registry.activate(name)` and update state
        - Satisfies: Req 2.3, 2.4
- [ ] 3.3 On activation failure: display error message in panel, set state
        to Failed, do not crash
        - Satisfies: Req 2.5
- [ ] 3.4 Persist disabled plugin names in session TOML under
        `plugins.disabled: Vec<String>`; restore on startup
        - Satisfies: Req 2.6
- [ ] 3.5 Write unit tests: `disable_active_plugin_calls_deactivate`,
        `enable_inactive_plugin_calls_activate`,
        `failed_activation_sets_failed_state`,
        `disabled_list_round_trips_through_session`
        - Satisfies: Req 2.2, 2.4, 2.5, 2.6

## Task 4. Plugin detail view (Req 3)

- [x] 4.1 Implement two-pane layout: left=list, right=detail area
        - Satisfies: Req 3.1
- [x] 4.2 Render detail area for selected plugin: description, author,
        licence, homepage URL, capabilities list, commands list with shortcuts
        - Satisfies: Req 3.1, 3.3
- [ ] 4.3 Render config keys section: key name, current value, link text
        "View in Settings" that opens SettingsPanel filtered to plugin namespace
        - Satisfies: Req 3.2
- [x] 4.4 Write unit tests: `detail_area_shows_plugin_metadata`,
        `detail_area_shows_commands_with_shortcuts`
        - Satisfies: Req 3.1, 3.3

## Task 5. Session persistence (Req 4)

- [x] 5.1 Add `PluginManager` to session tab kind serialisation in
        `session_manager.rs` and `PersistedTabKind` in `ff-session`
        - Satisfies: Req 4.1
- [x] 5.2 On restore: if session contains PluginManager tab, open it and
        reload registry state
        - Satisfies: Req 4.2
- [x] 5.3 Write unit test: `plugin_manager_tab_round_trips_through_session`
        - Satisfies: Req 4.1

## Task 6. TCR and documentation update

- [x] 6.1 Update `docs/quality/TCR.md` -- add plugin-manager-ui section
        with rows for Req 1.1-1.6, 2.1-2.6, 3.1-3.3, 4.1-4.2
        - Satisfies: project gate requirement
- [x] 6.2 Update `docs/specs/project-master/tasks.md` -- mark CO.5
        complete when all tasks above are [x]
        - Satisfies: project gate requirement
