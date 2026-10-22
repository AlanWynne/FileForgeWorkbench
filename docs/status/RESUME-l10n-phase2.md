# RESUME -- Localisation Phase 2 (CR-NR-103 L10N.4)

Updated 2026-10-21 after Slice 1 completed and merged. Open FileForgeWorkbench in
a FRESH Kiro session (picks up the consolidated steering + the compile-only
verification policy in testing.md "Execution split").

## One-line status

*** TASK 8 (8.1 -- ff-desktop render literals) COMPLETE and MERGED to `main`
(2026-10-22). *** Tip `4fc79d8`. OWNER gate CLEAN: 9889/9889 passed, 0 failed,
fmt ok, clippy clean (full nextest workspace run, 0:10:55). All 20 areas done
INLINE (areas 1-4 prior, 5-20 this session) after the delegated-workflow trial
hung at the commit/signal boundary on its first step (see DEFECT-terminal-
execution.md section 8 -- the 3-part step-completion pattern did NOT prevent the
hang; inline is the reliable path on this machine). 22 new per-area
`ff-desktop-*.ftl` catalogues under crates/ff-i18n/i18n/en/. Localised: all
user-facing render prose across the ff-desktop panels, shell chrome, dialogs,
tab bar, POM calendar, SCRM viewer. LEFT (documented in each .ftl header):
glyphs, mnemonic captions (Aa/W/.*, CAPS/NULLS/..., on/bar), egui ids, proper
nouns, menu/POM descriptions (Task 5). DEFERRED to Task 9 by design: all
argument-bearing STATUS/ERROR messages (open_error strings; status-bar
Ln/Col/lines/CAPS/phase labels). Worktree + branch removed post-merge.
NOT YET PUSHED: `main` is ahead of origin/main (push when ready).

NEXT: Task 9 (hard status/error messages -- open_error + status-bar, via t_args
with command verbs as NON-translated args; localise thiserror text at the
display boundary). THEN Task 10 (capstone: author fr/de .ftl catalogue DATA +
per-environment alias catalogues + per-locale help/ dirs). ALWAYS run
ftl_dupe_check.py after adding any .ftl; never reuse a key id.

--- historical log below (Slices 1-2, Task 7) ---

SLICE 1 (FEAT-001/002/003): DONE, merged to `main`, PUSHED. Gate-clean 9885.
SLICE 2 (task 7) IN PROGRESS: crate1 (ff-catalog-dialog) COMPLETE, committed
(b518513), gate-clean 9888. crate2 (ff-dataset-alloc-dialog) COMPLETE, committed
(8281a48, 2026-10-22) -- GATE-CLEAN: OWNER ran cargo gate --build = 9888/9888
passed, 0 failed, 2026-10-22 09:01 (count unchanged from crate1, as expected for
a behaviour-neutral literal->t() conversion).
crate3 (ff-files-panel) COMPLETE, committed (74f62cc, 2026-10-22) -- compile +
clippy -D warnings + fmt + ASCII all clean (agent compile-verify only; OWNER
still owes the full cargo gate --build run for this commit). NOTE: files-panel.ftl
is the FIRST .ftl to use Fluent unicode-escape placeables ({ "\u2014" } em dash,
{ "\u2026" } ellipsis) to keep the source ASCII while rendering byte-identical UI
prose -- the owner gate confirms this resolves correctly.
crate4 (ff-explorer-view) COMPLETE, committed (fbf222e, 2026-10-22) -- compile +
clippy -D warnings + fmt + ASCII all clean (agent compile-verify; OWNER owes the
full gate). Only the 9 row context-menu labels were localised; row glyphs and the
clipboard text-tree format (Req 19.5/19.6) stay as source literals.
crate5 (ff-theme-editor) COMPLETE, committed (4765c45, 2026-10-22) -- compile +
clippy -D warnings + fmt + ASCII clean; new unit test passes. OWNER owes the full
gate. Option B chosen (owner): the 48 colour-token labels ARE localised. Pattern:
EditableToken.label stays the stable English IDENTITY (PartialEq/tests untouched);
a new EditableToken::label_key() maps each label to a Fluent key via a total
token_label_key() match, resolved by t() only at the render site. A guard test
(every_token_has_a_unique_dedicated_label_key) asserts every SURFACE token maps to
a unique dedicated key. 64 keys total in theme-editor.ftl (16 chrome + 48 tokens);
the contrast-advisory ratio is pre-formatted to 1 decimal in Rust and passed as a
text placeable to keep rendering byte-identical.
GATE on crate5 (2026-10-22 10:13) found ISSUES: 4 ff-catalog-dialog delete_dialog
tests FAILED because the whole `en` Fluent bundle failed to BUILD -- crate3's
files-panel.ftl had reused the key id `files-panel-title`, which ALREADY exists
in main.ftl (`= Files`, a baseline key with tests). ff-i18n merges every en .ftl
into ONE bundle, so a duplicate id makes add_resource reject and takes the whole
bundle down; the first test to load it (ff-catalog-dialog) was the canary.
FIXED (commit 4a70994): renamed the crate3 key to `files-panel-title-bar` in the
.ftl + tree.rs. Verified: ff-i18n tests (15) PASS (they load the real shipped en
dir), ff-catalog-dialog delete_dialog tests (10) PASS, and a new guard tool
tools/python/ftl_dupe_check.py reports 7 files / 182 keys / NO duplicates.
LESSON: per-crate cargo check/clippy do NOT catch a cross-file key collision --
the bundle only builds at runtime/test time. Run ftl_dupe_check.py (or the ff-i18n
tests) after adding any .ftl, and NEVER reuse an existing key id for new content.
OWNER: please RE-RUN cargo gate --build -- the previous run was on the broken
state; crate5 + the fix need a clean authoritative gate.

