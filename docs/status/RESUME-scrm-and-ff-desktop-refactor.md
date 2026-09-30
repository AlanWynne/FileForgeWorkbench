# RESUME NOTE -- SCRM + ff-desktop decomposition (CR-NR-098)

Last updated: requirements draft complete, AWAITING OWNER REVIEW before the rest
of the gate (design/tasks/master/TCR) and before any build.

## DECOMPOSITION WAVE 1 -- ff-theme-editor -- DONE, owner-confirmed full gate CLEAN (2026-09-29)

The first behaviour-preserving PANEL-EXTRACTION wave (Req 19.3/19.4, distinct from
the SCRM engine crates below) landed:
- New crate `crates/ff-theme-editor/` (`src/lib.rs`): the pure Theme Editor --
  `EditableToken`, `ThemeEditorAction`, `ThemeEditorState` + methods, `render`,
  and its 4 unit tests. Deps: `ff_theme` + `egui` only.
- `ff-desktop/src/theme_editor_panel.rs` is now a 56-line THIN ADAPTER: it
  `pub use`s the crate's types (so `crate::theme_editor_panel::*` references
  resolve unchanged) and keeps the `WorkspaceContext` impl (the trait is local to
  ff-desktop). `apply_theme_editor_action`/`open_theme_editor`/
  `refresh_theme_editor_list` stay in `shell/commands.rs`.
- `ff-theme-editor` added as a path dep in `ff-desktop/Cargo.toml`.
- FIXED a pre-existing break: the crate was in the workspace `members` list with a
  `Cargo.toml` but no `src/`, so the whole workspace failed to load; Wave 1
  supplies `src/lib.rs`.
- Measurement: ff-desktop 54,545 -> 54,120 lines (-425); baseline doc wave log +
  TCR Req 19.3/19.4 row updated.
- Scoped gate CLEAN (cargo nextest): `nextest run -p ff-desktop` 1225/1225 passed;
  `clippy -p ff-desktop --all-targets -- -D warnings` exit 0; `fmt` clean;
  `test -p ff-theme-editor` 4/4. (5 unrelated config/history tests fail only under
  thread-parallel `cargo test` -- B048 env-var isolation -- and pass under nextest.)
- OWNER ran the full gate on 2026-09-29 and reported CLEAN (empty ai-review.log).
  Wave 1 is DONE (owner-confirmed), not merely scoped-clean.

## DECOMPOSITION WAVE 2 -- ff-toolchain-panel -- SCOPED CLEAN, awaiting owner full gate (2026-09-29)

The second panel-extraction wave landed:
- New crate `crates/ff-toolchain-panel/` (`src/lib.rs`): the pure Toolchain Panel
  (`ToolchainEntry`, `ToolchainPanelState`, `render` + its 10 unit tests). Deps:
  ff-toolchain-api, ff-gcc-toolchain, ff-rust-toolchain, egui. (The previous chat
  had created the crate + `src/lib.rs` + registered it in the workspace `members`
  list, but had NOT finished the extraction: `ff-desktop` still held the full
  duplicate implementation, and the crate was not yet a dependency of ff-desktop.)
- `ff-desktop/src/toolchain_panel.rs` is now a 15-line THIN ADAPTER:
  `pub use ff_toolchain_panel::{render, ToolchainPanelState};` (no
  `WorkspaceContext` impl -- the Toolchain Panel is a bottom dock, so ALL of it
  moved out; the shell still owns the `ToolchainPanelState` field, the
  `show_toolchain_panel` flag, and the diagnostic-navigation side effect).
- `ff-toolchain-panel` added as a path dep in `ff-desktop/Cargo.toml`.
- Measurement: `toolchain_panel.rs` 559 -> 15 lines; ff-desktop 54,120 -> ~53,576.
- Scoped gate CLEAN (cargo nextest): `fmt --check` exit 0;
  `clippy -p ff-toolchain-panel -p ff-desktop --all-targets -- -D warnings`
  exit 0; `nextest run -p ff-toolchain-panel -p ff-desktop` 1225 run / 1225
  passed / 0 skipped (the 10 panel tests moved with the code). Log:
  tools\logs\wave2-gate.txt.
