# Requirements Document

## Introduction

This feature specifies the layout and docking system for FileForgeWorkbench (`ff-layout` crate). The layout system provides dockable panels, tab groups with split views, floating OS-level windows, multi-monitor support, named layout personas (presets), drag-and-drop rearrangement, and full layout serialization. It is part of the Workbench Core -- the GUI shell renders the layout model but does not own it, adhering to the GUI-independence principle (Architecture Brief §3 Principle 1).

The layout system merges the general-purpose docking architecture from FileForgeEditor (panels can be attached/detached from dock zones, multi-monitor placement, persistence) with enhanced workbench concepts: tab groups with horizontal/vertical splits, named personas for rapid workspace switching, and a richer serialization model that supports layout export/import and graceful degradation when plugins are not loaded.

Panels are contributed by the plugin system -- the layout engine itself is panel-agnostic. Any component that implements the `DockablePanel` trait can participate in dock/undock operations, appear in tab groups, float as an independent OS window, and be included in persona configurations.

**Source references:**
- **FFE** = FileForgeEditor `dockable-panels` specification (12 requirements -- all incorporated)
- **WB** = Workbench Architecture Brief §12 Layout Architecture (layout-as-data, personas, GUI independence)

## Glossary

- **Layout_Engine**: The central coordinator within `ff-layout` that owns the layout tree, manages dock zones, tab groups, Detached Workspaces, and orchestrates all layout transitions. Replaces the FFE Dock_Manager with expanded responsibility. [FFE, WB]
- **Dock_Zone**: A designated area within the primary application window where panels can be attached. Standard zones are: left, right, bottom, and center. Each zone can hold one or more panels as tabs. [FFE]
- **Dockable_Panel**: Any UI panel that implements the `DockablePanel` trait, enabling it to be rendered inside a dock zone, within a tab group, or inside its own Detached Workspace. Panels are contributed via the plugin system. [FFE, WB]
- **Detached_Workspace**: A separate OS-level window (platform viewport) containing one or more panels or Workspaces that have been detached from the main application window. Previously called "Detached_Workspace" or "Detached Workspace". Replaces the FFE Undocked_Window concept with support for multi-panel floating containers. [FFE]
- **Panel_Registry**: The registry of all known panel types and their default dock zone assignments. Plugins register panels here during initialization. [FFE, WB]
- **Layout_State**: A serializable snapshot of the complete layout including dock zone contents, tab group arrangement, Detached Workspace positions/sizes, panel visibility, and splitter positions. [FFE, WB]
- **Primary_Window**: The main FileForgeWorkbench application window containing the menu bar, dock zones, tab groups, and docked panels. [FFE]
- **Tab_Group**: A subdivision of the center editor area that holds one or more Workspaces. Multiple tab groups can coexist via horizontal or vertical splits. [WB]
- **Persona**: A named, saved layout configuration (panel visibility, dock positions, tab group arrangement, splitter sizes) that can be activated to switch the entire workspace appearance with a single action. [WB]
- **Drop_Indicator**: A visual overlay shown during drag-and-drop operations that highlights valid dock zones or tab group targets where a panel can be attached. [FFE]
- **Splitter**: A draggable border between adjacent dock zones or tab groups that allows the user to resize the areas. [WB]
- **Layout_File**: A TOML or JSON file on disk that stores a serialized Layout_State for persistence or sharing. [WB]

## Requirements

### Requirement 1: Panel System

**User Story:** As a user, I want a system of dockable panels (file tree, help, output, properties, schema browser, etc.) that can be shown, hidden, and positioned in designated areas of the workbench window, so that I can organize my workspace according to my current task.

**Source:** FFE Reqs 1, 2, 10 -- merged with WB §12 layout-as-data principle. [FFE, WB]

#### Acceptance Criteria

1. WHEN the application starts, THE Layout_Engine SHALL initialize with a default layout containing dock zones for left, right, bottom, and center positions.
2. THE Layout_Engine SHALL maintain a Panel_Registry of all registered Dockable_Panel types and their default dock zone assignments.
3. WHEN a Dockable_Panel is registered with the Panel_Registry, THE Layout_Engine SHALL assign the panel to its default dock zone. IF the specified default dock zone is not one of left, right, bottom, or center, THEN THE Layout_Engine SHALL reject the registration and log an error message indicating the invalid zone name.
4. THE Dockable_Panel trait SHALL define a `panel_id(&self) -> &str` method that returns a unique string identifier (1 to 64 ASCII alphanumeric or underscore characters) for the panel type.
5. THE Dockable_Panel trait SHALL define a `default_dock_zone(&self) -> DockZone` method that returns the preferred dock zone. DockZone SHALL be an enum with variants `Left`, `Right`, `Bottom`, `Center`, and `Floating`.
6. THE Dockable_Panel trait SHALL define a `render(&mut self, ui: &mut egui::Ui)` method for drawing panel content that produces valid output regardless of whether the panel is docked, in a tab group, or displayed in a Detached Workspace.
7. THE Dockable_Panel trait SHALL define a `title(&self) -> &str` method returning the display title (1 to 128 characters).
8. THE Dockable_Panel trait SHALL define an `on_dock_state_changed(&mut self, state: DockState)` method that the Layout_Engine calls when the panel transitions between docked, floating, minimized, or hidden states.
9. WHEN a new type implements the Dockable_Panel trait and registers with the Panel_Registry, THE Layout_Engine SHALL dock, undock, show, hide, minimize, and maximize that panel using only the trait interface -- without code changes to the Layout_Engine.
10. IF a panel is registered with a `panel_id` that already exists in the Panel_Registry, THEN THE Panel_Registry SHALL reject the registration and return an error indicating a duplicate identifier.
11. WHEN the user triggers a show command for a hidden panel, THE Layout_Engine SHALL make the panel visible in its last known dock zone. WHEN the user triggers a hide command, THE Layout_Engine SHALL remove the panel from view while preserving its position in the Layout_State.
12. WHEN the user triggers a toggle command for a panel, THE Layout_Engine SHALL show the panel if it is currently hidden, or hide the panel if it is currently visible.
13. THE Layout_Engine SHALL support three panel display states: minimized (collapsed to a tab/icon in the dock zone header), normal (rendered at its assigned size), and maximized (expanded to fill the entire Primary_Window content area, overlaying other panels temporarily).
14. WHEN a plugin that contributes a panel is loaded, THE plugin system SHALL register the panel with the Panel_Registry via `PluginContext`. WHEN a plugin is unloaded, THE Layout_Engine SHALL remove the panel from display and mark its position as vacant in the Layout_State.

---

### Requirement 2: Tab Groups

**User Story:** As a user, I want the editor area to support multiple tab groups with split views, so that I can view and edit multiple files side-by-side without undocking windows.

**Source:** NEW -- from WB Architecture Brief enhanced layout concepts. [WB]

#### Acceptance Criteria

