# Requirements Document

## Introduction

This feature specifies the **multi-tab editor** for FileForgeWorkbench (`ff-tabs` crate). The multi-tab editor is the user-facing subsystem that manages open documents as a collection of tabs — each with independent editing state — and provides the UI and keyboard controls for navigating, reordering, pinning, splitting, and managing those tabs.

The multi-tab editor sits between the Layout_Engine's Tab_Group concept (which provides the structural container) and the Document model (which owns text content). It is responsible for the Tab_Collection data model, per-tab state isolation, MRU (Most Recently Used) ordering for rapid switching, tab overflow handling, context menus, drag-and-drop reordering, pinned tabs, duplicate detection, split editor views, tab title disambiguation, and keyboard navigation.

**All tab operations are dispatched through the command framework** (cross-cutting Requirement 4). Menu items, keyboard shortcuts, context menu actions, and macros all invoke the same registered commands — ensuring consistent undo integration, macro recordability, and a single audit trail.

This specification merges requirements from three primary sources:

- **FileForgeEditor `multi-tab-editor`** (10 requirements): Tab collection, per-tab state, tab bar display, close operations, context menu, keyboard navigation, drag-and-drop, command engine integration, file menu integration, application exit — all incorporated with workbench adaptations.
- **SciTE Buffer Management** (`SciTEBuffers.cxx`): MRU stack navigation (`IDM_PREVFILESTACK`/`IDM_NEXTFILESTACK`), configurable buffer count (`buffers.size`), tab move left/right, buffer dirty state tracking, fold state per-buffer.
- **Workbench Architecture Brief**: Tab_Group integration, split editor views, command-driven operations, VFS-aware resource addressing, session persistence contract.

### Design Principles

1. **Command-driven** — All tab operations are registered commands (`tabs.close`, `tabs.next`, `tabs.pin`, etc.) invocable from any source. [WB]
2. **Tab_Group integration** — Tabs live inside Tab_Groups managed by the Layout_Engine. The multi-tab subsystem owns per-tab state; the layout engine owns spatial arrangement. [WB]
3. **VFS-aware identity** — Tab identity is based on `ResourceUri` (not raw filesystem paths), enabling duplicate detection across VFS providers. [WB, FFW-ARCH-001]
4. **Session-serialisable** — The full Tab_Collection state (open URIs, tab order, per-tab viewport, MRU stack, pinned flags) is serialisable for session persistence. [FFE-MULTITAB, WB]
5. **GUI-independent model** — The tab data model lives in `ff-tabs` (platform-core layer); the GUI shell renders tab headers using the model but does not own it. [WB]
6. **No data loss** — Close operations on modified documents always prompt save/discard/cancel before discarding content. [FFE-MULTITAB]

### Source References

- **[FFE-MULTITAB]** = FileForgeEditor `multi-tab-editor` specification (10 requirements — priority source)
- **[SCI-STE-TABS]** = SciTE `SciTEBuffers.cxx` (MRU stack, buffer count, tab move, per-buffer state)
- **[WB]** = Workbench Platform Architecture Brief (command-driven, GUI independence, layout integration, session persistence)

### Cross-References

- **`file-operations`** — Open/Save/Revert commands create and modify tabs; unsaved-changes dialogs interact with tab close flows.
- **`document-model`** — Each tab references a `DocumentHandle` (`Arc<RwLock<Document>>`); split views share the same handle.
- **`layout-and-docking`** — Tab_Groups are spatial containers owned by the Layout_Engine; the multi-tab subsystem populates them with tab content.
- **`command-framework`** — All tab operations are registered commands with metadata, shortcuts, and enabled predicates.
- **`configuration-system`** — Provides settings for maximum tab count, MRU mode, tab title format, overflow behaviour, pinned tab position policy.
- **`startup-and-session`** — Session restore recreates the Tab_Collection from persisted state; session save captures it.

---

## Glossary

