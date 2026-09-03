# Requirements Document

## Introduction

This feature specifies the menu bar and status bar system for FileForgeWorkbench (`ff-menu` crate). The menu system provides a top-level menu bar with standard application menus (File, Edit, Search, View, Help), context menus, and a configurable multi-segment status bar that displays real-time editor and workbench state. All menu items route through the command framework — no menu action directly mutates application state.

The menu bar is the primary graphical command invocation surface for the workbench. It presents commands in a standard hierarchical structure that desktop users expect, while the underlying execution always flows through `command-framework` dispatch. This ensures macro recordability, shortcut consistency, and plugin extensibility for all menu-initiated operations.

The status bar is a workbench-level panel (not editor-specific) that displays contextual information in configurable segments. Core segments include editor mode, cursor position, file encoding, and modification state. Plugins can contribute additional segments via the plugin architecture.

The Command Field ("Command ===>") is an ISPF heritage element positioned above the editor area. It provides direct command entry and expands to fill all available horizontal space.

**Source references:**
- **FFE-MVP-4** = FileForgeEditor `mvp-implementation` Requirement 4 (menu bar, status bar, primary command field)
- **WB** = Workbench Architecture Brief §7, §12 (command-driven architecture, layout system)
- **SCI-STE** = Scintilla/SciTE concepts (recent files, view menu options, tab context menus, status bar segments)

**Cross-references:**
- `command-framework` — all menu items invoke registered commands
- `layout-and-docking` — status bar is a workbench panel; menu bar integrated with Primary_Window
- `file-operations` — File menu actions delegate to file operation commands
- `edit-operations` — Edit menu actions delegate to edit commands
- `find-and-replace` — Search menu actions delegate to find/change commands
- `theme-and-appearance` — View menu exposes theme switching
- `configuration-system` — status bar segment configuration, recent files persistence
- `plugin-architecture` — plugins contribute menu items and status bar segments

## Glossary

- **Tab_Window_Chrome**: The three-element header region rendered at the top of every tab's content area, consisting of (1) the Tab_Header row, (2) the Title_Line, and (3) the Primary_Command_Field. This chrome is present whether the tab is docked in the Primary_Window or detached into a Floating_Window.
- **Title_Line**: A read-only single-line display rendered below the Tab_Header row and above the Primary_Command_Field. Its content is context-dependent: for a POM tab it shows the application name and version; for a file editor tab it shows the full file path; for other tab kinds it shows the tab title.
- **Menu_Bar**: The horizontal menu bar rendered at the top of the Primary_Window, containing top-level menu headings (File, Edit, Search, View, Help) that open dropdown submenus when activated. [FFE-MVP-4]
- **Menu_Item**: An individual entry within a dropdown submenu. Each menu item is bound to a Command_ID in the command framework and displays the command's display name and keyboard shortcut (if any). [FFE-MVP-4, WB]
- **Menu_Separator**: A visual horizontal divider used to group related menu items within a submenu. [SCI-STE]
- **Submenu**: A nested menu that appears when hovering or clicking a parent menu item marked as a submenu container. [SCI-STE]
- **Context_Menu**: A popup menu triggered by a right-click or context-menu key, presenting context-sensitive actions for the clicked element (tab, editor area, panel). [SCI-STE]
- **Status_Bar**: A horizontal bar rendered at the bottom of the Primary_Window, divided into configurable segments that display real-time workbench and editor state. [FFE-MVP-4, WB]
- **Status_Segment**: An individual display region within the Status_Bar, showing a single piece of information (e.g., line/column, mode, encoding). Each segment has an ID, content provider, alignment, and minimum width. [WB]
- **Editor_Mode**: The current interaction mode of the active editor: Browse (read-only navigation), Edit (text modification enabled), or View (read-only, no commands). [FFE-MVP-4]
- **Insert_Overstrike_State**: Whether typed characters insert at the cursor position (Insert) or overwrite existing characters (Overstrike). [FFE-MVP-4]
- **Primary_Command_Field**: The single-line text input field labelled "Command ===>" positioned in the command area above the editor, used for direct ISPF-style command entry. [FFE-MVP-4]
- **Recent_Files_List**: An ordered collection of the most recently opened file paths, displayed as a submenu under the File menu. [SCI-STE]
- **Menu_Command_Binding**: The association between a Menu_Item and a Command_ID in the command registry, ensuring the menu item's enabled/visible state mirrors the command's predicates. [WB]

