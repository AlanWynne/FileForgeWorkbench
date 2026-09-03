# Requirements Document

## Introduction

This feature specifies the layout and docking system for FileForgeWorkbench (`ff-layout` crate). The layout system provides dockable panels, tab groups with split views, floating OS-level windows, multi-monitor support, named layout personas (presets), drag-and-drop rearrangement, and full layout serialization. It is part of the Workbench Core — the GUI shell renders the layout model but does not own it, adhering to the GUI-independence principle (Architecture Brief §3 Principle 1).

The layout system merges the general-purpose docking architecture from FileForgeEditor (panels can be attached/detached from dock zones, multi-monitor placement, persistence) with enhanced workbench concepts: tab groups with horizontal/vertical splits, named personas for rapid workspace switching, and a richer serialization model that supports layout export/import and graceful degradation when plugins are not loaded.

Panels are contributed by the plugin system — the layout engine itself is panel-agnostic. Any component that implements the `DockablePanel` trait can participate in dock/undock operations, appear in tab groups, float as an independent OS window, and be included in persona configurations.

**Source references:**
- **FFE** = FileForgeEditor `dockable-panels` specification (12 requirements — all incorporated)
- **WB** = Workbench Architecture Brief §12 Layout Architecture (layout-as-data, personas, GUI independence)

## Glossary

- **Layout_Engine**: The central coordinator within `ff-layout` that owns the layout tree, manages dock zones, tab groups, floating windows, and orchestrates all layout transitions. Replaces the FFE Dock_Manager with expanded responsibility. [FFE, WB]
- **Dock_Zone**: A designated area within the primary application window where panels can be attached. Standard zones are: left, right, bottom, and center. Each zone can hold one or more panels as tabs. [FFE]
- **Dockable_Panel**: Any UI panel that implements the `DockablePanel` trait, enabling it to be rendered inside a dock zone, within a tab group, or inside its own floating OS-level window. Panels are contributed via the plugin system. [FFE, WB]
- **Floating_Window**: A separate OS-level window (platform viewport) — also called a Detached View — containing one or more panels or editor tabs that have been detached from the main application window. Replaces the FFE Undocked_Window concept with support for multi-panel floating containers. [FFE]
- **Panel_Registry**: The registry of all known panel types and their default dock zone assignments. Plugins register panels here during initialization. [FFE, WB]
- **Layout_State**: A serializable snapshot of the complete layout including dock zone contents, tab group arrangement, floating window positions/sizes, panel visibility, and splitter positions. [FFE, WB]
- **Primary_Window**: The main FileForgeWorkbench application window containing the menu bar, dock zones, tab groups, and docked panels. [FFE]
- **Tab_Group**: A subdivision of the center editor area that holds one or more editor tabs. Multiple tab groups can coexist via horizontal or vertical splits. [WB]
- **Persona**: A named, saved layout configuration (panel visibility, dock positions, tab group arrangement, splitter sizes) that can be activated to switch the entire workspace appearance with a single action. [WB]
- **Drop_Indicator**: A visual overlay shown during drag-and-drop operations that highlights valid dock zones or tab group targets where a panel can be attached. [FFE]
- **Splitter**: A draggable border between adjacent dock zones or tab groups that allows the user to resize the areas. [WB]
- **Layout_File**: A TOML or JSON file on disk that stores a serialized Layout_State for persistence or sharing. [WB]

## Requirements

### Requirement 1: Panel System

**User Story:** As a user, I want a system of dockable panels (file tree, help, output, properties, schema browser, etc.) that can be shown, hidden, and positioned in designated areas of the workbench window, so that I can organize my workspace according to my current task.

**Source:** FFE Reqs 1, 2, 10 — merged with WB §12 layout-as-data principle. [FFE, WB]

#### Acceptance Criteria