| Term | Definition | Source |
|------|-----------|--------|
| **Tab** | A logical unit combining a DocumentHandle with its associated per-tab state (viewport position, cursor, selections, language, modified flag, undo stack reference, command line). Each Tab is identified by a unique `TabId`. | [FFE-MULTITAB] |
| **Tab_Collection** | The ordered collection of all open Tabs within a single Tab_Group. Maintains both insertion order and MRU order. | [FFE-MULTITAB], [SCI-STE-TABS] |
| **Tab_Group** | A spatial container within the center dock zone that holds one Tab_Collection. Multiple Tab_Groups can coexist via splits. Owned by the Layout_Engine (`ff-layout`). | [WB] |
| **Active_Tab** | The single Tab currently receiving user input and displayed in the editor viewport within its Tab_Group. | [FFE-MULTITAB] |
| **Tab_Bar** | The horizontal UI region within a Tab_Group that renders Tab_Headers for all tabs in that group's Tab_Collection. | [FFE-MULTITAB] |
| **Tab_Header** | A clickable element within the Tab_Bar displaying the tab title, modified indicator, pin indicator, and close button. | [FFE-MULTITAB] |
| **MRU_Stack** | A Most-Recently-Used ordering of tabs maintained alongside insertion order. Updated on every tab activation. Used for Ctrl+Tab cycling. | [SCI-STE-TABS] |
| **Pinned_Tab** | A tab marked as pinned by the user, visually distinct and immune to bulk-close operations (Close All, Close Others). Pinned tabs are always positioned at the left of the Tab_Bar. | [WB] |
| **Modified_Indicator** | A visual marker on the Tab_Header indicating the associated document has unsaved changes (typically a dot or asterisk). | [FFE-MULTITAB] |
| **Tab_Context_Menu** | The right-click context menu displayed on a Tab_Header, offering tab management actions. | [FFE-MULTITAB] |
| **Tab_Overflow** | The state where more tabs are open than can be rendered in the visible Tab_Bar width. Handled via scrolling or a dropdown list. | [SCI-STE-TABS], [WB] |
| **Split_View** | A configuration where the same Document is displayed in two or more Tab_Groups simultaneously, sharing the same DocumentHandle but maintaining independent viewport/cursor state. | [WB] |
| **TabId** | A unique, stable identifier for a tab within the workbench session. Does not change when a tab is moved or reordered. | [WB] |
| **ResourceUri** | The unified `vfs://provider/path` address for a document's backing resource, as defined by the `virtual-file-system` spec. | [WB] |
| **DocumentHandle** | An `Arc<RwLock<Document>>` providing shared ownership of a Document across tabs and background threads. | [WB] |
| **Tab_Title_Format** | The configurable strategy for displaying tab labels: filename-only, or filename with partial path disambiguation when ambiguous. | [FFE-MULTITAB], [SCI-STE-TABS] |
| **Maximum_Tab_Count** | A configurable upper limit on the number of simultaneously open tabs. When reached, the oldest non-pinned tab is closed or the user is prompted. | [SCI-STE-TABS] |

---

## Requirements

### Requirement 1: Tab Collection Management

**User Story:** As a user, I want to open multiple documents simultaneously in separate tabs within a Tab_Group, so that I can work across related files without closing and reopening them.

**Source:** FFE Reqs 1 — adapted for workbench VFS and Tab_Group integration. [FFE-MULTITAB, WB]

#### Acceptance Criteria

1. THE multi-tab subsystem SHALL maintain a Tab_Collection for each Tab_Group, where each Tab_Collection contains zero or more Tabs and each Tab holds a reference to an independent DocumentHandle plus per-tab state.
2. WHEN a document is opened (via `file.open` command, Recent Files, drag-drop, or CLI argument), THE system SHALL create a new Tab containing the DocumentHandle for that resource, insert it into the active Tab_Group's Tab_Collection, and set it as the Active_Tab.
3. WHEN `file.new` is executed, THE system SHALL create a new Tab containing an empty (untitled) DocumentHandle, insert it into the active Tab_Group's Tab_Collection, and set it as the Active_Tab.
4. THE Tab_Collection SHALL preserve insertion order so that Tabs appear in the Tab_Bar in the order they were opened (unless reordered by drag-and-drop or command).
5. WHEN a Tab is activated (clicked, keyboard-navigated, or programmatically selected), THE system SHALL update the MRU_Stack by moving that Tab to the top of the stack.
6. THE multi-tab subsystem SHALL support a configurable Maximum_Tab_Count (default: 100, minimum: 1, maximum: 500). WHEN a new tab open would exceed the Maximum_Tab_Count, THE system SHALL close the least-recently-used non-pinned, unmodified tab to make room. IF all non-pinned tabs are modified, THE system SHALL display an error notification and refuse to open the new tab.
7. THE system SHALL maintain tab-switching response time under 100 milliseconds regardless of the number of open tabs (up to the Maximum_Tab_Count).
8. IF a file cannot be opened (resource not found, permission denied, VFS provider error) during a tab-open operation, THEN THE system SHALL display an error notification containing the failure reason and ResourceUri, and leave the Tab_Collection unchanged.
9. WHEN the workbench starts with CLI file arguments, THE system SHALL create one Tab per argument in the order specified, setting the last as the Active_Tab. IF any argument cannot be opened, THE system SHALL log the error, skip that argument, and continue with the remaining arguments.
10. WHEN the workbench starts with no CLI arguments and no session to restore, THE system SHALL create a single Tab containing an empty DocumentHandle and set it as the Active_Tab.

---

### Requirement 2: Per-Tab State Isolation

**User Story:** As a user, I want each tab to maintain its own editing state independently, so that switching between tabs preserves my position, selections, command context, and undo history in each file.

**Source:** FFE Req 2 + SciTE Buffer per-buffer state (fold state, bookmarks, font override). [FFE-MULTITAB, SCI-STE-TABS]

#### Acceptance Criteria

