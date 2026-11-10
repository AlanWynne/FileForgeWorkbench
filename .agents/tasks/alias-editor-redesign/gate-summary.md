# Gate 2 summary -- ISPF-style Alias Editor redesign (CR-CH-068)

DESIGN-FIRST requirements gate, DOCS ONLY. No source code was written or changed.
All authored docs are plain ASCII. CR id RE-VERIFIED against change-log.md: the
highest allocated CR-CH was CR-CH-067, so this gate allocates CR-CH-068.

Gate 1 = CR-CH-067 (the consumed `EnvironmentRegistry` root-enumeration API). The
ACTUAL implemented signatures (verified in source,
`crates/ff-desktop/src/shell/environment_registry.rs`) are ASSOCIATED functions,
NOT `&self` methods:

    pub(crate) fn root_commands(env_name: &str) -> Vec<String>
    pub(crate) fn contains_root(env_name: &str, root: &str) -> bool

Called as `EnvironmentRegistry::root_commands("FFCMD")` /
`EnvironmentRegistry::contains_root("FFEDIT", "FIND")`. This corrects the Gate 1
`&self` doc drift (carry-over note a).

## Docs produced / edited

1. `docs/status/change-log.md` -- NEW entry CR-CH-068 (inserted above CR-CH-067 in
   the Change Requests section). Describes the ISPF redesign, the column<->model
   mapping, the consumed CR-CH-067 API, the framework-conformance stance, the four
   open decisions, and the superseded CR-NR-113 editor + resolved Add-visibility
   bug.
2. `docs/specs/command-configurator/requirements.md` -- REVISED Requirement 5: a
   revision note listing which 5.1-5.12 decisions REMAIN; inline supersede notes on
   5.3; and NEW EARS criteria 5.13-5.24 (columns, Command line, filter row, I/R/D
   line commands, empty-store usability, cascading CE->Root dropdowns, SAVE
   validation reuse, scrollbar, line-command focus reconciliation, primary-vs-line
   ordering, mandatory first-Tab focus, model+framework unchanged).
3. `docs/specs/command-configurator/design.md` -- NEW Section 9.10 (9.10.0-9.10.12):
   ISPF layout, column<->`UserAliasRowToml` mapping, Command line, I/R/D batch apply
   model (descending-index), cascading CE->Root dropdowns on `root_commands`
   (associated-fn), empty-store affordance, filter mechanism, the FOCUS/Tab
   reconciliation (the hard problem) IN DETAIL, testability, both Gate 1 carry-over
   notes resolved, what is NOT changing, and the framework-conformance statement.
4. `docs/specs/command-configurator/tasks.md` -- NEW Task 8 (8.1-8.9), TDD-first,
   each subtask cross-referencing its Req 5.x. Includes the MANDATORY full-shell
   first-Tab egui_kittest test (8.7.2) and the MANDATORY empty-store `I`-insert
   regression test (8.8.1).
5. `docs/quality/TCR.md` -- NEW Phase (alias-editor-redesign) CR-CH-068 block with
   one NOT-COVERED (red) row per new criterion Req 5.13-5.24.
6. `docs/project-management/ROADMAP.md` -- NEW CR-CH-068 bullet under the CR-CH-067
   entry.
7. `.agents/tasks/alias-editor-redesign/gate-summary.md` -- this file.

## Carry-forward resolved

- Add-visibility bug (add-row-bug-findings.md): RESOLVED BY REDESIGN, not separately
  patched. The footer Add button is superseded by the ISPF `I` line command; the
  empty-store case always renders the Command line + headings + FILTER row + an
  insertable row (design 9.10.6, Req 5.17), with a mandatory regression test
  (task 8.8.1).
- Gate 1 note (a) &self-vs-associated-fn doc drift: corrected to the actual
  associated-fn signatures throughout (design 9.10.5, 9.10.10a).
- Gate 1 note (b) per-environment no-shadow: DECISION recorded (design 9.10.10b) --
  KEEP the `builtin_ffcmd_roots()`-based no-shadow guard for CR-CH-068; defer full
  per-environment no-shadow to a later follow-up. Parked for owner confirmation
  (open decision 4).

## OPEN DECISIONS -- ALL CONFIRMED BY OWNER (CR-CH-068 approval)

The owner APPROVED the gate as authored and accepted all five recommended
decisions. Recorded here and folded into the criteria prose (Req 5.14/5.15/5.18
RECOMMENDED qualifiers dropped):