1. WHEN the application starts, THE Layout_Engine SHALL initialize with a default layout containing dock zones for left, right, bottom, and center positions.
2. THE Layout_Engine SHALL maintain a Panel_Registry of all registered Dockable_Panel types and their default dock zone assignments.
3. WHEN a Dockable_Panel is registered with the Panel_Registry, THE Layout_Engine SHALL assign the panel to its default dock zone. IF the specified default dock zone is not one of left, right, bottom, or center, THEN THE Layout_Engine SHALL reject the registration and log an error message indicating the invalid zone name.
4. THE Dockable_Panel trait SHALL define a `panel_id(&self) -> &str` method that returns a unique string identifier (1 to 64 ASCII alphanumeric or underscore characters) for the panel type.
5. THE Dockable_Panel trait SHALL define a `default_dock_zone(&self) -> DockZone` method that returns the preferred dock zone. DockZone SHALL be an enum with variants `Left`, `Right`, `Bottom`, `Center`, and `Floating`.
6. THE Dockable_Panel trait SHALL define a `render(&mut self, ui: &mut egui::Ui)` method for drawing panel content that produces valid output regardless of whether the panel is docked, in a tab group, or displayed in a floating window.
7. THE Dockable_Panel trait SHALL define a `title(&self) -> &str` method returning the display title (1 to 128 characters).
8. THE Dockable_Panel trait SHALL define an `on_dock_state_changed(&mut self, state: DockState)` method that the Layout_Engine calls when the panel transitions between docked, floating, minimized, or hidden states.
9. WHEN a new type implements the Dockable_Panel trait and registers with the Panel_Registry, THE Layout_Engine SHALL dock, undock, show, hide, minimize, and maximize that panel using only the trait interface — without code changes to the Layout_Engine.
10. IF a panel is registered with a `panel_id` that already exists in the Panel_Registry, THEN THE Panel_Registry SHALL reject the registration and return an error indicating a duplicate identifier.
11. WHEN the user triggers a show command for a hidden panel, THE Layout_Engine SHALL make the panel visible in its last known dock zone. WHEN the user triggers a hide command, THE Layout_Engine SHALL remove the panel from view while preserving its position in the Layout_State.
12. WHEN the user triggers a toggle command for a panel, THE Layout_Engine SHALL show the panel if it is currently hidden, or hide the panel if it is currently visible.
13. THE Layout_Engine SHALL support three panel display states: minimized (collapsed to a tab/icon in the dock zone header), normal (rendered at its assigned size), and maximized (expanded to fill the entire Primary_Window content area, overlaying other panels temporarily).
14. WHEN a plugin that contributes a panel is loaded, THE plugin system SHALL register the panel with the Panel_Registry via `PluginContext`. WHEN a plugin is unloaded, THE Layout_Engine SHALL remove the panel from display and mark its position as vacant in the Layout_State.

---

### Requirement 2: Tab Groups

**User Story:** As a user, I want the editor area to support multiple tab groups with split views, so that I can view and edit multiple files side-by-side without undocking windows.

**Source:** NEW — from WB Architecture Brief enhanced layout concepts. [WB]

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

### Requirement 3: Floating Windows

**User Story:** As a user, I want to undock panels or tabs into separate OS-level windows, so that I can arrange my workspace across multiple monitors or view content side-by-side independently.

**Source:** FFE Reqs 2, 3, 4, 5 — adapted for workbench multi-panel floating containers. [FFE, WB]

#### Acceptance Criteria

