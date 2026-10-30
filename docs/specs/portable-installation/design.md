# Portable Installation -- Design

## Overview

FileForgeWorkbench currently resolves on-disk locations independently in roughly
a dozen call sites, each calling the `dirs` crate directly and string-joining an
application name that is NOT consistent: config uses `FFWorkbench`
(and the session User Data Dir uses the lowercase `ffworkbench`), data and logs
use `FileForgeWorkbench`, and crash recovery uses `.fileforgewb` under the home
directory. There is no single notion of "where the application lives", so the
install is hard to find, cannot be relocated, and cannot run from removable
media.

This design introduces ONE path-resolution authority -- a new `ff-paths` crate
-- that owns an "application root" concept and resolves every subdirectory
against it by ROLE (shipped-resource root, config root, data root, disposable
root, extensions root). Every other crate calls `ff-paths` instead of `dirs::`.
On top of that authority sit two run modes: PORTABLE (the default, driven by a
`portable.ffw` marker / `FFW_HOME` / CLI flag) where everything lives under the
install directory, and MANAGED (the fallback) where writable state lives under
unified per-user `dirs::` locations. The existing six-layer config model is
preserved unchanged; project config stays with the project.

This is a deliberate FRAMEWORK CHANGE and is flagged as such below.

## Framework-change flag (framework-conformance)

Per the `framework-conformance` steering rule, session/config persistence root
resolution is a load-bearing framework contract (mechanism 6, descriptor-based
session persistence, and the config-layering model). This spec CHANGES how the
persistence/config ROOTS are resolved: instead of each crate calling `dirs::`,
all root resolution funnels through `ff-paths`, and the active mode
(PORTABLE/MANAGED) and overrides (`FFW_HOME`, `--ffw-home`) can move those roots.

The owner has approved this direction in conversation (see the original user
request that initiated this workflow). The change is ADDITIVE to everything
else: the config layering order, the `WorkspaceDescriptor` session-persistence
format, and the single command-dispatch path are all unchanged -- only the base
directory against which they resolve is now centralised and mode-aware. No
public framework type (`CommandTarget`, `WorkspaceContext`, `InteriorFocus`,
`WorkspaceDescriptor`, `KindConfig`, `BackendEnvironment`) is reshaped. This
section is the explicit owner-confirmed framework-change record the rule
requires.

## Path inventory (current state)

Every current user-facing location, the crate/file/function that resolves it,
the application name it uses today, and its lifecycle classification. "UDD" means
the location is resolved via `ff_session::UserDataDir::resolve(None)` (which
itself resolves `dirs::config_dir()/ffworkbench`, profile-aware) with a
`dirs::data_dir()/FileForgeWorkbench/...` fallback when the session layer is
unavailable -- so these locations have a SPLIT identity today.

