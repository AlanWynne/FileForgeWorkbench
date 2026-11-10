# Config model change -- owner decision 2026-11-04 (supersedes CR-CH-021)

## Decision
Switch from the current CODE-ONLY-WITH-OVERRIDES model to a WRITE-THROUGH-CONFIG
model, for the built-in menus (POM, Settings, SCRM) AND the command alias list:

- Hardcoded Rust defaults REMAIN, as the RECOVERY source only (not what runs).
  This is the real meaning of "never written to disk" the owner intended: the
  recovery baseline is in code so a corrupted/deleted config can always be rebuilt.
- FIRST RUN writes the defaults to TOML config files on disk.
- The application RUNS FROM THE CONFIG FILES at startup and during running --
  files are authoritative, not the constants + overlay.
- RESET BARE rebuilds the config files from the hardcoded Rust defaults (recovery
  action for corrupted/deleted/unwanted config).
- Nothing hidden: every menu and every alias is visible and editable on disk.

This REVERSES steering mechanism 4 / CR-CH-021 (built-in menus are code-only and
NEVER written to disk; a user file only overrides if present). The owner has
EXPRESSLY confirmed the reversal. The steering rule framework-conformance.md
mechanism 4 must be updated as part of this change.

## Rationale (owner)
Running from a single source (the files) is cleaner and simpler at execution
time than rebuilding from constants and overlaying each run. Everything is
exposed; the hardcoded values exist so Reset bare can rebuild if config is
corrupted or deleted.

## Deferred: upgrades (owner: "worry about it when one happens")
When a software update adds a NEW default menu option or alias, a user who
already has config files will not see it automatically (files are authoritative).
Accepted future answer (NOT built now): an UPGRADE command that builds the
reset-bare defaults and MERGES the existing user configuration into them. The
design should leave room for this (e.g. not preclude a version stamp or a merge
step) but must not implement it yet.

## Scope / affected areas (to confirm in the investigation + gate)
- Menu loader + first-run: write pom.toml / settings.toml / scrm.toml on first
  run; load from them thereafter.
- Alias loader: write the seed alias table (incl. X -> EXIT, see below) to the
  alias config on first run; load from it.
- Reset bare: rebuild all of the above from the Rust recovery defaults.
- Startup sequence and the Menus editor (which already reads user files).

## Items that now build ON this model (previously queued against the old model)
- X -> EXIT alias: X is NOT a command; it is a SEED alias of EXIT in the FFCMD
  alias config (written on first run, restored by Reset bare). POM X option
  command becomes EXIT; shows as `X EXIT : ...`. There is no command named X.
  Owner also wants X resolution: active command environment interpreter first,
  then FFCMD resolves X -> EXIT.
  OWNER DECISION 2026-11-04: ONLY X -> EXIT becomes a seed alias for now. Do NOT
  turn any other current built-in verb into a seed alias in this change.
- Menus editor SCRM bug (separate): the editor drop-down lists a hardcoded
  [POM, Settings] + on-disk *.toml and misses SCRM; and load_menu_for_editor
  falls back to recovery_pom_menu() for SCRM instead of recovery_scrm_menu().
  Under the new model SCRM has a scrm.toml on disk, so listing/loading it falls
  out naturally -- re-evaluate this fix against the new model.

## Note surfaced 2026-11-04 (menu-kind-gate test isolation)
The menu loader DEFAULTS a missing `show_calendar` to TRUE. A stray on-disk
`settings.toml` omitting the flag therefore made Settings render a calendar it
should not. This silent/implicit default is exactly what the write-through model
removes: first-run must write COMPLETE config with EVERY flag explicit (e.g.
Settings `show_calendar = false`), so there are no hidden defaults and the file
fully determines behaviour. Carry this into the config-model gate.

## X / END / RETURN / EXIT -- owner decision 2026-11-04 (ISPF model)
Three distinct commands, ISPF-authentic:
- END (PF3): pop ONE level. Already exists, bound to PF3. KEEP; PF3 stays END, NOT EXIT.
- RETURN: pop back to the application's PRIMARY panel. Confirm it exists and
  behaves this way (F4 Return key seen in the worklist); add/adjust if not.
- EXIT: terminate FFWB. Already exists (file.exit).
X = EXIT EVERYWHERE (owner confirmed, explicit): X is a SEED alias of EXIT;
EXIT terminates FFWB unconditionally on EVERY screen, including intermediate ones.
The old nav_x "collapse to visual root / close workspace / exit when last" verb is
RETIRED -- its step-back behaviour is now covered by RETURN (to primary panel) and
END (one level); the terminate-when-last nuance is gone (X just exits).
POM and SCRM X options' command becomes EXIT; displayed as `X EXIT`.
Resolution order preserved: active command environment interpreter first (so an
editor still resolves FFEDIT X -> EXCLUDE), then FFCMD resolves X -> EXIT.
FFEDIT X -> EXCLUDE must NOT be shadowed.

## Investigation correction
There IS a seed layer today: FFCMD seeds an IDENTITY table (every root maps to
itself) and FFEDIT base carries X -> EXCLUDE / INCLUDE -> SHOW. So adding the
FFCMD seed X -> EXIT is a natural extension, not a new mechanism. Reset bare today
does NOT touch aliases and rebuilds no files; both change here.

## Relation to portable-installation (docs/specs/portable-installation)
A gated-but-UNIMPLEMENTED plan exists: docs/specs/portable-installation
(requirements + design + tasks, "no .rs changed" out-of-scope). It introduces an
ff-paths authority, PORTABLE vs MANAGED run modes, and a single install layout
(resources/ shipped-defaults vs config/ + data/ user-state vs logs/cache/tmp
disposable). It will REROUTE every directory this config-model change writes
(menus, aliases, etc.) through ff-paths, and places SCRM captures under
data/screen-collections/.
RECONCILIATION TO FLAG when portable-installation is implemented: its Req 3.8
still states the OLD code-only-built-in rule (no built-in file written on first
launch). CR-CH-116 SUPERSEDES that with write-through (first run writes complete
config; defaults are the recovery source). The portable-installation design must
be reconciled with CR-CH-116 -- shipped defaults vs written user config under the
resources/ vs config//data/ split -- when it is picked up. The agent's SCRM-dir
helper correction (write scrm.toml under menus_dir to match the read path) is a
correct interim fix within TODAY's layout and does NOT pre-empt the ff-paths
reroute; portable-installation migrates all these dirs together later.

## Process
Confirmed framework change -> read-only investigation of current loaders /
first-run / Reset bare, then a DESIGN GATE (supersede CR-CH-021 in
framework-conformance.md; new/updated criteria in menu-workspace,
command-configurator alias specs, configuration-system, startup-and-session),
owner approval, then implementation. The X/EXIT, SCRM-editor and calendar items
build on the agreed model.