## Requirements

### Requirement 1: Menu Bar Structure

**User Story:** As a workbench user, I want a standard menu bar with familiar top-level menus, so that I can discover and invoke commands through a conventional hierarchical menu interface.

**Source:** FFE MVP Requirement 4 criteria 1–4. [FFE-MVP-4]

#### Acceptance Criteria

1. THE Menu_Bar SHALL be rendered at the top of the Primary_Window, below the window title bar and above the primary command area.
2. THE Menu_Bar SHALL contain the following top-level menus in this order: File, Edit, Search, View, and Help.
3. THE File menu SHALL contain, at minimum, the following items in order: New, Open, Open Recent (submenu), Save, Save As, Close, and Exit — separated by Menu_Separators between logical groups (New/Open/Open Recent | Save/Save As | Close | Exit).
4. THE Edit menu SHALL contain, at minimum, the following items in order: Undo, Redo, a Menu_Separator, Cut, Copy, Paste, a Menu_Separator, and Select All.
5. THE Search menu SHALL contain, at minimum, the following items in order: Find, Find Next, Find Previous, a Menu_Separator, Change (Replace), and Go to Line.
6. THE View menu SHALL contain, at minimum, the following items in order: Zoom In, Zoom Out, Reset Zoom, a Menu_Separator, Word Wrap (toggle), Show Whitespace (toggle), Show Line Numbers (toggle), a Menu_Separator, and Theme (submenu listing available themes).
7. THE Help menu SHALL contain, at minimum, the following items: Help Topics, Keyboard Shortcuts, and About.
8. EACH top-level menu SHALL open its dropdown submenu when the user clicks the menu heading or navigates to it with keyboard arrow keys while a menu is already open.

---

### Requirement 2: Menu–Command Integration

**User Story:** As a workbench developer, I want every menu item to invoke a registered command through the command framework, so that menu actions are undoable, macro-recordable, and consistent with keyboard shortcut invocations of the same command.

**Source:** WB Architecture Brief §7 — command-driven architecture. [WB]

#### Acceptance Criteria

1. EACH Menu_Item SHALL be associated with a Command_ID via a Menu_Command_Binding; WHEN the user activates the menu item, THE Menu_Bar SHALL invoke `execute_command` on the command framework dispatcher with the bound Command_ID and any required parameters.
2. WHEN a Menu_Item's bound command has a keyboard shortcut registered in the Shortcut_Registry, THE Menu_Item SHALL display the shortcut text right-aligned within the menu entry (e.g., "Save    Ctrl+S").
3. WHEN a Menu_Item's bound command has an enabled predicate that returns false for the current Execution_Context, THE Menu_Item SHALL be rendered in a disabled (greyed-out) state and SHALL NOT be activatable.
4. WHEN a Menu_Item's bound command has a visibility predicate that returns false for the current Execution_Context, THE Menu_Item SHALL be hidden from the menu (not rendered).
5. WHEN the user selects File > Open, THE Menu_Bar SHALL invoke the `file.open` command, which SHALL open a native file-picker dialog (via the `rfd` crate) and load the selected file into the active editor.
6. WHEN the user selects File > Save, THE Menu_Bar SHALL invoke the `file.save` command, executing the same save logic as the SAVE primary command or Ctrl+S shortcut.
7. WHEN the user selects File > Exit, THE Menu_Bar SHALL invoke the `workbench.exit` command, which SHALL initiate the application shutdown sequence (prompting for unsaved changes if applicable).
8. WHEN the user selects Edit > Undo, THE Menu_Bar SHALL invoke the `edit.undo` command; WHEN the user selects Edit > Redo, THE Menu_Bar SHALL invoke the `edit.redo` command.
9. WHEN the user selects Search > Find, THE Menu_Bar SHALL invoke the `find.show` command; WHEN the user selects Search > Change, THE Menu_Bar SHALL invoke the `find.replace_show` command.
10. NO Menu_Item SHALL directly mutate application state; all state changes SHALL flow exclusively through the command framework dispatch.

---

### Requirement 3: Recent Files Menu

**User Story:** As a user, I want a Recent Files submenu that lists my most recently opened files, so that I can quickly reopen files I've worked with recently without navigating the file system.