crate6 (ff-toolchain-panel) COMPLETE, committed (be852cb, 2026-10-22) -- compile +
clippy -D warnings + fmt + ASCII + dupe-check (8 files/196 keys, no dupes) all
clean. Localised the render-function literals (headings, status labels, buttons,
install-failed/detected/ready messages). LEFT unlocalised (have English-asserting
unit tests / are model-layer or terse codes): ToolchainEntry::build_status_text(),
severity_label() (E/W/N), and the plugin names (Rust/GCC, proper nouns).

*** TASK 7 CODE COMPLETE *** all six crates done (7.1-7.6), task 7 parent ticked.
*** BUT: a gate found a second regression -- NOT yet fully resolved (see below). ***

=== SECOND GATE FAILURE (2026-10-22 10:30) -- full-shell UI test ===============
Gate on be852cb: 1 test FAILED --
  ff-desktop shell::tests_focus::full_shell_theme_editor_type_name_and_save_creates_user_theme
Two layers:
 (A) FIXED: the test does harness.get_by_label("Save").click(). crate5 localised
     the Save button to t("theme-editor-btn-save"); but the test harness
     (harness_shell/make_shell in shell/tests_common.rs) never installed the i18n
     catalogue, so under nextest (each test = its own process) t() returned the
     KEY, not "Save", and get_by_label("Save") found nothing (panic at
     tests_focus.rs:1246). FIX APPLIED (uncommitted): added
     i18n_bootstrap::install_catalogue_for_tests() (new #[cfg(test)] pub(crate)
     helper calling install_catalogue_for_locale("en")) and call it at the top of
     harness_shell(). After this the label resolves and the click lands.
 (B) STILL FAILING (1 test only; fix A fixed ALL others -- latest gate 11:02 =
     2770 passed, 1 failed): the test gets PAST the label lookup but fails the
     file-write assertion ("clicking Save ... writes a USER theme") -- after
     get_by_label("Save").click() + run(), the user .toml is not on disk.
     DISPROVEN hypothesis: NOT frame timing. I tried `for _ in 0..4 { run() }`
     after the click -- STILL fails. Reverted that test change.
     CONFIRMED by code reading (not speculation):
       - apply path is SINGLE-FRAME and correct: render_body.rs renders the
         theme editor via render_workspace_context, takes panel.pending_action,
         and calls apply_theme_editor_action in the SAME update() frame; the
         ThemeEditorState WorkspaceContext::render does `self.pending_action =
         render(ui, self)` each frame.
       - apply_theme_editor_action(Save) on a built-in -> SaveAs(name_buffer) ->
         write_theme_file(new_name) to themes_dir(). Save-button wiring in
         theme-editor lib.rs is intact (action = ThemeEditorAction::Save).
       - test preconditions hold: name_buffer pre-filled (asserted), working is
         Some, dir_overrides.themes set to the TempDir.
     DIAGNOSTIC RUN (temporary panic after the click, then reverted) proved the
     SAVE ACTION NEVER FIRED. Immediately after get_by_label("Save").click() +
     run():
       new_name   = "Default Dark Copy"   (slug "default-dark-copy")
       error      = None       (no error set)
       name_buffer= "Default Dark Copy"    (NOT cleared)
       available  = [Default Dark, Default Light, Default High Contrast,
                     Default Legacy, Legacy Soft]   (new theme NOT present)
       themes dir = []          (empty -- no file written)
     SaveAs clears name_buffer + refreshes available on success; none of that
     happened, and error is None -> apply_theme_editor_action(Save) was NEVER
     invoked. So pending_action was not Save after the click: the click found a
     node (get_by_label did NOT panic as not-found/ambiguous, so exactly one
     "Save" node exists) but did NOT register as the theme-editor Save button's
     .clicked(). This is a kittest click-vs-widget issue, NOT a frame-count issue
     and NOT a bundle/label-resolution issue (the label clearly resolved to
     "Save", else get_by_label would have failed as before fix A).
     NEXT (unresolved -- needs egui_kittest know-how): determine why the clicked
     "Save" node does not drive the theme editor button's .clicked(). Candidates:
       - the Save button lives in the lower region AFTER the 48-row colour-token
         ScrollArea; the clicked node's rect may be off-screen / clipped so the
         simulated pointer does not hit it (a real kittest gotcha for widgets
         below a tall scroll area). Pre-slice-2 the button text was a literal but
         the LAYOUT was the same, so this would have been latent -- RE-CHECK
         whether this exact test actually passed at the slice-2 base (git log the
         test; it may be a NEWER test that never truly exercised the click path
         until the catalogue was installed by fix A).
       - OR enumerate the tree (egui_kittest Queryable::get_all_by / node debug)
         to see the Save node's role/rect. My attempts used unverified API names;
         confirm the correct egui_kittest 0.33 accessor first.
     Do NOT add frames (disproven) and do NOT weaken the assertion.
     NOTE: fix A (harness_shell catalogue install + install_catalogue_for_tests)
     is CORRECT and MUST be KEPT -- it got every OTHER full-shell label-lookup
     test green (gate 11:02 = 2770 passed, 1 failed). It is currently UNCOMMITTED
     (2 files: shell/i18n_bootstrap.rs + shell/tests_common.rs); commit it with
     the layer-B fix once B is understood.

