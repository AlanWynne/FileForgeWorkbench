# Portable Installation -- Requirements

## Summary

FileForgeWorkbench today resolves every on-disk location independently through
scattered `dirs::` calls in many crates, using three divergent application
names (`FFWorkbench` for config, `FileForgeWorkbench` for data/logs,
`.fileforgewb` for recovery) and no single notion of "where the application
lives". This makes the installation hard to find, hard to move, impossible to
run from a USB stick, and inconsistent to reason about on upgrade.

This specification introduces ONE path-resolution authority (a new `ff-paths`
crate) that owns an "application root" concept and resolves every subdirectory
against it. It defines TWO run modes -- PORTABLE (the default, driven by a
`portable.ffw` marker file / `FFW_HOME` env var / CLI flag) where everything
lives under the install directory, and MANAGED (the fallback) where state lives
under unified per-user `dirs::` locations. It defines a single visible install
layout that separates SHIPPED defaults (overwrite on upgrade) from USER state
(preserve on upgrade) from DISPOSABLE data (logs/cache/tmp, redirectable and
age-cleaned). The existing layered config model (System/User/Profile/Project/
Workspace) is preserved unchanged; portability applies only to the user/install
roots, never to project config.

This is a deliberate, owner-approved FRAMEWORK CHANGE: it alters how the
session/config persistence ROOTS are resolved, which the
`framework-conformance` steering rule treats as a load-bearing contract. The
change is additive to the layering model and dispatch; it reshapes only root
resolution. See `design.md` for the framework-change flag and the full path
inventory.

## Terminology

- **Application root** -- the single base directory against which every
  subdirectory is resolved. In PORTABLE mode it is the install directory
  (the directory containing the executable, or a forced override); in MANAGED
  mode the shipped-resource root is the install directory and the writable
  roots are the unified per-user `dirs::` locations.
- **PORTABLE mode** -- active when a `portable.ffw` marker file is present next
  to the executable, or forced by `FFW_HOME` / a CLI flag. Config, data,
  extensions, logs, cache, and tmp all resolve under the application root.
- **MANAGED mode** -- the fallback when no marker is present and no override
  forces a root. Shipped resources resolve next to the executable; writable
  state resolves under unified per-user `dirs::` locations.
- **SHIPPED-DEFAULT** -- content installed with the product, safe to overwrite
  on upgrade (e.g. `resources/` -- help, i18n, languages, default themes).
- **USER-OVERRIDE / STATE** -- content the user creates or customises, which
  MUST be preserved across upgrade (e.g. `config/`, `data/`, `extensions/`).
- **DISPOSABLE** -- regenerable content safe to age-clean (`logs/`, `cache/`,
  `tmp/`).

## Functional Requirements

### 1. Single path-resolution authority

- 1.1 WHEN any crate needs a user-facing directory (config, data subdirs, logs,
  cache, tmp, extensions, resources), THE application SHALL resolve it through
  the single `ff-paths` authority rather than calling `dirs::` directly.
- 1.2 WHEN `ff-paths` resolves a subdirectory, THE authority SHALL compute it as
  a child of the resolved application root (or the appropriate role root:
  shipped-resource root, config root, data root, disposable root).
- 1.3 WHEN the application starts, THE path authority SHALL be initialised ONCE,
  before any subsystem resolves a path, and SHALL expose the resolved roots and
  the active mode for inspection.
- 1.4 THE path authority SHALL expose a named accessor for every current
  location enumerated in the `design.md` path inventory (config file, profiles,
  languages, menus, macros, keymaps, themes, help, i18n, aliases,
  workspace-kinds, screen-collections, command config, logs, recovery, sessions,
  extensions) so no caller reconstructs a path by string-joining `dirs::`.

### 2. Two modes, PORTABLE default

- 2.1 WHEN a `portable.ffw` marker file exists in the executable's directory,
  THE application SHALL run in PORTABLE mode with the config/data/disposable
  roots all under the application root.
- 2.2 WHEN no marker is present and no override is set, THE application SHALL run
  in MANAGED mode and resolve writable state under the unified per-user
  `dirs::` locations.