- AWAITING owner's full verify.ps1 gate to mark Wave 2 DONE (owner-confirmed).

## DECOMPOSITION WAVE 3 -- ff-catalog-registry -- DONE, owner-confirmed full gate CLEAN (2026-09-30)

The third panel-extraction wave (DECOMP.3, decomposition-tasks.md task 11) landed:
- New crate `crates/ff-catalog-registry/` (`src/lib.rs`): the pure catalog-registry
  model -- `CatalogType`, `VirtualCatalog`, `CatalogRegistry`, `RegistryError`, the
  `dataset_node` helper, and its 22 unit tests. Deps: ff-dscatalog, ff-vfs,
  ff-file-tree, serde, toml (egui-free, zero `crate::` coupling). (The crate
  scaffold -- Cargo.toml + members entry + the lib.rs model body -- pre-existed
  from an earlier session; this wave supplied the missing test module and finished
  the extraction.)
- `ff-desktop/src/catalog_registry.rs` is now a 13-line THIN ADAPTER:
  `pub use ff_catalog_registry::*;` so every `crate::catalog_registry::*` reference
  (session_manager, shell/mod, shell/commands, shell/render, shell/reset_bare,
  files_panel, catalog_manager_dialog) resolves unchanged. No `WorkspaceContext`
  impl (the registry is a pure model, not a Context).
- `ff-catalog-registry` added as a path dep in `ff-desktop/Cargo.toml`.
- Measurement: `catalog_registry.rs` 801 -> 13 lines (-788); ff-desktop
  ~53,576 -> ~52,788 (owner to confirm exact total at gate time).
- VERIFIED THIS SESSION (early, before the terminal broke):
  `cargo test -p ff-catalog-registry` 22/22 pass; `cargo test -p ff-desktop`
  compiled cleanly (19.44 s) and 1186/1191 passed. The 5 failures
  (close_workspace_removes_settings_from_config, exit_saves_command_history_and_reloads,
  full_shell_theme_unknown_leaves_theme_unchanged,
  startup_missing_or_corrupt_history_is_empty_no_panic,
  theme_follow_os_can_be_set_to_true) contain NO catalog_registry reference and are
  the known B048 env-var config-isolation class that passes under process-isolated
  nextest -- identical to Wave 1/2.
- SCOPED GATE CLEAN (2026-09-30, after a terminal recovery): `cargo fmt --check`
  exit 0; `cargo clippy -p ff-catalog-registry -p ff-desktop -- -D warnings`
  exit 0 (0 warnings, both crates checked); `cargo nextest run
  -p ff-catalog-registry -p ff-desktop` -> 1213 tests run, 1213 passed, 0 skipped
  (the 5 B048 config/history/theme tests pass under process-isolated nextest).
  Logs: tools\logs\decomp3-fmt.txt / decomp3-clippy.txt / decomp3-nextest.txt.
  (Earlier in the session the interactive terminal was non-functional env-wide --
  every command returned Exit Code -1 with no output -- so the scoped gate was
  deferred; it ran clean once the terminal recovered.)
- OWNER ran the full gate on 2026-09-30 and reported CLEAN (verify.ps1 FULL
  nextest: 9452 run, 9452 passed, 0 failed; empty ai-review.log). Wave 3 is DONE
  (owner-confirmed), not merely scoped-clean.
- NEXT decomposition candidate: DECOMP.4 -- `catalog_manager_dialog.rs`
  (1292 lines; now that ff-catalog-registry is a crate, its only `crate::` dep is
  the adapter re-export), sliced by sub-dialog (New/Edit/Delete).

## DECOMPOSITION WAVE 4 -- ff-catalog-dialog -- SCOPED CLEAN, awaiting owner full gate (2026-09-30)

The fourth panel-extraction wave (DECOMP.4, decomposition-tasks.md tasks 12-16)
landed:
- New crate `crates/ff-catalog-dialog/`: the New/Edit/Delete virtual-catalog modal
  dialogs, sliced into one module each -- `new_dialog.rs` (NewCatalogForm,
  DialogOutcome, validate, build_catalog, render + field renderers; 25 tests),
  `edit_dialog.rs` (EditCatalogForm, validate_edit, render_edit; 9 tests),
  `delete_dialog.rs` (DeleteChoice, DeleteCatalogConfirm, render_delete,
  execute_delete with Home-catalog protection; 8 tests), and `lib.rs` (shared
  `pub(crate) catalog_type_label` + public re-exports). Deps: ff-catalog-registry,
  ff-dscatalog, egui. 44 tests total.