1. THE system SHALL maintain the following independent state per Tab: viewport position (top line, horizontal scroll offset), cursor position (line and column), selection ranges (including multi-caret selections), active language definition, modified flag, command line string, status message, fold state (set of collapsed fold regions), bookmark set, and a reference to the Tab's undo/redo TransactionStack within its Document.
2. WHEN the user switches from one Tab to another within the same Tab_Group, THE system SHALL persist the departing Tab's viewport position, cursor position, selection ranges, scroll offset, and command line string, then restore the arriving Tab's previously persisted state.
3. WHEN the user switches Tabs, THE system SHALL update the status bar to reflect the arriving Tab's language, encoding, line ending mode, cursor position, and modification state.
4. THE system SHALL maintain a separate TransactionStack (undo/redo history) for each Document. Tabs sharing the same DocumentHandle (split views) SHALL share the same TransactionStack.
5. THE system SHALL maintain a separate active language definition for each Tab, determined by the Tab's resource extension or content-based detection, independent of other Tabs.
6. WHEN a Tab's Document is modified (content changes via any edit operation), THE system SHALL set that Tab's modified flag to `true` and update the Tab_Header's Modified_Indicator.
7. WHEN a Tab's Document is saved successfully (save-point reached), THE system SHALL set that Tab's modified flag to `false` and remove the Modified_Indicator from the Tab_Header.
8. THE per-tab state SHALL be serialisable to the session file format defined by the `startup-and-session` spec, enabling full state restoration on workbench restart.

---

### Requirement 3: Tab Bar Display and Title Formatting

**User Story:** As a user, I want a tab bar showing clear, unambiguous titles for all open documents, with visual indicators for modification and pin state, so that I can identify and switch between files efficiently.

**Source:** FFE Req 3 + SciTE title formatting + workbench Tab_Group integration. [FFE-MULTITAB, SCI-STE-TABS, WB]

#### Acceptance Criteria

1. THE system SHALL render a Tab_Bar horizontally at the top of each Tab_Group, displaying one Tab_Header for each Tab in that group's Tab_Collection in insertion order (left to right), with pinned tabs always preceding unpinned tabs.
2. WHEN a Tab has an associated ResourceUri, THE Tab_Header SHALL display the file name (final path segment) as the tab label.
3. WHEN a Tab has no associated ResourceUri (untitled document), THE Tab_Header SHALL display "Untitled" as the tab label, with a numeric suffix for disambiguation when multiple untitled tabs exist (e.g., "Untitled", "Untitled-2", "Untitled-3").
4. IF two or more Tabs in the same Tab_Group share the same file name but different ResourceUris, THEN THE system SHALL append the minimum disambiguating parent directory segment(s) in parentheses (e.g., "main.rs (src/editor)" and "main.rs (src/commands)").
5. THE Active_Tab's Tab_Header SHALL be visually distinct from inactive Tab_Headers through a highlighted background colour consistent with the active theme.
6. WHEN a Tab's Document has unsaved modifications, THE Tab_Header SHALL display a Modified_Indicator (a filled circle "●" or asterisk "*" depending on configuration) adjacent to the tab title.
7. WHEN a Tab is pinned, THE Tab_Header SHALL display a pin icon and render with a more compact width (showing only the icon and optionally a truncated title).
8. EACH Tab_Header SHALL display a Close_Button ("×") to the right of the tab label. For pinned tabs, the Close_Button SHALL be hidden unless hovered.
9. WHEN the user left-clicks a Tab_Header, THE system SHALL activate that Tab (switch to it as the Active_Tab and update the MRU_Stack).
10. THE Close_Button SHALL appear visually muted on inactive tabs and prominent on the active tab or when the Tab_Header is hovered.
11. WHEN the user clicks the Close_Button on a Tab_Header, THE system SHALL execute the `tabs.close` command for that specific Tab, following close operation rules from Requirement 5.
12. THE Tab_Title_Format SHALL be configurable via the configuration-system with options: `filename_only` (default), `filename_with_directory` (always show one parent), or `auto_disambiguate` (show parent only when needed).

---

### Requirement 4: Tab Overflow Handling

**User Story:** As a user, I want to access all open tabs even when they exceed the visible Tab_Bar width, so that I don't lose track of open documents in a crowded workspace.

**Source:** FFE Req 3.9 expanded + SciTE buffer list dropdown. [FFE-MULTITAB, SCI-STE-TABS, WB]

#### Acceptance Criteria

1. WHEN the Tab_Collection contains more Tabs than can fit in the visible Tab_Bar width, THE system SHALL enter overflow mode, rendering only the tabs that fit and providing navigation controls for the hidden tabs.
2. WHILE in overflow mode, THE Tab_Bar SHALL display left/right scroll arrows (or equivalent navigation controls) at the ends of the Tab_Bar, allowing the user to scroll hidden tabs into view.
3. WHILE in overflow mode, THE Tab_Bar SHALL display a dropdown button that, when clicked, presents a scrollable list of all tabs in the Tab_Collection with their titles and modified indicators, allowing the user to activate any tab directly.
4. THE Active_Tab SHALL always be scrolled into the visible portion of the Tab_Bar. WHEN a tab outside the visible range is activated, THE Tab_Bar SHALL scroll to bring it into view.
5. THE overflow dropdown list SHALL indicate the Active_Tab with a checkmark or highlight, and SHALL indicate modified tabs with their Modified_Indicator.
6. THE overflow dropdown list SHALL support type-ahead filtering: as the user types characters while the dropdown is open, the list SHALL filter to show only tabs whose titles contain the typed characters (case-insensitive).
7. WHEN a tab is activated from the overflow dropdown, THE system SHALL close the dropdown and scroll the Tab_Bar to show the newly active tab.