1. THE Layout_Engine SHALL support subdividing the center dock zone into multiple Tab_Groups arranged via horizontal splits (side-by-side) or vertical splits (stacked top/bottom).
2. WHEN the user triggers a split-horizontal command, THE Layout_Engine SHALL divide the currently active Tab_Group into two side-by-side groups, moving the active tab to the new group and leaving remaining tabs in the original group.
3. WHEN the user triggers a split-vertical command, THE Layout_Engine SHALL divide the currently active Tab_Group into two stacked groups, moving the active tab to the new group and leaving remaining tabs in the original group.
4. THE user SHALL be able to move tabs between Tab_Groups via drag-and-drop: dragging a tab header from one group and dropping it onto another group's tab bar SHALL relocate the tab to the target group.
5. WHEN a tab is the last tab in a Tab_Group and is moved to another group, THE Layout_Engine SHALL automatically close the now-empty Tab_Group and redistribute its space to adjacent groups.
6. THE boundary between adjacent Tab_Groups SHALL be a draggable splitter handle that allows the user to resize the relative proportions of the groups.
7. THE Layout_Engine SHALL support a minimum Tab_Group size of 100 logical pixels in the split direction; dragging the splitter beyond this minimum SHALL NOT reduce the group further.
8. THE Layout_Engine SHALL preserve Tab_Group arrangement (split direction, proportions, tab order within each group) as part of the Layout_State for serialization.
9. WHEN a new file is opened, THE Layout_Engine SHALL add the tab to the currently active Tab_Group unless the open command specifies a target group.

---

### Requirement 3: Detached Workspaces

**User Story:** As a user, I want to undock panels or tabs into separate OS-level windows, so that I can arrange my workspace across multiple monitors or view content side-by-side independently.

**Source:** FFE Reqs 2, 3, 4, 5 -- adapted for workbench multi-panel floating containers. [FFE, WB]

#### Acceptance Criteria

1. WHEN the user triggers an undock action on a docked panel (via context menu, keyboard shortcut, or drag gesture), THE Layout_Engine SHALL remove the panel from its dock zone and create a Detached_Workspace containing the panel.
2. WHEN a panel is undocked, THE Detached_Workspace SHALL appear at a position offset from the Primary_Window by (50 × N) pixels right and (50 × N) pixels down, where N is the number of currently Detached Workspaces (starting at 1), with an initial size matching the panel's docked dimensions.
3. WHILE a panel is in a Detached_Workspace, THE panel SHALL render with full interactivity identical to its docked state, including all input handling, context menus, command dispatching, and keyboard shortcuts.
4. WHEN a panel is undocked, THE Layout_Engine SHALL update the Layout_State to reflect the panel's floating status and window position within 500 milliseconds.
5. WHEN the user triggers a redock action on a Detached_Workspace (via context menu, keyboard shortcut, or drag-to-dock gesture), THE Layout_Engine SHALL close the Detached_Workspace and reattach the panel to its most recent dock zone.
6. WHEN a panel is redocked, THE Layout_Engine SHALL restore the dock zone to its previous width or height if it was collapsed when the panel was undocked.
7. IF the most recent dock zone already contains another panel, THEN THE Layout_Engine SHALL stack the returning panel as a tab within that dock zone.
8. WHEN a Detached_Workspace is closed via the OS window close button, THE Layout_Engine SHALL redock the panel to its most recent dock zone rather than destroying the panel.
9. WHEN the user drags a tab header beyond 20 pixels outside the tab bar boundary and releases it outside the Primary_Window, THE Layout_Engine SHALL undock that tab into a new Detached_Workspace positioned at the mouse release coordinates.
10. WHILE a tab is in a Detached_Workspace, THE tab SHALL provide full editing functionality identical to the Primary_Window editor area, including command line, syntax highlighting, status bar, and all keyboard shortcuts.
11. WHEN an undocked tab's Detached_Workspace is closed via the OS window close button, THE Layout_Engine SHALL redock the tab back into the Tab_Group it originated from at its original position index; IF that index exceeds the current tab count, THEN THE tab SHALL be appended at the end.
12. IF a tab has unsaved modifications when the user attempts to close its Detached_Workspace, THEN THE Layout_Engine SHALL show the standard save confirmation dialog (Save, Discard, Cancel) before redocking; selecting Cancel SHALL abort the close.
13. Detached_Workspaces are full OS-level windows (platform viewports), not in-app overlays. They SHALL appear in the OS taskbar/dock and SHALL be independently movable, resizable, minimizable, and maximizable.
14. THE Layout_Engine SHALL support up to 16 simultaneous Detached_Workspaces; IF the user attempts to create an additional window beyond this limit, THEN THE Layout_Engine SHALL display a status message indicating the maximum has been reached and SHALL NOT undock the panel.
15. IF the operating system fails to create a Detached_Workspace, THEN THE Layout_Engine SHALL leave the panel in its current position and display a status message indicating the window could not be created.
16. Detached_Workspace state (position, size, contained panels) SHALL be persisted as part of the Layout_State.

---

### Requirement 4: Multi-Monitor Support

**User Story:** As a user, I want Detached Workspaces to work correctly across multiple monitors with different DPI settings, so that I can spread my workspace across my entire display setup.

**Source:** FFE Req 9 -- enhanced with per-monitor DPI handling. [FFE, WB]

#### Acceptance Criteria

1. WHEN a Detached_Workspace is created, THE Layout_Engine SHALL position the window at the requested coordinates regardless of which connected monitor contains those coordinates.
2. WHEN the user moves a Detached_Workspace to a different monitor, THE Layout_Engine SHALL update the window's recorded monitor identifier in the Layout_State.
3. THE Layout_State serialization SHALL include a monitor identifier for each Detached_Workspace to enable correct restoration on multi-monitor setups.
4. WHILE a Detached_Workspace is positioned on a monitor, THE window SHALL render using the DPI scale factor reported by the operating system for that specific monitor.
5. WHEN a Detached_Workspace is moved from one monitor to another with a different DPI scale factor, THE window SHALL adjust its rendered content to match the target monitor's DPI while preserving logical dimensions.
6. IF a monitor is disconnected while one or more Detached_Workspaces are displayed on it, THEN THE Layout_Engine SHALL move the affected windows to the center of the primary monitor's work area, preserving each window's logical size.
7. WHEN restoring persisted window positions at startup, IF a window's target position refers to a monitor that is no longer connected, THEN THE Layout_Engine SHALL reposition that window to the center of the primary monitor's work area.
8. WHEN restoring persisted window positions at startup, IF a window's position would result in less than 50% of the window being visible within any connected monitor's bounds, THEN THE Layout_Engine SHALL reposition that window to the center of the primary monitor's work area.

---

### Requirement 5: Personas (Layout Presets)

**User Story:** As a user, I want named layout configurations (personas) that I can switch between instantly, so that I can adapt my workspace to different tasks (editing, debugging, data analysis, database work) without manually rearranging panels each time.

**Source:** NEW -- from WB Architecture Brief §12 layout files concept. [WB]

#### Acceptance Criteria

