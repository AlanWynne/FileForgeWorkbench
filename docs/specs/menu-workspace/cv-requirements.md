# Requirements Document -- POM Redesign (Phase CV)

## Introduction

Phase CV revises the Primary Option Menu (POM) option list to reflect all
implemented functionality and defines the default `menus/pom.toml` content
that will be used when the POM is migrated to the Menu Workspace pattern
(Phase CU-impl).

The current 9 options (0-8) do not include entries for three fully implemented
features: the JES job monitor, Global Search, and Batch Command Execution.
This phase adds those entries and defines the canonical TOML content for the
default POM menu file.

### Scope

This document extends `docs/specs/menu-workspace/requirements.md` with
Phase CV-specific criteria. It does NOT modify the existing hardcoded POM
(`TabKind::PrimaryOptionMenu`) -- that migration happens in Phase CU-impl.

### Source References

- **[CR-NR-048]** = Change log entry for POM Options Review and Logical Grouping
- **[CR-NR-045]** = Menu Workspace Pattern (Phase CU)
- **[ISPF-POM]** = IBM ISPF Primary Option Menu heritage

---

## Glossary

| Term | Definition |
|------|-----------|
| **Default_POM_Content** | The TOML string written to `menus/pom.toml` on first launch. |
| **POM_Option_Key** | The short key the user types to navigate to a POM option (e.g. `0`, `9`, `S`). |
| **Backward_Compatible** | Existing numeric keys 0-8 retain their current commands and descriptions. |

---

## Requirement 6: Revised POM Option List

**User Story:** As an operator, I want the POM to include entries for all
major implemented features -- including the JES job monitor, Global Search,
and Batch Execution -- so that I can navigate to any feature from the home
screen without memorising undocumented commands.

**Source:** [CR-NR-048], [ISPF-POM]

### Acceptance Criteria

1. THE revised POM option list SHALL contain the following entries in order,
   with the specified keys, labels, and descriptions:

   | Key | Label | Description |
   |-----|-------|-------------|
   | `0` | Settings | FFWB Settings and Client Parameters |
   | `1` | File Catalogs | Virtual File Catalogs -- Mainframe, POSIX, Native |
   | `2` | Files | File Explorer -- Browse catalogs and files in a tree view |
   | `3` | Utilities | Perform utility functions |
   | `4` | Compilers | Interactive language processing |
   | `5` | Lua Scripts | Run and manage Lua macros |
   | `6` | Terminals | Enter TSO or Workstation commands |
   | `7` | Databases | Database tool and query browser |
   | `8` | Plugins | Vendor added plugins |
   | `9` | Jobs | JES job monitor and spool viewer |
   | `S` | Search | Global search and replace across files |
   | `B` | Batch | Batch command execution (IKJEFT01 analogue) |

2. WHEN the user types `9` in any `Command ===>` field of a POM tab and
   presses Enter, THE shell SHALL navigate to the JES job monitor panel
   (equivalent to the existing `=JES` fastpath).

3. WHEN the user types `S` (case-insensitive) in any `Command ===>` field
   of a POM tab and presses Enter, THE shell SHALL open the Global Search
   panel (equivalent to `Ctrl+Shift+F` or the `SEARCH` command).

4. WHEN the user types `B` (case-insensitive) in any `Command ===>` field
   of a POM tab and presses Enter, THE shell SHALL display a status message
   `Batch execution is available via the --batch CLI flag or the BATCH command.`
   (Batch is a headless mode; there is no interactive panel to navigate to.)

5. THE existing numeric keys 0-8 SHALL retain their current commands and
   navigation behaviour unchanged -- this is a Backward_Compatible extension.

6. THE POM option list SHALL be divided into two visual groups in the TOML
   file using the `group` field:
   - Group `"Core"`: options 0-8 (existing options)
   - Group `"Extended"`: options 9, S, B (new options)

---

## Requirement 7: Default menus/pom.toml Content

**User Story:** As a first-time user, I want the workbench to create a
`menus/pom.toml` file on first launch that matches the revised option list,
so that when the POM is migrated to the Menu Workspace pattern it works
correctly out of the box.

**Source:** [CR-NR-048], [CR-NR-045] Req 4.1

### Acceptance Criteria

1. THE `DEFAULT_POM_TOML` constant in `menu_workspace/defaults.rs` SHALL
   contain valid TOML that, when parsed by `load_menu_file()`, produces a
   `MenuFile` with:
   - `title = "FileForge Workbench -- Primary Option Menu"`
   - 12 options matching the table in Requirement 6.1 exactly (keys, labels,
     descriptions, groups).

2. WHEN `ensure_default_menu_files()` is called and `menus/pom.toml` does
   not exist, THE function SHALL write `DEFAULT_POM_TOML` to that path.

3. WHEN `menus/pom.toml` already exists, `ensure_default_menu_files()` SHALL
   NOT overwrite it -- user customisations are preserved.

4. THE `DEFAULT_POM_TOML` string SHALL be valid TOML parseable by the
   `toml` crate with no errors.

5. THE `DEFAULT_POM_TOML` string SHALL use only plain ASCII characters
   (code points 0x00-0x7F) -- no curly quotes, em dashes, or Unicode symbols.

---

## Requirement 8: startup-and-session Req 14.3 Update

**User Story:** As a requirements reader, I want `startup-and-session`
Requirement 14.3 to reflect the revised 12-option list so that the spec
remains the source of truth for POM content.

**Source:** [CR-NR-048]

### Acceptance Criteria

1. `docs/specs/startup-and-session/requirements.md` Requirement 14.3 SHALL
   be updated to list all 12 options (0-8, 9, S, B) with their keys, labels,
   and descriptions as defined in Requirement 6.1 of this document.

2. THE updated Req 14.3 SHALL retain the existing forward-reference note
   pointing to `docs/specs/menu-workspace/requirements.md` and Phase CV.

3. THE updated Req 14.3 SHALL note that options 9, S, and B are new in
   Phase CV and that the existing options 0-8 are unchanged.