| # | Location (today) | Crate / file / function | Name used | Classification |
|---|------------------|-------------------------|-----------|----------------|
| 1 | System config `config.toml` | `ff-config/src/paths.rs::system_config_path` (Win `%PROGRAMDATA%\FFWorkbench`, Linux `/etc/ffworkbench`, mac `/Library/Application Support/FFWorkbench`) | FFWorkbench / ffworkbench | USER-OVERRIDE/STATE (system-managed) |
| 2 | User config `config.toml` | `ff-config/src/paths.rs::user_config_path` (`dirs::config_dir()/ffworkbench/config.toml`, profile-aware; env seam `FFWB_USER_CONFIG_PATH`) | ffworkbench | USER-OVERRIDE/STATE |
| 3 | User config dir | `ff-config/src/paths.rs::user_config_dir` (`dirs::config_dir()/ffworkbench`) | ffworkbench | USER-OVERRIDE/STATE |
| 4 | Profiles dir | `ff-config/src/paths.rs::user_profiles_dir` (`dirs::config_dir()/ffworkbench/profiles`) | ffworkbench | USER-OVERRIDE/STATE |
| 5 | Languages dir | `ff-config/src/paths.rs::languages_dir` (`dirs::config_dir()/ffworkbench/languages`) | ffworkbench | SHIPPED-DEFAULT + USER-OVERRIDE (mixed today) |
| 6 | Project config | `ff-config/src/paths.rs::project_config_path` (`<project>/.ffworkbench/config.toml`) | .ffworkbench (per-project) | USER-OVERRIDE/STATE (stays with project, NOT in app root) |
| 7 | User Data Dir base | `ff-session/src/user_data_dir.rs::platform_default_path` / `default_base` (`dirs::config_dir()/ffworkbench`, profile-aware; const `APP_DIR_NAME = "ffworkbench"`) | ffworkbench | USER-OVERRIDE/STATE |
| 8 | UDD `sessions/` | `ff-session/src/user_data_dir.rs::sessions_dir` | ffworkbench | USER-OVERRIDE/STATE |
| 9 | UDD `recovery/` | `ff-session/src/user_data_dir.rs::recovery_dir` | ffworkbench | USER-OVERRIDE/STATE |
| 10 | UDD `profiles/` | `ff-session/src/user_data_dir.rs::profiles_dir` | ffworkbench | USER-OVERRIDE/STATE |
| 11 | UDD `plugins/` (REQUIRED_SUBDIRS) | `ff-session/src/user_data_dir.rs::plugins_dir` / `REQUIRED_SUBDIRS` | ffworkbench | USER-OVERRIDE/STATE -- RENAME to `extensions/` (Req 5.1) |
| 12 | Session user-data-dir default | `ff-session/src/user_data_dir.rs` (platform default = `dirs::config_dir()`) | ffworkbench | USER-OVERRIDE/STATE |
| 13 | Logs dir | `ff-logging/src/config.rs::default_log_directory` (Win `%LOCALAPPDATA%/FileForgeWorkbench/logs`, Unix `$XDG_DATA_HOME/file-forge-workbench/logs`) | FileForgeWorkbench / file-forge-workbench | DISPOSABLE |
| 14 | Crash recovery (unsaved) | `ff-undo-redo/src/recovery.rs::unsaved_recovery_path` (`~/.fileforgewb/recovery/<id>.recovery`) | .fileforgewb | USER-OVERRIDE/STATE (recoverable work) |
| 15 | Per-file recovery | `ff-undo-redo/src/recovery.rs::recovery_path_for` (`.<stem>.recovery` next to source) | (per-file) | USER-OVERRIDE/STATE (stays next to source, NOT in app root) |
| 16 | Help content | `ff-help/src/content_loader.rs::from_config` + `ff-desktop/src/shell/help.rs` (exe-dir `/help`, then `dirs::data_dir()/FileForgeWorkbench/help`) | FileForgeWorkbench | SHIPPED-DEFAULT + USER-OVERRIDE (mixed today) |
| 17 | i18n tree | `ff-desktop/src/shell/i18n_bootstrap.rs::resolve_i18n_root_from` (exe-dir `/i18n`, dev-tree marker, then `dirs::data_dir()/FileForgeWorkbench/i18n`) | FileForgeWorkbench | SHIPPED-DEFAULT (+ user overlay) |
| 18 | Themes dir | `ff-desktop/src/theme_defaults.rs::themes_dir` (UDD `/themes`, fallback `dirs::data_dir()/FileForgeWorkbench/themes`) | FileForgeWorkbench | SHIPPED-DEFAULT + USER-OVERRIDE (mixed today) |
| 19 | Menus dir | `ff-desktop/src/shell/commands_menu.rs::menus_dir` (UDD `/menus`, fallback `dirs::data_dir()/FileForgeWorkbench/menus`) | FileForgeWorkbench | USER-OVERRIDE/STATE (built-ins are code-only) |
| 20 | Macros dir | `ff-desktop/src/shell/actions.rs::macro_dirs` (`dirs::data_dir()/FileForgeWorkbench/macros`) | FileForgeWorkbench | USER-OVERRIDE/STATE |
| 21 | Keymaps dir | `ff-desktop/src/shell/keys_editor.rs::keymaps_dir` + `construct.rs` (UDD `/keymaps`, fallback `dirs::data_dir()/FileForgeWorkbench/keymaps`) | FileForgeWorkbench | USER-OVERRIDE/STATE |
| 22 | Aliases dir | `ff-desktop/src/shell/user_aliases.rs::user_alias_override_base` (`dirs::data_dir()/FileForgeWorkbench/aliases`; env seam `FFWB_USER_ALIASES_DIR`) | FileForgeWorkbench | USER-OVERRIDE/STATE |
| 23 | Workspace-kinds dir | `ff-desktop/src/shell/kinds_editor.rs::kinds_dir` + `construct.rs` (UDD `/workspace-kinds`, fallback `dirs::data_dir()/FileForgeWorkbench/workspace-kinds`) | FileForgeWorkbench | USER-OVERRIDE/STATE |
| 24 | Screen-collections dir | `ff-desktop/src/shell/commands_session.rs` (UDD `/screen-collections`, fallback `dirs::data_dir()/FileForgeWorkbench/screen-collections`) | FileForgeWorkbench | USER-OVERRIDE/STATE |
| 25 | Command config store | `ff-desktop/src/shell/construct.rs` (`dirs::data_dir()/FileForgeWorkbench/commands/commands.toml`) | FileForgeWorkbench | USER-OVERRIDE/STATE |
| 26 | i18n/help locale overlays | `construct.rs` + `alias_reader` (per-locale subdirs of the i18n/help trees) | FileForgeWorkbench | SHIPPED-DEFAULT (+ user overlay) |