1. R (Repeat) semantics -- CONFIRMED: R duplicates the current row's field values
   into a NEW editable row inserted after it (design 9.10.4, Req 5.16).
2. Filter application + scope -- CONFIRMED: applied on the `FILTER` command / Enter
   (ISPF fidelity), NOT live-as-you-type; filter scope CE + Root ONLY (not ALIAS /
   Parameters). (Req 5.15)
3. Primary commands on the `Command ===>` line -- CONFIRMED: v1 = SAVE, CANCEL/END,
   FILTER, RESET; locale switch OUT of v1. (Req 5.14)
4. Per-environment no-shadow timing -- CONFIRMED: DEFERRED to a later follow-up;
   v1 keeps the current `builtin_ffcmd_roots()` guard, and the CR-CH-067
   `contains_root` save-side hook enforces root-in-declared-environment validity.
   (design 9.10.10b)
5. CE dropdown membership -- CONFIRMED: v1 CE dropdown lists the two
   user-addressable alias environments `AliasEnvironment::ALL` (`ffcmd` / `ffedit`)
   only; full EnvironmentRegistry enumeration is a deferred, purely additive
   follow-up (no model or framework change). (Req 5.18)

Also approved as part of this gate: the L18N round-trip fix (persist
`localisation_key` instead of hardcoded `String::new()` in
`validate_user_alias_rows`). The three remaining editorial NITs are non-blocking.

## Framework-conformance

Builds ON the framework: one dispatch front door (`ALIASES` ->
`builtin_workspace_target_for` -> `custom("aliases")` -> `open_alias_editor`,
unchanged), one navigation stack, one `WorkspaceContext` focus-latch path, one
persistence model (`CustomWorkspace { CommandConfigurator }` + `editor = "aliases"`,
no new WorkspaceKind), code-only built-ins, and the UNCHANGED on-disk
`UserAliasRowToml` shape. Consumes the CR-CH-067 additive read-only API. Reshapes
NO core framework type. NOT a framework change -- a UI/interaction redesign of one
Context. The five open decisions are UI/scope choices, not framework items.

## Gate 2 review iteration 2 -- findings resolved

The Gate 2 design review (`design-review.md`, verdict CHANGES_REQUESTED: 2 MEDIUM
+ 2 NIT) was addressed docs-only this iteration:

- Finding 1 (MEDIUM, L18N save-path contradiction) -- RESOLVED by making the L18N
  column genuinely round-trip (review option (a)). The on-disk `UserAliasRowToml`
  ALREADY carries `localisation_key`, so persisting an edited L18N value is NOT an
  on-disk-shape change (Req 5.24 preserved). The save path
  `validate_user_alias_rows` (`crates/ff-desktop/src/shell/alias_editor.rs`) today
  hardcodes `localisation_key: String::new()`; task 8.9 now has a subtask to carry
  `row.localisation_key.trim().to_string()` plus a round-trip unit test. Req 5.13
  made explicit that L18N persists via `localisation_key`; design 9.10.2 now
  states the round-trip choice unambiguously. This matches the owner's mock (an
  `en` value typed in the L18N column).
- Finding 2 (MEDIUM, ambiguous first-interior target) -- RESOLVED: Req 5.23 now
  names EXACTLY ONE first interior control (the filter CE dropdown, `response.id`
  captured fresh each frame), matching design 9.10.8 part 3, so the mandatory
  first-Tab egui_kittest asserts a single deterministic `egui::Id`. Tasks 8.7.1 /
  8.7.2 already target the filter CE combo.
- Finding 3 (NIT, CE membership vs "registered command environments") -- recorded
  as OPEN DECISION 5 above and noted inline on Req 5.18.
- Finding 4 (NIT, Tab-order prose) -- Req 5.21(c) reworded to separate the FILTER
  row field set (filter CE -> filter Root) from each data row's field set.

No framework change was required by any finding; the on-disk shape, dispatch front
door, WorkspaceKind, focus contract, and v1 user-store-only scope are all
preserved.

## Status

GATE APPROVED BY OWNER. Docs-first; no source changed. All five open decisions
CONFIRMED (and folded into Req 5.14/5.15/5.18); the L18N round-trip fix approved;
the three editorial NITs non-blocking. Task 8 implementation (TDD, compile-verify
by the agent; full suite/gate by the owner) now proceeds as a SEPARATE Gate 3 run.
