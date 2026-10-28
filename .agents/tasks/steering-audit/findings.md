# Steering Rule-Set Audit -- FileForgeWorkbench

READ-ONLY correctness / consistency / redundancy sweep of both steering tiers.
NO steering file was edited. All findings below are for the owner to action.

Audited 2024 session. Scope: GLOBAL (`C:\Users\A142367\.kiro\steering\`, 6 files)
and WORKSPACE (`c:\workspace\apps\FileForgeWorkbench\.kiro\steering\`, 9 files)
plus the pointer `c:\workspace\apps\FileForgeWorkbench\.amazonq\rules\steering-pointer.md`.

Each finding is tagged **[GLOBAL]** (higher blast radius -- affects every project
on this machine) or **[WORKSPACE]** (this repo only).

---

## Summary (answer first)

The rule set is in good shape. The charset discipline is clean (0 violations in
either tier and in the pointer), the `_ignore/` convention landed correctly in
the GLOBAL `tooling.md` / `terminal-discipline.md` / `testing.md` (0 lingering
`tools/logs` in the global tier), and the `plugin`->`extension` rename is
correctly scoped as gradual/in-place (the two `plugin-*` spec folders still exist
on disk under their old names, so `specs.md` referencing them is NOT stale yet).

The material issues are concentrated in TWO places:

1. **CORRECTNESS -- the `_ignore/` migration was not applied to
   `workflow-orchestration.md` or to `.gitignore`.** `workflow-orchestration.md`
   still writes logs to `tools/logs/` in four places and asserts "`tools/logs/`
   is git-ignored"; `.gitignore` carries a self-contradicting comment+entry pair
   (says tools/logs was relocated, then still ignores it) and points at a
   non-existent rule file. These are the only places the recent `_ignore/` change
   is factually incomplete.

2. **STALE -- `specs.md` and the pointer file carry superseded references.**
   `specs.md` points at `documentation.md` (the file is `documentation-charset.md`),
   describes Catalogs/Files as live POM options (removed by CR-CH-073), and omits
   the on-disk `extension-integration` spec folder. The pointer lists a
   `shell-exit-code.md` universal rule that does not exist in the global tier.

Lower-severity consistency/redundancy items (log-capture duplication, the
task-format duplication between `task-format.md` and `specs.md`, the
command-dispatch overlap between `wiring-standard.md` and `framework-conformance.md`)
are all currently COMPLEMENTARY, not contradictory -- noted below with a
consolidation recommendation, but none is urgent.

---

## CORRECTNESS (now factually wrong given the recent changes)

### C1 [WORKSPACE] `workflow-orchestration.md` still uses `tools/logs/` -- 4 occurrences
File: `.kiro/steering/workflow-orchestration.md`

- Line ~31: "writing the full report to `tools/logs/liveness-probe.txt`"
- Line ~49: heartbeat path "`tools/logs/<run-label>-progress.txt`"
- Line ~89: "append a timestamped progress line to `tools/logs/<run-label>-progress.txt`"
- Line ~92: "Keep these logs ephemeral (`tools/logs/` is git-ignored)."

The recent `_ignore/` convention (now documented in GLOBAL `tooling.md`) states
ALL non-committed data -- including script/report logs -- lives under `_ignore/`,
and that script logs go to `_ignore/logs/`. These four references are the single
largest lingering `tools/logs` cluster in steering. The phrase "`tools/logs/` is
git-ignored" is the specific phrasing the audit brief called out: it is only
still true because of the stale `.gitignore` leftover (finding C3), and it
contradicts the convention that new logs belong in `_ignore/logs/`.

Recommended fix (do not apply): change all four to `_ignore/logs/...`, and change
"`tools/logs/` is git-ignored" to "`_ignore/logs/` is git-ignored (the whole
`_ignore/` tree)". Note this file is NOT a tool itself -- it also drives the
`liveness-probe.ps1` output path, so if the probe script writes to `tools/logs`
the SCRIPT path should be reconciled too (script out of audit scope; flag only).

Evidence: `grep_search 'tools[/\\]logs'` over `.kiro/steering/**` returned exactly
these matches; the GLOBAL tier returned 0 (`Select-String ... 'tools.{0,2}logs'`
count = 0), confirming the lingering usage is isolated to this one workspace file.

### C2 [WORKSPACE] `specs.md` cross-reference to `documentation.md` is broken
File: `.kiro/steering/specs.md`, line ~138
Text: "All documentation MUST use plain ASCII (see `documentation.md`)."

There is no `documentation.md` in either tier. The ASCII rule lives in
`documentation-charset.md` (confirmed: global tier file list is exactly
documentation-charset.md, task-format.md, terminal-discipline.md, testing.md,
tooling.md, workflow.md). The pointer file and `rust-standards.md` both correctly
cite `documentation-charset.md`.

Recommended fix: change the reference to `documentation-charset.md`.

### C3 [WORKSPACE] `.gitignore` -- self-contradicting `tools/logs` comment+entry and a broken rule reference
File: `.gitignore` (repo root -- not a steering file, but it is the factual
ground truth the steering rules in C1 depend on, so reported here)

Two problems:
- A comment block says "tools/logs/ has been relocated to _ignore/logs/ (see
  tools/powershell/sweep-to-ignore.ps1)" and the `_ignore/` entry is present --
  GOOD. But a few lines later a SEPARATE active entry still ignores `tools/logs/`
  under the header "Script execution logs (ephemeral, see
  `.amazonq/rules/script-output.md`)". Keeping the `tools/logs/` entry is what
  keeps C1's stale "is git-ignored" claim technically true and masks the
  incomplete migration.
- That header references `.amazonq/rules/script-output.md`, which does NOT exist
  (`Test-Path` = False; `.amazonq/rules/` contains only `steering-pointer.md`).

Recommended fix: once C1 is applied (logs move to `_ignore/logs/`), drop the
dedicated `tools/logs/` entry (the `_ignore/` entry already covers the new
location), or keep a short transition note; and fix/remove the dead
`.amazonq/rules/script-output.md` reference (the live rule is GLOBAL `tooling.md`
"Script output -- MANDATORY log capture"). This is a judgement call for the owner
-- flagged, not prescribed.

---

## CONSISTENCY / CONTRADICTION

### N1 [GLOBAL vs WORKSPACE] workflow.md gate vs specs.md gate -- COMPLEMENTARY (no contradiction)
GLOBAL `workflow.md` section 3 defines the universal 7-step requirements gate in
the abstract ("IDENTIFY the correct spec area"). WORKSPACE `specs.md` supplies
the concrete spec-folder layout (`docs/specs/<sub-project>/` with
requirements/design/tasks) and the sub-project list the gate operates over. These
are correctly layered: the global rule stays project-agnostic and the workspace
rule fills in the specifics. The "workspace-level rules take precedence" convention
resolves the one overlap (the task-checkbox format, see R2) cleanly. No change
needed -- recorded to confirm the overlap was checked.

### N2 [WORKSPACE] gate-policy.md vs testing.md cadence -- COMPLEMENTARY (no contradiction)
`gate-policy.md` explicitly declares it REFINES the GLOBAL `testing.md` execution
split and governs CADENCE for this repo, deferring everything else to the global
file. The two agree: agents compile-verify with the scoped trio; the owner runs
the full gate. No contradiction. `gate-policy.md` is correctly the more-specific
workspace rule.

### N3 [WORKSPACE] wiring-standard.md vs framework-conformance.md command dispatch -- CONSISTENT, slight terminology drift
Both describe the single command-dispatch seam. They do not contradict, but they
name the entry point differently:
- `framework-conformance.md` mechanism 1: resolves through
  `ff_command::resolve_target` -> `shell/target_dispatch.rs`.
- `wiring-standard.md` "Command dispatch is unified (B080)": front door is
  `shell/dispatch.rs::dispatch_command_string` -> `resolve_target` ->
  `run_command_ladder`, via `command_config/mod.rs::builtin_workspace_target_for`.

These are two layers of the same path (dispatch.rs calls resolve_target), so they
are complementary, but a reader could think there are two seams. `wiring-standard.md`
already says "the more specific rule wins; this file is the integration recipe".

Recommended fix (optional): add a one-line cross-reference in
`framework-conformance.md` mechanism 1 pointing to `wiring-standard.md` "Command
dispatch is unified" for the concrete call chain, so the two stay in sync when the
dispatch code evolves. No contradiction to resolve today.

---

## REDUNDANCY (duplication that could be consolidated or cross-referenced)

### R1 [WORKSPACE within-file] workspace-conformance.md repeats "The test" section twice
File: `.kiro/steering/workspace-conformance.md`
The section "## The test -- every workspace MUST prove it" appears once with the
full `egui_kittest` code sample, then a second near-duplicate "## The test --
every workspace MUST prove it (unchanged)" lower down that re-states the same
requirement and points back up ("See the 'The test' section below" -- which also
mis-points, since the referenced block is ABOVE it). This is internal redundancy
plus a wrong directional pointer.

Recommended fix: keep the first (detailed) "The test" section as the owner; reduce
the second to a one-line pointer, and fix "below" -> "above".

### R2 [WORKSPACE duplicates GLOBAL] task-format.md checkbox rules are re-embedded verbatim in specs.md
GLOBAL `task-format.md` OWNS the `[ ]`/`[x]`-only + title + status-on-sub-bullet
rules. WORKSPACE `specs.md` re-states substantially the same rules in its
"## Task File Format" section (allowed markers table, title rule, intermediate
states, summary checklist).

Tier split constraint: the GLOBAL file must stay self-contained for other
projects, so it keeps its copy. The WORKSPACE copy is the redundant one.
Recommended fix: `specs.md` should OWN only the spec-specific bit (tasks live
under `docs/specs/<sub-project>/tasks.md`) and CROSS-REFERENCE GLOBAL
`task-format.md` for the checkbox/title mechanics, rather than duplicating them.
(Workspace may reference global; global must not reference workspace -- respected.)

### R3 [GLOBAL within/across] log-capture discipline stated in both tooling.md and terminal-discipline.md
The "write to a log, read it back, verify by effect, never trust Exit Code"
discipline appears in GLOBAL `tooling.md` ("Script output -- MANDATORY log
capture") and GLOBAL `terminal-discipline.md` (rule 5 / rule 6). This is
deliberate and already cross-referenced ("See `tooling.md` for the mandatory
stdout-capture discipline"), so it is low-priority. Both are GLOBAL so the tier
split is not at risk.

Recommended fix (optional): no change needed; the cross-reference already exists.
If trimming, let `tooling.md` OWN the log-capture pattern detail and have
`terminal-discipline.md` rule 6 keep only the pointer it already has.

---

## STALE / NIT

### S1 [WORKSPACE] steering-pointer.md lists a non-existent `shell-exit-code.md`
File: `.amazonq/rules/steering-pointer.md`, universal-rules table
The table lists a 7th universal rule: "Shell exit codes | `shell-exit-code.md` |
Trust the sentinel, not the reported `Exit Code`". The global steering dir
contains NO such file (verified file list: documentation-charset.md,
task-format.md, terminal-discipline.md, testing.md, tooling.md, workflow.md). The
exit-code guidance now lives inside `terminal-discipline.md` (rule 5, "Verify by
EFFECT, not by the reported exit code").

Also note the pointer's exit-code row advises "Trust the sentinel" whereas the
current `terminal-discipline.md` rule 3/5 now says the `;`-sentinel is BLOCKED and
you must rely on the log+effect instead -- so the row is doubly stale (names a
missing file AND stale advice).

Recommended fix: remove the `shell-exit-code.md` row (or repoint it to
`terminal-discipline.md`) and drop the "trust the sentinel" wording.

The pointer's other references check out: `c:\workspace\_steering\02-TIERING-STRATEGY.md`
and `c:\workspace\_steering\user-space\steering` both exist (Test-Path = True).

### S2 [WORKSPACE] specs.md describes Catalogs/Files as live POM options (pre-CR-CH-073)
File: `.kiro/steering/specs.md`, "Context names by Workspace kind"
- "Catalog Explorer Context -- POM option 1, virtual catalog management"
- "File Explorer Context -- POM option 2, VFS tree browser"

CR-CH-073 (DONE) trimmed the bare-core POM to exactly 0 Settings / 3 Help / X and
REMOVED the Catalogs/Files menu options (their commands+Contexts remain
reachable-if-typed, Option B). Describing them as "POM option 1/2" is now stale --
they are reachable Contexts but no longer POM options.

Recommended fix: drop the "POM option 1" / "POM option 2" qualifiers (keep the
Context descriptions), and optionally note they are typed-command reachable per
CR-CH-073. Low blast radius -- descriptive prose only.

### S3 [WORKSPACE] specs.md sub-project list omits the on-disk `extension-integration` folder
File: `.kiro/steering/specs.md`, sub-project list
`docs/specs/extension-integration/` exists on disk (verified) but is not in the
list. Given the new extension work (CR-CH-074) this is the folder most likely to
be consulted. (The list also still includes `plugin-architecture` and
`plugin-manager-ui`, which is CORRECT -- those folders still exist under those
names; see note below.)

Recommended fix: add an `extension-integration` entry to the sub-project list.

### S4 [WORKSPACE] "plugin" in prose -- mostly NOT actionable yet; one genuinely stale item
Per the extension-terminology rule the rename is gradual/in-place, so a reference
to the still-named `plugin-architecture` / `plugin-manager-ui` spec folders is
NOT stale (folders verified present under those names). The remaining "plugin"
occurrences in steering and whether to touch them:
- `extension-terminology.md`: "plugin" appears as the SUBJECT of the rule (the
  naming map, "formerly called plugin"). CORRECT -- must stay.
- `specs.md`: `plugin-architecture`, `plugin-manager-ui` (folder names -- correct
  until renamed), "Plugin Manager Context", and "record-structure View/Edit
  plugin" (CR-NR-061 description). These are references to not-yet-renamed
  entities -- leave until the gradual rename reaches them, EXCEPT consider whether
  "Plugin Manager Context" should track the UI rename to "Extension Manager".
- `workspace-conformance.md` line ~82 and `wiring-standard.md` line ~17-19: both
  list "Plugin Manager" among migrated/mirrored Contexts. These are PROSE in the
  steering rules describing the shell, and the UI is being renamed to "Extension
  Manager" -- these are the prose spots the audit brief says SHOULD update when
  the code Context is renamed. Flag as "update in lockstep with the
  Plugin Manager -> Extension Manager code rename", not an immediate edit.

No GLOBAL file contains "plugin" (verified count = 0) -- correct, since the term
is FFWB-specific and global rules are universal.

### S5 [WORKSPACE] recent CRs not reflected where the framework list is authoritative
`framework-conformance.md` enumerates the load-bearing mechanisms with their CR
tags but does not mention CR-CH-074's two additive extension points (the
runtime-extensible root registry/classifier and the single `TabKind::Extension`
dispatch arm). Since CR-CH-074 is APPROVED and deliberately ADDITIVE to mechanism
1 (command dispatch) and the TabKind set, the framework list will be incomplete
once that lands.

Recommended fix (when CR-CH-074 lands, not now): add a note under mechanism 1 /
the TabKind discussion that the extension system plugs in via the approved
registry/classifier extension point and the single `TabKind::Extension` arm, so a
future reader does not mistake it for a bypass of the single-dispatch rule. Flag
only -- this is forward-looking.

---

## Items checked and found CLEAN (no action)

- **Charset / ASCII (documentation-charset.md enforcement)**: ran the file's own
  pattern `[^\x00-\x7F\u2500-\u257F]` over both steering dirs and the pointer.
  Match counts: GLOBAL = 0, WORKSPACE = 0, pointer = 0. No non-ASCII outside the
  allowed box-drawing range anywhere.
- **GLOBAL tier `tools/logs`**: 0 occurrences -- `tooling.md`,
  `terminal-discipline.md`, `testing.md` all correctly use `_ignore/logs`. The
  `_ignore/` migration of the GLOBAL rules is complete.
- **GLOBAL tier "plugin"**: 0 occurrences -- correctly kept FFWB-specific.
- **Pointer tiering references**: `02-TIERING-STRATEGY.md` and
  `user-space/steering` both exist.
- **extension-terminology.md**: internally consistent; naming map is complete and
  the "gradual, no per-rename confirmation" policy matches the owner's stated
  intent (user messages 2-3, 10-11).

---

## Recommendations, ranked for the owner

1. (C1+C3) Fix the `_ignore/` migration gap: repoint `workflow-orchestration.md`'s
   four `tools/logs` references to `_ignore/logs`, and reconcile the `.gitignore`
   leftover entry + dead `script-output.md` reference. This is the only
   factually-wrong-given-recent-changes cluster. [WORKSPACE]
2. (C2, S1) Fix the two broken cross-references: `specs.md` -> `documentation.md`
   should be `documentation-charset.md`; pointer's `shell-exit-code.md` row should
   be removed/repointed to `terminal-discipline.md`. [WORKSPACE]
3. (S2, S3) Refresh `specs.md` prose for CR-CH-073 (drop "POM option 1/2") and add
   the `extension-integration` sub-project. [WORKSPACE]
4. (R1) De-duplicate the twin "The test" section in `workspace-conformance.md` and
   fix its "below" pointer. [WORKSPACE]
5. (R2) Make `specs.md` cross-reference GLOBAL `task-format.md` instead of
   duplicating the checkbox rules. [WORKSPACE]
6. (N3, R3, S4, S5) Optional / forward-looking: tighten the dispatch cross-ref,
   leave the log-capture cross-ref as-is, and update "Plugin Manager" prose +
   add the CR-CH-074 extension-point note when those code changes land.

No GLOBAL steering file requires a correctness edit; all actionable items are
WORKSPACE-tier (plus the repo-root `.gitignore`), which keeps the blast radius to
this project.

---

## Verification method note

All global-dir searches were run via the clean pwsh7 wrapper
(`-NoProfile -NonInteractive`), one command per invocation (no `;` chaining -- the
guard hook blocked two early `;`-chained attempts, confirming the discipline),
redirected to `_ignore/logs/*.log` and read back. The reported `Exit Code: -1` on
every pwsh call is the documented capture artifact; results were confirmed by
EFFECT (reading each log). Match-count probes (`@(Select-String ...).Count`) were
used to distinguish a genuine zero-match from a swallowed write, since the write
mechanism was independently proven working by the non-empty logs in the same
batches.