- `ff-desktop/src/catalog_manager_dialog.rs` is now a 12-line THIN ADAPTER:
  `pub use ff_catalog_dialog::*;` so every `crate::catalog_manager_dialog::*`
  reference (files_panel, shell/render, shell/update) resolves unchanged.
- `ff-catalog-dialog` added as a path dep in `ff-desktop/Cargo.toml`.
- Measurement: catalog_manager_dialog.rs 1292 -> 12 lines; ff-desktop
  ~52,788 -> ~51,508 (-1280, owner to confirm exact total).
- Scoped gate CLEAN (2026-09-30): `cargo fmt -p ff-catalog-dialog -p ff-desktop
  -- --check` exit 0; `cargo clippy -p ff-catalog-dialog -p ff-desktop
  -- -D warnings` exit 0 (0 warnings); `cargo nextest run -p ff-catalog-dialog
  -p ff-desktop` -> 1191 tests run, 1191 passed, 0 skipped. Logs:
  tools\logs\decomp4-fmt-scoped.txt / decomp4-clippy.txt / decomp4-nextest.txt.
- PRE-EXISTING gate blockers (NOT DECOMP.4) FIXED in the same session (owner
  approved as a separate cleanup):
  - ff-pdf-export declared `#[cfg(feature = "protected")] pub mod protected;`
    with no `protected.rs`. Created the missing module: a GENERIC
    `protect_pdf` / `export_pdf_protected` (owner-password encryption,
    copy-enabled + edit-locked, optional user password, optional content-hash
    embedded in /Info) that operates on plain PDF bytes / pages instead of the
    SCRM-specific types the ff-scrm `pdf_protected` uses. lopdf aligned 0.34 ->
    0.36 to match ff-scrm. `cargo test -p ff-pdf-export --features protected`
    7/7 pass; `clippy --features protected -- -D warnings` exit 0.
  - Formatting drift in the prior-batch markdown-viewer crates (ff-md-viewer,
    ff-mdx-app, ff-mdx-installer, ff-mdx-plugin) absorbed with `cargo fmt`.
  - Workspace-wide `cargo fmt --check` now exit 0.
- AWAITING owner's full verify.ps1 gate to mark Wave 4 DONE (owner-confirmed).
- NEXT decomposition candidate: DECOMP.5 -- explorer substrate
  (`ff-context-menu`, `ff-nav-model`, `ff-explorer-view`).

## One-line status

Owner APPROVED requirements AND the full gate package. BUILD IN PROGRESS.
WAVE 0 COMPLETE AND GATE-VERIFIED: verify.ps1 FULL nextest CLEAN on 2026-09-26
(9404 run, 9404 passed, 0 failed; tools\logs\ai-review.log EMPTY). Engine-side
TCR rows flipped to PASS; project-master SCRM.0/SCRM.1 marked [x]; change-log
CR-NR-098 = IN PROGRESS (build). NEXT UP: Wave 1 (first ff-desktop change).

## TERMINAL WORKAROUND THAT WORKS (use this)

CORRECTION (see tooling.md "Terminal Invocation"): the Postgres/AUTH_TOKEN banner
comes ONLY from Windows PowerShell 5.1's machine profile. Using
`C:\tools\powershell7\pwsh.exe` avoids the banner entirely -- it is not a reason
to avoid the terminal. The SEPARATE mangling issue (character-by-character echo,
glued commands) comes from long interactive one-liners; run
`pwsh.exe -NoProfile -NonInteractive -Command "<short command>"` and prefer the
dedicated read/grep/list tools. The background-process + Python-waiter pattern
below is still the right choice for LONG jobs (verify.ps1, cargo) where you wait
on a log, not for quick inspection.

