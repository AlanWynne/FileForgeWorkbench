# Requirements Document -- Configurable Workspace Kinds (CR-NR-090)

## Introduction

FileForgeWorkbench opens Workspaces; each Workspace is of a particular Workspace
Kind and is shown as a tab (dockable or detachable). Today a Kind is a compiled
`TabKind` with fixed presentation: fixed title label, a per-kind key map
(`keymaps/<kind>.toml`, function-keys Req 14.6 / CR-CH-027), and a single global
menu bar. This spec makes a Workspace Kind CONFIGURABLE -- each Kind carries its
own title, menu bar, key list, and profile attributes -- and lets a user CREATE a
new Kind "modelled on" a built-in one (a flat copy that shares the base's command
and behaviour but overrides presentation + profile).

There is deliberately NO separate "Workspace Definition" layer between Kind and
the live Workspace (owner decision): the flexibility of named presets is achieved
by letting a user create a new Kind modelled on an existing base, keeping the
runtime a single two-level model (Kind -> live Workspace tab).

This spec delivers the deferred later slices of CR-NR-082 (per-workspace-kind
menu-bar + keymap + Profile store) and CR-NR-080 Slice C (menu-bar per kind).

### Relationship to other specs

- **`workspace-model`** owns the PROJECT sense of "Workspace" (a `.ffwb-workspace`
  file of root directories + project settings). This spec is UNRELATED to that;
  it configures Workspace KINDS. The two never share a type or a file.
- **`function-keys-and-history`** Req 14.6 (`context_name_for_kind`, per-kind key
  maps) is the seam this spec generalises: a Kind name already selects a key map;
  a Kind now also selects a menu bar, a title, and a profile.
- **`menu-workspace`** Req 17 (Menu_Bar) Slice C (per-workspace-kind assignment)
  is delivered here; Req 18 (CR-NR-082) deferred the per-workspace menu-bar +
  keymap + Profile store to "later slices" -- this is those slices.
- **`configuration-system`** Req 19 (RESET BARE) gains a compiled default set of
  Kinds.
- **`workspace-framework`** (CR-NR-078 `WorkspaceContext` trait) is reused by the
  B.4 Kind-configuration editor Context; not otherwise coupled.

### Source references

- **[CR-NR-090]** = this change request (owner: configurable Kinds, "modelled on"
  instead of a Def layer, external-ready for future Lua/REXX Kinds).
- **[CR-NR-082]** = Unified Menu Workspace + deferred per-workspace slices.
- **[CR-NR-080]** = named menu bars (Slice C: per-kind assignment).

## Glossary

- **Workspace_Kind**: The type of a Workspace. Either BUILT-IN (compiled, maps to
  a core command) or USER (created by the user, modelled on a built-in base).
  Identified by a stable, unique `name`.
- **Base_Kind**: What a Kind is modelled on. `Builtin(<compiled kind>)` in v1;
  the persisted schema also admits `External(<name>)` for a future Lua/REXX
  -provided base (unresolved in v1). A BUILT-IN Kind's base is itself.
- **Kind_Config**: The configurable record for a Workspace_Kind:
  `{ name, modelled_on: Base_Kind, title, menu_bar, key_list, profile }`.
- **Kind_Profile**: The profile attributes carried by a Kind_Config: edit-profile
  defaults (CAPS/NULLS/STATS/LOCK/HILITE), tab size, line endings (extensible).
- **Kind_Registry**: The runtime set of all Workspace_Kinds (compiled built-ins
  plus user Kinds loaded from disk).
- **Built-in Kind**: A compiled Kind (Editor, Catalogs, Files, Config, Search,
  Plugins, Log, Macros, Menu, Commands, POM, and the editor Contexts). Its
  Kind_Config has compiled defaults and `modelled_on = Builtin(self)`.
- **User Kind**: A Kind created via the configuration dialog, persisted as a TOML
  under `<User_Data_Dir>/workspace-kinds/<name>.toml`, with `modelled_on` a
  built-in base.

---

## Requirements

<!-- ===================== SLICE B.1 ===================== -->

### Requirement 1: Kind configuration data model (Slice B.1)

**User Story:** As a workbench developer, I want a single data model that
describes every Workspace Kind's configuration -- built-in or user-created -- so
that a Kind's title, menu bar, key list, and profile are data, not hard-coded.

**Source:** [CR-NR-090] Slice B.1

#### Acceptance Criteria

