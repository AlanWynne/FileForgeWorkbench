# Design Review -- ISPF-style Alias Editor redesign (CR-CH-068, Gate 2)

Reviewer: design-review subagent (fresh context). Docs-only design gate; no
build/test run, no doc/source edited. Verification used dedicated read/search
tools against the actual source, not the design's word.

## Documents reviewed

- docs/specs/command-configurator/requirements.md (revised Requirement 5 + new
  EARS criteria 5.13-5.24)
- docs/specs/command-configurator/design.md (Section 9.10)
- docs/specs/command-configurator/tasks.md (Task 8)
- docs/status/change-log.md (CR-CH-068 entry)
- docs/quality/TCR.md (CR-CH-068 block)
- docs/project-management/ROADMAP.md (CR-CH-068 bullet)
- .agents/tasks/alias-editor-redesign/gate-summary.md

## Cross-checks performed against source and sibling specs

- crates/ff-desktop/src/shell/environment_registry.rs -- the ACTUAL CR-CH-067
  signatures.
- crates/ff-desktop/src/shell/alias_editor.rs -- `validate_user_alias_rows`
  (the save path, the `localisation_key` hardcoding, the existing `contains_root`
  hook).
- crates/ff-desktop/src/alias_editor_panel/state.rs and render.rs -- the current
  `AliasRowEdit` / `AliasEditorState` shape and `AliasEnvironment::ALL`.
- crates/ff-desktop/src/shell/user_aliases.rs -- `AliasEnvironment`,
  `env_name()`, `UserAliasRowToml`.
- docs/specs/command-environments/requirements.md Requirement 20 (CR-CH-067).
- workspace-conformance.md (WorkspaceContext / InteriorFocus focus contract).

---

## Findings

No HIGH or MEDIUM findings. All findings below are NIT.

1. NIT -- Task 8.9 subtask ordering is non-monotonic.
   WHERE: docs/specs/command-configurator/tasks.md, Task 8.9. The subtasks are
   listed 8.9.1, 8.9.4, 8.9.5, 8.9.2, 8.9.3 (8.9.4/8.9.5 inserted ahead of
   8.9.2/8.9.3 when the L18N round-trip work was added for review finding 1).
   The content is complete and each line has a title, so this does not affect
   the gate, but the out-of-order numbering reads awkwardly.
   FIX: renumber the save/round-trip subtasks into ascending order, e.g.
   8.9.1 (SAVE-path verification) -> 8.9.2 (L18N round-trip test) -> 8.9.3
   (save-path `localisation_key` change) -> 8.9.4 (TCR update) -> 8.9.5
   (record the five open decisions). Pure editorial.

2. NIT -- Req 5.14 and Req 5.15 bundle "RECOMMENDED, owner to confirm" scope
   into otherwise-testable EARS criteria, while Task 8.3.1 / 8.5.1 already
   assume `FILTER`/`RESET` and applied-on-command filtering.
   WHERE: requirements.md Req 5.14 (the `FILTER <ce> <root>` / `RESET` clause),
   Req 5.15 (filter-applied-on-command clause); tasks.md 8.3.1, 8.5.1. These are
   explicitly parked as OPEN DECISIONS 2 and 3 for the owner CONFIRM step, so the
   ambiguity is deliberate and recorded, not an oversight. The SHALL core of each
   criterion (SAVE + CANCEL/END; filter by CE and/or Root only; empty filters
   list all) is unambiguous and testable on its own.
   FIX: none required for the gate -- the parked decisions are correctly flagged.
   When the owner confirms, fold the decision into the criterion prose and drop
   the "RECOMMENDED" qualifier so the final criterion is a flat SHALL.

3. NIT -- A stale prior-iteration review artifact will be overwritten.
   WHERE: .agents/tasks/alias-editor-redesign/design-review.md (the Gate 2
   iteration-2 review) is referenced by design 9.10.13 and the gate-summary. This
   review replaces it. No action needed; noted so the authors know the
   iteration-2 verdict record is superseded by this one.

---

## Point-by-point verification (the seven mandated checks)

### 1. ISPF Act-field line command vs WorkspaceContext/InteriorFocus/Tab -- CONCRETE

The hard, novel part is reconciled with an actual mechanism, not hand-waving.
Design 9.10.8 gives four concrete parts, each grounded in an EXISTING seam:

- No mid-frame restructuring. A typed Act letter is PENDING state on the row
  (`row.act: String`); the row list is restructured only in the shell apply pass
  AFTER the frame, through the existing owned-panel swap (`mem::take` ->
  `render_workspace_context` -> put back -> `apply_alias_editor_action`). The
  render iterates a stable `rows` for the whole frame, so egui assigns stable ids
  and the Tab ring is well-defined. This is the existing `pending_action`
  discipline extended with new actions, NOT a new mechanism -- verified against
  `AliasEditorState.pending_action` in state.rs.