---

### Requirement 5: Tab Close Operations

**User Story:** As a user, I want to close tabs individually or in groups with appropriate save prompts, so that I can manage my workspace without accidentally losing work.

**Source:** FFE Reqs 4, 10 — adapted with pinned tab protection. [FFE-MULTITAB, WB]

#### Acceptance Criteria

1. WHEN the `tabs.close` command is executed on a Tab that has no unsaved modifications, THE system SHALL remove that Tab from the Tab_Collection immediately.
2. IF a Tab has unsaved modifications WHEN a close operation is initiated, THEN THE system SHALL display an unsaved-changes confirmation dialog presenting exactly three options: Save, Discard, and Cancel.
3. WHEN the user selects "Save" in the close confirmation dialog and the save completes successfully, THE system SHALL remove the Tab from the Tab_Collection.
4. WHEN the user selects "Discard" in the close confirmation dialog, THE system SHALL remove the Tab from the Tab_Collection without saving.
5. WHEN the user selects "Cancel" in the close confirmation dialog, THE system SHALL abort the close operation and leave the Tab unchanged.
6. WHEN the last Tab in a Tab_Group is closed, THE system SHALL create a new Tab containing an empty DocumentHandle (the Tab_Group never has zero Tabs unless it is being removed by the Layout_Engine).
7. WHEN the Active_Tab is closed and other Tabs remain, THE system SHALL activate the next tab in MRU order (the most recently used remaining tab). IF MRU order is disabled in configuration, THE system SHALL activate the Tab immediately to the right; IF no Tab exists to the right, the Tab immediately to the left.
8. WHEN `tabs.close_all` is executed, THE system SHALL close all non-pinned Tabs in the Tab_Group in left-to-right order, following the confirmation dialog for each modified Tab. Pinned Tabs SHALL remain open.
9. WHEN `tabs.close_others` is executed, THE system SHALL close all non-pinned Tabs except the target Tab, following the confirmation dialog for each modified Tab.
10. IF the user selects "Cancel" during any bulk close operation, THEN THE system SHALL abort the entire bulk operation, leaving all remaining Tabs (including the current one) unchanged.
11. WHEN the user initiates application exit and one or more Tabs have unsaved modifications, THE system SHALL present the unsaved-changes confirmation dialog for each modified Tab in Tab_Bar order across all Tab_Groups.
12. IF the user selects "Cancel" for any modified Tab during exit, THEN THE system SHALL abort the exit operation and return to the workbench with all Tabs intact.
13. WHEN all modified Tabs during exit have been handled (saved or discarded), THE system SHALL proceed with the exit sequence (session save, plugin shutdown, window close).
14. WHEN a pinned Tab's close button is clicked, THE system SHALL unpin the tab rather than closing it. To close a pinned Tab, the user must explicitly select "Close" from the context menu or use `tabs.close_pinned` command.

---

### Requirement 6: Tab Context Menu

**User Story:** As a user, I want a right-click context menu on tabs with comprehensive tab management operations, so that I can efficiently manage my open files and work contexts without navigating top-level menus.

**Source:** [ISPF-POM] + FFE Req 5 — expanded with ISPF-style operations. [FFE-MULTITAB, WB, ISPF-POM]

#### Acceptance Criteria

1. WHEN the user right-clicks a Tab_Header, THE system SHALL display a Tab_Context_Menu positioned at the cursor location.
2. THE Tab_Context_Menu contents SHALL be determined by the kind of the right-clicked tab, as defined in criteria 6.2a–6.2c.

   6.2a The following items SHALL appear for ALL tab kinds:
   - Close
   - Close All BUT This
   - Close All to the Left
   - Close All to the Right
   - Close All Unchanged
   - *(separator)*
   - Clone to Other Tab
   - Move to Other View
   - *(separator)*
   - Pin Tab / Unpin Tab *(toggle based on current pin state)*
   - *(separator)*
   - Exit

   6.2b The following items SHALL appear ONLY when the right-clicked tab is a file editor tab:
   - *(separator)*
   - Open Containing Folder in Explorer
   - Open Containing Folder in CMD
   - Open Containing Folder in PowerShell
   - Open Containing Folder in Terminal
   - *(separator)*
   - Rename
   - Copy Name to Clipboard
   - Copy Path to Clipboard
   - *(separator)*
   - Read-Only
   - Clear Read-Only Flag
   - *(separator)*
   - Save
   - Save As
   - Reload

   6.2c The Tab_Context_Menu for a Primary Option Menu tab SHALL contain ONLY the universal items from 6.2a. No file-specific items SHALL appear — not even in a disabled state.
