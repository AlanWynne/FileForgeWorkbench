# Portable Installation -- Tasks

Implementation tasks for the portable, self-contained installation. Each task
cross-references the requirement(s) it satisfies. This is the requirements gate
output; no source code is written until these tasks are approved.

- [ ] 1. Create the ff-paths crate skeleton and application-root model
  - [ ] 1.1 Add a new `ff-paths` crate (depends on `std` + `dirs` only; sits below ff-config/ff-session) with an `AppRoot` type carrying the resolved root and a `Mode` enum (`Portable`/`Managed`) (Req 1.1, 1.3)
  - [ ] 1.2 Implement `resolve()` with precedence CLI flag > `FFW_HOME` > `portable.ffw` marker (next to the executable) > Managed fallback (Req 2.1-2.5)
  - [ ] 1.3 Add a one-time startup init so the authority is resolved before any subsystem reads a path, exposing the active mode and roots (Req 1.3)

- [ ] 2. Implement role roots and named subdir accessors
  - [ ] 2.1 Implement `resources_root`, `config_root`, `data_root`, `extensions_root`, and disposable `logs_dir`/`cache_dir`/`tmp_dir` for both modes (Req 1.2, 3.1-3.6)
  - [ ] 2.2 Implement named accessors for every inventory location: menus, macros, keymaps, themes, help, i18n, aliases, workspace-kinds, screen-collections, command store, sessions, recovery, profiles, languages, user config (Req 1.4)
  - [ ] 2.3 Apply the active Application_Profile (CR-NR-081) as a profile segment on config/data roots, preserving existing profile isolation (Req 4.1, 4.3)
  - [ ] 2.4 Honour the retained test/host env seams (`FFWB_USER_CONFIG_PATH`, `FFWB_USER_ALIASES_DIR`, `FFWB_HISTORY_PATH`) verbatim in the matching accessors (Req 1.4)

- [ ] 3. Unify the application name in Managed mode
  - [ ] 3.1 Resolve the `FFWorkbench` / `ffworkbench` / `FileForgeWorkbench` / `file-forge-workbench` / `.fileforgewb` divergence to the single name `FileForgeWorkbench` for all per-user roots (Req 2.6)
  - [ ] 3.2 Record the old-name-to-new-name mapping and note that user-data migration is a follow-up (Req 2.6; migration out of scope)

- [ ] 4. Relative vs absolute path resolution
  - [ ] 4.1 Implement path-value resolution: absolute honoured verbatim, relative joined under the application root (Req 6.1, 6.2)
  - [ ] 4.2 Switch shipped/default config to root-relative `./resources/...` forms instead of machine-specific absolute paths (Req 6.3)

- [ ] 5. Retarget all crates onto ff-paths (remove scattered dirs:: path-joins)
  - [ ] 5.1 Retarget `ff-config/src/paths.rs` (user config, profiles, languages roots) onto ff-paths while keeping `project_config_path` project-relative (Req 1.1, 4.2)
  - [ ] 5.2 Retarget `ff-session/src/user_data_dir.rs` onto ff-paths and rename the `plugins/` required subdir to `extensions/` (Req 1.1, 5.1)
  - [ ] 5.3 Retarget `ff-logging/src/config.rs` log directory onto ff-paths `logs_dir` (Req 1.1, 3.6)
  - [ ] 5.4 Retarget `ff-undo-redo/src/recovery.rs::unsaved_recovery_path` onto ff-paths `recovery_dir`, keeping per-file recovery next to the source (Req 1.1)
  - [ ] 5.5 Retarget `ff-help` and `ff-desktop/src/shell/help.rs` help search onto ff-paths shipped+user help roots (Req 1.1, 3.7)
  - [ ] 5.6 Retarget `ff-desktop` i18n bootstrap, themes, menus, macros, keymaps, aliases, workspace-kinds, screen-collections, and command store onto ff-paths accessors (Req 1.1, 3.4, 3.7)
  - [ ] 5.7 Add an acceptance check (repo search) that no user-facing directory is resolved by a direct `dirs::` path-join outside ff-paths (Req 1.1)

- [ ] 6. Install layout and shipped/user lifecycle split
  - [ ] 6.1 Define the portable layout (exe, resources/, config/, data/, extensions/, logs/, cache/, tmp/, portable.ffw, README/LICENSE) (Req 3.1)
  - [ ] 6.2 Split mixed shipped+user content (help, i18n, themes, languages) so shipped defaults live under resources/ and user overrides under config//data/, searching user root first then resources/ (Req 3.2-3.5, 3.7)
  - [ ] 6.3 Confirm built-in menus/themes stay code-only constants and are never written to resources/ or data/ on first launch (Req 3.8)
  - [ ] 6.4 Name the installed-extensions directory `extensions/` everywhere, never `plugins/` (Req 5.1)

