# Requirements Document -- Workspace Model

## Introduction

This spec defines the Workspace Model for FileForgeWorkbench. A workspace is a named,
persistable grouping of one or more root directories (catalog mount points) together with
workspace-scoped settings, a recent-files list, and a workspace file that can be saved,
opened, and shared. The workspace model is the foundational architectural layer that
unblocks the Command Palette (which needs a workspace scope for search) and Global Search
(which needs a workspace root set).

The workspace model is implemented in the `ff-session` crate (extended) and wired into
`ff-desktop`. It does not introduce a new library crate -- it extends the existing session
and configuration layers.

**Source references:**
- **WB** = Workbench Architecture Brief -- workspace layer concept
- **GAP** = Phase BQ gap-analysis.md section 2.1 (Multi-Root Workspaces, High priority)
- **EXEC** = Phase BQ executive-assessment.md Recommendation 2

## Glossary

- **Workspace**: A named collection of root directories (catalog mount points), workspace-scoped
  settings, and a recent-files list, persisted as a `workspace.toml` file.
- **Workspace_File**: A TOML file (extension `.ffwb-workspace`) that stores the workspace
  definition: name, root paths, workspace-scoped config overrides, and MRU list.
- **Workspace_Root**: A directory path registered as a top-level entry point in the workspace.
  Corresponds to a catalog mount point in the Virtual Catalog Manager.
- **Workspace_Settings**: Configuration key overrides scoped to the workspace, stored in the
  Workspace_File and applied as the Workspace layer in the configuration precedence chain.
- **Active_Workspace**: The single workspace currently loaded in the workbench session.
  At most one workspace is active at any time.
- **MRU_List**: The most-recently-used files list, scoped per workspace.

---

## Requirements

### Requirement 1: Workspace File Format

**User Story:** As a developer, I want workspaces stored as human-readable TOML files so
that I can share them with teammates via version control and open them with any text editor.

**Source:** WB, GAP 2.1

#### Acceptance Criteria

1. WHEN a workspace is saved, THE workbench SHALL write a TOML file with extension
   `.ffwb-workspace` containing: `name` (string), `roots` (array of path strings),
   `[settings]` table of workspace-scoped config overrides, and `[[recent_files]]` array
   of recently opened file paths with timestamps.

2. THE Workspace_File format SHALL be valid TOML v1.0 and SHALL be human-readable and
   editable with any text editor without special tooling.

3. WHEN a Workspace_File is opened, THE workbench SHALL validate that all required fields
   (`name`, `roots`) are present; IF any required field is missing, THE workbench SHALL
   display an error message and SHALL NOT load the workspace.

4. THE `roots` array SHALL contain absolute path strings; relative paths SHALL be resolved
   relative to the directory containing the Workspace_File at load time.

5. THE `[settings]` table SHALL use the same key namespace as the main configuration system
   (`editor.*`, `theme.*`, etc.) and SHALL be applied as the Workspace layer in the
   configuration precedence chain (highest priority, above Project layer).

---

### Requirement 2: Workspace Lifecycle Commands

**User Story:** As a developer, I want Open Workspace, Save Workspace, and Close Workspace
commands so that I can switch between project contexts without restarting the workbench.

**Source:** GAP 2.1, WB

#### Acceptance Criteria

1. WHEN the user issues `WORKSPACE OPEN <path>` or selects `File > Open Workspace...`,
   THE workbench SHALL load the Workspace_File at the given path, register its roots as
   catalog mount points, apply its Workspace_Settings to the configuration layer, and
   restore its MRU_List as the active recent-files list.

2. WHEN the user issues `WORKSPACE SAVE` or selects `File > Save Workspace`, THE workbench
   SHALL write the current workspace state (roots, settings overrides, MRU list) to the
   current Workspace_File path; IF no Workspace_File path is set, THE workbench SHALL
   behave as `WORKSPACE SAVE AS`.

3. WHEN the user issues `WORKSPACE SAVE AS <path>` or selects `File > Save Workspace As...`,
   THE workbench SHALL write the workspace to the specified path and update the active
   Workspace_File path.

4. WHEN the user issues `WORKSPACE CLOSE` or selects `File > Close Workspace`, THE workbench
   SHALL unload the Active_Workspace: remove workspace roots from the catalog mount list,
   remove the Workspace_Settings layer from the configuration chain, and clear the
   workspace-scoped MRU_List.