3. WHEN "Close" is selected, THE system SHALL execute `tabs.close` on the right-clicked Tab, following Requirement 5 unsaved-changes rules.
4. WHEN "Close All BUT This" is selected, THE system SHALL close all tabs except the right-clicked tab, following Requirement 5 confirmation rules for each modified tab.
5. WHEN "Close All to the Left" is selected, THE system SHALL close all non-pinned Tabs positioned to the left of the right-clicked Tab, following Requirement 5 confirmation rules for each modified Tab.
6. WHEN "Close All to the Right" is selected, THE system SHALL close all non-pinned Tabs positioned to the right of the right-clicked Tab, following Requirement 5 confirmation rules for each modified Tab.
7. WHEN "Close All Unchanged" is selected, THE system SHALL close all tabs that have no unsaved modifications, leaving modified tabs open. No confirmation dialog is required.
8. WHEN "Clone to Other Tab" is selected, THE system SHALL create a duplicate of the right-clicked tab (same content, same type) as a new tab appended to the tab bar.
9. WHEN "Move to Other View" is selected, THE system SHALL detach the right-clicked tab into a new floating OS window.
10. WHEN "Open Containing Folder in Explorer" is selected on a file tab, THE system SHALL open the folder containing the file in the platform file explorer.
11. WHEN "Open Containing Folder in CMD" is selected on a file tab, THE system SHALL open a CMD window at the folder containing the file.
12. WHEN "Open Containing Folder in PowerShell" is selected on a file tab, THE system SHALL open a PowerShell window at the folder containing the file.
13. WHEN "Open Containing Folder in Terminal" is selected on a file tab, THE system SHALL open the platform default terminal at the folder containing the file.
14. WHEN "Rename" is selected on a file tab, THE system SHALL allow the user to rename the file on disk and update the tab title accordingly.
15. WHEN "Copy Name to Clipboard" is selected, THE system SHALL copy the file name (without path) to the system clipboard.
16. WHEN "Copy Path to Clipboard" is selected, THE system SHALL copy the full absolute path of the file to the system clipboard.
17. WHEN "Read-Only" is selected on a writable file tab, THE system SHALL set the tab to read-only mode, preventing edits and updating the tab header with a read-only indicator.
18. WHEN "Clear Read-Only Flag" is selected on a read-only file tab, THE system SHALL restore the tab to editable mode.
19. WHEN "Pin Tab" is selected, THE system SHALL mark the right-clicked Tab as pinned, move it to the rightmost position among pinned tabs, and update the Tab_Header to show the pin indicator.
20. WHEN "Unpin Tab" is selected (shown only for pinned tabs), THE system SHALL remove the pin flag, move the tab to the leftmost position among unpinned tabs, and update the Tab_Header.
21. WHEN "Save" is selected on a modified file tab, THE system SHALL save the file to disk.
22. WHEN "Save As" is selected on a file tab, THE system SHALL prompt the user for a new file path and save the content there, updating the tab title.
23. WHEN "Reload" is selected on a file tab, THE system SHALL reload the file content from disk. IF the tab has unsaved modifications, THE system SHALL prompt the user to confirm discarding changes before reloading.
24. File-specific Tab_Context_Menu items (those listed in 6.2b) SHALL be OMITTED ENTIRELY from the menu when the right-clicked tab is not a file editor tab — they SHALL NOT appear in a disabled or greyed-out state.
25. WHILE there are no Tabs to the left of the right-clicked Tab, THE "Close All to the Left" menu item SHALL appear disabled.
26. WHILE there are no Tabs to the right of the right-clicked Tab, THE "Close All to the Right" menu item SHALL appear disabled.
27. WHILE only one Tab exists in the Tab_Group, THE "Close All BUT This" menu item SHALL appear disabled.
28. WHEN the user selects "Exit" from the Tab_Context_Menu, THE system SHALL initiate the application exit sequence, closing the entire application.

---

### Requirement 7: MRU Tab Ordering

**User Story:** As a user, I want Ctrl+Tab to cycle through tabs in most-recently-used order, so that I can quickly switch back to the file I was editing previously — similar to Alt+Tab window switching in operating systems.

**Source:** SciTE `IDM_PREVFILESTACK`/`IDM_NEXTFILESTACK` + workbench configuration. [SCI-STE-TABS, WB]

#### Acceptance Criteria