- [ ] 7. Redirectable disposable directories
  - [ ] 7.1 Read a `[paths]` section (`logs`/`cache`/`tmp`) from `config/fileforge.toml` and apply it in the disposable accessors (Req 7.1, 7.4)
  - [ ] 7.2 Resolve relative `[paths]` values under the root and absolute values verbatim (Req 7.2)
  - [ ] 7.3 Reject `config`/`data` redirection via `[paths]` with a WARN (Req 7.3)

- [ ] 8. Retention / maintenance pass
  - [ ] 8.1 Read a `[maintenance]` section with defaults cleanup_on_startup=true and 10-day retention for logs/cache/tmp (Req 8.1)
  - [ ] 8.2 Implement a best-effort startup cleanup that never blocks startup and tolerates partial failure (Req 8.2, NF.4)
  - [ ] 8.3 Restrict deletion to files inside the resolved logs/cache/tmp dirs, age-filtered, preferring app-owned files (Req 8.3, 8.4)
  - [ ] 8.4 Treat a retention value of 0 as never-clean for that directory (Req 8.5)
  - [ ] 8.5 Guarantee data/ and config/ are never auto-cleaned (Req 8.6)
  - [ ] 8.6 For a redirected disposable dir, apply age + ownership filters only (never blind-sweep) (Req 8.7)
  - [ ] 8.7 Optionally clear tmp/ on clean shutdown (Req 8.8)

- [ ] 9. Writability fallback
  - [ ] 9.1 Probe the portable root for writability at resolve time (create+remove a write-test) (Req 9.1)
  - [ ] 9.2 On a non-writable root, keep resources_root on the install dir but redirect config/data/disposable roots to per-user locations, and log a WARN naming the root and fallback (Req 9.1, 9.2)
  - [ ] 9.3 Document and default the installer destination to %USERPROFILE%\Apps\FileForgeWorkbench to avoid the fallback in the common case (Req 9.3)

- [ ] 10. Preserve the config layering model
  - [ ] 10.1 Verify the six-layer order (Defaults/System/User/Profile/Project/Workspace) is unchanged after retargeting roots onto ff-paths (Req 4.1, 4.3)
  - [ ] 10.2 Verify project config is still read from <project>/.ffworkbench/config.toml independent of the application root (Req 4.2)

- [ ] 11. Optional Windows integration (ffw-setup)
  - [ ] 11.1 Confirm the portable baseline runs with no installer via ZIP extraction (Req 10.1, 6.5)
  - [ ] 11.2 Specify an optional lightweight ffw-setup.exe for shortcuts/associations, defaulting to %USERPROFILE%\Apps\FileForgeWorkbench, no admin, no portability-affecting registry keys (Req 10.2-10.4)

- [ ] 12. Packaging
  - [ ] 12.1 Produce dist\FileForgeWorkbench-<version>-windows-x64\ with the full install tree (Req 11.1)
  - [ ] 12.2 Produce the ZIP of that tree as the distribution artifact (Req 11.2)

- [ ] 13. Tests (TDD, written before/with each implementation task)
  - [ ] 13.1 Unit tests for resolve() precedence (flag > env > marker > managed) with synthesised inputs (Req 2.5)
  - [ ] 13.2 Unit tests for each role-root and subdir accessor in both modes (Req 1.2, 1.4, 3.1-3.6)
  - [ ] 13.3 Unit tests for relative-vs-absolute resolution (Req 6.1, 6.2)
  - [ ] 13.4 Unit tests for [paths] redirection including config/data rejection (Req 7.1-7.4)
  - [ ] 13.5 Unit tests for maintenance age/ownership filtering, 0-means-never, and never-touch-data/config (Req 8.3-8.7)
  - [ ] 13.6 Unit test for the writability fallback using a read-only temp root (Req 9.1, 9.2)
  - [ ] 13.7 Integration test: PORTABLE startup in a temp tree resolves every subsystem under the root (Req 2.1, 3.1)
  - [ ] 13.8 Integration test: MANAGED startup uses the single unified name (Req 2.6)
  - [ ] 13.9 Integration test: move/copy the portable tree and relaunch still runs (Req 6.4)

- [ ] 14. Framework-change and steering records
  - [ ] 14.1 Record the owner-approved framework change (root/persistence resolution) per framework-conformance in design.md (Req NF.2)
  - [ ] 14.2 Add the portable-installation sub-project to the specs steering list (done in this gate) (Req NF.1)