- 2.3 WHEN the `FFW_HOME` environment variable is set to a directory path, THE
  application SHALL force PORTABLE mode with the application root set to that
  path, overriding marker detection.
- 2.4 WHEN a `--ffw-home <path>` (or equivalent) command-line flag is supplied,
  THE application SHALL force PORTABLE mode with the application root set to that
  path, and THE CLI flag SHALL take precedence over `FFW_HOME`.
- 2.5 WHEN both marker detection and an explicit override are considered, THE
  precedence SHALL be: CLI flag, then `FFW_HOME`, then `portable.ffw` marker,
  then MANAGED fallback.
- 2.6 WHEN MANAGED mode resolves per-user locations, THE application SHALL use
  ONE unified application name for all of them, resolving the current
  `FFWorkbench` / `FileForgeWorkbench` / `.fileforgewb` divergence to a single
  documented name (see `design.md`).

### 3. Portable install layout

- 3.1 WHEN the product is installed portably, THE install directory SHALL
  contain, under one visible root: the executable, `resources/`, `config/`,
  `data/`, `extensions/`, `logs/`, `cache/`, `tmp/`, the `portable.ffw` marker,
  and the README/LICENSE files.
- 3.2 THE `resources/` directory SHALL hold SHIPPED defaults (help, i18n,
  languages, default themes) and SHALL be classified as overwrite-on-upgrade.
- 3.3 THE `config/` directory SHALL hold user configuration and SHALL be
  classified as preserve-on-upgrade.
- 3.4 THE `data/` directory SHALL hold user state (menus, macros, keymaps,
  aliases, workspace-kinds, screen-collections, command config, sessions,
  recovery) and SHALL be classified as preserve-on-upgrade. NOTE: the SCRM
  capture / Save-As default directory (screen-snapshot-scrm Req 24, the
  discoverable captures folder the capture-STOP Save-As dialog defaults to) IS
  the `screen-collections` location named here. When this portability reorg
  lands, that default SHALL resolve through `ff-paths` to the
  `data/screen-collections/` root (preserve-on-upgrade), NOT a bespoke path.
  Until then, SCRM resolves it against the current ffwb domain convention; the
  reorg migrates it to this root.
- 3.5 THE `extensions/` directory SHALL hold installed extensions and SHALL be
  classified as preserve-on-upgrade.
- 3.6 THE `logs/`, `cache/`, and `tmp/` directories SHALL be classified as
  disposable.
- 3.7 WHERE content today mixes shipped defaults and user overrides in one
  location (menus, themes, help, i18n), THE design SHALL split it by lifecycle
  so shipped defaults live under `resources/` and user overrides live under
  `config/` or `data/`.
- 3.8 WHEN a user menu/theme/keymap file is absent, THE application SHALL fall
  back to the compiled built-in default (no built-in file is ever written to
  `resources/` or `data/` on first launch), consistent with the code-only
  built-in-menu framework rule.

### 4. Preserve the layered config model

- 4.1 THE application SHALL preserve the existing config layering
  (Defaults -> System -> User -> Profile -> Project -> Workspace) unchanged.
- 4.2 WHEN resolving PROJECT config, THE application SHALL continue to read it
  from `<project>/.ffworkbench/config.toml` relative to the project root,
  independent of the application root, so project config stays with the project
  and in version control.
- 4.3 WHEN portability changes a root, THE change SHALL affect only the
  user/install config and data roots, and SHALL NOT collapse, reorder, or remove
  any config layer.

### 5. Extensions directory naming

- 5.1 THE directory for installed extensions SHALL be named `extensions/`
  (NOT `plugins/`), consistent with the standing extension-terminology steering
  rule that renames "plugin" to "extension".

### 6. Relative vs absolute paths and relocatability

- 6.1 WHEN a path value in config is RELATIVE, THE application SHALL resolve it
  against the application root.
- 6.2 WHEN a path value in config is ABSOLUTE, THE application SHALL honour it
  verbatim.
- 6.3 THE application SHALL prefer root-relative path forms (e.g.
  `./resources/help`) over machine-specific absolute paths in shipped and
  default config.