- Stable ids in visual order: `alias_editor_command_line` -> filter CE combo ->
  filter Root combo -> each data row Act -> CE -> Root -> L18N -> ALIAS ->
  Parameters, with per-row controls salted by the row's BACKING index
  (`("alias_act", idx)` etc.), mirroring the existing `surface_id(i)` pattern.
- First interior control = the FILTER row's filter CE combo, its `response.id`
  captured fresh each frame (the combo-id round-trip workspace-conformance.md
  requires). The `Command ===>` line is the ENTRY control, so the first Tab FROM
  it lands on the filter CE combo -- no phantom stop. Req 5.23 pins exactly ONE
  deterministic target, so the mandatory first-Tab egui_kittest asserts a single
  `egui::Id`.
- Primary-vs-line ordering: Enter emits ApplyLineCommands then CommandLine(text)
  so pending Act codes apply before SAVE (Req 5.22).

The filtered-view index hazard (line commands operate on `rows` by index but the
user sees the filtered subset) is explicitly handled: design 9.10.7 captures each
displayed row's backing `rows` index for the pending-line-command snapshot, and
task 8.5.1 tests that a line command on a filtered-visible row targets the
correct BACKING index. Concrete and correct. PASS.

### 2. CE->Root cascade consumes the REAL CR-CH-067 API; anchor by construction

VERIFIED in crates/ff-desktop/src/shell/environment_registry.rs: the actual
signatures are ASSOCIATED functions keyed by name, not `&self` methods:

    pub(crate) fn root_commands(env_name: &str) -> Vec<String>
    pub(crate) fn contains_root(env_name: &str, root: &str) -> bool

Design 9.10.5, 9.10.9, 9.10.10a, Req 5.18, the change-log, and tasks 8.4.1/8.4.2
all reference the associated-fn form (`EnvironmentRegistry::root_commands(
row.environment.env_name())`), explicitly correcting the Gate 1 `&self` doc
drift. `AliasEnvironment::env_name()` returning `FFCMD_NAME`/`FFEDIT_NAME` and
`AliasEnvironment::ALL` both exist in source (user_aliases.rs, render.rs). The
(environment, root) anchor is enforced BY CONSTRUCTION: the Root combo offers
only `root_commands(CE)` entries, so a chosen pair cannot be one `contains_root`
would reject; on CE change the Root is cleared/revalidated via `contains_root`.
PASS.

### 3. No on-disk-model change, no framework change

- On-disk shape: Req 5.24, design 9.10.11, change-log, TCR Req 5.24 row all
  state the `UserAliasRowToml` shape is unchanged. The L18N persistence (finding
  1 of the prior iteration) is NOT a shape change: `UserAliasRowToml` ALREADY
  carries `localisation_key` (verified in user_aliases.rs and the construction in
  alias_editor.rs). The change is to stop hardcoding `localisation_key:
  String::new()` in `validate_user_alias_rows` (the hardcoding is REAL -- verified
  at the construction site) and carry `row.localisation_key.trim()`. Adding
  `localisation_key` to the in-memory `AliasRowEdit` (task 8.1.1) is a UI-state
  change, not an on-disk change.
- Framework: single dispatch front door (ALIASES -> `builtin_workspace_target_for`
  -> `custom("aliases")` -> `open_alias_editor`), WorkspaceKind::CommandConfigurator
  + `editor = "aliases"`, per-tab Navigation_Stack, single WorkspaceContext
  focus-latch path, session-persistence format, and v1 user-store-only scope are
  all PRESERVED (design 9.10.11/9.10.12, Req 5.24). The `Command ===>` line is the
  editor's OWN primary-command field (Menus Editor pattern), not a second global
  dispatcher. No core framework type is reshaped; CR-CH-067 is consumed as an
  already-approved additive read-only API. Correctly classified as NOT a framework
  change. No framework change sneaks in. PASS.

### 4. EARS criteria testable; no silent contradiction of CR-NR-113

Criteria 5.13-5.24 are WHEN/THE/SHALL form and testable. The revision note at the
head of Req 5 lists the settled CR-NR-113 decisions that REMAIN (5.1 storage
shape, 5.2 dispatch/verb, 5.4 validations, 5.6 live refresh, 5.7 resolver model,
5.8 expansion delegation, 5.9 nav return, 5.10 focus contract, 5.11 v1 scope,
5.12 ASCII posture) and notes each supersession inline: 5.3's footer Add button
-> the `I` line command; the free-text Root -> the dropdown (5.18). The L18N
change is noted as persisting via the pre-existing on-disk field (explicitly NOT
a model change). No settled decision is silently contradicted. PASS.