1. THE workbench SHALL define a `Kind_Config` record with fields: `name` (stable
   unique identifier string), `modelled_on` (a `Base_Kind`), `title` (the tab /
   Title_Line label), `menu_bar` (a named menu, optional), `key_list` (a named
   key map, optional), and `profile` (a `Kind_Profile`). Absent `menu_bar` /
   `key_list` mean "use the base Kind's default".

2. THE `Base_Kind` type SHALL be an OPEN model with (at least) two forms:
   `Builtin(<compiled kind>)` and `External(<name>)`. In v1 only `Builtin` is
   resolvable; an `External` base SHALL be accepted by the persisted schema but,
   when it cannot be resolved, SHALL be reported with a clear message and SHALL
   NOT crash (the Kind falls back to a safe built-in base or is skipped with a
   notice). This keeps the model external-ready (future Lua/REXX Kinds) with no
   schema change.

3. A USER Kind's `modelled_on` SHALL be a BUILT-IN base in v1 (no user Kind may be
   modelled on another user Kind -- no inheritance chains, no cycles). THE model
   SHALL reject or ignore-with-message a user Kind whose base is another user
   Kind.

4. THE runtime SHALL resolve any Workspace_Kind to its base for COMMAND and
   BEHAVIOUR (a user Kind behaves as its `modelled_on` built-in for dispatch,
   central-panel rendering, navigation, and session reconstruction); ONLY the
   presentation (title, menu bar, key list) and profile are overridden by the
   Kind_Config. Resolution SHALL be a single hop (user Kind -> its built-in base).

5. A user Kind's `name` SHALL be its STABLE ID: it SHALL be the identifier used
   for the tab title source, the per-Kind key map file (`keymaps/<name>.toml`),
   the keymap context (`context_name_for_kind` equivalent), and the session
   descriptor. Built-in Kinds SHALL keep their existing stable names (`editor`,
   `files`, `config`, `search`, `plugins`, `log`, `macros`, `menu`, `commands`,
   `pom`, `theme`, `menus`, `keys`).

6. THE `Kind_Config` SHALL be serialisable to and from a TOML file under
   `<User_Data_Dir>/workspace-kinds/<name>.toml` (human-readable, editable),
   using a schema that round-trips `modelled_on` in both `Builtin` and `External`
   forms.

### Requirement 2: Kind registry with compiled built-in defaults (Slice B.1)

**User Story:** As a workbench developer, I want one registry that yields the
configuration for any Kind, so all presentation/profile lookups go through a
single source of truth with sane compiled defaults.

**Source:** [CR-NR-090] Slice B.1

#### Acceptance Criteria

1. THE workbench SHALL provide a `Kind_Registry` that, given a Kind name, returns
   its effective `Kind_Config`. THE registry SHALL contain a COMPILED default
   `Kind_Config` for every built-in Kind, so a Kind is always resolvable even
   with no user files present.

2. WHEN a user Kind file `<User_Data_Dir>/workspace-kinds/<name>.toml` exists and
   is valid, THE registry SHALL include that user Kind; a user file whose `name`
   equals a built-in Kind name SHALL OVERRIDE that built-in's config (so a user
   can reconfigure a built-in Kind), consistent with the user-override-of-compiled
   -default pattern used for menus and themes (CR-CH-021).

3. WHEN a user Kind file is absent or fails to parse, THE registry SHALL fall back
   to the compiled default for a built-in name, or skip an unparseable user Kind
   with a non-blocking notice (an absent file is silent), never crashing.

4. THE compiled default `Kind_Config` for each built-in Kind SHALL carry that
   Kind's CORRECT title label, resolving the current defect where distinct Kinds
   share a label: e.g. the Catalog Explorer Kind SHALL default to title
   `[CATALOGS]` and the File Explorer / navigator Kind to `[FILES]` (they SHALL
   NOT both be `[FILES]`).

### Requirement 3: Tab title and Title_Line derived from the Kind config (Slice B.1)

**User Story:** As a user, I want each Workspace's tab header and Title_Line to
show its Kind's configured title, so a reconfigured or user-created Kind shows the
right label everywhere.

**Source:** [CR-NR-090] Slice B.1

#### Acceptance Criteria

1. WHEN rendering a tab header or the Title_Line for a Workspace whose Kind is a
   system/panel or Menu Kind, THE label SHALL be derived from the active Kind's
   effective `Kind_Config.title` (via the registry), through the existing shared
   derivation (`title_line_text` / the tab-bar header helper, menu-and-statusbar
   Req 17.10) -- NOT a hard-coded per-`TabKind` string.