1. THE system SHALL maintain an MRU_Stack per Tab_Group that records the order in which tabs were last activated, with the most recently activated tab at the top.
2. WHEN a Tab is activated (by any means: click, keyboard shortcut, command, open), THE system SHALL move that Tab to the top of the MRU_Stack.
3. WHEN Ctrl+Tab is pressed, THE system SHALL begin an MRU navigation session and activate the second tab in MRU order (the previously active tab). Subsequent Ctrl+Tab presses while Ctrl is held SHALL cycle deeper into the MRU_Stack.
4. WHEN Ctrl+Shift+Tab is pressed during an MRU navigation session, THE system SHALL cycle backwards (towards more recently used tabs) in the MRU_Stack.
5. WHEN the Ctrl key is released after an MRU navigation session, THE system SHALL commit the currently displayed tab as the new MRU top and end the navigation session.
6. WHILE an MRU navigation session is active, THE system SHALL display a transient popup showing the MRU-ordered tab list with the current selection highlighted, allowing the user to see which tab they will land on.
7. THE MRU navigation mode SHALL be configurable: `mru` (default — cycle in MRU order) or `sequential` (cycle in Tab_Bar insertion order). WHEN sequential mode is configured, Ctrl+Tab SHALL move to the next tab to the right (wrapping) and Ctrl+Shift+Tab to the left (wrapping).
8. WHEN a Tab is closed, THE system SHALL remove it from the MRU_Stack. The ordering of remaining tabs in the stack SHALL be preserved.
9. THE MRU_Stack SHALL be serialised as part of the session state so that MRU order is preserved across workbench restarts.

---

### Requirement 8: Tab Keyboard Navigation

**User Story:** As a user, I want keyboard shortcuts to switch between tabs, create new tabs, and close tabs, so that I can navigate open files without reaching for the mouse.

**Source:** FFE Req 6 + SciTE keyboard bindings + workbench command framework. [FFE-MULTITAB, SCI-STE-TABS, WB]

#### Acceptance Criteria

1. WHEN the user presses Ctrl+Tab, THE system SHALL switch to the next tab per the configured MRU or sequential mode (see Requirement 7).
2. WHEN the user presses Ctrl+Shift+Tab, THE system SHALL switch to the previous tab per the configured mode.
3. WHEN the user presses Ctrl+W, THE system SHALL execute `tabs.close` on the Active_Tab, following Requirement 5 close rules.
4. WHEN the user presses Ctrl+N, THE system SHALL execute `file.new`, creating a new Tab with an empty DocumentHandle and setting it as the Active_Tab.
5. WHEN the user presses Ctrl+1 through Ctrl+9, THE system SHALL activate the tab at position 1 through 9 in the Tab_Bar (insertion order, 1-indexed). Ctrl+9 SHALL always activate the last tab regardless of position count.
6. IF the user presses Ctrl+N (where N is a digit) and the Tab_Collection has fewer than N tabs, THEN THE system SHALL take no action (no error, no new tab).
7. WHEN the user presses Ctrl+Shift+T, THE system SHALL reopen the most recently closed tab (restoring its ResourceUri and loading the resource from VFS). THE system SHALL maintain a stack of up to 20 recently closed tab URIs.
8. ALL keyboard shortcuts SHALL be configurable through the command framework's Shortcut_Registry, allowing users to rebind or disable any tab navigation shortcut.

---

### Requirement 9: Tab Drag-and-Drop Reordering

**User Story:** As a user, I want to drag tabs to reorder them within a Tab_Group or move them between Tab_Groups, so that I can organize my workspace by grouping related files together.

**Source:** FFE Req 7 + SciTE `IDM_MOVETABRIGHT`/`IDM_MOVETABLEFT` + workbench Layout_Engine Tab_Group moves. [FFE-MULTITAB, SCI-STE-TABS, WB]

#### Acceptance Criteria

1. WHEN the user presses and holds the left mouse button on a Tab_Header and moves the cursor horizontally beyond a 5-pixel dead zone, THE system SHALL initiate a drag operation, visually indicating the Tab is being dragged (e.g., semi-transparent ghost of the Tab_Header attached to the cursor).
2. WHILE a drag operation is active within the same Tab_Bar, THE system SHALL display a drop indicator (vertical line or highlighted gap) showing where the Tab will be inserted if released at the current cursor position.
3. WHEN the user releases the mouse button during a drag operation within the same Tab_Bar, THE system SHALL move the dragged Tab to the indicated insertion position in the Tab_Collection, preserving all per-tab state.
4. IF the user releases the mouse button at the Tab's original position during a drag operation, THEN THE system SHALL take no action (no reorder occurs).
5. WHEN a Tab is dragged over the Tab_Bar of a different Tab_Group within the same window, THE system SHALL display a drop indicator in that Tab_Bar and, upon release, move the Tab from its original Tab_Group to the target Tab_Group at the indicated position.
6. WHEN a Tab is the last tab in a Tab_Group and is dragged to another Tab_Group, THE Layout_Engine SHALL close the now-empty Tab_Group and redistribute its space (delegated to `layout-and-docking`).
7. WHEN a Tab is dragged outside all Tab_Bars and released over the editor content area between Tab_Groups, THE system SHALL create a new Tab_Group at the drop location (split) and place the Tab there.
8. WHILE a pinned Tab is being dragged, THE system SHALL constrain its drop position to the pinned tab region (left side of the Tab_Bar, before all unpinned tabs).
9. WHEN a Tab is reordered via drag-and-drop, THE MRU_Stack position of that Tab SHALL remain unchanged (reordering does not affect MRU).
10. THE drag-and-drop operation SHALL support pressing Escape to cancel the drag, returning the Tab to its original position.