Naming divergence summary: config/session use `ffworkbench` (lowercase) and the
system path uses `FFWorkbench`; data/logs use `FileForgeWorkbench`
(and `file-forge-workbench` on Unix); recovery uses `.fileforgewb`. This spec
resolves these to ONE name (see "Unified name" below). Locations 6 and 15
(project config, per-file recovery) deliberately stay OUTSIDE the application
root because they belong to the project/document, not the installation.

## Resolved design decisions

### The `ff-paths` authority

A new crate `ff-paths` (depending only on `std` + `dirs`) owns:

- `AppRoot` -- the resolved application root plus the active `Mode`
  (`Portable` or `Managed`).
- `resolve() -> AppRoot` -- run ONCE at startup. It reads, in precedence order:
  1. the `--ffw-home <path>` CLI flag (if the host passes it in),
  2. the `FFW_HOME` environment variable,
  3. a `portable.ffw` marker file in the executable's directory
     (`std::env::current_exe().parent()`),
  4. otherwise MANAGED fallback.
- Role roots, each a method on `AppRoot`:
  - `resources_root()` -- shipped defaults. PORTABLE: `<root>/resources`.
    MANAGED: the executable directory's `resources/` (shipped alongside the
    binary), with the dev-tree marker walk retained for `cargo run`.
  - `config_root()` -- PORTABLE `<root>/config`; MANAGED
    `dirs::config_dir()/<unified>`.
  - `data_root()` -- PORTABLE `<root>/data`; MANAGED
    `dirs::data_dir()/<unified>`.
  - `extensions_root()` -- PORTABLE `<root>/extensions`; MANAGED
    `data_root()/extensions`.
  - disposable roots `logs_dir()`, `cache_dir()`, `tmp_dir()` -- defaults under
    `<root>/logs|cache|tmp` (PORTABLE) or `dirs::data_local_dir()/<unified>/...`
    (MANAGED), each overridable by the `[paths]` config section (see below).