Inline `execute_pwsh` is unreliable (PS 5.1 machine profile prints a Postgres/
AUTH_TOKEN banner and mangles capture; ad-hoc commands sometimes don't run at
all). RELIABLE PATTERN, per tooling.md:
- Run long jobs (verify.ps1, cargo) via the BACKGROUND process tool.
- For waiting/polling/status, use PYTHON at C:\tools\python\python.exe (no PS
  profile). Helper added: `tools\python\scrm_gate_status.py` -- `--wait <secs>`
  blocks until tools\logs\ai-review.log appears, then prints progress + whether
  the gate is CLEAN (size 0). This is how Wave 0's gate result was confirmed.
- verify.ps1 writes tools\logs\verify.progress.txt (live phase/test count) and
  tools\logs\ai-review.log (empty == clean). Read those, not stdout.

## WAVE 1 -- COMPLETE + GATE-VERIFIED (2026-09-26)

Delivered the first ff-desktop change:
- `crates/ff-desktop/src/screen_snapshot.rs` -- POM ScreenProvider:
  `menu_workspace_screen_model(state, command_line)` builds a ScreenModel from
  the active Menu_Workspace; `render_snapshot(model, format)`.
- SNAPSHOT command intercept in `shell/commands.rs` handle_command (after MENUS):
  `SNAPSHOT [TEXT|ANSI|MARKDOWN|MD|HTML|YAML|AI]` -> `handle_snapshot(arg)` ->
  pure `snapshot_text_for_active(arg)` + clipboard (arboard) + status via
  `open_error`. `snapshot_format_label` helper in `shell/helpers.rs`.
- `ff-screen-model` added to `crates/ff-desktop/Cargo.toml`; module registered in
  `main.rs`.
- 9 tests: 4 in screen_snapshot module + 4 shell unit + 1 full-shell egui_kittest
  `full_shell_snapshot_on_pom_captures_selectable_text`.
- verify.ps1 FULL nextest CLEAN: 9413 run, 9413 passed, 0 failed; ai-review.log
  EMPTY. project-master SCRM.2 = [x]; TCR ff-desktop rows (Req 4.1, 6.1-6.3,
  2.2-2.3, 11.2-11.4) flipped to PASS.

## WAVE 2 -- COMPLETE + GATE-VERIFIED (2026-09-26)

ff-scrm wired into ff-desktop. Delivered:
- `scrm_session.rs` -- ScrmSession (active ScreenCollection + auto-capture flag);
  start/stop/capture/purge/status/list; auto-start default collection on first
  capture.
- CAPTURE command family in `shell/commands.rs` (`handle_capture`):
  START/STOP/SCREEN/STATUS/LIST/PURGE/REPLAY via the single handle_command path.
  Plus `active_screen_model()` / `active_screen_name()` (single provider seam) and
  `auto_capture_active_context()`.
- Auto-capture hook in `shell/nav_stack.rs::navigate_to` (Context-transition
  choke point; no-op unless auto-capture on).
- `scrm_viewer_panel.rs` -- ScrmViewerState + WorkspaceContext (First/Prev/Next/
  Last + selectable-text screen render + stable first-control id).
- Full Context wiring: WorkspaceKind::ScrmViewer (ff-session), TabKind::ScrmViewer
  + scrm_viewer() ctor, open_scrm_viewer_tab, reconstruct arm, descriptor maps
  (session_manager + nav_stack), set_active_tab_context arm, render dispatch in
  render.rs (stages the active collection then owned-panel swap), exhaustive-match
  arms (helpers/workspace_kind/shell mod/session_manager).
- 16 tests incl. `full_shell_scrm_viewer_first_tab_focuses_first_control`.
- verify.ps1 FULL nextest CLEAN: 9429 run, 9429 passed, 0 failed; ai-review.log
  EMPTY. (One clippy redundant-guard warning was fixed en route: `"" =>` instead
  of `other if other.is_empty()`.) project-master SCRM.3/SCRM.4 = [x]; TCR rows
  flipped.

Deferred within Wave 2 (recorded in project-master + TCR as separate rows, not
gate-blocking): shell wiring of conditional-capture rules + configurable interval
(Req 9.2-9.4, 9.6-9.9) and DIDL replay filter UI (Req 15.3) -- the ff-scrm ENGINE
already implements these; only the shell surface is deferred. Off-frame async
capture write (Req 18.3) also deferred (in-memory capture is cheap).