---

### Requirement 10: Pinned Tabs

**User Story:** As a user, I want to pin important tabs so they stay open and are protected from bulk-close operations, so that I don't accidentally close files I'm actively working on.

**Source:** NEW — workbench concept adapted from VS Code and modern editors. [WB]

#### Acceptance Criteria

1. WHEN the `tabs.pin` command is executed on an unpinned Tab, THE system SHALL mark the Tab as pinned, move it to the rightmost position among existing pinned tabs in the Tab_Bar, and update the Tab_Header to display the pin icon.
2. WHEN the `tabs.unpin` command is executed on a pinned Tab, THE system SHALL remove the pin flag, move the Tab to the leftmost position among unpinned tabs, and remove the pin icon from the Tab_Header.
3. PINNED Tabs SHALL always appear to the left of all unpinned Tabs in the Tab_Bar. The relative order among pinned tabs SHALL be maintained based on their pin sequence.
4. PINNED Tabs SHALL be immune to `tabs.close_all` and `tabs.close_others` bulk-close operations. Only explicit `tabs.close` targeting a pinned tab (via context menu "Close" or `tabs.close_pinned` command) SHALL close a pinned Tab.
5. PINNED Tab_Headers SHALL render with a compact width (pin icon + optional truncated title) to conserve Tab_Bar space.
6. WHEN a pinned Tab has unsaved modifications, THE Modified_Indicator SHALL be displayed on the pinned Tab_Header alongside the pin icon.
7. THE pinned state of each Tab SHALL be serialised as part of the session state, so that pinned tabs remain pinned across workbench restarts.
8. WHEN a pinned Tab is duplicated (via context menu or command), THE duplicate SHALL be created as an unpinned Tab.

---

### Requirement 11: Duplicate Detection

**User Story:** As a user, I want the editor to detect when I try to open a file that's already open in another tab, so that I don't create duplicate tabs and accidentally edit the same file in two places with diverging state.

**Source:** FFE Req 1.6 — adapted for VFS ResourceUri-based identity. [FFE-MULTITAB, WB]

#### Acceptance Criteria

1. IF the resource being opened is already open in an existing Tab within any Tab_Group (determined by comparing canonicalized ResourceUris), THEN THE system SHALL activate the existing Tab (switch to its Tab_Group and set it as Active_Tab) instead of creating a duplicate.
2. THE duplicate detection SHALL operate across all Tab_Groups in the workbench — a resource open in Tab_Group A will be detected when the same resource is opened in Tab_Group B.
3. THE duplicate detection SHALL normalize ResourceUris before comparison: for local filesystem URIs, the system SHALL resolve symlinks, normalize case on case-insensitive filesystems, and resolve relative segments.
4. WHEN a duplicate is detected and the existing Tab is in a different Tab_Group than the one requesting the open, THE system SHALL focus the Tab_Group containing the existing Tab and activate that Tab.
5. IF the user explicitly requests opening the same resource in a split view (via "Split Right", "Split Down", or a command parameter), THEN THE system SHALL create a new Tab sharing the same DocumentHandle rather than rejecting the open as a duplicate. This is governed by Requirement 12 (Split Editor).

---

### Requirement 12: Split Editor (Same Document in Multiple Views)

**User Story:** As a user, I want to open the same document in multiple Tab_Groups simultaneously, so that I can view and edit different sections of a large file side-by-side without scrolling back and forth.

**Source:** NEW — workbench concept for multi-view editing. Layout_Engine provides Tab_Group splits; this requirement defines the tab-level semantics. [WB]

#### Acceptance Criteria

1. WHEN the user executes `tabs.split_right` or selects "Split Right" from the context menu, THE system SHALL request a horizontal split from the Layout_Engine and create a new Tab in the new Tab_Group that shares the same DocumentHandle as the source Tab.
2. WHEN the user executes `tabs.split_down` or selects "Split Down" from the context menu, THE system SHALL request a vertical split and create a new Tab sharing the same DocumentHandle.
3. TABS sharing the same DocumentHandle SHALL maintain independent per-tab state: viewport position, cursor position, and selections SHALL be independent in each view. Edits, undo/redo, modification state, and language definition SHALL be shared (since they are properties of the Document).
4. WHEN an edit is made in one view of a shared Document, THE system SHALL immediately reflect the change in all other views of the same Document (content stays synchronized).
5. WHEN a shared Document is saved, THE Modified_Indicator SHALL be cleared on all Tab_Headers that reference that DocumentHandle.
6. WHEN one view of a shared Document is closed and other views remain, THE Document SHALL remain in memory (reference-counted via DocumentHandle). THE Document SHALL only be released when the last Tab referencing it is closed.
7. THE Split_View Tab_Header SHALL display the same title as the original Tab, with an optional view indicator (e.g., "[1]", "[2]") when configured to distinguish multiple views of the same resource.