**Source:** SciTE recent files concept, FFE `file-menu-operations` Recent_Files_List. [SCI-STE, FFE-MVP-4]

#### Acceptance Criteria

1. THE File menu SHALL contain an "Open Recent" submenu that lists the most recently opened or saved file paths, ordered from most recent to least recent.
2. THE Recent_Files_List SHALL store a configurable maximum number of entries, specified via the configuration system under `menu.recent_files_max`, with a default of 10 and a maximum of 50.
3. WHEN a file is successfully opened or saved, THE Recent_Files_List SHALL add or promote the file's absolute path to the top of the list; IF the list exceeds its maximum, THE oldest entry SHALL be removed.
4. WHEN the user selects an entry from the Open Recent submenu, THE Menu_Bar SHALL invoke the `file.open` command with the selected path as a parameter.
5. IF a path in the Recent_Files_List no longer exists on disk, THEN the submenu entry SHALL be rendered with a visual indication (e.g., italic or greyed text) and THE system SHALL remove it from the list after a failed open attempt.
6. THE Recent_Files_List SHALL be persisted across application sessions via the configuration system's workbench data directory.
7. THE Open Recent submenu SHALL include a "Clear Recent Files" item at the bottom (below a Menu_Separator) that removes all entries from the list.

---

### Requirement 4: Context Menus

**User Story:** As a user, I want right-click context menus that present relevant actions for the element I clicked, so that I can quickly access common operations without navigating the menu bar.

**Source:** SciTE tab context menus, editor context menu concepts. [SCI-STE]

#### Acceptance Criteria

1. WHEN the user right-clicks within the editor text area, THE system SHALL display a Context_Menu containing at minimum: Cut, Copy, Paste, Select All, a Menu_Separator, Find, and Change.
2. WHEN the user right-clicks on a tab header in a Tab_Group, THE system SHALL display a Context_Menu containing at minimum: Close, Close Others, Close All, Close to the Right, a Menu_Separator, Copy Path, and Reveal in File Tree.
3. EACH Context_Menu item SHALL be associated with a Command_ID and SHALL respect the same enabled/visibility predicates as its equivalent Menu_Bar item.
4. WHEN a Context_Menu item's bound command is disabled, THE item SHALL be rendered in a disabled state and SHALL NOT be activatable.
5. THE plugin system SHALL provide an extension point allowing plugins to contribute additional Context_Menu items for specific context types (editor area, tab header, panel header, file tree node) via command registration and menu contribution descriptors.

---

### Requirement 5: Status Bar Layout

**User Story:** As a user, I want a status bar at the bottom of the workbench window that shows me important real-time information about the current editor state, so that I can see my cursor position, editing mode, and file status at a glance.

**Source:** FFE MVP Requirement 4 criterion 10. [FFE-MVP-4, WB]

#### Acceptance Criteria

1. THE Status_Bar SHALL be rendered as a horizontal bar at the bottom of the Primary_Window, spanning the full width of the window.
2. THE Status_Bar SHALL be divided into Status_Segments, each occupying a defined portion of the bar width with configurable alignment (left, center, or right within the segment).
3. THE default Status_Bar layout SHALL contain the following segments from left to right: editor mode, insert/overstrike state, file encoding, cursor line and column, modified indicator, and total line count.
4. EACH Status_Segment SHALL have a unique string identifier (1 to 64 ASCII alphanumeric or underscore characters) for configuration and plugin reference.
5. THE Status_Bar SHALL have a fixed height sufficient to display one line of text at the current UI font size, and SHALL NOT be resizable by the user.
6. THE Status_Bar SHALL be a workbench-level panel (not editor-specific) managed by the layout system; it SHALL remain visible regardless of which panel or editor tab has focus.
7. WHEN no editor tab is open, THE Status_Bar SHALL display placeholder values for editor-specific segments (e.g., mode shows "—", line/column shows "—/—", encoding shows "—").

---

### Requirement 6: Status Bar Content — Mode and State Indicators

**User Story:** As a user, I want the status bar to clearly indicate my current editing mode and insert/overstrike state, so that I understand how the editor will respond to my keystrokes.

**Source:** FFE MVP Requirement 4 criterion 10. [FFE-MVP-4]

#### Acceptance Criteria

