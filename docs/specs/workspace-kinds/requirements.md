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

### Requirement 5: Per-Kind profile attributes applied on open (Slice B.3)

**User Story:** As a user, I want a Workspace Kind's profile defaults (its ISPF
edit profile, and the default line endings for a new buffer) applied when a
Workspace of that Kind is opened, so a Kind I configured for e.g. mainframe
editing starts with CAPS ON without my having to set it each time.

**Source:** [CR-NR-090] Slice B.3; delivers CR-NR-082's deferred per-workspace
Profile store.

#### Acceptance Criteria

1. WHEN an EDITOR Workspace (a file-editor or untitled buffer) is OPENED, THE
   workbench SHALL initialise that tab's Edit_Profile (CAPS/NULLS/STATS/LOCK/
   HILITE) from the active Kind's effective `Kind_Profile.edit_profile` (via the
   registry, keyed by the Kind's stable name). This applies ONCE at open, not on
   every activation, so a user's subsequent per-tab toggle (e.g. `CAPS OFF`) is
   NOT clobbered.

2. WHEN a NEW / UNTITLED buffer is created, THE workbench SHALL set its
   Line_End_Mode from the active Kind's effective `Kind_Profile.line_end_mode`
   (mapping the stored name: `"default"` / `"unicode"`). WHEN a file is LOADED
   from disk, its Line_End_Mode SHALL remain the mode DETECTED from the file
   content (the file's real encoding wins over the Kind default); the Kind's
   `line_end_mode` is a default for NEW buffers only.

3. THE `Kind_Profile.tab_size` field SHALL be carried and editable (B.4) and
   round-trip in the Kind file, but is NOT applied to a per-tab tab size in B.3
   (there is no per-tab tab-size field; `editor.tab_size` remains a global config
   key). This is a DOCUMENTED deferral -- applying a per-Kind tab size is picked
   up when a per-tab tab-size exists (or via the B.4 dialog writing config); no
   silent gap.

4. A user Kind's `Kind_Profile` SHALL follow the same "modelled on" rule: WHERE
   the user Kind does not set a profile value, the compiled default profile is
   used (the built-in defaults are the neutral `EditProfile::default()` / tab
   size / `"default"` line endings), so an unconfigured Kind opens exactly as
   today (behaviour-preserving).

5. THE profile application SHALL be behaviour-preserving for the built-in Kinds
   with default profiles: opening an editor Workspace of a built-in Kind SHALL
   produce the SAME initial Edit_Profile and (for new buffers) Line_End_Mode as
   before this slice.

### Requirement 6: Kind configuration Context (Slice B.4)

**User Story:** As a user, I want a dialog to configure a Workspace Kind's title,
menu bar, key list, and profile, and to create a NEW Kind modelled on an
existing one, so I can tailor Kinds without hand-editing TOML.

**Source:** [CR-NR-090] Slice B.4; makes B.1-B.3 user-facing. Modelled on the
Keys Workspace / Menus Editor Context pattern (function-keys Req 22, menu
-workspace Req 13): a pure render that stashes an action the shell applies.

#### Acceptance Criteria

1. THE workbench SHALL provide a Kinds Editor Context (a Workspace of a new
   `Kinds Editor` kind, title `[KINDS]`) that presents a SELECTOR of the
   configurable Workspace Kinds (built-in + user) and, for the selected Kind, an
   editable form for its `Kind_Config`: `title`, `menu_bar`, `key_list`,
   `modelled_on` (base), and `profile` (edit-profile toggles CAPS/NULLS/STATS/
   LOCK/HILITE, `tab_size`, `line_end_mode`).

2. THE Kinds Editor SHALL provide a "New Kind modelled on <base>" action that
   creates a NEW user Kind whose `modelled_on` is a selected BUILT-IN base
   (Requirement 1.3), with a user-entered unique `name`; the new Kind starts as a
   copy of the base's presentation/profile that the user can then edit.

3. WHEN the user Saves the selected Kind, THE workbench SHALL write its
   `Kind_Config` to `<User_Data_Dir>/workspace-kinds/<name>.toml` and reload the
   Kind_Registry so the change is live (tab titles / menu bar / key list / new
   opens reflect it) WITHOUT a restart. Saving a built-in Kind writes a user
   override file of the same name (Requirement 2.2).

4. THE Kinds Editor SHALL be reachable BOTH by a command (`KINDS`, command
   parity: the same code path whether typed or invoked from a menu) AND by a
   Settings menu entry; opening it on the Home Context transforms in place (END/
   RETURN returns to the POM), else opens/activates a dedicated tab, consistent
   with the other editor Contexts.

5. THE Kinds Editor render SHALL be a PURE function returning an editor Action;
   all file writes and registry reloads SHALL be applied by the shell command
   layer (mirroring the Keys / Menus / Theme editors), and the Context SHALL
   participate in the unified Tab-order model (report its `InteriorFocus` via the
   `WorkspaceContext` trait, CR-NR-078).

6. THE `Kinds Editor` kind SHALL itself be a built-in Kind in the registry (its
   own compiled default `Kind_Config`, title `[KINDS]`), so it is consistent with
   every other Context.

### Requirement 7: RESET BARE restores the compiled default Kinds (Slice B.4)

**User Story:** As a user, I want RESET BARE to drop my Kind customisations back
to the compiled defaults, consistent with how it resets menus/themes/config.

**Source:** [CR-NR-090] Slice B.4; configuration-system Req 19.

#### Acceptance Criteria

1. WHEN RESET BARE resets a profile's in-memory state to the compiled baselines
   (configuration-system Req 19.6), THE workbench SHALL also reset the
   Kind_Registry to the compiled built-in defaults (dropping any loaded user
   Kind overrides for the active profile) so Kinds return to their compiled
   configuration.

2. WHEN RESET BARE archives a profile's user data (configuration-system Req 19.5),
   THE profile's `workspace-kinds/` directory SHALL be included in the archived
   set (so user Kind files are moved to the timestamped archive, not deleted),
   consistent with the treatment of `menus/` / `themes/` / config.

3. THE compiled default Kind set SHALL always be present after RESET BARE (the
   registry's built-in defaults, Requirement 2.1), so every Workspace Kind
   remains resolvable.