- 6.4 WHEN a PORTABLE installation directory is moved or copied to a new
  location, THE application SHALL continue to run correctly from the new
  location without edits to absolute paths.
- 6.5 WHEN the product is distributed as a ZIP, extracting the ZIP SHALL be a
  valid install method, and deleting the extracted directory SHALL fully
  uninstall a PORTABLE installation (no registry keys, no files left outside
  the directory for the portable case).

### 7. Redirectable disposable directories

- 7.1 WHEN `config/fileforge.toml` contains a `[paths]` section with entries for
  `logs`, `cache`, or `tmp`, THE application SHALL resolve each disposable
  directory to the configured value.
- 7.2 WHEN a `[paths]` disposable value is RELATIVE, THE application SHALL
  resolve it under the application root; WHEN it is ABSOLUTE, THE application
  SHALL honour it verbatim.
- 7.3 THE application SHALL NOT allow the `config/` and `data/` roots to be
  redirected through the `[paths]` section (they define the installation).
- 7.4 WHEN no `[paths]` entry is present for a disposable directory, THE
  application SHALL use the default location under the application root.

### 8. Retention / cleanup

- 8.1 THE application SHALL read a `[maintenance]` config section with the
  defaults `cleanup_on_startup = true`, `logs_retention_days = 10`,
  `cache_retention_days = 10`, and `tmp_retention_days = 10`.
- 8.2 WHEN `cleanup_on_startup` is true, THE application SHALL run a best-effort
  maintenance pass at startup that deletes aged files from the disposable
  directories, and THE pass SHALL NOT be a hard gate on startup (a failure or
  partial pass SHALL NOT prevent the application from starting).
- 8.3 WHEN the maintenance pass runs, THE application SHALL delete ONLY files
  located inside the RESOLVED `logs/`, `cache/`, and `tmp/` directories.
- 8.4 WHEN deciding whether to delete a file, THE application SHALL delete only
  files older than the configured retention age for that directory, and SHALL
  prefer to delete only files the application created or owns.
- 8.5 WHEN a retention value is `0`, THE application SHALL NEVER clean that
  directory.
- 8.6 THE application SHALL NEVER auto-clean the `data/` or `config/`
  directories under any maintenance setting.
- 8.7 WHEN a disposable directory has been redirected (via `[paths]`) to a
  shared or non-default location, THE maintenance pass SHALL NOT blind-sweep it:
  it SHALL apply the age filter AND the ownership filter only.
- 8.8 WHERE `tmp/` is concerned, THE application MAY additionally clear `tmp/` on
  a clean shutdown.

### 9. Writability fallback

- 9.1 WHEN PORTABLE mode is selected but the resolved application root is NOT
  writable, THE application SHALL detect the condition and fall back to per-user
  `dirs::` locations for writable state (config, data, logs, cache, tmp),
  while still reading shipped resources from the install directory.
- 9.2 WHEN the writability fallback is engaged, THE application SHALL record a
  warning identifying the non-writable root and the fallback location used.
- 9.3 THE default installer destination SHALL be `%USERPROFILE%\Apps\
  FileForgeWorkbench` (a writable, no-admin, no-registry location) so the common
  case never triggers the writability fallback.

### 10. Optional Windows integration

- 10.1 THE product SHALL be fully usable with NO installer by extracting the
  distribution ZIP (the portable baseline).
- 10.2 WHERE an optional lightweight `ffw-setup.exe` is provided, it SHALL offer
  Start Menu / desktop shortcuts and file associations as OPTIONAL conveniences.
- 10.3 WHEN the optional setup runs, it SHALL default its install destination to
  `%USERPROFILE%\Apps\FileForgeWorkbench`, require no administrator rights, and
  write no registry keys that affect portability.
- 10.4 THE presence or absence of shortcuts/associations created by the optional
  setup SHALL NOT affect whether the installation runs portably.

### 11. Packaging

- 11.1 WHEN the product is built for distribution, THE build SHALL produce a
  `dist/FileForgeWorkbench-<version>-windows-x64/` directory containing the full
  install tree (executable, `resources/`, empty/initial `config/`, `data/`,
  `extensions/`, `logs/`, `cache/`, `tmp/`, `portable.ffw`, README/LICENSE).