1. THE editor mode segment SHALL display one of the following values: "Browse", "Edit", or "View", reflecting the current Editor_Mode of the active editor tab.
2. WHEN the Editor_Mode changes (e.g., from Browse to Edit), THE mode segment SHALL update immediately to reflect the new mode.
3. THE insert/overstrike segment SHALL display either "INS" (insert mode) or "OVR" (overstrike mode), reflecting the current Insert_Overstrike_State of the active editor.
4. WHEN the Insert_Overstrike_State toggles (e.g., via the Insert key), THE segment SHALL update immediately.
5. THE modified indicator segment SHALL display a visual marker (e.g., "●" or "[Modified]") when the active document has unsaved changes; WHEN the document has no unsaved changes, THE segment SHALL display nothing or a clean-state indicator.
6. WHEN a save operation completes successfully, THE modified indicator SHALL clear immediately.

---

### Requirement 7: Status Bar Content — Position and File Information

**User Story:** As a user, I want the status bar to show my cursor position (line and column), file encoding, and total line count, so that I always know where I am in the file and its basic properties.

**Source:** FFE MVP Requirement 4 criteria 10–11. [FFE-MVP-4]

#### Acceptance Criteria

1. THE line/column segment SHALL display the current cursor position in the format "Ln {line}, Col {col}" where line and col are 1-based integers.
2. WHEN the cursor moves to a different line or column (via arrow keys, mouse click, search navigation, or any other cursor movement), THE line/column segment SHALL update to reflect the new position within one frame of the cursor movement event.
3. THE file encoding segment SHALL display the detected or configured encoding of the active document (e.g., "UTF-8", "UTF-16LE", "ISO-8859-1", "EBCDIC").
4. THE total line count segment SHALL display the total number of lines in the active document in the format "{count} lines".
5. WHEN lines are added or removed from the document (via editing, line commands, or file reload), THE total line count segment SHALL update immediately.
6. WHEN the active editor tab changes (user switches tabs), ALL status bar segments SHALL update to reflect the state of the newly active document within one frame of the tab switch.

---

### Requirement 8: Status Bar Extensibility

**User Story:** As a plugin developer, I want to contribute custom segments to the status bar, so that my plugin can display relevant state information (e.g., Git branch, language server status, build state) alongside the core editor information.

**Source:** WB Architecture Brief — plugin-contributed panels and segments. [WB]

#### Acceptance Criteria

1. THE plugin system SHALL provide a `StatusSegmentProvider` trait that plugins implement to contribute custom Status_Segments to the Status_Bar.
2. THE `StatusSegmentProvider` trait SHALL define: `segment_id(&self) -> &str` (unique identifier), `render(&self, ui: &mut egui::Ui)` (draw segment content), `alignment(&self) -> SegmentAlignment` (left, center, or right grouping), and `priority(&self) -> u32` (ordering within the alignment group, lower values render first).
3. WHEN a plugin registers a StatusSegmentProvider, THE Status_Bar SHALL include the plugin's segment in the bar layout according to its alignment and priority.
4. WHEN a plugin is unloaded, THE Status_Bar SHALL remove the plugin's contributed segments from the layout.
5. THE Status_Bar segment layout (order, visibility, minimum widths) SHALL be configurable via the configuration system under the `statusbar.segments` table, allowing users to hide, reorder, or resize segments.
6. IF a plugin registers a segment with a `segment_id` that already exists, THEN THE Status_Bar SHALL reject the registration and log a WARN-level message indicating the duplicate ID.

---

### Requirement 9: Primary Command Field

**User Story:** As an ISPF-familiar user, I want a primary command text field at the top of the editor area that expands to fill available width, so that I can type ISPF commands directly without using menus or keyboard shortcuts.

**Source:** FFE MVP Requirement 4 criterion 12. [FFE-MVP-4]

#### Acceptance Criteria