- Named subdir accessors mirroring the inventory, e.g. `menus_dir()`,
  `macros_dir()`, `keymaps_dir()`, `themes_dir()`, `help_dir()`, `i18n_dir()`,
  `aliases_dir()`, `workspace_kinds_dir()`, `screen_collections_dir()`,
  `command_store_path()`, `sessions_dir()`, `recovery_dir()`, `profiles_dir()`,
  `languages_dir()`, `user_config_path()`. Each is a pure join on the correct
  role root, so no caller reconstructs a path from `dirs::` (Req 1.1, 1.4).

The existing test/host override env seams (`FFWB_USER_CONFIG_PATH`,
`FFWB_USER_ALIASES_DIR`, `FFWB_HISTORY_PATH`) are RETAINED and continue to win
verbatim where they apply -- `ff-paths` consults them in the matching accessor
so test isolation (B048) is preserved.

Other crates (`ff-config`, `ff-session`, `ff-logging`, `ff-undo-redo`,
`ff-help`, `ff-desktop`) call `ff-paths` accessors instead of `dirs::`. To keep
the dependency graph acyclic (`ff-config` and `ff-session` are low-level),
`ff-paths` sits BELOW them; it must not depend on `ff-config` or `ff-session`.
The active Application_Profile (CR-NR-081), which currently layers a
`profiles/<slug>/` segment onto config/data roots, is applied by `ff-paths` as a
profile segment on `config_root()`/`data_root()` so the existing profile
isolation is preserved on top of the new roots.

### Unified name

MANAGED-mode per-user locations use ONE name. CHOSEN: `FileForgeWorkbench`
(the data/logs name, and the human-facing product name), for all of config,
data, logs, cache, tmp, extensions, recovery. Rationale over the alternatives:
`ffworkbench` is an internal lowercase abbreviation and `.fileforgewb` is a
hidden-dir abbreviation; the full product name is the clearest for a
"easy-to-find" installation and already dominates the data/log locations.
The lowercase Unix variant `file-forge-workbench` is dropped in favour of the
single name for cross-platform consistency (the directory is app-private, so the
XDG lowercase convention is not required). Migration of existing users from the
old names is noted as a follow-up (out of scope here).

### Two modes and precedence

`Mode::Portable` when a marker/override selects it; `Mode::Managed` otherwise.
Precedence (Req 2.5): CLI flag > `FFW_HOME` > `portable.ffw` marker > Managed.
An explicit override (flag/env) ALWAYS forces Portable with the given root --
this is the critical seam for tests, CI, and USB use, where the root must be
deterministic regardless of where the binary sits.

### Install layout and lifecycle split

```
FileForgeWorkbench/              <- application root (PORTABLE)
  fileforge.exe
  portable.ffw                   <- marker (presence => PORTABLE)
  README.txt  LICENSE.txt
  resources/                     <- SHIPPED, overwrite on upgrade
    help/   i18n/   languages/   themes/   (default themes only)
  config/                        <- USER, preserve on upgrade
    fileforge.toml               <- user config incl. [paths] and [maintenance]
    profiles/   languages/       (user language overrides)
  data/                          <- USER, preserve on upgrade
    menus/ macros/ keymaps/ aliases/ workspace-kinds/
    screen-collections/ commands/ sessions/ recovery/
  extensions/                    <- USER, preserve on upgrade
  logs/   cache/   tmp/          <- DISPOSABLE
```

The locations that today MIX shipped + user content (help, i18n, themes,
languages -- inventory rows 5, 16, 17, 18, 26) are SPLIT: the shipped defaults
move under `resources/`, and user overrides live under `config/` (languages) or
`data/` (themes/menus) or are read as an overlay on top of `resources/` (help,
i18n locale overlays). Built-in menus/themes remain code-only constants and are
never written to disk (Req 3.8, consistent with the code-only built-in-menu
framework rule).

Resolution order for mixed content: `ff-paths` exposes BOTH the shipped root and
the user root for help/i18n/themes; the loader searches the user root first,
then the shipped `resources/` root, preserving today's "user overrides shipped"
behaviour while giving each a clean lifecycle.

### Relative vs absolute, relocatability