1. WHEN the user triggers an undock action on a docked panel (via context menu, keyboard shortcut, or drag gesture), THE Layout_Engine SHALL remove the panel from its dock zone and create a Floating_Window containing the panel.
2. WHEN a panel is undocked, THE Floating_Window SHALL appear at a position offset from the Primary_Window by (50 × N) pixels right and (50 × N) pixels down, where N is the number of currently floating windows (starting at 1), with an initial size matching the panel's docked dimensions.
3. WHILE a panel is in a Floating_Window, THE panel SHALL render with full interactivity identical to its docked state, including all input handling, context menus, command dispatching, and keyboard shortcuts.
4. WHEN a panel is undocked, THE Layout_Engine SHALL update the Layout_State to reflect the panel's floating status and window position within 500 milliseconds.
5. WHEN the user triggers a redock action on a Floating_Window (via context menu, keyboard shortcut, or drag-to-dock gesture), THE Layout_Engine SHALL close the Floating_Window and reattach the panel to its most recent dock zone.
6. WHEN a panel is redocked, THE Layout_Engine SHALL restore the dock zone to its previous width or height if it was collapsed when the panel was undocked.
7. IF the most recent dock zone already contains another panel, THEN THE Layout_Engine SHALL stack the returning panel as a tab within that dock zone.
8. WHEN a Floating_Window is closed via the OS window close button, THE Layout_Engine SHALL redock the panel to its most recent dock zone rather than destroying the panel.
9. WHEN the user drags a tab header beyond 20 pixels outside the tab bar boundary and releases it outside the Primary_Window, THE Layout_Engine SHALL undock that tab into a new Floating_Window positioned at the mouse release coordinates.
10. WHILE a tab is in a Floating_Window, THE tab SHALL provide full editing functionality identical to the Primary_Window editor area, including command line, syntax highlighting, status bar, and all keyboard shortcuts.
11. WHEN an undocked tab's Floating_Window is closed via the OS window close button, THE Layout_Engine SHALL redock the tab back into the Tab_Group it originated from at its original position index; IF that index exceeds the current tab count, THEN THE tab SHALL be appended at the end.
12. IF a tab has unsaved modifications when the user attempts to close its Floating_Window, THEN THE Layout_Engine SHALL show the standard save confirmation dialog (Save, Discard, Cancel) before redocking; selecting Cancel SHALL abort the close.
13. Floating_Windows are full OS-level windows (platform viewports), not in-app overlays. They SHALL appear in the OS taskbar/dock and SHALL be independently movable, resizable, minimizable, and maximizable.
14. THE Layout_Engine SHALL support up to 16 simultaneous Floating_Windows; IF the user attempts to create an additional window beyond this limit, THEN THE Layout_Engine SHALL display a status message indicating the maximum has been reached and SHALL NOT undock the panel.
15. IF the operating system fails to create a Floating_Window, THEN THE Layout_Engine SHALL leave the panel in its current position and display a status message indicating the window could not be created.
16. Floating_Window state (position, size, contained panels) SHALL be persisted as part of the Layout_State.

---

### Requirement 4: Multi-Monitor Support

**User Story:** As a user, I want floating windows to work correctly across multiple monitors with different DPI settings, so that I can spread my workspace across my entire display setup.

**Source:** FFE Req 9 — enhanced with per-monitor DPI handling. [FFE, WB]

#### Acceptance Criteria

1. WHEN a Floating_Window is created, THE Layout_Engine SHALL position the window at the requested coordinates regardless of which connected monitor contains those coordinates.
2. WHEN the user moves a Floating_Window to a different monitor, THE Layout_Engine SHALL update the window's recorded monitor identifier in the Layout_State.
3. THE Layout_State serialization SHALL include a monitor identifier for each Floating_Window to enable correct restoration on multi-monitor setups.
4. WHILE a Floating_Window is positioned on a monitor, THE window SHALL render using the DPI scale factor reported by the operating system for that specific monitor.
5. WHEN a Floating_Window is moved from one monitor to another with a different DPI scale factor, THE window SHALL adjust its rendered content to match the target monitor's DPI while preserving logical dimensions.
6. IF a monitor is disconnected while one or more Floating_Windows are displayed on it, THEN THE Layout_Engine SHALL move the affected windows to the center of the primary monitor's work area, preserving each window's logical size.
7. WHEN restoring persisted window positions at startup, IF a window's target position refers to a monitor that is no longer connected, THEN THE Layout_Engine SHALL reposition that window to the center of the primary monitor's work area.
8. WHEN restoring persisted window positions at startup, IF a window's position would result in less than 50% of the window being visible within any connected monitor's bounds, THEN THE Layout_Engine SHALL reposition that window to the center of the primary monitor's work area.

---

### Requirement 5: Personas (Layout Presets)

**User Story:** As a user, I want named layout configurations (personas) that I can switch between instantly, so that I can adapt my workspace to different tasks (editing, debugging, data analysis, database work) without manually rearranging panels each time.

**Source:** NEW — from WB Architecture Brief §12 layout files concept. [WB]

#### Acceptance Criteria

1. THE Layout_Engine SHALL support named Persona configurations, each defining: panel visibility per panel_id, dock zone assignments, Tab_Group arrangement (splits and proportions), Floating_Window positions and sizes, and splitter positions.
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

**Source:** FFE Req 8 — enhanced with export/import, graceful degradation, and reset-to-default. [FFE, WB]

#### Acceptance Criteria