1. THE Layout_Engine SHALL support named Persona configurations, each defining: panel visibility per panel_id, dock zone assignments, Tab_Group arrangement (splits and proportions), Detached_Workspace positions and sizes, and splitter positions.
2. THE workbench SHALL provide built-in personas including at minimum: "Editor Focus" (minimal panels, maximized editor area), "Debug" (output and variable panels visible), "FileForge" (file tree and structure panels prominent), and "Database" (schema browser, SQL editor, result grid visible).
3. THE user SHALL be able to create custom personas by saving the current Layout_State under a user-chosen name via a save-persona command.
4. WHEN the user activates a persona (via command palette, keyboard shortcut, or menu), THE Layout_Engine SHALL transition the layout to match the persona's configuration within 500 milliseconds.
5. WHEN switching personas, THE Layout_Engine SHALL NOT close, discard, or lose any open documents or their unsaved state. All open editor tabs SHALL remain in the Tab_Group structure defined by the target persona; IF the target persona defines fewer Tab_Groups than currently open tabs require, THEN THE excess tabs SHALL be placed in the last available group.
6. THE user SHALL be able to delete custom personas. Built-in personas SHALL NOT be deletable but MAY be overridden by a custom persona with the same name.
7. Persona definitions SHALL be stored as individual TOML files in the `layouts/` directory (e.g., `layouts/editor-focus.toml`, `layouts/debug.toml`).
8. IF a persona references a panel_id that is not currently registered (plugin not loaded), THEN THE Layout_Engine SHALL skip that panel entry and apply the remainder of the persona configuration without error.
9. THE Layout_Engine SHALL track which persona is currently active and display its name in the status bar or a designated UI indicator.
10. WHEN the user modifies the layout while a persona is active (moves a panel, changes a split), THE Layout_Engine SHALL mark the persona as "modified" in the UI indicator. The user MAY update the persona to capture the changes or revert to the saved persona state.

---

### Requirement 6: Layout Serialization

**User Story:** As a user, I want the workbench to remember my layout between sessions and allow me to export/import layouts, so that my workspace arrangement is never lost and can be shared with colleagues.

**Source:** FFE Req 8 -- enhanced with export/import, graceful degradation, and reset-to-default. [FFE, WB]

#### Acceptance Criteria

1. WHEN the application exits normally, THE Layout_Engine SHALL serialize the current Layout_State to the session file at `config/layout_state.toml`.
2. WHEN the application starts and a valid `config/layout_state.toml` exists, THE Layout_Engine SHALL restore the layout from the persisted state, including dock zone contents, Tab_Group arrangement, Detached_Workspace positions, splitter sizes, and panel visibility.
3. IF the persisted `config/layout_state.toml` is missing, fails to parse as valid TOML, or has a schema version mismatch, THEN THE Layout_Engine SHALL fall back to the default layout and log a warning indicating the reason.
4. THE Layout_State SHALL include for each docked panel: panel_id, dock zone assignment, and zone width/height in logical pixels. For each floating panel: panel_id, window position (x, y), window size (width, height) with minimum 200×150 logical pixels, and monitor identifier. For each Tab_Group: split direction, proportional size, and ordered list of tab identifiers.
5. IF a persisted Layout_State references a panel_id that is not currently registered in the Panel_Registry (plugin not loaded), THEN THE Layout_Engine SHALL omit that panel from the restored layout and log an INFO-level message, without failing or blocking the layout restoration.
6. WHEN the user triggers a layout-export command, THE Layout_Engine SHALL serialize the current Layout_State to a user-specified file path in TOML format.
7. WHEN the user triggers a layout-import command with a valid layout file, THE Layout_Engine SHALL apply the imported Layout_State, subject to the same graceful degradation rules (missing panels skipped) as startup restoration.
8. WHEN the user triggers a layout-reset command, THE Layout_Engine SHALL discard the current Layout_State and restore the built-in default layout.
9. WHEN a Detached_Workspace is moved or resized, THE Layout_Engine SHALL update the in-memory Layout_State within 500 milliseconds of the operation completing.
10. IF the Layout_Engine fails to write `config/layout_state.toml` at exit (due to permission error, disk full, or I/O failure), THEN THE Layout_Engine SHALL log a warning and allow exit to proceed without blocking shutdown.
11. THE Layout_State serialization format SHALL include a schema version number to enable forward-compatible migration of layout files across application versions.

---

### Requirement 7: Drag-and-Drop

**User Story:** As a user, I want to rearrange panels and tabs by dragging them to dock zones, tab groups, or outside the window to float, so that I can intuitively organize my workspace.

**Source:** FFE Reqs 6, 7 -- enhanced with tab group drop targets. [FFE, WB]

#### Acceptance Criteria

1. WHEN the user initiates a drag on a Detached_Workspace's title bar and moves it over a valid dock zone in the Primary_Window, THE Layout_Engine SHALL display a Drop_Indicator highlighting the target zone.
2. WHEN the user releases the drag over a valid dock zone with a visible Drop_Indicator, THE Layout_Engine SHALL dock the panel into that zone and close the Detached_Workspace.
3. WHEN the user releases the drag outside any valid dock zone or tab group, THE Layout_Engine SHALL leave the panel in its Detached_Workspace at the release position.
4. THE Drop_Indicator SHALL render as a semi-transparent overlay with a distinct border color covering the target dock zone or tab group area.
5. WHEN a drag enters a dock zone or tab group drop target, THE Drop_Indicator SHALL appear within 16 milliseconds (one frame at 60 FPS).
6. WHEN a drag leaves a dock zone without release, THE Drop_Indicator SHALL disappear immediately.
7. WHEN the user drags a tab header from one Tab_Group and drops it onto another Tab_Group's tab bar area, THE Layout_Engine SHALL move the tab to the target group at the insertion index determined by the horizontal drop position.
8. WHEN the user drags a docked panel header and drops it onto a different dock zone, THE Layout_Engine SHALL move the panel to the target zone.
9. WHEN the user drags a panel or tab outside the Primary_Window boundaries and releases, THE Layout_Engine SHALL undock the item into a new Detached_Workspace at the release coordinates (drag-to-float).
10. WHEN the user drags a Detached_Workspace's title bar and drops it onto a valid dock zone (drag-to-dock), THE Layout_Engine SHALL dock the panel into that zone and close the Detached_Workspace.
11. WHEN the user drags a tab header vertically more than 30 pixels away from the tab bar, THE Layout_Engine SHALL begin a tear-off preview, displaying the tab as a floating thumbnail anchored at the cursor position.
12. WHEN the user releases a torn-off tab back within 30 pixels of a tab bar, THE Layout_Engine SHALL cancel the tear-off and reorder the tab at the insertion index determined by horizontal cursor position.
13. WHILE a drag-to-dock operation is in progress, THE Primary_Window SHALL highlight all valid dock zones with a visible border of at least 2 pixels in a color distinct from the zone's default appearance.

---

### Requirement 8: Resizing

**User Story:** As a user, I want to resize panels and tab groups by dragging their borders, so that I can allocate screen space according to my current needs.

**Source:** NEW -- enhanced layout management for workbench. [WB]

#### Acceptance Criteria