---

### Requirement 13: Command Framework Tab Integration

**User Story:** As a user, I want all tab operations to be accessible as named commands in the command framework, so that they can be invoked from keyboard shortcuts, menus, macros, and plugins uniformly.

**Source:** FFE Reqs 8, 9 — adapted for workbench command framework. [FFE-MULTITAB, WB]

#### Acceptance Criteria

1. THE multi-tab subsystem SHALL register the following commands with the Command_Registry during initialization: `tabs.close`, `tabs.close_all`, `tabs.close_others`, `tabs.close_to_left`, `tabs.close_to_right`, `tabs.close_pinned`, `tabs.next`, `tabs.previous`, `tabs.next_mru`, `tabs.previous_mru`, `tabs.pin`, `tabs.unpin`, `tabs.move_left`, `tabs.move_right`, `tabs.goto_1` through `tabs.goto_9`, `tabs.split_right`, `tabs.split_down`, `tabs.reopen_closed`, `tabs.duplicate`.
2. WHEN a command is executed via the command line or shortcut, THE system SHALL operate on the Active_Tab's Tab_Group unless the command parameters specify a target TabId.
3. WHEN `file.save` or `file.save_as` is executed, THE system SHALL operate on the Active_Tab's DocumentHandle and update the Active_Tab's Tab_Header to reflect any new resource name.
4. WHEN `file.new` is executed, THE system SHALL create a new Tab (not replace the current one) and set it as Active_Tab.
5. WHEN `file.open` is executed and a resource is selected, THE system SHALL open the resource in a new Tab (following Requirement 1 and Requirement 11 duplicate detection).
6. WHEN a recent file is selected from the Recent_Files_List, THE system SHALL open it in a new Tab following the same rules as `file.open`.
7. THE system SHALL NOT modify any Tab other than the target Tab during command execution. Commands that operate on multiple tabs (bulk close) SHALL iterate explicitly.
8. EACH registered tab command SHALL include Command_Metadata with: display name, description, category "Tabs", default keyboard shortcut, and an enabled predicate that disables the command when it cannot meaningfully execute (e.g., `tabs.close_to_left` disabled when the tab is leftmost).

---

### Requirement 14: Session Persistence Contract

**User Story:** As a user, I want my open tabs, their order, pinned state, and per-tab state to be restored exactly when I reopen the workbench, so that I can resume work without manually reopening files and finding my place.

**Source:** FFE startup-and-session contract + SciTE session file + workbench session model. [FFE-MULTITAB, SCI-STE-TABS, WB]

#### Acceptance Criteria

1. THE multi-tab subsystem SHALL expose a `serialize_tab_collection()` method that produces a serialisable representation of the complete tab state for a Tab_Group: ordered list of TabIds, each with ResourceUri (or untitled marker), viewport position, cursor position, selection ranges, pinned flag, and language override (if any).
2. THE multi-tab subsystem SHALL expose a `deserialize_tab_collection(data)` method that reconstructs a Tab_Collection from serialised data, opening resources via the VFS and restoring per-tab state.
3. WHEN session restore encounters a ResourceUri that cannot be opened (file deleted, provider unavailable), THE system SHALL skip that tab, log a warning, and continue restoring the remaining tabs without aborting the entire restore.
4. THE serialised Tab_Collection SHALL include the MRU_Stack ordering so that Ctrl+Tab order is preserved across restarts.
5. THE serialised Tab_Collection SHALL include the Active_Tab identifier so that the previously active tab is re-activated after restore.
6. THE session serialisation format SHALL be versioned. WHEN the system encounters an older format version, it SHALL attempt to migrate the data. IF migration fails, THE system SHALL discard the stored tab state, log a warning, and start with a single empty Tab.

---

### Requirement 15: Tab Window Chrome

**User Story:** As a user, I want every tab to display a consistent three-element header
(Tab_Header row, Title_Line, Command_Line) so that the layout is predictable regardless
of tab kind or whether the tab is docked or floating.

**Source:** ISPF 3270 screen layout convention; menu-and-statusbar Requirement 17; user
requirement (Phase AL).

#### Acceptance Criteria

15.1 EACH Tab SHALL expose a `title_line_text() -> String` method that returns the
     context-appropriate text for the Title_Line:
     - POM tab: `"FileForge Workbench  v{version}"`
     - File editor tab with open file: full absolute path of the file
     - File editor tab untitled: `"[Untitled]"`
     - All other tab kinds: the tab's title string

15.2 THE Tab_Bar rendering contract (TabBarModel) SHALL include the Title_Line text for
     the active tab so that the GUI shell can render it without querying the document model
     directly.

15.3 WHEN the active tab changes, THE Title_Line text SHALL update within one frame to
     reflect the newly active tab's context.

15.4 THE per-tab state (Requirement 2) SHALL include the Title_Line text as a derived,
     read-only field — it is computed from the tab's ResourceUri and kind, not stored
     independently.