- 11.2 THE distribution artifact SHALL be the ZIP of that `dist/...` tree.

## Non-Functional Requirements

- NF.1 All new and changed documentation and config keys SHALL be ASCII-only
  (documentation-charset steering).
- NF.2 The change SHALL be additive to the config layering and command-dispatch
  frameworks; it SHALL alter ONLY root/path resolution, and SHALL be recorded as
  an owner-approved framework change per `framework-conformance`.
- NF.3 Path resolution SHALL be deterministic and testable: given a fixed set of
  marker/env/flag inputs and a root, `ff-paths` SHALL resolve the same paths on
  every call (no reliance on ambient state beyond the documented inputs).
- NF.4 The maintenance pass SHALL be bounded and best-effort: it SHALL not block
  startup and SHALL tolerate partial failure (unreadable file, permission
  error) by skipping and continuing.

## Acceptance Criteria

1. A new `ff-paths` crate/module exists and exposes a named accessor for every
   location in the path inventory; a repository search finds no direct `dirs::`
   path-join for a user-facing directory outside `ff-paths` (Req 1.1, 1.4).
2. With a `portable.ffw` file beside the executable, all of config/data/logs/
   cache/tmp/extensions resolve under the executable's directory (Req 2.1, 3.1).
3. With no marker and no override, all writable state resolves under ONE unified
   per-user name (not a mix of `FFWorkbench` / `FileForgeWorkbench` /
   `.fileforgewb`) (Req 2.2, 2.6).
4. Setting `FFW_HOME` forces the root to that path; passing `--ffw-home` forces
   the root and overrides `FFW_HOME`; precedence is flag > env > marker >
   managed (Req 2.3, 2.4, 2.5).
5. Project config is still read from `<project>/.ffworkbench/config.toml`
   regardless of application root, and the six-layer order is unchanged
   (Req 4.1, 4.2, 4.3).
6. The installed extensions directory is named `extensions/`, never `plugins/`
   (Req 5.1).
7. A relative config path resolves under the root; an absolute one is honoured
   verbatim; moving the whole portable directory and launching still works
   (Req 6.1, 6.2, 6.4).
8. Extracting the ZIP runs the product; deleting the directory leaves nothing
   behind for the portable case (Req 6.5, 11.2).
9. A `[paths]` entry redirects logs/cache/tmp (relative under root, absolute
   verbatim); config/data cannot be redirected via `[paths]` (Req 7.1-7.4).
10. The maintenance pass with defaults deletes only aged, app-owned files inside
    the resolved logs/cache/tmp; a retention value of 0 disables cleaning for
    that directory; data/ and config/ are never touched; a redirected disposable
    dir is age+ownership filtered, never blind-swept; startup still succeeds if
    the pass fails (Req 8.1-8.8).
11. With a non-writable portable root, the app falls back to per-user writable
    state, logs a warning, and still reads shipped resources from the install
    dir (Req 9.1, 9.2).
12. The build emits `dist/FileForgeWorkbench-<version>-windows-x64/` with the
    full tree and a ZIP of it (Req 11.1, 11.2).
13. The path inventory in `design.md` lists every current location with its
    resolving crate/file/function and a SHIPPED-DEFAULT / USER-OVERRIDE /
    DISPOSABLE classification, and documents the naming divergence (Req 1.4,
    2.6, 3.7).

## Out of Scope

- Any source-code implementation. This spec is the requirements gate only;
  no `.rs` file is changed here.
- Non-Windows installers. The layout is cross-platform by construction (the
  MANAGED fallback uses platform `dirs::`), but the packaging/installer
  requirements target Windows x64; Linux/macOS packaging is a future increment.
- Auto-update / in-place upgrade mechanics beyond classifying each directory as
  overwrite vs preserve. The actual upgrade tool is out of scope.
- Signing/notarisation and distribution-channel logistics.
- Migrating existing users' data from the old divergent locations to the unified
  layout (a one-time migration helper may be specified as a follow-up).