1. THE boundary between a dock zone and the center editor area SHALL be a draggable splitter handle that the user can move to resize the adjacent areas.
2. THE boundary between adjacent Tab_Groups SHALL be a draggable splitter handle for resizing the relative proportions of the groups.
3. EACH panel SHALL define a minimum size constraint (width and height in logical pixels). THE Layout_Engine SHALL NOT allow a splitter drag to reduce any panel below its declared minimum size.
4. IF no minimum size is declared by a panel, THE Layout_Engine SHALL enforce a default minimum of 48 logical pixels in both dimensions.
5. WHEN the Primary_Window is resized by the user or the operating system, THE Layout_Engine SHALL resize all dock zones and Tab_Groups proportionally according to their current relative sizes, subject to minimum size constraints.
6. IF proportional resizing would violate a minimum size constraint, THEN THE Layout_Engine SHALL prioritize the center editor area and reduce side/bottom panels first, collapsing them to their minimum size before reducing the editor area.
7. THE Layout_Engine SHALL persist splitter positions (as proportional values between 0.0 and 1.0) in the Layout_State, restoring them on next application start.
8. WHEN the user double-clicks a splitter handle, THE Layout_Engine SHALL reset that splitter to its default proportional position as defined by the active persona or built-in default.
9. WHILE the user is dragging a splitter, THE Layout_Engine SHALL provide real-time visual feedback by rendering both sides at their new sizes on each frame (no deferred resize).

---

### Requirement 9: Keyboard Shortcuts

**User Story:** As a user, I want keyboard shortcuts for common layout operations (dock/undock, split, persona switch), so that I can manage my workspace efficiently without the mouse.

**Source:** FFE Req 11 -- adapted for workbench command-framework integration. [FFE, WB]

#### Acceptance Criteria

1. WHEN the user presses Ctrl+Shift+D while a Dockable_Panel has keyboard focus, THE Layout_Engine SHALL toggle the focused panel between docked and floating states.
2. IF the user presses Ctrl+Shift+D and no Dockable_Panel currently has keyboard focus, THEN THE Layout_Engine SHALL take no action.
3. WHEN the user presses Ctrl+Shift+T while an editor tab is the active tab in the Primary_Window, THE Layout_Engine SHALL undock the active tab into a new Detached_Workspace.
4. IF the user presses Ctrl+Shift+T while the active tab is the only tab in the only Tab_Group, THEN THE Layout_Engine SHALL take no action (preventing an empty editor area).
5. WHEN the user presses Ctrl+Shift+T while focus is in a Detached_Workspace containing a tab, THE Layout_Engine SHALL redock the tab back to its originating Tab_Group.
6. ALL layout keyboard shortcuts SHALL be registered with the command-framework's shortcut registry to prevent conflicts and enable user remapping via the key map system.
7. WHEN the user invokes a persona-switch command (via keyboard shortcut or command palette), THE Layout_Engine SHALL activate the specified persona following Requirement 5 criteria.
8. WHEN the user invokes a split-horizontal or split-vertical command via keyboard shortcut, THE Layout_Engine SHALL split the active Tab_Group following Requirement 2 criteria.

---

### Requirement 10: Visual Feedback and Indicators

**User Story:** As a user, I want clear visual indicators showing panel states, drop targets, and active persona, so that I can understand and control my workspace layout at all times.

**Source:** FFE Req 12 -- adapted for workbench personas and tab groups. [FFE, WB]

#### Acceptance Criteria

1. WHILE a panel is floating, THE Primary_Window SHALL display a placeholder indicator in the panel's former dock zone showing the panel name and a clickable "redock" button.
2. WHEN the user hovers over the placeholder indicator for at least 300 milliseconds, THE Primary_Window SHALL display a tooltip showing "Click to redock [panel name]" and the associated keyboard shortcut.
3. WHEN the user clicks the placeholder indicator's redock button, THE Layout_Engine SHALL redock the associated panel following the same behavior defined in Requirement 3 criterion 5.
4. THE Detached_Workspace title bar SHALL display the panel title followed by " -- FileForge", truncated to a maximum of 80 characters if necessary.
5. THE status bar or a designated UI region SHALL display the name of the currently active persona, with a "modified" indicator when the user has changed the layout from the persona's saved state.
6. WHEN the user drags a panel or tab and enters a valid drop zone, THE Layout_Engine SHALL display a Drop_Indicator showing exactly where the item will be placed upon release (left/right/top/bottom split, or tab insertion point).
7. WHILE a panel is minimized, THE dock zone header SHALL display an icon or label for the panel that the user can click to restore it to normal state.


---

### Requirement 11: Tab Window Chrome in Detached Workspaces

**User Story:** As a user, I want Detached Workspaces to show the same Title_Line and
Command Field chrome as the docked tab, so that the experience is consistent whether a
tab is in the main window or detached independently.

**Source:** menu-and-statusbar Requirement 17 and 18; user requirement (Phase AL).

#### Acceptance Criteria

1. WHEN a tab is displayed in a Detached Workspace (Detached_Workspace), THE Detached Workspace SHALL render the full Tab_Window_Chrome: Tab_Header row, Title_Line, and Primary_Command_Field -- in that order at the top of the window, above the tab's content area.

2. THE Title_Line in a Detached Workspace SHALL display the same context-dependent text as it would when the tab is docked (per menu-and-statusbar Requirement 17.3–17.6).

3. THE Primary_Command_Field in a Detached Workspace SHALL be fully functional: it SHALL accept keyboard focus on window activation, accept typed commands, and dispatch them through the same CommandEngine as the Primary_Window Command Field.

4. WHEN the Legacy theme is active, THE Title_Line in a Detached Workspace SHALL use the same blue background / white text styling as the docked Title_Line (menu-and-statusbar Requirement 17.8).

5. THE Detached Workspace title bar (OS chrome) SHALL display the Title_Line content followed by " -- FileForge Workbench" (menu-and-statusbar Requirement 18.5).


---

### Requirement 12: Shell Layout Tree Foundation (Slice 2a)

**User Story:** As a workbench maintainer, I want the shell's tab management to be backed by a
layout tree (starting as a single group) instead of a flat list, so that in-window splits can be
added later without a pipeline rewrite -- and I want this foundation to change NOTHING that the
user can observe, so it is safe to land before any visible split feature.

**Source:** [B046] Slice 2a; owner: "creating tab groups is a great idea ... build the invisible
layout-tree foundation first, alone, behaviour-identical." Wires the existing but unused
`ff-layout::TabGroupTree` (Requirement 2) into `ff-desktop`.