5. WHEN a workspace is opened and another workspace is already active, THE workbench SHALL
   close the current workspace (per criterion 4) before loading the new one; IF the current
   workspace has unsaved changes, THE workbench SHALL prompt the user to save or discard.

6. AT MOST one workspace SHALL be active at any time.

---

### Requirement 3: Workspace Root Management

**User Story:** As a developer, I want to add and remove root directories from the active
workspace so that I can organise multi-project work under a single workspace context.

**Source:** GAP 2.1

#### Acceptance Criteria

1. WHEN the user issues `WORKSPACE ADD ROOT <path>`, THE workbench SHALL add the specified
   directory as a new Workspace_Root, register it as a catalog mount point in the Virtual
   Catalog Manager, and mark the workspace as having unsaved changes.

2. WHEN the user issues `WORKSPACE REMOVE ROOT <path>`, THE workbench SHALL remove the
   specified Workspace_Root from the workspace and unregister its catalog mount point;
   any open editor tabs referencing files under that root SHALL remain open but SHALL
   display a warning indicator that their root is no longer mounted.

3. THE workbench SHALL display all Workspace_Roots in the File Explorer sidebar as top-level
   nodes, visually grouped under the workspace name.

4. WHEN a workspace is loaded, THE workbench SHALL automatically register all `roots` entries
   as catalog mount points of type Native, using the root path as the catalog path.

5. IF a root path in the Workspace_File does not exist on disk at load time, THE workbench
   SHALL display a warning in the status bar identifying the missing root and SHALL continue
   loading the remaining roots.

---

### Requirement 4: Workspace-Scoped Settings

**User Story:** As a developer, I want workspace-specific configuration overrides so that
project-level settings (indent style, theme, line endings) apply automatically when I open
a workspace without affecting my global user settings.

**Source:** GAP 2.1, configuration-system Req 2.1

#### Acceptance Criteria

1. WHEN a workspace is active, THE configuration system SHALL apply the workspace's
   `[settings]` table as the Workspace layer -- the highest-priority layer in the
   configuration precedence chain (above Project, Profile, User, System, Defaults).

2. WHEN the user changes a setting in the Settings panel while a workspace is active and
   chooses to save at workspace scope, THE workbench SHALL write the override to the
   Workspace_File `[settings]` table rather than the user-layer config file.

3. WHEN a workspace is closed, THE workbench SHALL remove the Workspace layer from the
   configuration chain and recompute all effective values using the remaining layers,
   invoking hot-reload callbacks for any keys whose effective value changed.

4. THE workspace settings SHALL support all configuration keys that the user layer supports;
   no key SHALL be restricted from workspace-scope override.

---

### Requirement 5: Workspace Session Persistence

**User Story:** As a developer, I want the workbench to remember which workspace was open
when I last closed it and restore it automatically on next launch.

**Source:** startup-and-session Req 4

#### Acceptance Criteria

1. WHEN the workbench exits normally with an Active_Workspace, THE session manager SHALL
   persist the Workspace_File path in `session.toml` under `active_workspace_path`.

2. WHEN the workbench starts and `session.toml` contains a valid `active_workspace_path`,
   THE workbench SHALL automatically load that workspace as part of the startup sequence,
   before restoring open tabs.

3. IF the persisted `active_workspace_path` does not exist or cannot be read at startup,
   THE workbench SHALL start without an active workspace, display a warning in the status
   bar, and clear the stale path from `session.toml`.

4. WHEN no workspace was active at last exit, THE workbench SHALL start without an active
   workspace (existing behaviour unchanged).

---

### Requirement 6: Workspace-Scoped Recent Files

**User Story:** As a developer, I want a per-workspace recent-files list so that switching
workspaces restores the MRU list relevant to that project context.

**Source:** GAP 2.3

#### Acceptance Criteria

1. WHEN a workspace is active, THE workbench SHALL maintain a workspace-scoped MRU_List
   separate from the global recent-files list; files opened while the workspace is active
   SHALL be added to the workspace MRU_List.

2. THE workspace MRU_List SHALL be persisted in the Workspace_File `[[recent_files]]` array
   and restored when the workspace is loaded.

3. WHEN a workspace is closed, THE workbench SHALL revert to the global recent-files list.

4. THE workspace MRU_List SHALL have a configurable maximum depth under
   `workspace.recent_files_depth` (default: 50).