2. A user-assigned per-tab `workspace_name` (CX Requirement 1.4), when set, SHALL
   still take PRECEDENCE over the Kind's configured title (the NAME command
   overrides the Kind label for that one tab). The Home Context (POM) app-banner
   Title_Line and the file-editor path (path / `[Untitled]`, Requirement 17.4/
   17.5 of menu-and-statusbar) are UNCHANGED.

3. WHEN the effective Kind title changes (a user reconfigures a Kind or a user
   Kind is created with a distinct title), THE tab header and Title_Line of open
   Workspaces of that Kind SHALL reflect the new title without a restart
   (recomputed from the registry on render), consistent with the live-derivation
   rule of menu-and-statusbar Req 17.10.

<!-- ===================== SLICES B.2 - B.4 (to be gated when reached) ===================== -->

### Requirement 4: Per-Kind menu bar and key list (Slice B.2)

**User Story:** As a user, I want each Workspace Kind to use its configured menu
bar and key list, so a Kind (built-in or user-created) presents the menu and
key bindings I assigned to it.

**Source:** [CR-NR-090] Slice B.2; delivers CR-NR-082 deferred per-workspace-kind
menu-bar + keymap and CR-NR-080 Slice C (menu-bar per kind).

#### Acceptance Criteria

1. WHEN the Menu_Bar is rendered for the active Workspace, THE bar SHALL be the
   named menu given by the active Kind's effective `Kind_Config.menu_bar`
   (resolved via the registry, keyed by the Kind's stable name); WHERE the Kind's
   `menu_bar` is `None`, THE bar SHALL fall back to the compiled Default_Menu_Bar
   name (`DEFAULT_MENU_BAR_NAME`), preserving current behaviour. Resolution reuses
   the existing named-menu resolver (`resolve_menu_bar_menu` -> `menus/<slug>.toml`
   with the compiled fallback, menu-workspace Req 17.8).

2. WHEN the active Workspace changes (tab switch, navigate-in-place, open), THE
   rendered Menu_Bar SHALL update to the newly-active Kind's configured bar. A
   Detached_Workspace's own Menu_Bar (menu-and-statusbar Req 18.12) SHALL likewise
   use ITS Kind's configured bar.

3. WHEN the active Workspace's key map context is selected, THE context name SHALL
   be the active Kind's effective `Kind_Config.key_list` WHERE set; WHERE
   `key_list` is `None`, THE context SHALL be the Kind's base context name
   (`context_name_for_kind` / `pom` for Home), preserving current per-kind keymap
   behaviour (function-keys Req 14.6, CR-CH-027). The selected context resolves a
   loaded `keymaps/<context>.toml` map exactly as today; a `key_list` naming a
   context with no loaded map falls back to the global key map (existing
   full-replacement precedence, function-keys Req 14.3/14.5).

4. A user Kind modelled on a built-in base SHALL, by default (no `menu_bar` /
   `key_list` override), present the SAME menu bar and key list as its base Kind
   (the base's defaults), and SHALL present its OWN configured bar / key list
   where the override is set -- so "modelled on" means "inherits the base's
   presentation unless overridden".

5. THE menu-bar and key-list resolution SHALL be behaviour-preserving for the
   built-in Kinds with default (unset) `menu_bar` / `key_list`: no existing
   menu-bar rendering or key binding SHALL change for a Kind that has not been
   reconfigured.

### Requirement 5: Per-Kind profile attributes (Slice B.3 -- PENDING GATE)

*(Placeholder -- criteria to be finalised at the B.3 gate.)* A Workspace Kind's
`Kind_Profile` (edit-profile defaults CAPS/NULLS/STATS/LOCK/HILITE, tab size,
line endings) SHALL be applied when a Workspace of that Kind is opened. Delivers
CR-NR-082's deferred per-workspace Profile store.

### Requirement 6: Kind configuration dialog + command + RESET BARE defaults (Slice B.4 -- PENDING GATE)

*(Placeholder -- criteria to be finalised at the B.4 gate.)* THE workbench SHALL
provide a Kind configuration Context (dialog) to edit a built-in Kind's config
and to create a NEW user Kind "modelled on" a selected built-in base, reachable
via a Settings menu entry and a command (command parity). RESET BARE SHALL
restore the compiled default Kinds and drop user Kinds.