`ff-paths` resolves a config path value by: if `Path::is_absolute()`, honour
verbatim; else join under the application root (Req 6.1, 6.2). Shipped/default
config uses `./resources/...` relative forms (Req 6.3). Because PORTABLE roots
derive from `current_exe().parent()` at runtime and all internal references are
root-relative, moving or copying the whole directory relocates cleanly
(Req 6.4). ZIP extract is a valid install; deleting the directory is a full
uninstall for the portable case (Req 6.5).

### Redirectable disposable dirs

`config/fileforge.toml` may carry:

```toml
[paths]
logs  = "./logs"       # relative => under app root
cache = "D:/ffw-cache" # absolute => verbatim
tmp   = "./tmp"
```

`ff-paths` reads this section when resolving `logs_dir()`/`cache_dir()`/
`tmp_dir()`. `config/` and `data/` are NOT keys here and cannot be redirected
(Req 7.3) -- they define the installation. A relative value resolves under the
root; an absolute value is verbatim (Req 7.2). This supports synced
`%USERPROFILE%` folders (redirect cache/tmp out of the sync set) and read-only
installs (redirect disposable writes elsewhere).

### Retention / maintenance

```toml
[maintenance]
cleanup_on_startup   = true
logs_retention_days  = 10
cache_retention_days = 10
tmp_retention_days   = 10
```

A `maintenance::run_startup_cleanup(&AppRoot, &MaintenanceConfig)` pass, invoked
once at startup AFTER `ff-paths` resolves and BEFORE heavy init, iterates each
disposable dir and deletes files whose modified-age exceeds the per-dir
retention, restricted to files inside the RESOLVED dir and (where determinable)
owned/created by the app. Guardrails enforced in code and stated in Req 8:
value 0 => never clean that dir; `data/`/`config/` never touched; a redirected
disposable dir is age+ownership filtered, never blind-swept; the whole pass is
best-effort (errors skip-and-continue, never block startup). `tmp/` MAY also be
cleared on clean shutdown.

### Writability fallback

At `resolve()` time in PORTABLE mode, `ff-paths` probes the root for writability
(create+remove a `.write_test`, mirroring `UserDataDir::initialise`). If the
root is NOT writable (e.g. unzipped into `Program Files`), it keeps
`resources_root()` pointed at the install dir but redirects `config_root()`,
`data_root()`, and the disposable roots to the MANAGED per-user locations, and
logs a WARN naming the non-writable root and the fallback (Req 9.1, 9.2). The
installer default of `%USERPROFILE%\Apps\FileForgeWorkbench` avoids this for the
common case (Req 9.3).

### Optional Windows integration and packaging

The portable baseline needs NO installer (Req 10.1). An OPTIONAL
`ffw-setup.exe` (a separate, lightweight binary -- not specified in detail here)
offers shortcuts/associations, defaults to `%USERPROFILE%\Apps\
FileForgeWorkbench`, needs no admin, writes no portability-affecting registry
keys, and its outputs never change portability (Req 10.2-10.4). The build
produces `dist/FileForgeWorkbench-<version>-windows-x64/` with the full tree and
a ZIP of it as the distribution artifact (Req 11).

## Error handling

- Root resolution (`ff-paths::resolve`): CLI flag / `FFW_HOME` pointing at a
  non-existent path is a recoverable condition -- `ff-paths` creates the root if
  its parent is writable, else treats the root as non-writable and engages the
  writability fallback (Req 9). The active mode and any fallback are returned on
  `AppRoot` and logged at startup. Fatal only if neither the forced root nor the
  managed fallback can be made writable; the caller surfaces a clear startup
  error.
- Config `[paths]` value that is malformed or points at an uncreatable location:
  recoverable -- log a WARN naming the key and fall back to the default location
  under the root; the application continues.
- Maintenance pass: every file operation is best-effort. A permission error,
  a file held open, or an unreadable entry is logged at DEBUG/WARN and skipped;
  the pass never returns an error that blocks startup (Req 8.2, NF.4).