### 5. Empty-store usability AND mandatory first-Tab criterion both present

- Empty-store: Req 5.17 ("WHEN the User_Alias_Store is absent or empty ... SHALL
  still render the column headings and the FILTER row, AND SHALL present at least
  one empty insertable row ..."), design 9.10.6 (always-present scratch blank row
  promoted into `rows` on edit/`I`), task 8.6.2, and the mandatory regression test
  8.8.1. PRESENT.
- First-Tab: Req 5.23 (MANDATORY full-shell egui_kittest first Tab lands exactly
  on the filter CE dropdown, no phantom stop), design 9.10.8 part 3 / 9.10.9,
  task 8.7.2. PRESENT. PASS.

### 6. Tasks TDD-first; required tests present; CR id; TCR

- TDD-first: tasks 8.1.2, 8.2.1, 8.3.1, 8.4.1, 8.5.1, 8.9.2 all "Write failing
  ... tests" ahead of the implementing subtask. PASS.
- Mandatory full-shell first-Tab egui_kittest: task 8.7.2 (Validates Req 5.23).
  PRESENT.
- Empty-store `I`-insert regression test: task 8.8.1 (Validates Req 5.17) -- the
  exact test the add-row-bug-findings report identified as missing (the file
  exists at .agents/tasks/alias-editor-workspace/add-row-bug-findings.md).
  PRESENT.
- CR id: CR-CH-068 appears ONLY in this gate's docs; the prior highest allocation
  is CR-CH-067. No other spec claims CR-CH-068. No collision. PASS.
- TCR: the CR-CH-068 block has one NOT-COVERED (red) row per new/changed
  criterion Req 5.13-5.24 (13 rows, with 5.13 split into a columns row and an
  L18N-round-trip row). PASS.
- Task-format: every task line has a title; checkboxes are `[ ]`/`[x]` only;
  Validates/Covers cross-refs present. PASS.

### 7. ASCII (documentation-charset.md)

requirements.md, design.md, and tasks.md: grep for any non-ASCII byte returned NO
matches -- plain ASCII. The TCR CR-CH-068 block uses the status emoji
(red/green/check) that documentation-charset.md explicitly permits for a
test-coverage report table. PASS.

---

## Verified Assumptions (checked against source / sibling specs)

- CR-CH-067 API is `pub(crate) fn root_commands(env_name: &str) -> Vec<String>`
  and `pub(crate) fn contains_root(env_name: &str, root: &str) -> bool`,
  associated functions -- matches every design/task reference.
- `validate_user_alias_rows` (alias_editor.rs) today constructs
  `UserAliasRowToml { ..., localisation_key: String::new() }` -- the hardcoding
  the design/task claims is real, so the L18N round-trip fix is correctly scoped.
- `validate_user_alias_rows` ALREADY calls
  `EnvironmentRegistry::contains_root(row.environment.env_name(), &canonical)`
  (check (e)); so Req 5.19's save-side root validity is already in source and
  task 8.9.1 correctly treats it as verification, not new logic.
- `UserAliasRowToml` (user_aliases.rs) carries `localisation_key`, so persisting
  the L18N value is not an on-disk-shape change (Req 5.24 preserved).
- `AliasEnvironment::ALL` and `env_name()` exist -- the v1 CE dropdown membership
  (OPEN DECISION 5) is implementable as stated.
- `AliasEditorState` already has `first_interior_id` / `last_interior_id` /
  `pending_action` -- the focus mechanism builds on existing fields.
- `AliasRowEdit` today has surface / canonical / environment / command_template
  and NO `localisation_key` / `act` -- task 8.1.1 correctly ADDS the transient
  fields.
- command-environments Requirement 20 (20.1-20.8) exists and matches the consumed
  API, including the SAVE-side hook (20.8).
- CR-CH-068 is uniquely allocated; add-row-bug-findings.md exists.

## Unverified / Wrong Assumptions

- None material. The design's assumptions that were checkable were all confirmed
  against source. No wrong assumption found.
- Not independently re-derived (out of scope for a docs gate, not a defect): the
  CR-NR-112 `command_template` expansion (correctly flagged by the design as a
  hard build-order prerequisite not yet in source) and the egui `response.id`
  combo-id round-trip behaviour (relied on per workspace-conformance.md, which is
  the established project convention).

---

## Verdict

HIGH findings: 0. MEDIUM findings: 0. NIT findings: 3.

Per the mechanical rule (HIGH + MEDIUM == 0), the verdict is APPROVED. The three
NITs are editorial and may be folded in during implementation or at the owner
CONFIRM step; none blocks the gate.