1. THE Primary_Command_Field SHALL be rendered in the command area above the active editor content, labelled with "Command ===>" to the left.
2. THE Primary_Command_Field SHALL expand horizontally to fill all available width between the "Command ===>" label and the right edge of the editor panel.
3. WHEN the user types text into the Primary_Command_Field and presses Enter, THE workbench SHALL submit the entered text to the CommandEngine for parsing and dispatch.
4. WHEN a command is successfully dispatched from the Primary_Command_Field, THE field SHALL clear its content.
5. IF the command entered in the Primary_Command_Field is not recognized by the CommandEngine, THEN THE system SHALL display an error message in the status bar indicating the unrecognized command, and the field content SHALL remain for correction.
6. THE Primary_Command_Field SHALL support command history recall: pressing the Up Arrow key while the field is focused SHALL cycle backwards through previously entered commands; pressing Down Arrow SHALL cycle forwards.
7. WHEN no text has been entered in the Primary_Command_Field and the user presses the Down Arrow key, THE keyboard focus SHALL move to the editor area (first editable line) rather than cycling command history.

---

### Requirement 10: Menu Extensibility

**User Story:** As a plugin developer, I want to contribute menu items and submenus to the menu bar, so that my plugin's commands are discoverable through the standard menu hierarchy alongside built-in commands.

**Source:** WB Architecture Brief — plugin architecture, command-driven menus. [WB]

#### Acceptance Criteria

1. THE plugin system SHALL provide a menu contribution descriptor that allows plugins to specify: the target menu path (e.g., "File", "Tools", "View > Panels"), the Command_ID to bind, the desired position (before/after a reference item, or at end), and an optional Menu_Separator specification.
2. WHEN a plugin registers a menu contribution, THE Menu_Bar SHALL insert the item at the specified position within the target menu, respecting the command's metadata for display name, shortcut, and icon.
3. PLUGINS SHALL be able to contribute entirely new top-level menus (e.g., "Tools", "Macros") by specifying a menu path that does not yet exist; THE Menu_Bar SHALL create the new top-level menu and insert it before the Help menu.
4. WHEN a plugin is unloaded, THE Menu_Bar SHALL remove all menu items contributed by that plugin and collapse any top-level menus that become empty after removal.
5. THE Menu_Bar SHALL support a "Tools" top-level menu (between View and Help) for plugin-contributed tool commands, macro execution, shell commands, and extension management actions.
6. PLUGIN-contributed menu items SHALL respect the same enabled/visibility predicates and shortcut display rules as built-in menu items.

---

### Requirement 13: Help > About Dialog

**User Story:** As a user, I want a Help > About dialog that identifies the application, its
creator, and the AI assistant that helped build it, so that I can see version and attribution
information at any time.

**Source:** [WB] standard desktop application About dialog.

#### Acceptance Criteria

1. WHEN the user selects `Help > About` from the menu bar, THE shell SHALL open a modal
     About dialog.

2. THE About dialog SHALL display the application name `FileForge Workbench` as a prominent
     heading.

3. THE About dialog SHALL display the current application version string.

4. THE About dialog SHALL credit the creator: `Created by Alan R Wynne`.

5. THE About dialog SHALL credit the AI assistant: `Built with Amazon Q Developer,
     an AI coding assistant by Amazon Web Services (AWS)`.

6. THE About dialog SHALL display a copyright notice in the form
     `© {year} Alan R Wynne. All rights reserved.`

7. THE About dialog SHALL display a brief description of the application:
     `A cross-platform enterprise file editor and mainframe workstation
     inspired by IBM ISPF and File-AID.`

8. WHEN the user clicks the `Close` button or presses `Escape`, THE About dialog SHALL close.

---

### Requirement 11: Keyboard Navigation

**User Story:** As a keyboard-centric user, I want full keyboard navigation of the menu bar, so that I can access all menu commands without a mouse.

**Source:** Standard desktop accessibility, SciTE keyboard menu access. [SCI-STE]

#### Acceptance Criteria

1. WHEN the user presses Alt+{underlined letter} (access key) for a top-level menu, THE Menu_Bar SHALL open that menu's dropdown (e.g., Alt+F opens File).
2. WHEN a dropdown menu is open, THE user SHALL be able to navigate items with Up/Down Arrow keys, open submenus with Right Arrow, close submenus with Left Arrow, and activate the highlighted item with Enter.
3. WHEN a dropdown menu is open, THE user SHALL be able to jump between top-level menus using Left/Right Arrow keys without closing and reopening.
4. WHEN the user presses Escape while a menu is open, THE Menu_Bar SHALL close the open menu and return keyboard focus to the previously focused element.
5. EACH menu item SHALL support an access key (underlined character) that activates the item when pressed while the parent menu is open.
6. WHEN the F10 key is pressed (or platform-specific menu activation key), THE Menu_Bar SHALL activate and focus the first top-level menu, enabling keyboard navigation.