**Design note (three-layer model):** the shell separates (1) a TAB STORE (owns `TabState`s by
`TabId`), (2) a LAYOUT TREE (`ff-layout::TabGroupTree`: `Leaf(TabGroup)` regions and `Split`
nodes with a `SplitDirection` and relative `proportion`), and (3) a FOCUS MODEL (which Tab_Group
is focused; "the active tab" = the focused group's active tab). Slice 2a delivers layers 1-3 with
the tree constrained to a SINGLE leaf, so it is behaviour-identical to today's flat model. The
visible split (multiple leaves, splitter render, focus routing, split command) is Slice 2b.

**Glossary:**
- **Tab_Group**: A leaf region of the layout tree holding an ordered list of tabs (by `TabId`) and
  the index of its active tab. In Slice 2a there is exactly one.
- **Layout_Tree**: The shell's `TabGroupTree` instance modelling how Tab_Groups are split. In Slice
  2a it is always a single `Leaf`.
- **Focused_Group**: The Tab_Group whose active tab is "the active tab" the command/render/focus
  pipeline operates on. In Slice 2a it is always the sole group.

#### Acceptance Criteria

1. THE `ff-desktop` shell SHALL depend on the `ff-layout` crate and hold a `TabGroupTree` as the
   authoritative model of Tab_Group arrangement. On startup and after any operation in Slice 2a,
   the tree SHALL be a single `Leaf` Tab_Group containing all open tabs in their existing order.

2. THE `TabManager` SHALL retain a single flat store of `TabState`s keyed by stable `TabId`
   (identity + content), UNCHANGED by this slice; the layout tree SHALL reference tabs by `TabId`,
   not by cloning `TabState`.

3. THE `TabManager` SHALL maintain a `Focused_Group` identifier. WHEN the layout tree is a single
   `Leaf` (always, in Slice 2a), the Focused_Group SHALL be that leaf.

4. THE existing `TabManager` accessors `active_tab()`, `active_tab_mut()`, and `active_index()`
   SHALL be RESOLVED THROUGH the Focused_Group's active tab (Focused_Group -> active `TabId` ->
   store), and SHALL return EXACTLY the same tab they return today for every single-group layout.
   No caller of these accessors SHALL require modification in Slice 2a.

5. EVERY tab-lifecycle operation (`open_file`, `new_untitled_tab`, the per-kind openers,
   `close_tab`, `remove_at`, `insert_at`, `move_tab`, `set_active`, `previous_active_index`) SHALL
   keep the layout tree's single Tab_Group consistent with the store (same tab set, same order,
   same active tab, same previous-active semantics) so that all existing behaviour -- including
   detach/redock (menu-and-statusbar Req 18) and bare-SWAP toggle (multi-tab-editor Req 18.7) --
   is preserved bit-for-bit.

6. THE layout tree model SHALL be serialization-ready (it reuses `ff-layout`'s serde-derived
   `TabGroupTree`), but Slice 2a SHALL NOT change the on-disk session format: session save/restore
   SHALL continue to persist the flat tab list exactly as today (the single-group tree is implied).
   Persisting the tree is deferred to a later slice.

7. Slice 2a SHALL introduce NO user-visible change: NO split command, NO splitter rendering, NO new
   key binding, NO new menu entry, and NO change to the tab bar, Title_Line, Command Field, focus
   order, or any panel. This requirement is satisfied only if the FULL existing test suite passes
   unchanged and a manual smoke test shows identical behaviour.

8. THE foundation SHALL be covered by unit tests proving the single-leaf invariant and the
   resolve-through-focused-group equivalence: after each lifecycle operation, the tree is a single
   `Leaf` whose tab order and active tab match `TabManager`'s store, and `active_tab()` returns the
   focused group's active tab.


---

### Requirement 13: Visible In-Window Split (Slice 2b)

**User Story:** As a user, I want to split the workbench's Workspace area into two side-by-side (or
stacked) regions so I can see two Workspaces at once -- like VS Code editor groups or ISPF
split-screen -- without detaching a window.

**Source:** [B046] Slice 2b; owner: "creating tab groups is a great idea and extends the product
very nicely." Builds on Requirement 12 (Slice 2a layout-tree foundation) and realises Requirement 2
(Tab_Groups). Fulfils the deferred menu-and-statusbar Req 19.11-19.14 (CR-CH-040 reserved `SPLIT`).

**Design note:** the split is rendered by walking the `TabGroupTree` and drawing each `Leaf`
Tab_Group's focused tab through the EXISTING `render_active_tab_body` (the same path detached
windows reuse), inside the region the tree allocates. The FOCUSED Tab_Group (Slice 2a focus model)
is where the command line, keys, and new-tab opens act. Slice 2b is limited to EXACTLY ONE split
(two leaves); recursive nesting, drag-tab-between-groups, and session persistence are Slice 2c.

**Glossary:**
- **Split**: A `TabGroupTree::Split { direction, proportion, first, second }` node dividing the area
  into two child regions at a relative `proportion` in [0,1].
- **Splitter**: The draggable boundary between the two regions that adjusts `proportion`.
- **Focused_Group**: The Tab_Group (leaf) that receives command/key/new-tab input (Requirement 12.3),
  visually indicated.

#### Acceptance Criteria

1. THE shell SHALL provide a `SPLIT` command that divides the Focused_Group into two Tab_Groups: a
   `TabGroupTree::Split` node whose first child keeps the group's tabs and whose second child is a
   new empty (or active-tab-cloned -- see 13.3) Tab_Group. `SPLIT` and `SPLIT RIGHT` SHALL use
   `SplitDirection::Horizontal` (side-by-side); `SPLIT DOWN` SHALL use `SplitDirection::Vertical`
   (stacked). The default proportion SHALL be 0.5. (Command parity: menu/keys invoke the command.)

2. WHEN there is already one split (two leaves) and `SPLIT` is issued again, THE shell SHALL show a
   non-blocking status message that only one split is supported in this slice and SHALL NOT create a
   nested split (Slice 2b scope: exactly one split). Recursive nesting is Slice 2c.

3. WHEN a `SPLIT` creates the second Tab_Group, THE new group SHALL open showing the Home Context
   (POM) by default so the user chooses what to place there (ISPF-style), UNLESS the command
   specifies otherwise. (The exact new-group Context MAY be refined during design; the default is a
   usable Workspace, never an empty non-interactive region.)

4. THE shell SHALL render BOTH Tab_Groups simultaneously: each region draws its focused tab's real
   Context via the existing `render_active_tab_body`, with its OWN tab bar showing that group's tabs.
   The two regions SHALL be separated by a draggable Splitter.

5. THE Splitter SHALL adjust the split `proportion` by dragging, honouring a minimum region size of
   `MIN_TAB_GROUP_SIZE` (100 logical pixels) in the split direction; dragging beyond the minimum
   SHALL NOT shrink a region further. Sizing SHALL be RELATIVE (proportion), never pixel-absolute.

6. THE Focused_Group SHALL be visually indicated (e.g. an active-border/title highlight) so the user
   can see which region has the keyboard. Exactly one group is focused at a time.

7. THE shell SHALL provide a command/key to move focus between the two Tab_Groups (framework verb,
   e.g. `FOCUS NEXT` / `FOCUS OTHER`, and/or a default key binding). Moving focus SHALL update the
   Focused_Group so subsequent commands, keys, and new-tab opens act on the newly focused group.

8. WHILE split, the command line, function keys, and new-tab/open operations SHALL act on the
   Focused_Group's active tab (resolved through the Slice 2a focus model), NOT on the other group.