## WAVE 3 -- COMPLETE + GATE-VERIFIED (2026-09-26)

PDF checkpoint PASSED (lopdf 0.36 encryption module writes owner-password +
permission-flag encryption). Delivered:
- `ff-scrm/src/pdf.rs` -- dependency-free hand-written PDF 1.7 writer (Base-14
  Courier, real BT/Tj selectable text; title page + screen index + per-capture
  pages; honours masking). Chosen over printpdf (churning API).
- `ff-scrm/src/evidence.rs` -- EvidencePackage (user/date/session/collection/
  screens + test-case id + pass/fail) + `content_hash` (sha256, `sha2` crate) +
  `to_json` (keeps serde_json in ff-scrm; ff-desktop has none).
- `ff-scrm/src/pdf_protected.rs` -- `export_pdf_protected` + `ProtectionOptions`:
  build plain PDF -> lopdf load -> set /Info (+content hash) -> set trailer /ID
  (REQUIRED for key derivation) -> EncryptionState::try_from(EncryptionVersion::
  V2{owner_pw,user_pw,key_length:16,permissions}) -> doc.encrypt -> save. Perms =
  PRINTABLE|COPYABLE|COPYABLE_FOR_ACCESSIBILITY|PRINTABLE_IN_HIGH_QUALITY
  (edit-locked, copy-allowed). Optional user/open password.
- Shell commands: CAPTURE EXPORT TEXT|MD|HTML|PDF [file]; CAPTURE EXPORT PDF
  PROTECTED [owner-pw] [file]; CAPTURE SAVE|LOAD|OPEN [file]; CAPTURE EVIDENCE
  [test-case-id] [PASS|FAIL] [file]. scrm_dir()/scrm_dir_override + resolve_scrm_path.
- verify.ps1 FULL nextest CLEAN: 9449 run, 9449 passed, 0 failed; ai-review.log
  EMPTY. (One clippy redundant-guard fixed in Wave 2; Wave 3 clean first pass.)
  Note: the gate took ~18 min this run -- ~11 min was the one-time compile of the
  fresh crypto/PDF dep tree (aes/cbc/md-5/jiff/rayon/lopdf/zip/sha2); the test run
  itself was normal. project-master SCRM.5/SCRM.6 = [x]; TCR flipped.

DEFERRED add-ons (recorded as 🔴 TCR rows, NOT gate-blocking): Req 20.4a digital
signature; Req 14.5 evidence-as-protected-PDF; Req 15.4 DIDL state-transition
export; Req 9.2-9.4/9.6-9.9 conditional-capture-rule shell wiring; Req 15.3 DIDL
replay filter UI; Req 18.3 off-frame async capture write. The ff-scrm ENGINE
already implements the rule/replay-filter logic; only shell surfaces are deferred.

## CR-NR-098 STATUS: core capability COMPLETE (Waves 0-3). Owner review point.

Screen Snapshot + SCRM is functionally delivered end to end: text-first capture
(SNAPSHOT), collection lifecycle + auto-capture (CAPTURE), replay viewer Context,
exports (text/md/html/pdf), persistence (save/load), evidence packages, and the
protected/tamper-evident PDF. Remaining items are the deferred add-ons above --
each is a discrete future slice, none blocks the core.

## WAVE 3 -- ORIGINAL PLAN (done) -- kept for reference (tasks.md tasks 7-9)

- CAPTURE EXPORT TEXT/MD/HTML wired to ff-scrm exporters (async). (Req 12.1-12.3)
- PDF export (printpdf) with title page/TOC/index/page numbers, real selectable
  text. (Req 12.4, 12.6)
- Evidence packages (user/date/session/collection/screens + test-case id +
  pass/fail). (Req 14)
- Protected/tamper-evident PDF: PRE-SLICE CHECKPOINT first (confirm the pinned PDF
  crate can apply owner-password encryption + permission flags; surface as blocker
  if not). Then CAPTURE EXPORT PDF PROTECTED: copy-enabled, edit-locked, optional
  read password, content hash embedded. (Req 20)