---

### Requirement 16: Tab-Order Focus Cycle

**User Story:** As a keyboard-centric user, I want Tab and Shift+Tab (Back Tab) to cycle focus
through the interactive elements of the workbench shell in a predictable ISPF-style order, so
that I can navigate the entire UI without a mouse.

**Source:** ISPF 3270 terminal tab-order convention; standard desktop accessibility.

#### Acceptance Criteria

1. WHEN the application launches, THE keyboard focus SHALL be placed on the
     Primary_Command_Field ("Command ===>") automatically, so that the user can begin
     typing a command immediately without clicking.

2. WHEN the user types any printable character while the Primary_Command_Field has focus,
     THE character SHALL appear in the command field.

3. WHEN the Primary_Command_Field has focus and the user presses Tab (forward), THE focus
     SHALL move to the first POM option row (option `0 Settings`) if a POM tab is active,
     otherwise to the first top-level menu bar item.

4. WHEN a POM option row has focus and the user presses Tab (forward), THE focus SHALL
     advance to the next option row in sequence (0 → 1 → 2 → … → 8).

5. WHEN the last POM option row (option `8 Plugins`) has focus and the user presses Tab
     (forward), THE focus SHALL move to the POM exit line
     ("Enter X to Terminate using log/list defaults").

6. WHEN the POM exit line has focus and the user presses Tab (forward), THE focus SHALL
     move to the calendar `<` (previous-month) button.

7. WHEN the calendar `<` button has focus and the user presses Tab (forward), THE focus
     SHALL move to the calendar `>` (next-month) button.

8. WHEN the calendar `>` button has focus and the user presses Tab (forward), THE focus
     SHALL move to the first top-level menu bar item (the leftmost menu heading, `Settings`).

9. WHEN a menu bar item has focus and the user presses Tab (forward), THE focus SHALL
     advance to the next menu bar item to the right.

10. WHEN the last menu bar item (`Help`) has focus and the user presses Tab (forward),
      THE focus SHALL move to the first tab header in the tab bar (the leftmost tab).

11. Shift+Tab (Back Tab) SHALL be the exact reverse of the forward Tab cycle:
      - From Primary_Command_Field → last tab header
      - From first tab header → last menu bar item (`Help`)
      - From any tab header → previous tab header
      - From first menu bar item (`Settings`) → calendar `>` (if POM active) or last tab header (if not POM)
      - From any menu bar item → previous menu bar item
      - From `<` calendar button → POM exit line
      - From `>` calendar button → `<` calendar button
      - From POM exit line → last POM option row (option `8`)
      - From first POM option row (option `0`) → Primary_Command_Field
      - From any POM option row → previous POM option row

20. WHEN a tab header has focus and the user presses Tab (forward), THE focus SHALL
      advance to the next tab header to the right.

21. WHEN the last tab header has focus and the user presses Tab (forward), THE focus
      SHALL wrap back to the Primary_Command_Field.

22. WHEN the Tab cycle is active and the current tab is NOT a POM tab, the cycle
      SHALL be: Primary_Command_Field → menu bar items → tab headers → Primary_Command_Field
      (POM option rows, exit line, and calendar buttons are still skipped).

12. WHEN a POM option row has focus (via Tab navigation), THE option row SHALL be rendered
      with reversed colours — its background SHALL use the option label colour and its text
      SHALL use the panel background colour — providing a clear visual focus indicator.

13. WHEN a focused POM option row is activated by pressing Enter or Space, THE shell SHALL
      perform the same navigation action as clicking that option button.

14. WHEN a focused POM exit line is activated by pressing Enter or Space, THE shell SHALL
      initiate the application exit sequence.

15. WHEN a focused calendar `<` button is activated by pressing Enter or Space, THE calendar
      SHALL navigate to the previous month.

16. WHEN a focused calendar `>` button is activated by pressing Enter or Space, THE calendar
      SHALL navigate to the next month.

17. WHEN a menu bar item has focus (via Tab navigation), THE item SHALL receive a visible
      focus indicator (highlight or border) so the user can see which item is currently focused.

18. WHEN a focused menu bar item is activated by pressing Enter or Space, THE item's
      dropdown menu SHALL open; subsequent Tab/Shift+Tab presses SHALL navigate within the
      open dropdown rather than moving to the next shell focus stop.