1. WHEN the application exits normally, THE Layout_Engine SHALL serialize the current Layout_State to the session file at `config/layout_state.toml`.
2. WHEN the application starts and a valid `config/layout_state.toml` exists, THE Layout_Engine SHALL restore the layout from the persisted state, including dock zone contents, Tab_Group arrangement, Floating_Window positions, splitter sizes, and panel visibility.
3. IF the persisted `config/layout_state.toml` is missing, fails to parse as valid TOML, or has a schema version mismatch, THEN THE Layout_Engine SHALL fall back to the default layout and log a warning indicating the reason.
4. THE Layout_State SHALL include for each docked panel: panel_id, dock zone assignment, and zone width/height in logical pixels. For each floating panel: panel_id, window position (x, y), window size (width, height) with minimum 200×150 logical pixels, and monitor identifier. For each Tab_Group: split direction, proportional size, and ordered list of tab identifiers.
5. IF a persisted Layout_State references a panel_id that is not currently registered in the Panel_Registry (plugin not loaded), THEN THE Layout_Engine SHALL omit that panel from the restored layout and log an INFO-level message, without failing or blocking the layout restoration.
6. WHEN the user triggers a layout-export command, THE Layout_Engine SHALL serialize the current Layout_State to a user-specified file path in TOML format.
7. WHEN the user triggers a layout-import command with a valid layout file, THE Layout_Engine SHALL apply the imported Layout_State, subject to the same graceful degradation rules (missing panels skipped) as startup restoration.
8. WHEN the user triggers a layout-reset command, THE Layout_Engine SHALL discard the current Layout_State and restore the built-in default layout.
9. WHEN a Floating_Window is moved or resized, THE Layout_Engine SHALL update the in-memory Layout_State within 500 milliseconds of the operation completing.
10. IF the Layout_Engine fails to write `config/layout_state.toml` at exit (due to permission error, disk full, or I/O failure), THEN THE Layout_Engine SHALL log a warning and allow exit to proceed without blocking shutdown.
11. THE Layout_State serialization format SHALL include a schema version number to enable forward-compatible migration of layout files across application versions.

---

### Requirement 7: Drag-and-Drop

**User Story:** As a user, I want to rearrange panels and tabs by dragging them to dock zones, tab groups, or outside the window to float, so that I can intuitively organize my workspace.

**Source:** FFE Reqs 6, 7 — enhanced with tab group drop targets. [FFE, WB]

#### Acceptance Criteria

1. WHEN the user initiates a drag on a Floating_Window's title bar and moves it over a valid dock zone in the Primary_Window, THE Layout_Engine SHALL display a Drop_Indicator highlighting the target zone.
2. WHEN the user releases the drag over a valid dock zone with a visible Drop_Indicator, THE Layout_Engine SHALL dock the panel into that zone and close the Floating_Window.
3. WHEN the user releases the drag outside any valid dock zone or tab group, THE Layout_Engine SHALL leave the panel in its Floating_Window at the release position.
4. THE Drop_Indicator SHALL render as a semi-transparent overlay with a distinct border color covering the target dock zone or tab group area.
5. WHEN a drag enters a dock zone or tab group drop target, THE Drop_Indicator SHALL appear within 16 milliseconds (one frame at 60 FPS).
6. WHEN a drag leaves a dock zone without release, THE Drop_Indicator SHALL disappear immediately.
7. WHEN the user drags a tab header from one Tab_Group and drops it onto another Tab_Group's tab bar area, THE Layout_Engine SHALL move the tab to the target group at the insertion index determined by the horizontal drop position.
8. WHEN the user drags a docked panel header and drops it onto a different dock zone, THE Layout_Engine SHALL move the panel to the target zone.
9. WHEN the user drags a panel or tab outside the Primary_Window boundaries and releases, THE Layout_Engine SHALL undock the item into a new Floating_Window at the release coordinates (drag-to-float).
10. WHEN the user drags a Floating_Window's title bar and drops it onto a valid dock zone (drag-to-dock), THE Layout_Engine SHALL dock the panel into that zone and close the Floating_Window.
11. WHEN the user drags a tab header vertically more than 30 pixels away from the tab bar, THE Layout_Engine SHALL begin a tear-off preview, displaying the tab as a floating thumbnail anchored at the cursor position.
12. WHEN the user releases a torn-off tab back within 30 pixels of a tab bar, THE Layout_Engine SHALL cancel the tear-off and reorder the tab at the insertion index determined by horizontal cursor position.
13. WHILE a drag-to-dock operation is in progress, THE Primary_Window SHALL highlight all valid dock zones with a visible border of at least 2 pixels in a color distinct from the zone's default appearance.

---

### Requirement 8: Resizing

**User Story:** As a user, I want to resize panels and tab groups by dragging their borders, so that I can allocate screen space according to my current needs.