- CAPTURE OPEN/SAVE/LOAD commands (collection persistence via ff-scrm
  save_archive/load_archive). (Req 11.1 remainder)

## WAVE 2 -- ORIGINAL PLAN (done) -- kept for reference (tasks.md tasks 4-6)

- CAPTURE lifecycle commands (START/STOP/SCREEN/STATUS/LIST/OPEN/SAVE/LOAD/PURGE)
  wired to ff-scrm; auto-start default collection on first capture. (Req 7, 8, 11.1)
- Auto-capture hook at `nav_stack::reconstruct_context` + post-command model-diff
  guard; configurable interval; off-frame async write. (Req 9, 18.2-18.3)
- SCRM viewer Context: WorkspaceKind::ScrmViewer + TabKind + open_scrm_viewer_tab
  + reconstruct_context arm + descriptor mapping; WorkspaceContext impl with
  InteriorFocus + mandatory full-shell first-Tab test; CAPTURE REPLAY + DIDL
  filter. (Req 10, 15.3, 16)
Add `ff-scrm` to `crates/ff-desktop/Cargo.toml` when Wave 2 wiring begins.

## TERMINAL: reliable pattern (confirmed this session)

Run long jobs via the BACKGROUND process tool with output redirected to a log
(`cargo ... 2>&1 | Tee-Object -FilePath <log>; "EXIT=$LASTEXITCODE" | Out-File
-Append <log>`). Then run a PYTHON waiter (no PS profile) as a background process
and read ITS output:
- `C:\tools\python\python.exe tools\python\wait_for_marker.py <log> "EXIT=" <secs>`
  for cargo/test runs.
- `C:\tools\python\python.exe tools\python\scrm_gate_status.py --wait <secs>` for
  the verify.ps1 gate (reports ai-review.log CLEAN/PROBLEMS). Also read
  `tools\logs\verify.progress.txt` for live test counts.

## PRIOR WAVE 1 PLAN (done) -- kept for reference (tasks.md task 3)

3.1 Register SNAPSHOT + TEXT/ANSI/MARKDOWN(MD)/HTML/YAML(AI) as Function-target
    Command_IDs, resolved via resolve_target -> Function; NO new `if upper==...`
    intercept. Command handlers in shell/commands.rs area. (Req 4.1-4.6, 11.2-11.3)
3.2 First ScreenProvider impl: the POM / Home Context. Build a ScreenModel from
    the POM's state (primary_option_menu.rs / menu_workspace). (Req 2.1, 2.2)
3.3 Deliver rendered snapshot to the clipboard (arboard is already a ff-desktop
    dep) + status confirmation; "not capturable" path when the active Context has
    no provider. (Req 6.1-6.3, 2.3)
3.4 Full-shell egui_kittest: SNAPSHOT on the POM copies selectable text of the
    expected fields. (Req 1.1, 6.1)
Add ff-screen-model (and ff-scrm for Wave 2) to crates/ff-desktop/Cargo.toml when
wiring begins. Keep each new ff-desktop file <=400 non-test lines.

## PRIOR STATE (kept for reference)

## WAVE 0 STATUS -- code complete, per-crate gate green

Both new crates built with TDD and PASS in isolation:
- `crates/ff-screen-model/` -- ScreenModel + elements (model.rs), ScreenProvider
  trait + SnapshotFormat (lib.rs), renderers (render.rs coordinator +
  render/{text,ansi,markdown,html,yaml}.rs), tests/property_tests.rs.
- `crates/ff-scrm/` -- model.rs (ScreenCollection/ScreenCapture + DIDL + optional
  image), rules.rs (CaptureRule/RuleSet + MaskingRules), replay.rs (ReplaySession
  state machine + dialog-state filter), export.rs (text/md/html + Masking on/off),
  persist.rs (zip archive save/load + crash-recovery journal), error.rs,
  tests/property_tests.rs (archive round-trip proptest + 10,000-capture test).

Per-crate gate result (all clean):
- `cargo test -p ff-screen-model -p ff-scrm` -> 43 passed, 0 failed (EXIT=0).
- `cargo clippy -p ff-screen-model -p ff-scrm --all-targets -- -D warnings` -> EXIT=0.
- `cargo fmt -p ff-screen-model -p ff-scrm -- --check` -> clean after running fmt.

