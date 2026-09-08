# Requirements Document -- Settings as a Menu Workspace (Phase CW)

## Introduction

Phase CW restructures the Settings Context as a Menu Workspace. Instead of
presenting a single flat list of all configuration keys, the Settings Context
becomes a menu whose options correspond to the major configuration namespaces.
Selecting an option opens a filtered view of that namespace's keys using the
existing flat-list Settings panel widget.

The default `menus/settings.toml` file is defined here. It is written on first
launch by `ensure_default_menu_files()` (Menu Workspace Req 4.2).

### Scope

This document extends `docs/specs/menu-workspace/requirements.md` and
`docs/specs/configuration-system/requirements.md` with Phase CW-specific
criteria. It does NOT modify the existing `SettingsPanel` implementation --
that migration happens in Phase CW-impl.

### Source References

- **[CR-NR-047]** = Change log entry for Settings Context as a Menu Workspace
- **[CR-NR-045]** = Menu Workspace Pattern (Phase CU)
- **[WB]** = Workbench Architecture Brief (Configuration as Data)

---

## Glossary

| Term | Definition |
|------|-----------|
| **Settings_Menu** | The Menu_Workspace backed by `menus/settings.toml`. |
| **Settings_Namespace_View** | The existing flat-list Settings panel filtered to a single namespace. |
| **Default_Settings_Content** | The TOML string written to `menus/settings.toml` on first launch. |
| **Namespace_Option** | One entry in the Settings_Menu corresponding to one configuration namespace. |

---

## Requirement 9: Settings Menu Option List

**User Story:** As a workbench user, I want the Settings Context to open as a
menu of configuration namespaces, so that I can navigate directly to the group
of settings I need without scrolling through an undifferentiated flat list.

**Source:** [CR-NR-047], [WB]

### Acceptance Criteria

1. THE Settings_Menu SHALL contain the following Namespace_Options in order,
   with the specified keys, labels, and descriptions:

   | Key | Label | Description |
   |-----|-------|-------------|
   | `E` | Editor | Text editing behaviour -- indentation, line endings, encoding |
   | `T` | Theme | Appearance -- active theme, font size, OS dark/light follow |
   | `C` | Catalogs | Default catalog roots for Mainframe and POSIX catalogs |
   | `V` | VFS | Virtual File System provider settings |
   | `L` | Logging | Log level, output directory, file rotation |
   | `K` | Key Maps | Function key bindings and per-context key maps |
   | `S` | Session | Session persistence, restore behaviour, recent files |
   | `P` | Plugins | Plugin-specific configuration namespaces |
   | `X` | Accessibility | Reduce motion, focus indicators, contrast settings |
   | `A` | All Settings | Browse all configuration keys (unfiltered flat list) |

2. WHEN the user types a Namespace_Option key (case-insensitive) in the
   `Command ===>` field of the Settings_Menu and presses Enter, THE shell
   SHALL open a Settings_Namespace_View filtered to that namespace.

3. WHEN the user clicks a Namespace_Option row in the Settings_Menu, THE
   shell SHALL open the corresponding Settings_Namespace_View, identical to
   typing the key and pressing Enter.

4. WHEN the user selects option `A` (All Settings), THE shell SHALL open the
   existing unfiltered flat-list Settings panel, identical to the current
   `SETTINGS` command behaviour.

5. THE Settings_Menu title SHALL be `FileForge Workbench -- Settings`.

6. THE Settings_Menu options SHALL be divided into two visual groups in the
   TOML file using the `group` field:
   - Group `"Namespaces"`: options E, T, C, V, L, K, S, P, X
   - Group `"All"`: option A

---

## Requirement 10: Settings Namespace View

**User Story:** As a workbench user, I want each namespace option to open a
filtered view showing only that namespace's keys, so that I can focus on the
settings relevant to my current task without distraction.

**Source:** [CR-NR-047], [WB]

### Acceptance Criteria

1. WHEN a Settings_Namespace_View is opened for namespace `N`, THE shell SHALL
   display the existing flat-list Settings panel with the filter field
   pre-populated with the namespace prefix (e.g., `editor.` for option `E`).

2. THE pre-populated filter SHALL be applied immediately on open -- only keys
   matching the namespace prefix SHALL be visible without the user needing to
   type anything.

3. THE user SHALL be able to clear or modify the filter field to broaden or
   narrow the visible keys within the Settings_Namespace_View.

4. WHEN the user presses `F3` or types `END` in the Settings_Namespace_View
   command field, THE shell SHALL return to the Settings_Menu (not to the POM).

5. THE Settings_Namespace_View tab title SHALL be `[SETTINGS:<namespace>]`
   (e.g., `[SETTINGS:editor]`) to distinguish it from the full Settings panel
   and from the Settings_Menu tab.

6. THE Settings_Namespace_View SHALL persist in the session and be restored on
   next launch with the same namespace filter applied.

---

## Requirement 11: Default menus/settings.toml Content

**User Story:** As a first-time user, I want the workbench to create a
`menus/settings.toml` file on first launch that matches the Settings_Menu
option list, so that when the Settings Context is migrated to the Menu
Workspace pattern it works correctly out of the box.

**Source:** [CR-NR-047], [CR-NR-045] Req 4.2

### Acceptance Criteria

1. THE `DEFAULT_SETTINGS_TOML` constant in `menu_workspace/defaults.rs` SHALL
   contain valid TOML that, when parsed by `load_menu_file()`, produces a
   `MenuFile` with:
   - `title = "FileForge Workbench -- Settings"`
   - 10 options matching the table in Requirement 9.1 exactly (keys, labels,
     descriptions, groups).

2. WHEN `ensure_default_menu_files()` is called and `menus/settings.toml` does
   not exist, THE function SHALL write `DEFAULT_SETTINGS_TOML` to that path.

3. WHEN `menus/settings.toml` already exists, `ensure_default_menu_files()`
   SHALL NOT overwrite it -- user customisations are preserved.

4. THE `DEFAULT_SETTINGS_TOML` string SHALL be valid TOML parseable by the
   `toml` crate with no errors.

5. THE `DEFAULT_SETTINGS_TOML` string SHALL use only plain ASCII characters
   (code points 0x00-0x7F) -- no curly quotes, em dashes, or Unicode symbols.

---

## Requirement 12: configuration-system Req 15 Update

**User Story:** As a requirements reader, I want `configuration-system`
Requirement 15 to reflect the Menu Workspace restructure so that the spec
remains the source of truth for Settings Context behaviour.

**Source:** [CR-NR-047]

### Acceptance Criteria

1. `docs/specs/configuration-system/requirements.md` Requirement 15 SHALL be
   updated to describe the two-level Settings navigation: the Settings_Menu
   (namespace selector) as the primary entry point, and the
   Settings_Namespace_View (filtered flat list) as the secondary view.

2. THE updated Req 15 SHALL retain all existing criteria 15.1-15.11 with
   adjustments to reflect that:
   - Criterion 15.1 (opening Settings) now opens the Settings_Menu, not the
     flat list directly.
   - Criterion 15.2 (grouped display) is now the Settings_Namespace_View
     behaviour, not the primary Settings entry point.
   - Criterion 15.10 (F3/END) now returns to the Settings_Menu when in a
     Settings_Namespace_View, and returns to the POM when in the Settings_Menu.

3. THE updated Req 15 SHALL add a note that the Settings_Menu is backed by
   `menus/settings.toml` and that the migration from the current flat-list
   implementation happens in Phase CW-impl.