- Writability probe failure: recoverable -- engage the per-user fallback and log
  a WARN (Req 9.2).
- Missing shipped `resources/` subtree (help/i18n/themes): recoverable and
  already handled by the existing loaders -- an absent dir degrades to the
  compiled defaults / key-fallback, unchanged by this spec.

## Input validation

- `FFW_HOME` / `--ffw-home`: must be a directory path. Required? No (absence =>
  marker/managed). On a value that cannot be created or made writable => treat
  as non-writable root, fall back, WARN.
- `portable.ffw`: presence-only; contents are ignored for mode selection (a
  future version MAY read key=value hints from it, out of scope now).
- `[paths]` entries (`logs`/`cache`/`tmp`): optional strings; relative or
  absolute path; unknown keys ignored with a WARN; `config`/`data` keys here are
  rejected with a WARN (Req 7.3).
- `[maintenance]` values: `cleanup_on_startup` boolean (default true); retention
  days are non-negative integers (default 10); 0 means never clean; a negative
  or non-integer value is clamped to the default with a WARN.

## Invariants and ownership

- Single-authority invariant: no user-facing directory is resolved outside
  `ff-paths`. Owned by `ff-paths`; enforced by a repo-search acceptance check
  (AC 1) and by the other crates depending on `ff-paths` accessors.
- Lifecycle invariant: `resources/` is overwrite-on-upgrade; `config/`,
  `data/`, `extensions/` are preserve-on-upgrade; `logs/`/`cache/`/`tmp/` are
  disposable. Owned by the install layout + the maintenance pass guardrails.
- Cleanup-containment invariant: the maintenance pass only ever deletes inside
  the resolved disposable dirs, age+ownership filtered. Owned by
  `ff-paths::maintenance`; it is the single layer that performs deletion so the
  guardrail lives in one place.
- Layering invariant: the six config layers and project-config location are
  unchanged. Owned by `ff-config` (`layer.rs`, `paths.rs::project_config_path`),
  untouched except that user/system roots are now obtained from `ff-paths`.

## Testability

Unit-testable (pure, deterministic given inputs -- NF.3):
- `ff-paths::resolve` precedence (flag > env > marker > managed) with
  synthesised exe-dir, env, and flag inputs.
- Each role-root and subdir accessor for both modes.
- Relative-vs-absolute resolution.
- `[paths]` redirection (including the config/data rejection).
- Maintenance age/ownership filtering and the 0-means-never and
  never-touch-data/config guardrails, using a temp dir with staged mtimes.
- Writability fallback using a read-only temp root.

Integration-testable:
- Full startup in a temp PORTABLE tree (marker present) asserting every
  subsystem resolves under the root.
- Managed startup asserting the single unified name.
- Move-the-directory relocatability (copy the temp tree, relaunch headless).

These are plain `std`/filesystem tests; no GUI harness is required for the path
authority itself. (Any GUI surface that later exposes the resolved paths would
follow the egui_kittest rule, but that is not in this spec.)

## Alternatives considered

- Keep per-crate `dirs::` calls and only unify the NAME: rejected -- it fixes
  the divergence but not relocatability, portability, or the single-authority
  need; the scattered calls would drift again.
- Make `ff-config` the path authority: rejected -- `ff-config` is a config
  layer, and `ff-session`/`ff-logging`/`ff-undo-redo` would then depend on it,
  risking cycles; a thin low-level `ff-paths` below all of them is cleaner.
- Marker-only mode selection (no env/flag): rejected -- tests/CI/USB need a
  deterministic forced root independent of exe location, so `FFW_HOME` and
  `--ffw-home` are required overrides.
- Freely redirectable `config/`/`data/` via `[paths]`: rejected -- those roots
  DEFINE the installation; redirecting them would reintroduce the "where does it
  live" ambiguity this spec removes. Only disposable dirs are redirectable.