9. THE shell SHALL provide `UNSPLIT` (and END/close semantics on a split, per menu-and-statusbar Req
   19.14) that collapses the split back to a single Tab_Group, preserving the surviving group's tabs
   and active tab; the tree returns to a single `Leaf`. Closing the last tab of one group SHALL also
   collapse the split (the surviving group becomes the whole area), reusing
   `TabGroupTree::remove_empty_groups`.

10. Slice 2b SHALL NOT change session persistence: a split layout is NOT persisted yet (on restart
    the workbench opens unsplit); `LayoutState` serialization is Slice 2c. All existing single-group
    behaviour (no split active) SHALL remain identical to Slice 2a.

11. THE split model operations (create split, collapse/unsplit, move focus, adjust proportion,
    resolve the focused group's active tab) SHALL be covered by unit tests on the tree + focus
    model; the rendered two-region behaviour (both regions drawn, focused-group highlight, splitter
    present, command acts on focused group) SHALL have an `egui_kittest` full-shell test per the GUI
    Behaviour Testing rule.

### Requirement 14: Split Rework Slice 2c -- Recursive Nesting, Drag-Move, Persistence, Detached Fold-in

**User Story:** As a user, I want to split any region again (not just once), drag a tab from one
region into another, have my split layout survive a restart, and treat a detached window as just
another region of the same workbench -- so the split feature is complete, not a one-shot.

**Source:** [B046] Slice 2c; owner: "full set." Completes the four capabilities Slice 2b deferred
(Requirement 13.2 "one split only" and 13.10 "not persisted"). Builds on Requirement 13 (Slice 2b
visible split) and Requirement 12 (Slice 2a layout-tree foundation).

**Design note:** the `ff-layout::TabGroupTree` is ALREADY a recursive, serde-serialisable binary
tree with `remove_empty_groups`. Slice 2c makes that tree (not the Slice 2b hardcoded two-group
`SplitState`) the authoritative split model, and reuses the EXISTING `SessionState.layout:
Option<LayoutSnapshot>` slot (documented for "tab groups, splitters, persona") for persistence. The
unsplit path MUST remain byte-identical to Slice 2a/2b (a session with no layout, and a workbench
with no split, behave exactly as before). Delivered in four internally-gated sub-slices
(2c.1 nesting, 2c.2 drag-move, 2c.3 persistence, 2c.4 detached fold-in), each shippable alone.

**Glossary (additions to Requirement 13):**
- **Nested_Split**: a `TabGroupTree::Split` whose `first` or `second` child is itself a `Split`
  (arbitrary depth).
- **Drop_Zone**: a target region highlighted during a tab-header drag; dropping there moves the tab
  into that region's Tab_Group (an edge Drop_Zone MAY create a new split around the target).
- **Layout_Snapshot**: the persisted split arrangement (tree shape + direction + proportion per
  internal node, per-leaf tab ids + active index, focused leaf id) stored in `SessionState.layout`.
- **Focus_Context**: the unified abstraction over an in-window Focused_Group and a Detached_Workspace
  -- both a "place a command/keys/new-tab act on," reached through the same active-tab-swap path.
  *(CR-CH-041 reframing: a Focus_Context is a PLACEMENT of a Workspace instance -- the region leaf or
  the OS window the instance is currently drawn in -- NOT a chrome owner. The command line, menu bar,
  and keylist that act there belong to the INSTANCE, not to the place; see Requirement 16.)*

#### Acceptance Criteria -- 2c.1 Recursive nesting

1. WHEN `SPLIT` / `SPLIT RIGHT` / `SPLIT DOWN` is issued while the Workspace is ALREADY split, THE
   shell SHALL split the FOCUSED Tab_Group again (replacing that leaf with a new `Split` node whose
   first child is the focused group and whose second child is a new POM group), to ARBITRARY depth --
   removing the Slice 2b "one split only" rejection (Requirement 13.2 is superseded for 2c).

2. THE authoritative split model SHALL be the `ff-layout::TabGroupTree` itself (not a fixed
   two-element structure); `TabManager` SHALL maintain the tree and derive per-leaf membership from
   it, so any number of leaves at any depth is representable. The flat `TabState` store keyed by
   `TabId` SHALL remain authoritative for tab CONTENT (Requirement 12.2); the tree owns ARRANGEMENT.

3. WHEN a leaf's last tab closes, or `UNSPLIT`/END collapses the focused leaf, THE shell SHALL
   collapse exactly that leaf via `remove_empty_groups`, preserving every other leaf and its
   proportion; when only one leaf remains the tree returns to a single `Leaf` (unsplit).

4. THE render SHALL walk the full tree recursively, drawing a draggable Splitter at EACH internal
   `Split` node and each `Leaf` as a region with its own tab bar (Requirement 13.4 generalised to
   arbitrary depth); exactly one leaf is the Focused_Group (Requirement 13.6).

5. `FOCUS` / `FOCUS OTHER` SHALL move focus among ALL leaves (a defined traversal order, e.g.
   left-to-right/top-to-bottom leaf order), not just two (Requirement 13.7 generalised).

#### Acceptance Criteria -- 2c.2 Drag a tab between groups

6. THE user SHALL be able to drag a Tab_Header out of its region's tab bar and drop it onto another
   region; on drop THE shell SHALL move that `TabState` from the source Tab_Group to the target
   Tab_Group (by `TabId`; content unchanged), make it the target's active tab, and focus the target.

7. WHEN the source Tab_Group becomes empty after a move, THE shell SHALL collapse it via
   `remove_empty_groups` (Requirement 14.3), so a drag-out that empties a region removes that region.

8. Dragging a tab beyond the workbench (the existing >20px-outside-bar gesture, Requirement 13/18.6)
   SHALL still DETACH (create a Detached_Workspace), unchanged; the in-window move is the NEW
   behaviour for a drop ONTO another region. A drop onto the tab's OWN region SHALL be a no-op.

9. THE drag SHALL show a Drop_Zone highlight on the region under the pointer so the target is
   unambiguous before release.

#### Acceptance Criteria -- 2c.3 Split persistence

10. ON exit, THE shell SHALL serialise the current split arrangement (tree shape, per-node direction
    and proportion, per-leaf tab ids + active index, focused leaf id) into
    `SessionState.layout` (the existing `LayoutSnapshot`), alongside the flat tab list.

11. ON launch, WHEN a persisted `layout` is present and its referenced tab ids resolve against the
    restored tab store, THE shell SHALL rebuild the split tree (reconciling any missing/extra ids as
    `sync_layout` already does) and open split as saved; the focused leaf SHALL be restored.

12. WHEN no `layout` is persisted (older session, or a workbench that was not split), THE shell SHALL
    open UNSPLIT exactly as Slice 2a/2b (Requirement 13.10). The addition SHALL be backward-compatible
    (`#[serde(default)]`, NO schema-version bump); an older `session.toml` SHALL load without error.

13. THE persisted layout SHALL be self-consistent on restore: a tab id in the layout that no longer
    exists in the tab store SHALL be dropped; a tab in the store not referenced by the layout SHALL
    be placed in the focused (or first) leaf, so no tab is lost and no dangling id remains.

#### Acceptance Criteria -- 2c.4 Detached fold-in

14. THE shell SHALL model an in-window Focused_Group and a Detached_Workspace through ONE
    Focus_Context abstraction: the SAME code path that swaps the active tab + per-window command
    context for a region SHALL serve a detached window (generalising `with_workspace_context`), so
    command/key/new-tab dispatch is defined once for both.

15. THE existing Detached_Workspace behaviour (Requirement 18: detach via drag-out / "Move to Other
    View" / `SPLIT DETACH`; `DOCK` re-attach; the 16-window limit; per-window command line; F-keys
    act on the focused window) SHALL remain intact after the fold-in (no regression).

16. WHEN a Detached_Workspace is re-docked (`DOCK`), THE shell SHALL re-attach its tab into a
    Tab_Group of the tree (its origin leaf when still present, else the focused leaf), consistent
    with the recursive model rather than a flat origin index.

#### Acceptance Criteria -- coverage

17. ALL 2c model operations (nested split/collapse, move-tab-between-groups, layout snapshot
    to/from tree, focus traversal over N leaves, detached fold-in swap) SHALL have unit tests; the
    rendered behaviour (recursive regions drawn, drag-drop move, focused highlight at depth, restore
    from a persisted layout) SHALL have full-shell `egui_kittest` tests per the GUI Behaviour Testing
    rule. The drag GESTURE pixels and the OS detached-window chrome remain justified-MANUAL rows.

### Requirement 15: Per-Region Command Lines for Split Tab_Groups

**User Story:** As a user, when the Workspace is split I want EACH region to have its own
`Command ===>` line so I can run a command against a specific region directly, without first moving
focus, and so it is unambiguous which region a command applies to.

**Source:** [B046] Slice 2d; owner: "each split window should have its own command line relevant to
that workspace." Deferred out of CR-NR-093 (design.md 2c.4 "out of scope: per-region command
lines"). Builds on Requirement 13/14 (visible split) and reuses menu-and-statusbar Requirement 18.10
(a Detached_Workspace already has its own independent command context).

**Design note:** the machinery already exists -- a `WorkspaceCommandContext` (command_text, scroll,
status/open_error, focus + outcome latches) per window, swapped into the shell via
`with_workspace_context(tab_index, &mut ctx, |shell| ...)` so the UNCHANGED command pipeline runs
against that window's tab. This requirement gives each in-window split LEAF its own
`WorkspaceCommandContext` and renders a per-region command field, dispatched through the same path.

**CR-CH-041 reframing (ownership, not mechanism):** the criteria below say "each region has its own
command line," which was the framing before the instance/region/placement model was locked. Under
CR-CH-041 the command line is owned by the WORKSPACE INSTANCE and rendered at the instance's
PLACEMENT (the region leaf it currently occupies, or its detached window); a region does NOT own a
command line -- it hosts the instance whose command line is drawn there. The mechanism is unchanged
(one `WorkspaceCommandContext` per placed instance, dispatched via `with_workspace_context`), and
every criterion 15.1-15.9 still holds as written when "the region's command line" is read as "the
command line of the instance placed in that region." Requirement 16 states the general principle;
this note records that Requirement 15 is a specialisation of it, not a competing model. In
particular, per CR-CH-041 the instance also renders its menu bar and Title_Line in the same region
(Requirement 16), so a split region shows the placed instance's FULL chrome, not just its command
line.

**Glossary (additions to Requirement 13/14):**
- **Region_Command_Line**: the `Command ===>` field rendered inside a split region, bound to that
  region's `WorkspaceCommandContext`.
- **Region_Context**: a split leaf's `WorkspaceCommandContext` (its own command_text, SCROLL,
  status, focus/outcome latches), the in-window analogue of a Detached_Workspace's `cmd_ctx`.