**Source:** NEW — enhanced layout management for workbench. [WB]

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

**Source:** FFE Req 11 — adapted for workbench command-framework integration. [FFE, WB]

#### Acceptance Criteria

1. WHEN the user presses Ctrl+Shift+D while a Dockable_Panel has keyboard focus, THE Layout_Engine SHALL toggle the focused panel between docked and floating states.
2. IF the user presses Ctrl+Shift+D and no Dockable_Panel currently has keyboard focus, THEN THE Layout_Engine SHALL take no action.
3. WHEN the user presses Ctrl+Shift+T while an editor tab is the active tab in the Primary_Window, THE Layout_Engine SHALL undock the active tab into a new Floating_Window.
4. IF the user presses Ctrl+Shift+T while the active tab is the only tab in the only Tab_Group, THEN THE Layout_Engine SHALL take no action (preventing an empty editor area).
5. WHEN the user presses Ctrl+Shift+T while focus is in a Floating_Window containing a tab, THE Layout_Engine SHALL redock the tab back to its originating Tab_Group.
6. ALL layout keyboard shortcuts SHALL be registered with the command-framework's shortcut registry to prevent conflicts and enable user remapping via the key map system.
7. WHEN the user invokes a persona-switch command (via keyboard shortcut or command palette), THE Layout_Engine SHALL activate the specified persona following Requirement 5 criteria.
8. WHEN the user invokes a split-horizontal or split-vertical command via keyboard shortcut, THE Layout_Engine SHALL split the active Tab_Group following Requirement 2 criteria.

---

### Requirement 10: Visual Feedback and Indicators

**User Story:** As a user, I want clear visual indicators showing panel states, drop targets, and active persona, so that I can understand and control my workspace layout at all times.

**Source:** FFE Req 12 — adapted for workbench personas and tab groups. [FFE, WB]

#### Acceptance Criteria

1. WHILE a panel is floating, THE Primary_Window SHALL display a placeholder indicator in the panel's former dock zone showing the panel name and a clickable "redock" button.
2. WHEN the user hovers over the placeholder indicator for at least 300 milliseconds, THE Primary_Window SHALL display a tooltip showing "Click to redock [panel name]" and the associated keyboard shortcut.
3. WHEN the user clicks the placeholder indicator's redock button, THE Layout_Engine SHALL redock the associated panel following the same behavior defined in Requirement 3 criterion 5.
4. THE Floating_Window title bar SHALL display the panel title followed by " — FileForge", truncated to a maximum of 80 characters if necessary.
5. THE status bar or a designated UI region SHALL display the name of the currently active persona, with a "modified" indicator when the user has changed the layout from the persona's saved state.
6. WHEN the user drags a panel or tab and enters a valid drop zone, THE Layout_Engine SHALL display a Drop_Indicator showing exactly where the item will be placed upon release (left/right/top/bottom split, or tab insertion point).
7. WHILE a panel is minimized, THE dock zone header SHALL display an icon or label for the panel that the user can click to restore it to normal state.


---

### Requirement 11: Tab Window Chrome in Detached Views

**User Story:** As a user, I want Detached Views to show the same Title_Line and
Command Field chrome as the docked tab, so that the experience is consistent whether a
tab is in the main window or detached independently.

**Source:** menu-and-statusbar Requirement 17 and 18; user requirement (Phase AL).

#### Acceptance Criteria

1. WHEN a tab is displayed in a Detached View (Floating_Window), THE Detached View SHALL render the full Tab_Window_Chrome: Tab_Header row, Title_Line, and Primary_Command_Field — in that order at the top of the window, above the tab's content area.

2. THE Title_Line in a Detached View SHALL display the same context-dependent text as it would when the tab is docked (per menu-and-statusbar Requirement 17.3–17.6).

3. THE Primary_Command_Field in a Detached View SHALL be fully functional: it SHALL accept keyboard focus on window activation, accept typed commands, and dispatch them through the same CommandEngine as the Primary_Window Command Field.

4. WHEN the Legacy theme is active, THE Title_Line in a Detached View SHALL use the same blue background / white text styling as the docked Title_Line (menu-and-statusbar Requirement 17.8).

5. THE Detached View title bar (OS chrome) SHALL display the Title_Line content followed by " — FileForge Workbench" (menu-and-statusbar Requirement 18.5).