IMPORTANT UNKNOWN: the gate used nextest and stopped reporting after the FIRST
ff-desktop failure. There may be OTHER full-shell tests that click now-localised
buttons by English label (e.g. catalog-dialog "OK"/"Cancel"/"Delete Catalog",
toolchain "Build", explorer ctx items). The harness_shell catalogue install (fix
A) should fix ALL of those at once (they all go through harness_shell). After
fixing (B), the OWNER must RE-RUN the full gate to surface any remaining ones.

Terminal note: the shell channel hit a dead episode during diagnosis (commands
returning no output / not writing logs). File reads stayed reliable.

*** LAYER B RESOLVED (2026-10-22) -- the theme-editor save test now PASSES. ***

Root cause (proven, not guessed):
  - The test was byte-identical to its pre-Slice-2 (d9dd2f9) form and passed then.
  - DIAG2 proved the PRODUCT is correct: with working=Some, calling
    apply_theme_editor_action(Save) directly WROTE the user .toml and added the
    theme to `available`. So Save -> SaveAs -> write_theme_file + the dir override
    all work. The failure was purely that get_by_label("Save").click() did not
    deliver a .clicked() to the button.
  - Trigger: fix A installs the i18n catalogue in the harness, so labels render
    as real English (different widths) instead of the long key fallback. The test
    queried get_by_label("Save") in the SAME frame the button was first laid out
    with the new widths, so the simulated click targeted an unsettled rect.

Fix (layer B): add ONE harness.run() BEFORE the get_by_label("Save") query so the
accessibility tree + layout are settled at query time; then click + run to apply.
This is a test-harness timing fix only -- product behaviour and the assertions are
unchanged. VERIFIED: the scoped test now reports `ok` (1 passed).

