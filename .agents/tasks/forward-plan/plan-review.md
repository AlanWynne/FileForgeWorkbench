# FORWARD-PLAN design review (docs-only)

Reviewer: design-review subagent, fresh read (no authoring context).
Scope: `docs/status/FORWARD-PLAN.md` and its supporting status edits
(`current-work.md`, the eight `RESUME-*.md` banners), cross-checked against the
investigation, `ROADMAP.md`, `core-acceptance-test-plan.md`, the
`extension-integration` spec (requirements + design), and the
`framework-conformance` / `wiring-standard` steering rules.

This was a DOCS-ONLY planning review. No build or test was run (there is no code
to build). File evidence was read directly rather than taken from the plan's or
the investigation's word.

Verdict: CHANGES_REQUESTED (1 MEDIUM, 0 HIGH, 3 NIT).

---

## Findings

### 1. MEDIUM -- Phase 4 and decision 4 misstate the test-plan structure: there is no "Help group"; the Help rows live inside Group 7 "Function keys"

Where: FORWARD-PLAN Phase 4 ("a Help group exists -- rows 7.1 / 7.1a / 7.1b --
but predates the rich renderer") and Resolved decision 4 ("rich-Help rows ...
alongside the existing 7.1/7.1a/7.1b"). The investigation repeats the same claim
in sections (E) and (H).

Problem: I read `core-acceptance-test-plan.md`. Group 7 is titled
"Group 7 -- Function keys (Core: whole function-keys spec)". Rows 7.1 / 7.1a /
7.1b / 7.1c / 7.1d ARE present, but they sit INSIDE the Function-keys group
(because F1 opens Help), not in a dedicated "Help group". There is no group
titled Help anywhere in the document (groups are: 1 Shell, 2 Menu/Settings,
3 File navigator, 4 Editor, 5 Command-line, 6 Macros, 7 Function keys, 8 Theme,
9 Config recovery). So the plan's statement "a Help group exists" is factually
wrong, and it leaves the Phase-4 executor an unresolved choice: add the new
rich-renderer Help rows into Group 7, or stand up a new Help group? Because the
plan asked me specifically to verify the Phase-4 / test-plan reconciliation, this
imprecision in the instrument the phase depends on is a blocking consistency gap,
not a cosmetic one.

Fix: correct the wording in both Phase 4 and decision 4, and make the placement
explicit. Suggested replacement for the Phase 4 bullet:

```
- ADD Help rows for the new rich renderer + the authored core help content. The
  existing Help acceptance rows (7.1 / 7.1a / 7.1b / 7.1c / 7.1d) currently live
  INSIDE Group 7 "Function keys" (F1 opens Help); there is no standalone Help
  group today. Decide and state whether the new rich-renderer rows extend Group 7
  in place or a new "Help" group is split out, and record that choice here.
```

Apply the same correction to decision 4 ("alongside the existing 7.1/7.1a/7.1b
rows in Group 7 (Function keys)").

### 2. NIT -- "two additive framework points" (plan/requirements) vs "one additive generalization" (extension-integration design.md)

Where: FORWARD-PLAN CURRENT REALITY and Phase 5+ both describe CR-CH-074 as TWO
additive framework extension points (runtime-extensible root registry; single
`TabKind::Extension { id }` dispatch arm). This matches
`extension-integration/requirements.md` Req 10.3 (which enumerates both a and b)
and the investigation's CR-CH-074 record.

Observation: `extension-integration/design.md` (lines 23-27) still says the
design "requires exactly one additive core generalization -- a single
identifier-keyed extension-Context dispatch arm". That is the design doc lagging
the requirements, NOT a defect in the FORWARD-PLAN -- the plan correctly follows
the authoritative requirements and CR-CH-074. No change to the plan is required.
Flagged only so the owner knows the design.md "one additive" sentence is the doc
that is behind; correcting design.md is out of scope for this plan.

### 3. NIT -- SCRM renderer reuse claim is accurate but slightly narrow

Where: FORWARD-PLAN decision 2 / Phase 3 ("adopt `egui_commonmark` (the in-tree
engine, used by the ffmdx app)").

Observation: verified against the tree -- `egui_commonmark = "0.22"` is a
workspace dependency and `crates/ff-mdx-app/src/viewer.rs` uses
`CommonMarkViewer`. The claim is grounded. For completeness, the change-log notes
the same engine also backs the in-shell Markdown viewer path
(`ff-mdx-plugin` / `ff-viewers`), so the engine is reused more widely than "the
ffmdx app" alone. Not wrong, just narrower than the full picture. No change
required; optionally broaden to "used by the ffmdx app and the in-shell Markdown
viewer".

### 4. NIT -- SCRM surfacing placement: plan is more specific than the design, which is fine but worth a one-line cross-reference

Where: FORWARD-PLAN Phase 2 / decision 1 resolves SCRM surfacing as a MENU-BAR
ENTRY, explicitly not a bare-core POM option and not typed-only.

Observation: `extension-integration/design.md` D4 says the owner wants SCRM
surfaced "via a menu option AND the menu bar". The design does not bind the exact
placement (it defers the wiring to "a separate core effort"), so the plan's
narrower, owner-confirmed decision (menu-bar entry, POM untouched) does NOT
contradict it. The wiring mechanism the plan specifies (menu-bar entry whose
`command` value is the existing SNAPSHOT / CAPTURE verb, routed through
`resolve_target` -> `dispatch_command_target`, content in `menus/*.toml` /
`DEFAULT_*_TOML`, no new verb or dispatcher) is exactly what `wiring-standard.md`
("Menu Integration" / "Toolbar Integration") and `framework-conformance.md`
(mechanisms 1, 2, 4) require. No change required; optionally add "(design.md D4
leaves placement to this core effort)" so a future reader sees the two are
reconciled, not at odds.

---

## Verified assumptions (checked against source, held true)

- current-work.md IS a thin pointer. It names FORWARD-PLAN.md as the single
  read-first document, states the five-phase sequence, points at ROADMAP (control
  tower) and core-acceptance-test-plan (sign-off instrument), and confirms the
  stale phase-ledger tables were removed. Matches the plan's claim.
- All EIGHT `RESUME-*.md` in `docs/status/` carry a top-of-file subordinate
  banner (`> STATUS: ...`) that states real status and points at
  `docs/status/FORWARD-PLAN.md`: l10n-phase2 (ACTIVE), volume-model
  (ACTIVE-but-DEFERRED), scrm-and-ff-desktop-refactor (DONE), ffdesktop-
  simplification (DONE), decomp-wave6 (DONE), decomp-wave7 (DONE/SUPERSEDED),
  dataset-rationalisation (DONE), menus-editor-typing-bug (DONE pending
  close-out). The banners' statuses agree with the investigation's RESUME-file
  inventory (B). FORWARD-PLAN genuinely works as the single entry point.
- ROADMAP Section 6 DOES point at FORWARD-PLAN. Section 6 carries a "FIRST-READ
  POINTER" paragraph naming `docs/status/FORWARD-PLAN.md` as the active-phase
  ordering, deferring finalise-core detail to Section 6a, and preserving the
  control-tower role -- exactly the reconciliation the plan's "Authority"
  section and decision 5 claim. No silent contradiction: the ROADMAP's CORE-
  before-extension split, six core capabilities, and Section 5 plugin order are
  untouched and the plan explicitly defers to them.
- The finalise-core task detail (localisation -> Volume UI -> catalog/navigator
  rework -> editor F3-F5 -> CORE #4 editor test pass) lives in ROADMAP Section 6a
  (steps 0-8). FORWARD-PLAN Phase 1 correctly DEFERS to 6a and does not restate
  the ordering, matching its stated authority split.
- SCRM is BUILT and core-wired, not greenfield. The investigation's file
  citations are consistent with the plan: `ff-scrm` + `ff-screen-model` crates,
  SNAPSHOT / CAPTURE verbs on the single front door, `[REPLAY]` WorkspaceContext.
  ROADMAP 6a records SCRM done/merged (residual SCRM.7 bookkeeping). The plan's
  "surfacing not building" framing is sound.
- There is NO SCRM acceptance group in `core-acceptance-test-plan.md` (grep for
  SCRM / SNAPSHOT / CAPTURE / REPLAY returns only Regression-map CR references,
  no test group). The plan's Phase-4 instruction to ADD a SCRM group is therefore
  correct and necessary.
- Help: `help_context.rs` renders the body as a monospace label today (the
  "needs a rich renderer" premise), `egui_commonmark` exists in-tree, and the
  `ff-help` model navigation + `plugin_help.rs` HelpPluginBridge seam exist. The
  per-component build-vs-reuse split in Phase 3 (reuse Context + model nav + seam;
  build the rich-renderer integration + rendered-link-click bridge; author core
  content) is grounded in the investigation's file evidence, which is itself
  consistent with the tree I sampled (egui_commonmark dependency confirmed).
- Phase 5+ matches `extension-integration`: CR-CH-074's two additive points =
  Req 10.3 (a) + (b); Extension Manager live registry = Req 1 + Req 8; Catalogs/
  Files as the FIRST extension = Req 9; Extension Manager reached by the
  `EXTENSIONS` verb and NOT compiled into the bare-core POM = Req 8.1. The plan
  does not contradict the spec and correctly sequences all of this AFTER the
  Phase-4 gate.
- Framework conformance of the three new wirings holds. SCRM menu-bar entry,
  Help rich-renderer Context, and the extension-help seam are all expressed
  through existing seams (single `resolve_target` / `dispatch_command_target`
  dispatch, `render_workspace_context` single focus-latch, `WorkspaceDescriptor`
  persistence, code-only `DEFAULT_*_TOML`). The plan explicitly forbids a
  parallel dispatcher, a second navigation stack, and a per-arm focus mechanism,
  matching `framework-conformance.md` and `wiring-standard.md`. No conformance
  violation found.
- Internal sequencing is coherent: Phases 2 (SCRM) and 3 (Help) are explicitly
  marked PARALLEL; Phase 4 (full-core TCR) is the explicit hard gate before
  Phase 5+ extensions; CR-CH-073 commit is the first concrete Phase-1 step;
  CR-CH-074 build is deferred to Phase 5+ so the core stays frozen during
  sign-off. The CURRENT REALITY section agrees with the phases and the
  investigation.
- ASCII-only: a scan of FORWARD-PLAN.md and current-work.md for characters
  outside `\x00-\x7F` and the allowed box-drawing range returned no matches. The
  RESUME banners read are ASCII. Compliant with `documentation-charset.md`.
- Task-format: the plan uses prose and bullet lists, not `tasks.md` checkbox
  lists, so `task-format.md`'s `[ ]`/`[x]` marker rule does not apply here; no
  violation.

## Unverified / wrong assumptions

- WRONG (finding 1): the claim that "a Help group exists" in
  core-acceptance-test-plan.md. The Help rows exist but inside Group 7 "Function
  keys"; there is no standalone Help group.
- LAGGING-DOC (finding 2, not a plan defect): extension-integration/design.md
  still says "exactly one additive core generalization" while requirements.md
  Req 10.3 and the plan say two. The plan follows the authoritative requirements;
  the design.md sentence is the stale one.
- NOT INDEPENDENTLY RE-VERIFIED (relied on the investigation's file citations):
  the precise code locations for the SCRM verb routing
  (`commands_ladder_b.rs`), the `[REPLAY]` Context wiring across
  tab_state/session_manager/tab_manager/render_body, and the `ff-help`
  navigation/plugin_help internals were taken from the investigation rather than
  re-opened line-by-line in this pass. I did directly confirm the two load-bearing
  premises most likely to be wrong: the Help body is a monospace label, and
  `egui_commonmark` is an in-tree dependency used by ffmdx. Both held. The
  remaining citations are consistent with ROADMAP 6a (which independently records
  SCRM done/merged), so I have no reason to doubt them, but they were not
  re-derived here.