19. WHEN the Tab cycle is active and the current tab is NOT a POM tab (e.g., a file editor
      tab), THE POM option rows, exit line, and calendar buttons SHALL be skipped; the cycle
      SHALL be: Primary_Command_Field → menu bar items → Primary_Command_Field.


---

### Requirement 17: Tab Window Chrome — Title Line and Command Line per Tab

**User Story:** As an ISPF-familiar user, I want every tab's content area to have a consistent
three-element header — a Tab_Header row, a Title_Line, and a Command_Line — so that I always
know what I am looking at and can issue commands without hunting for the input field.

**Source:** ISPF 3270 screen layout convention; user requirement (Phase AL).

#### Acceptance Criteria

1. WHEN any tab is displayed (whether docked or in a Floating_Window), THE tab's content
     area SHALL render the following three elements at the top, in order from top to bottom:
     (1) Tab_Header row, (2) Title_Line, (3) Primary_Command_Field ("Command ===>").

2. THE Title_Line SHALL be a read-only, single-line display rendered between the Tab_Header
     row and the Primary_Command_Field. It SHALL NOT be editable by the user.

3. WHEN the active tab is a Primary Option Menu tab, THE Title_Line SHALL display the
     application name and version in the format:
     `FileForge Workbench  vX.Y.Z`

4. WHEN the active tab is a file editor tab with an open file, THE Title_Line SHALL display
     the full absolute path of the open file.

5. WHEN the active tab is a file editor tab with no file open (untitled), THE Title_Line
     SHALL display `[Untitled]`.

6. WHEN the active tab is any other tab kind (Settings, Files Panel, etc.), THE Title_Line
     SHALL display the tab's title string.

7. THE Title_Line SHALL be styled using the active theme's primary text colour and SHALL
     be visually distinct from the editor content area (e.g., different background or a
     separator line below it).

8. WHEN the Legacy theme is active, THE Title_Line SHALL be rendered with a blue background
     (`#0000AA`) and white text (`#FFFFFF`), consistent with ISPF primary menu heading
     colour semantics (Requirement 13.2 of theme-and-appearance).

9. THE Primary_Command_Field SHALL remain the third element in the chrome, directly below
     the Title_Line, and SHALL retain all existing behaviour defined in Requirement 9.

---

### Requirement 18: Detachable Tab Windows

**User Story:** As a user, I want to detach any tab from the main window into its own
independent OS-level window, so that I can arrange my workspace across multiple monitors
or view content side-by-side independently.

**Source:** Layout-and-docking Requirement 3 (Floating Windows); user requirement (Phase AL).

#### Acceptance Criteria

1. WHEN the user selects "Move to Other View" from a tab's context menu, THE shell SHALL
     detach that tab into a new Floating_Window containing the full Tab_Window_Chrome
     (Tab_Header row, Title_Line, Primary_Command_Field) and the tab's content area.

2. WHILE a tab is in a Floating_Window, THE tab SHALL provide full functionality identical
     to the Primary_Window: the Title_Line SHALL update to reflect the tab's current state,
     the Primary_Command_Field SHALL accept commands, and all keyboard shortcuts SHALL work.

3. WHEN a Floating_Window containing a tab is closed via the OS window close button,
     THE shell SHALL redock the tab back into the Primary_Window's tab bar at its original
     position index; IF that index exceeds the current tab count, THE tab SHALL be appended
     at the end.

4. WHEN a tab is detached into a Floating_Window, THE Primary_Window's tab bar SHALL
     remove that tab's Tab_Header from the bar. WHEN the tab is redocked, THE Tab_Header
     SHALL be restored at the correct position.

5. THE Floating_Window title bar SHALL display the tab's Title_Line content followed by
     " — FileForge Workbench", truncated to a maximum of 80 characters if necessary.

6. WHEN the user drags a Tab_Header beyond 20 pixels outside the tab bar boundary and
     releases it outside the Primary_Window, THE shell SHALL detach that tab into a new
     Floating_Window positioned at the mouse release coordinates.

7. THE shell SHALL support up to 16 simultaneous Floating_Windows containing detached
     tabs. IF the user attempts to detach a tab beyond this limit, THE shell SHALL display
     a status message and SHALL NOT detach the tab.