Both crates added to workspace Cargo.toml members (ff-screen-model, ff-scrm,
placed before ff-desktop).

## FIRST THING TOMORROW

1. Run the COMPLETION GATE (full workspace, no scope switch):
   `powershell -ExecutionPolicy Bypass -File tools\powershell\verify.ps1`
   (I started it but stopped it to save state; it had only printed the PS-profile
   Postgres banner, no cargo results yet.) Then read `tools\logs\ai-review.log`
   (empty == clean). This confirms the two new crates did not break the workspace.
2. If clean: flip the ff-screen-model + ff-scrm TCR rows (the engine-side Req 1.1/
   1.2, 2.1/2.4, 3.x, 4.2-4.6, 5.x, 7.x, 8.2/8.3, 9.7-9.9, 10.x, 12.1-12.3, 12.5,
   13.1-13.3, 15.1-15.3, 17.x, 18.1, 18.4, 20.4-engine) from 🔴 to ✅. The
   ff-desktop-side rows STAY 🔴 (Wave 1+ not started). Mark project-master SCRM.0
   and SCRM.1 [x].
3. Then Wave 1 (tasks.md task 3): SNAPSHOT commands as Function targets + POM
   ScreenProvider impl + clipboard delivery + full-shell egui_kittest. THIS is the
   first ff-desktop change.

## ENVIRONMENT GOTCHAS (carry forward)

- Inline terminal output is UNRELIABLE this session (the PS 5.1 machine profile
  prints a Postgres/creds banner and mangles capture). PATTERN THAT WORKS:
  `cargo ... *> tools\logs\<name>.log; "EXIT=$LASTEXITCODE" | Out-File -Append <log>`
  then read the log with read_file. Re-run + re-read if a build lock truncates it.
  verify.ps1 already writes tools\logs\ai-review.log -- read that, not stdout.
- No ff-desktop code touched yet; nothing is half-edited. Repo is consistent.

## Code-grounded seams found (verified, for the build)

- Command dispatch: ff_command::resolve_target -> CommandTarget (command_target.rs);
  shell/target_dispatch.rs dispatch_command_target; Function -> handle_command.
  Simple verbs are `if upper=="..."` intercepts in shell/commands.rs ~L384+.
- AUTO-CAPTURE HOOK (Q3): nav_stack::reconstruct_context (via navigate_to ~L280)
  is the single Context-transition choke point. Plus a post-handle_command
  model-diff guard for command-driven screen changes.
- New Context wiring pattern (like EventLog/MacroLibrary): WorkspaceKind variant
  (ff-session) + TabKind + open_*_tab + reconstruct_context arm (commands.rs
  ~L2180) + descriptor mapping (session_manager.rs / nav_stack.rs).
- WorkspaceContext trait: shell/workspace_context.rs, render -> InteriorFocus,
  ShellRequest drain. Panels split state/render (config_panel, menu_workspace,
  search_results_panel) so ScreenProvider reads panel STATE, not egui.
- PDF (Q8): printpdf for selectable text + layout, lopdf to apply encryption/
  permissions/passwords; content hash for tamper-evidence; signatures deferred.

## The two owner goals (one combined program)

1. Refactor/optimise `ff-desktop` so it compiles and links faster (it is a
   ~49k-line monolithic binary crate; every panel edit recompiles the whole crate
   incl. `shell/tests.rs` 9,672 lines and relinks `ffwb` against ~35 path-deps).
2. Extend FFWB core with the Screen Snapshot + SCRM capability from the two owner
   source docs.

Owner decisions recorded:
- Do BOTH together (owner confirmed an earlier, lost chat recommended this).
- Build LIKE a plugin, package LIKE core: capture couples only via a small
  `ScreenProvider` trait + existing command-dispatch / WorkspaceContext seams;
  logic lives in new core crates compiled into the app (no install step); can be
  lifted to a real plugin later with no rewrite.
- HARD CONSTRAINT: capture is copy/paste-able TEXT from a logical Screen_Model,
  NOT jpeg/png. Bitmap is optional secondary only.