#### Acceptance Criteria

1. WHILE the Workspace is split, EACH region (each `TabGroupTree` leaf) SHALL render its OWN
   `Command ===>` field (a Region_Command_Line) within that region, in addition to the region's tab
   bar and Context body.

2. WHEN a command is submitted (Enter) in a region's Region_Command_Line, THE shell SHALL dispatch
   it against THAT region's active tab -- regardless of which region currently holds keyboard focus
   -- by running the existing command pipeline under that region's Region_Context (via the same
   `with_workspace_context` swap a Detached_Workspace uses). The command SHALL NOT act on any other
   region's tab.

3. EACH region SHALL maintain its OWN command text, SCROLL amount, and status/error line in its
   Region_Context; text typed in one region's field SHALL NOT appear in another region's field, and
   a status/error produced by one region's command SHALL show only in that region.

4. Submitting a command in a region's Region_Command_Line SHALL also FOCUS that region (make it the
   Focused_Group), so subsequent keys/new-tab opens act there, consistent with Requirement 14.5/14.8.

5. THE per-region Region_Context SHALL stay in lockstep with the tree across split model changes: a
   new leaf (from `SPLIT`) SHALL get a fresh Region_Context; a collapsed/merged leaf's Region_Context
   SHALL be discarded; moving a tab between regions (Requirement 14.6) SHALL NOT carry command text
   between regions. No Region_Context SHALL leak to an unrelated leaf.

6. WHEN the Workspace is UNSPLIT (single leaf), THE shell SHALL render the single top-level
   `Command ===>` field exactly as today (byte-identical behaviour); per-region command lines apply
   ONLY while split. (Design MAY choose to hide or repurpose the top-level field while split; the
   chosen behaviour SHALL be specified in design.md and SHALL NOT change the unsplit case.)

7. THE Region_Command_Line SHALL participate in the workspace tab-order / Boundary_Policy model per
   the workspace-conformance rule: a region's field has a STABLE `egui::Id` (salted per leaf), and
   the design SHALL define how Tab moves between a region's command field, its interior controls, and
   the menu bar so no phantom stop is introduced. (The exact focus contract is a design decision;
   this criterion requires it be defined and tested, not left implicit.)