Three uncommitted test-infra files (fix A + layer B), ALL in ff-desktop:
  crates/ff-desktop/src/shell/i18n_bootstrap.rs   (install_catalogue_for_tests)
  crates/ff-desktop/src/shell/tests_common.rs     (harness_shell installs it)
  crates/ff-desktop/src/shell/tests_focus.rs      (settle-frame before the click)
COMMITTED: 8de5b2d "fix(l10n/test): install i18n catalogue in full-shell harness,
settle a frame before the theme-editor Save click" (3 files, 28 insertions).

Slice 2 branch commits (feature/l10n-phase2-slice2), in order:
  b518513 (7.1 catalog-dialog)  8281a48 (7.2 dataset-alloc-dialog)
  74f62cc (7.3 files-panel)     fbf222e (7.4 explorer-view)
  4765c45 (7.5 theme-editor)    4a70994 (ftl key-collision fix)
  be852cb (7.6 toolchain-panel) 8de5b2d (test-infra: harness catalogue + settle frame)

*** SLICE 2 COMPLETE AND MERGED TO MAIN (2026-10-22). ***
- OWNER full gate on tip 8de5b2d: CLEAN -- 9889/9889 passed, 0 failed (the +1 vs
  9888 is the new theme-editor label-key guard test).
- Fast-forwarded main 9b07352..8de5b2d (all 8 Slice 2 commits).
- Docs commit 6108ed3 (task 7 ticked, this resume updated).
- Guard tool commit a7d825e (tools/python/ftl_dupe_check.py).
- Worktree .worktrees/l10n-phase2-slice2 removed; branch feature/l10n-phase2-slice2
  deleted (was 8de5b2d, fully merged).
- main is AHEAD of origin/main by 10 commits -- NOT yet pushed. OWNER: push when ready.

FOLLOW-UP (optional, not blocking): wire tools/python/ftl_dupe_check.py into the
gate so a cross-file .ftl key collision is caught at build time, not test time.

Slice 2 pushed to origin/main (2026-10-22).

=== TASK 8 IN PROGRESS (workflow ABORTED -- now continuing INLINE) ===
Worktree .worktrees/l10n-phase2-task8, branch feature/l10n-phase2-task8 (off main).
Workflow wf_7a5d44144739f873 was launched but its code step STALLED ~15 min at the
signal/commit boundary (the documented terminal hang) after doing only area 1.
ABORTED it. Area-1 work was committed by hand (75f4c9b feat(l10n): task8
command-config localise render literals). Decision: finish the remaining 19 areas
INLINE (file tools reliable; the workflow coder keeps hanging at the commit
boundary on this machine) following the planner's plan at
.worktrees/l10n-phase2-task8/.agents/tasks/plan.md -- 20 ordered areas, each with
files + exact literals + leave-unlocalised rules + a scoped egui_kittest test.
Per-area commit, scoped cargo check/clippy/fmt + ftl_dupe_check after each .ftl.
OWNER runs the authoritative full cargo gate --build at the end; ORCHESTRATOR then
ff-merges to main + removes the worktree.
Progress: [x] 1 command-config (75f4c9b). [x] 2 config-panel (ddd3dfb). Areas
3-20 delegated to a TRIAL workflow (see below).