- ADDED (owner): protected PDF export -- copy-enabled but edit-locked, as an
  unalterable activity record. Captured as Requirement 20 (+ Req 12.7, Req 14.5,
  command CAPTURE EXPORT PDF PROTECTED). Framed honestly as "copy-enabled,
  edit-locked, tamper-EVIDENT" (owner-password permission lock is advisory;
  integrity hash/signature makes tampering DETECTABLE) -- NOT "physically
  uneditable". New open questions Q6-Q8 for the owner.

## What exists on disk NOW (this is the whole footprint)

- `docs/status/change-log.md` -- CR-NR-098 logged, PENDING GATE.
- `docs/specs/screen-snapshot-scrm/requirements.md` -- THE deliverable to review.
  19 requirements: Req 1 (logical-not-raster constraint), Req 2 (ScreenProvider
  seam), Req 3 (model preservation), Req 4-6 (SNAPSHOT command + renderers +
  copy), Req 7-10 (collection/manual/auto+conditional/replay), Req 11 (commands
  via single dispatch), Req 12-15 (export/masking/evidence/DIDL), Req 16 (viewer
  as WorkspaceContext), Req 17 (data model), Req 18 (non-functional), Req 19
  (ff-desktop decomposition delivered together; new crates ff-screen-model +
  ff-scrm). Ends with 5 OPEN QUESTIONS (Q1-Q5) for the owner.
- `.kiro/steering/specs.md` -- screen-snapshot-scrm registered in the list.

## ff-desktop measured facts (for the design phase)

88 .rs files, ~48,935 lines. Biggest: shell/tests.rs 9672, shell/render.rs 2616,
tab_manager.rs 2211, shell/commands.rs 2119, files_panel.rs 1998, shell/mod.rs
1613, editor_panel.rs 1512, shell/update.rs 1503, catalog_manager_dialog.rs 1176,
explorer_view.rs 1082, menu_workspace/render.rs 1029, main.rs 840 ...
NOTE: `ff-shell` crate is NOT the GUI shell -- it is the OS-command/terminal
subsystem. The GUI shell (WorkbenchShell) is still entirely inside ff-desktop.

## OWNER ANSWERS (received; baked into requirements.md)

- Q1 auto-start default collection: YES (Req 8.4).
- Q2 PDF text-selectable required: YES (Req 12.6).
- Q3 auto-capture transition events: DEFERRED to design.md (proposed: navigate_to
  push, command-completion screen change, open_menu_by_name).
- Q4 crate names ff-screen-model / ff-scrm: CONFIRMED.
- Q5 sensitive field = Context-marked (passwords/hidden) OR config rule: BOTH
  (Req 13.1, 13.5).
- Q6 protected PDF v1 = permission lock + content hash, signature as add-on:
  CONFIRMED (Req 20.4 / 20.4a).
- Q7 optional read/open password: YES (Req 20.2a).
- Q8 PDF library selection: DEFERRED to design.md (shortlist to present).

Only Q3 and Q8 remain open, and both are design-phase decisions (present in
design.md for confirmation, not blockers to approving requirements).

## NEXT STEPS (only after owner approves the requirements)

1. Owner approves requirements.md (Q3/Q8 will be settled inside design.md).
2. Write design.md: crate boundaries (ff-screen-model, ff-scrm), the
   ScreenProvider extraction template, the ff-desktop decomposition WAVES (each
   behaviour-preserving, each keeping verify.ps1 full green, each measured
   before/after), PDF text-selectable library choice, auto-capture event mapping.
3. Write tasks.md (independently completable, `[ ]` only, cross-ref criteria).
4. Add tasks to docs/project-management/project-master/tasks.md.
5. Add NOT COVERED rows to docs/quality/TCR.md (one per new criterion).
6. Present the full gate package; wait for approval.
7. Only then: TDD build, SCRM first (as the first extracted vertical).

## Do NOT

- Do not start any build before the owner approves the requirements + gate.
- Do not add SCRM code inside the ff-desktop monolith (defeats goal 1).
- Do not introduce a parallel command-dispatch / nav / focus / persistence
  mechanism (framework-conformance).