8. Per-region command state SHALL NOT be persisted across restart in this slice (transient, like the
   detached windows' `cmd_ctx`); the split STRUCTURE persistence (Requirement 14.10-14.13) is
   unchanged. (If persistence of region command text is desired later, it is a separate requirement.)

9. ALL per-region command-line behaviour (a region field renders per leaf; a region command acts on
   that region's tab not another; per-region command text/status isolation; submit-focuses-region;
   Region_Context lifecycle across split/move/collapse) SHALL be covered by unit + full-shell
   `egui_kittest` tests. Pixel-exact field placement is a justified-MANUAL exception only.

---

### Requirement 16: Workspace Instance Owns Its Chrome; Region Is Placement; Placement Is Derived

**User Story:** As a user, I want each Workspace to be a self-contained unit that carries its own
title, menu bar, keylist, and command line wherever it is shown -- whether it fills the whole window,
sits in a split region, or floats in a detached window -- so the experience is uniform and it is
always unambiguous which Workspace a command, menu, or key acts on.

**Source:** [CR-CH-041] owner: "regions should be an attribute of an instance of a workspace ... the
instance should have all the attributes of title, menubar, keylist, command line"; "regions as an
attribute of a workspace instance: whether it is detached, docked and its position and size";
"we must render the kinds instance menu bar inside the workspace instance"; "placement is derived
from the layout snapshot on restore, not double-stored on the descriptor"; and "the complexity of a
workspace window having tabs will be left to the design and build of a workspace kind, i.e. internal
to the workspace kind not part of the core." This requirement states the unifying principle that
Requirement 14.14 (Focus_Context) and Requirement 15 (per-region command lines) are specialisations
of; it also finalises the CR-CH-040 Slice 2 open question ("is a Panel a layout region, a Kind-like
preset, or both?") in favour of: a region is geometry/placement; chrome belongs to the instance.

**Design note (no mechanism change):** this reframes OWNERSHIP, not machinery. The command-line
per-window context (`WorkspaceCommandContext` + `with_workspace_context`), the per-Kind menu bar and
keylist resolution (workspace-kinds Requirement 4), the `TabGroupTree` arrangement model
(Requirement 12/13/14), the `Workspace_Descriptor` persistence (startup-and-session Requirement 21),
and the split `Layout_Snapshot` (Requirement 14.10-14.13) all already exist. Requirement 16 assigns
each of these to exactly one owner and defines how they compose. The UNSPLIT single-instance case is
visually and behaviourally identical to today (the region is the whole main window), so this
requirement is behaviour-preserving for the common case.

**Glossary (additions):**
- **Workspace_Instance**: the tab-level Workspace -- the unit of work. It is `TabState` at runtime
  and a `Workspace_Descriptor` (startup-and-session Requirement 21) when persisted. It owns its Kind,
  a unique instance id, and its chrome (title, menu bar, keylist, command line, scroll). NOT the
  project Workspace of `workspace-model` (a `.ffwb-workspace` of roots + settings + MRU), which is a
  separate, unrelated concept that keeps the "Workspace" name only in that spec.
- **Region**: a leaf of the `TabGroupTree` -- a geometry/placement slot with a position, a size, and
  an id, that HOSTS a set of Workspace_Instances and shows one at a time. A Region is NOT an owner of
  a menu bar, keylist, or command line.
- **Placement**: the attribute of a Workspace_Instance describing WHERE it is currently drawn. It has
  two forms: `Detached` (its own OS window) or `Docked { position, size }` (a `TabGroupTree` region
  leaf, whose position/size come from the tree). Placement is DERIVED, not independently stored (see
  criterion 16.6).

#### Acceptance Criteria

1. THE tab-level Workspace_Instance SHALL be the owner of its chrome: its title, menu bar, keylist,
   and command line (and its SCROLL field) SHALL be attributes resolved FOR THE INSTANCE (title and
   menu bar and keylist via its Kind's effective config -- workspace-kinds Requirement 3 and 4;
   command line via its own command context). A Region SHALL NOT own any of these; it contributes
   only geometry (position, size) and identity.

2. THE full per-instance chrome -- menu bar, Title_Line, and command line, in that order (the
   Tab_Window_Chrome of menu-and-statusbar Requirement 17) -- SHALL be rendered INSIDE the
   instance's placement (its region, or its detached window), NOT as a single shared application-level
   bar. WHILE the Workspace area is split, there SHALL be no shared top-level menu bar or command
   line spanning regions; each region draws the FULL chrome of the instance placed in it.

3. WHEN the Workspace area is UNSPLIT (a single instance filling the main window), THE instance's
   chrome SHALL occupy the top of the main window exactly as today (the region is the whole window),
   so criterion 16.2 changes rendering OWNERSHIP without changing the unsplit APPEARANCE or
   behaviour (behaviour-preserving; consistent with Requirement 15.6).

4. THE three placements of a Workspace_Instance -- docked-filling-the-window, docked-in-a-split-region,
   and detached-in-an-OS-window -- SHALL render the SAME per-instance chrome through the SAME path;
   the only difference between them SHALL be the instance's Placement. (This generalises the
   Requirement 14.14 Focus_Context: an in-window region and a detached window are two Placements of an
   instance, not two kinds of chrome owner.)

5. EACH Workspace_Instance SHALL carry a stable, unique instance id (the `TabId`), distinct from its
   Region's id (the `TabGroupId`): the instance id identifies the unit of work and its content; the
   Region id identifies the geometry slot. Moving an instance between regions (Requirement 14.6) or
   detaching/redocking it SHALL preserve the instance id and its chrome.

6. THE instance's Placement SHALL be DERIVED from the persisted split `Layout_Snapshot`
   (Requirement 14.10-14.13) on restore, and SHALL NOT be duplicated on the `Workspace_Descriptor`
   (startup-and-session Requirement 21). The `Workspace_Descriptor` persists WHAT the instance is
   (its Kind + params); the `Layout_Snapshot` persists WHERE instances sit (the tree shape). On
   restore the shell SHALL reconstruct instances from their descriptors, then resolve each instance's
   Placement from the restored layout tree; WHERE no `Layout_Snapshot` is present (unsplit, or an
   older session), every instance's Placement SHALL be the single docked region exactly as today.
   The two persisted models SHALL NOT disagree: the layout is the single source of truth for
   placement, reconciled against the descriptor set as `sync_layout` already does (no instance lost,
   no dangling placement -- consistent with Requirement 14.13).

7. CORE SHALL provide exactly ONE tab system: the Region / `TabGroupTree` that hosts Workspace_Instances.
   CORE SHALL NOT provide a mechanism for a single Workspace_Instance to contain its own tab list, and
   SHALL NOT add a universal "tab-container on/off" attribute to Workspace Kinds. A Workspace Kind that
   requires an internal tabbed / multi-region composition SHALL implement it WITHIN the Kind (it MAY
   reuse a `TabGroupTree` internally as an implementation detail), and that composition is the Kind's
   own concern, invisible to and unmanaged by the core tab/region model.

8. THE per-instance in-region chrome SHALL satisfy the shell-level Boundary_Policy and the
   workspace-conformance focus contract (menu-and-statusbar Requirement 16): each rendered command
   field and menu bar SHALL have a STABLE `egui::Id` (salted per instance/region so multiple placed
   instances do not collide), and Tab / Shift+Tab SHALL move between an instance's command field, its
   interior controls, and its menu bar with NO phantom stop. This is a specialisation of the
   workspace-conformance rule, not an exception to it.

9. ALL Requirement 16 behaviour (instance owns chrome; chrome renders at the instance's placement and
   not as a shared bar; unsplit appearance unchanged; the same chrome across the three placements;
   instance id preserved across move/detach/redock; placement derived from the layout snapshot and not
   double-stored; core provides no intra-workspace tab system) SHALL be covered by unit tests on the
   model and full-shell `egui_kittest` tests on the rendered behaviour, per the GUI Behaviour Testing
   rule. Pixel-exact chrome placement and the OS detached-window frame remain justified-MANUAL rows.
