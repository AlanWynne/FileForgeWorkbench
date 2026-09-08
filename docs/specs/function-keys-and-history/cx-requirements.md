# Requirements Document -- Named Workspaces, KEYS Name Argument, and SPLIT Alias (Phase CX)

## Introduction

Phase CX adds three related capabilities:

1. **Named Workspaces** (CR-NR-046): Each Workspace (tab) gains a user-visible
   name string. The name is displayed in the tab header and can be set or
   changed at any time.

2. **KEYS name argument** (CR-NR-046): The `KEYS` command is extended to accept
   an optional name argument (`KEYS <name>`) so the user can open the Key
   Configuration Dialog pre-loaded with any named key map, not just the current
   Workspace's map.

3. **SPLIT command alias** (CR-CH-010): A `SPLIT` primary command is added as
   an ISPF-heritage alias for the existing Workspace detach operation
   (Move to Other View / Ctrl+Shift+T). This replaces the ISPF split-screen
   concept with modern OS window management.

   Note: The existing `SPLIT` command in the editor context implements
   split-screen view (menu-and-statusbar Req 19.11). That behaviour is
   preserved for Editor Context Workspaces. The new `SPLIT` alias for detach
   applies when the active Workspace is NOT an Editor Context (i.e., POM,
   Settings, Files, etc.), or when the user explicitly types `SPLIT DETACH`.

### Source References

- **[CR-NR-046]** = Change log entry for Named Workspaces and Per-Workspace KEYS
- **[CR-CH-010]** = Change log entry for SPLIT Command Alias for Workspace Detach
- **[ISPF-POM]** = IBM ISPF Primary Option Menu heritage

---

## Glossary

| Term | Definition |
|------|-----------|
| **Workspace_Name** | A user-visible string label assigned to a Workspace (tab). Distinct from the tab title (which is derived from content). |
| **Named_Key_Map** | A key map identified by a user-chosen name string, stored in `[context_key_maps.<name>]` in the workbench configuration. |
| **Detach_Alias** | The `SPLIT` command when used to detach the current Workspace into a separate OS window. |

---

## Requirement 1: Workspace Name Property

**User Story:** As a workbench user, I want to assign a name to each Workspace
so that I can identify it in the tab bar and reference it by name in commands
like `KEYS <name>`.

**Source:** [CR-NR-046]

### Acceptance Criteria

1. EACH `TabState` SHALL have a `workspace_name: Option<String>` field. When
   `None`, the Workspace has no user-assigned name and the tab header displays
   only the content-derived title (existing behaviour).

2. WHEN the user types `NAME <text>` in any `Command ===>` field and presses
   Enter, THE shell SHALL set the active Workspace's `workspace_name` to
   `<text>` (trimmed, max 32 characters).

3. WHEN the user types `NAME` with no argument, THE shell SHALL clear the
   active Workspace's `workspace_name` (set to `None`), restoring the
   content-derived title.

4. WHEN a Workspace has a `workspace_name` set, THE tab header SHALL display
   the name in addition to (or instead of) the content-derived title, in the
   format `[<name>]` for system tabs or `<name>: <title>` for file editor tabs.

5. THE `workspace_name` SHALL be persisted in the session state alongside the
   existing tab state fields and restored on next launch.

6. THE `workspace_name` SHALL be included in the `PersistedTab` session TOML
   as an optional `workspace_name` string field.

---

## Requirement 2: KEYS Command Name Argument

**User Story:** As a workbench user, I want to type `KEYS <name>` to open the
Key Configuration Dialog pre-loaded with the named key map, so that I can edit
any key map by name without first switching to a Workspace that uses it.

**Source:** [CR-NR-046]

### Acceptance Criteria

1. WHEN the user types `KEYS` with no argument, THE shell SHALL open the Key
   Configuration Dialog with the Default (Global) scope tab active, as before.

2. WHEN the user types `KEYS <name>` where `<name>` matches a context name
   registered in `[context_key_maps]` (e.g., `KEYS editor`, `KEYS pom`), THE
   shell SHALL open the Key Configuration Dialog with the tab for that context
   name pre-selected.

3. WHEN the user types `KEYS <name>` where `<name>` does not match any
   registered context name, THE shell SHALL open the Key Configuration Dialog
   with the Default scope tab active and display a status message:
   `Key map '<name>' not found -- showing Default map.`

4. THE `KEYS <name>` argument matching SHALL be case-insensitive.

5. THE Key Configuration Dialog SHALL gain a read-only `Map Name` field in the
   header area showing the name of the currently selected scope tab, so the
   user can confirm which map they are editing.

---

## Requirement 3: SPLIT Command as Workspace Detach Alias

**User Story:** As an ISPF-familiar operator, I want to type `SPLIT` to detach
the current Workspace into its own OS window, consistent with the ISPF heritage
of `SPLIT` meaning "open a new independent view".

**Source:** [CR-CH-010]

### Acceptance Criteria

1. WHEN the user types `SPLIT DETACH` in any `Command ===>` field and presses
   Enter, THE shell SHALL detach the current Workspace into a separate OS
   window, identical to the existing "Move to Other View" context menu action
   (layout-and-docking Req 3.1).

2. WHEN the user types `SPLIT` (no argument) in a `Command ===>` field of a
   non-Editor-Context Workspace (POM, Settings, Files, Plugins, etc.), THE
   shell SHALL detach the current Workspace, identical to `SPLIT DETACH`.

3. WHEN the user types `SPLIT` (no argument) in a `Command ===>` field of an
   Editor Context Workspace, THE shell SHALL perform the existing split-screen
   operation (menu-and-statusbar Req 19.11), preserving backward compatibility.

4. THE `SPLIT DETACH` form SHALL work from any Workspace kind including Editor
   Context, providing an explicit way to detach even from an editor tab.

5. WHEN `SPLIT DETACH` is issued and the 16-window limit has been reached, THE
   shell SHALL display the existing status message indicating the maximum has
   been reached and SHALL NOT detach.

6. THE `SPLIT` command SHALL be registered in the command framework with
   Command_ID `layout.split` for the detach variant.

---

## Requirement 4: Session Persistence for Workspace Names

**User Story:** As a workbench user, I want my Workspace names to survive
restarts so that my named tabs are restored exactly as I left them.

**Source:** [CR-NR-046]

### Acceptance Criteria

1. WHEN the session is saved (on exit or periodic save), THE `workspace_name`
   field of each tab SHALL be written to `session.toml` as an optional string
   field in the tab's persisted state.

2. WHEN the session is restored on launch, THE `workspace_name` SHALL be read
   from `session.toml` and applied to the restored `TabState`.

3. WHEN a persisted tab entry has no `workspace_name` field (older session
   file), THE restored tab SHALL have `workspace_name = None` -- backward
   compatible with existing session files.