=== TASK 8 TRIAL WORKFLOW (areas 3-20) -- testing a clean step-completion pattern ===
After the first task-8 workflow hung at the commit/signal boundary (defect doc
section 7), the owner confirmed they PREFER workflows and want them to END cleanly
rather than fall back to inline. Diagnosis: the hang is the COMPLETION SIGNAL not
landing (a long/quiet shell command as the step's last act losing completion in
Kiro's wrapper), NOT the commit itself. Owner ideas folded in: (a) scripted commit
to a log, (b) async/fire-and-forget the long command.
Trial infrastructure (committed on the branch): tools/powershell/wf-commit.ps1
(dd19f14) -- bounded, logged git-commit helper; stages only listed files, commits,
writes resulting HEAD to tools/logs/wf-commit.log for verify-by-effect. Smoke-
tested OK.
Launched workflow wf_df38cebc46b21acb (label l10n-task8-areas3-20-trial) for areas
3-20, briefed with the 3-part STEP-COMPLETION PATTERN: (1) no long/quiet command as
a step's last act (background compile-verify to a log + read it), (2) commit via
wf-commit.ps1 and confirm via its log, (3) signal completion via the agent/tool
layer (send_message success / status), NOT a trailing shell command. The run also
reports whether this pattern avoided the boundary hang -- that is the key
diagnostic for whether we can return to the preferred delegated-workflow path for
tasks 9 and 10.
Orchestrator: ff-merge to main + remove the worktree after the run; OWNER runs the
authoritative full cargo gate --build.

NEXT L10N WORK after Task 8: Task 9 (hard status/error messages, argument-bearing),
Task 10 (capstone: author fr/de catalogue DATA + per-locale help dirs). Same
compile-only agent policy; ALWAYS run ftl_dupe_check.py after adding any .ftl, and
never reuse a key id.

NEXT (do NOT skip the gate): OWNER runs cargo gate --build on the slice2 branch
tip (be852cb). The last owner gate (10:13) ran on the BROKEN pre-fix state; the
fix (4a70994) + crate6 have only been AGENT-verified (compile/clippy/fmt/ASCII +
the ff-i18n & ff-catalog-dialog tests I ran scoped). A clean full gate is required
before ff-merge. THEN: ff-merge feature/l10n-phase2-slice2 -> main, push, remove
the .worktrees/l10n-phase2-slice2 worktree + branch.
Slice 2 commits: b518513 (7.1) 8281a48 (7.2) 74f62cc (7.3) fbf222e (7.4)
4765c45 (7.5) 4a70994 (collision fix) be852cb (7.6).
Also note: tools/python/ftl_dupe_check.py was added in the MAIN checkout (not the
worktree, so not in this branch) -- commit it to main separately as a reusable
guard tool; run it after adding any .ftl.

## SLICE 2 resume steps (do these next)

Worktree: c:\workspace\apps\FileForgeWorkbench\.worktrees\l10n-phase2-slice2
Branch:   feature/l10n-phase2-slice2 (off main). The slice2 WORKFLOW was ABORTED.

1. DONE -- crate1 (ff-catalog-dialog) committed b518513, task 7.1 ticked.
2. DONE -- crate2 (ff-dataset-alloc-dialog) committed 8281a48, task 7.2 ticked.
   Pattern used (copy for remaining crates): add ff-i18n path dep to the crate's
   Cargo.toml; author a per-crate .ftl in crates/ff-i18n/i18n/en/ with the EXACT
   current English literals (ASCII only, trailing-space alignment preserved via a
   { "  " } quoted placeable); swap each displayed literal at the render/validate
   sites for ff_i18n::t("key") / t_args("key", MessageArgs::new().set(...) /
   .set_number(...)); leave command verbs, enum logic keys, id_salt values, and
   technical mnemonic codes (e.g. DSORG/RECFM PS/PO/FB...) UNLOCALISED. Agent
   verification = scoped cargo check + clippy -D warnings + fmt --check + ASCII
   scan, reading each log; the OWNER runs the full cargo gate --build.
3. DO the remaining task-7 crates HANDS-ON, NOT via a workflow (workflow coder
   agents reliably WRITE code but have stalled repeatedly at the verify/commit/
   signal shell step). Remaining: ff-files-panel, ff-explorer-view,
   ff-theme-editor, ff-toolchain-panel -- one commit per crate.
4. After the last crate: tick task 7 parent, ff-merge the branch to main, remove
   the worktree. OWNER runs the final cargo gate --build.

NOTE (assistant-emission bug 2026-10-21 pm): this session repeatedly emitted
MALFORMED tool calls (rendering as garbled "court ..." plaintext no-ops) across
all tools, and the shell channel hit dead episodes. A Kiro RESTART cleared it
earlier. If tool calls garble again, restart Kiro -- nothing is lost (work is on
disk; workflow state is server-side).

(Historical detail of Slice 1 and later tasks follows below.)

## Where things stand

- `main` is at `44312e4` -- carries FEAT-001 (ff-i18n bootstrap wired in),
  FEAT-002 (menu title/description localisation), FEAT-003 (per-locale help).
  `main` is now 3 commits AHEAD of `origin/main` (UNPUSHED -- push when ready;
  that is 8ec2217..44312e4 = FEAT-001/002/003 plus the earlier session's commits
  already counted). Confirm with `git -C <repo> status -sb` next session.
- Branch `feature/l10n-phase2-slice1` is fully merged (same tip as main). It can
  be deleted: `git -C <repo> branch -d feature/l10n-phase2-slice1`.
- Worktree `.worktrees\l10n-phase2-slice1`: the git registration + merge are
  done, but the on-disk directory removal kept hitting terminal dead-episodes and
  may still be present. LOOSE END -- remove it when convenient:
  `git -C <repo> worktree remove .worktrees/l10n-phase2-slice1 --force`
  (or delete the folder in Explorer, then `git worktree prune`).
- Do NOT confuse with the OLD pre-reorg branch `feature/localization-phase2`
  (`d666b59`) -- stale, unrelated.

## Gate status

OWNER-run full gate 2026-10-21 16:08 from the worktree: CLEAN -- 9885 run,
9885 passed, 0 failed (fmt ok, clippy clean, build ok, app-build ok). The +13
over the 9872 baseline are FEAT-002 (5 i18n_keys tests) + FEAT-003 help tests.
No gate re-run is needed unless main is touched again.

## What Slice 1 delivered

- FEAT-001 (2c748ed): wired `ff-i18n` into `ff-desktop` (shell/i18n_bootstrap.rs
  installs the English catalogue at startup + registers the ui.locale hot-reload
  callback -- the first production caller of set_catalogue); 13 English menu keys
  in crates/ff-i18n/i18n/en/menus.ftl; closed task-4 bookkeeping.
- FEAT-002 (ee7c7bf): menu TITLE + option DESCRIPTIONS resolve through t() at
  render time, keyed by menu+command, with English fallback; command values stay
  unlocalised; logic in new menu_workspace/i18n_keys.rs (render.rs only swaps
  call sites). Req 7.1-7.4, 4.2.
- FEAT-003 (44312e4): ff-help content_loader resolves help/<locale>/ then English
  help/; registry gained additive replace_file_topics for the ui.locale
  hot-reload; ff-desktop help wiring passes the active locale. Topic_Key /
  cross-ref keys stay unlocalised. Req 6.1-6.4.
- Tasks 4, 5, 6 ticked in docs/specs/localization/tasks.md.

## What is NEXT -- Slice 2 and beyond (tasks.md Phase 2)

Each is its OWN slice, SAME compile-only verification policy (agents prove
compile-clean: scoped cargo check + clippy -D warnings + fmt --check, reading the
log; the OWNER runs the full gate manually -- NEVER have an agent run cargo
test/nextest/gate/full-gate, that wedges the terminal).

- Task 7: convert the clean dialog/panel-crate render literals to t("key")
  lookups -- one crate per step: ff-catalog-dialog, ff-dataset-alloc-dialog,
  ff-files-panel, ff-explorer-view, ff-theme-editor, ff-toolchain-panel. Keep
  each crate's egui_kittest behaviour tests green (behaviour-neutral at
  ui.locale=en). Add the English keys to crates/ff-i18n/i18n/en/ as data.
- Task 8: convert the ff-desktop render literals (labels/buttons/hover text).
- Task 9: convert the hard status/error messages (argument-bearing via t_args;
  embedded command verbs stay English as non-translated placeables).
- Task 10 (LATER capstone): author other-language DATA (fr/de .ftl catalogues +
  per-environment alias catalogues + per-locale help dirs). NOT until the
  extraction (tasks 7-9) is substantially complete.

## Known workflow caveat (recurring this project)

Workflow coder steps reliably do the WORK and compile clean, but have TWICE hung
at the step's final verify/commit/SIGNAL boundary (FEAT-001 and FEAT-003), not
advancing even after nudge messages. Recovery that worked both times: confirm the
edits compile (owner ran cargo check/clippy/build in the worktree), commit the
agent's staged work by hand, abort the stuck workflow, then ff-merge. If a slice-2
workflow hangs the same way, use the same recovery -- the work is on disk, just
uncommitted. (The per-step commit-after-each-FEAT structure makes this safe.)

## After L10N.4 -- ROADMAP Section 6a forward path

Volume UI (step 2), Volume-in-POM (3), Catalog UI rework (4), ff-vfs
volume/catalog wiring (5), File Navigator dataset/volume duality (6), editor
windowed-record F3-F5 (7, parallel), converging on the CORE #4 editor test pass.
