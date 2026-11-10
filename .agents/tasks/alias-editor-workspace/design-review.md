# Independent Design Review -- CR-NR-113 Alias Editor Workspace Context

Reviewer: independent design-review subagent (fresh context).
Scope: documentation-only review of the CR-NR-113 gate
(`command-configurator` Requirement 5, design Section 9, Task 7, change-log,
TCR, ROADMAP). No doc or source file was edited. Load-bearing claims were
spot-verified against live source in the current worktree.

Verdict: APPROVED (no HIGH, no MEDIUM; NITs only).

The prior grounding pass surfaced six likely findings (two HIGH, three MEDIUM,
two NIT). Reading the authored docs fresh, those six are ALL already resolved in
the revised text -- the docs explicitly carry the corrections the grounding
predicted. I independently re-verified each load-bearing claim rather than
trusting that. The remaining issues are genuine NITs that do not block
implementation.

---

## Sources verified against live source

- `crates/ff-desktop/src/shell/alias_reader.rs` -- `builtin_ffcmd_roots()`
  (line 136, `pub(crate)`), `ACTIVE_ALIAS_STORE` static (line 293),
  `publish_active_alias_store` (301), `reload_active_alias_store_for_locale`
  (317), `seed_and_overlay_for_locale` (211), `apply_overlay_from_dir` (227).
- `crates/ff-desktop/src/command_config/mod.rs` --
  `BUILTIN_WORKSPACE_VERB_ROOTS` (line 167, `pub(crate)`) and the co-location
  guard test that keeps it in sync with the classifier.
- `crates/ff-desktop/src/shell/environment.rs` --
  `apply_locale_overlay_keyed` (line 305): SAME canonical UPDATES the row,
  DIFFERENT canonical returns `AliasOverlayError::Collision` against a scratch
  copy so `self` is left untouched.
- `crates/ff-desktop/src/shell/alias_overlay.rs` -- `load_into_keyed` (207).
- `docs/specs/command-semantics/requirements.md` -- Req 14.2/14.3/14.9 (the
  CR-NR-112 worked example and `&*` semantics).
- Repo-wide search for `command_template` -> NO matches in any `.rs` file.

---

## Area 1 -- The one genuinely new mechanism (user store + serialiser + refresh)

### 1(a) Load precedence -- well-defined and verified

Design 9.5 states the precedence unambiguously as three ordered layers:
identity base -> per-locale overlay (`i18n/<locale>/aliases/`) -> User_Alias_Store
overlay. Req 5.7 says the same ("a THIRD overlay layered on top of the
Identity_Base and the per-locale overlay"). The ordering is internally
consistent across Req 5.6, 5.7, design 9.5, change-log, and ROADMAP.

The store-to-environment mapping gap flagged by grounding is now closed: each
`[[alias]]` row carries an optional `environment` field (`ffcmd`|`ffedit`,
default `ffcmd`) and the loader routes the row into that environment's
`AliasTable` through the same keyed seam (`apply_locale_overlay_keyed`). The
design justifies the explicit field over inference (9.5) -- reasonable and
unambiguous. VERIFIED: the per-environment keyed seam exists and loads one
environment's rows into one table.

The "coexist vs reject" tension grounding flagged is also closed and, critically,
is CORRECT against live source. Design 9.5 and Req 5.7 define the cross-layer
rule as: SAME root -> no-op (mappings agree); DIFFERENT root -> loader rejects the
user row with a WARN and retains the per-locale surface. I verified
`apply_locale_overlay_keyed` already behaves exactly this way (SAME canonical
updates in place; DIFFERENT canonical returns `Collision` and leaves `self`
untouched). So the design's claim that this "reuses the existing collision
discipline" is accurate, not aspirational. No finding.

### 1(b) No-shadow-built-in-root rule -- correctly re-grounded, misattribution fixed

The grounding's HIGH finding was that the no-shadow rule was attributed to
command-framework Req 8.10, which is a `resolve_target` stage-order rule that
never sees aliases. The revised docs FIX this comprehensively:

- Req 5 carries an explicit "NO-SHADOW RULE PROVENANCE" note stating the rule is
  a NEW CR-NR-113 invariant of the alias PRE-PASS, NOT inherited from Req 8.10,
  and explains why (aliases rewrite in the `rewrite_verb_to_root` pre-pass that
  runs BEFORE `resolve_target`).
- Design 9.5 ("No-shadow invariant (Req 5.4, 5.7) -- NEW, not Req 8.10") repeats
  the correction with the mechanism.
- The rule is grounded on the REAL set: `builtin_ffcmd_roots()` folding in
  `command_config::BUILTIN_WORKSPACE_VERB_ROOTS`. I VERIFIED both exist and are
  `pub(crate)`, and that `builtin_ffcmd_roots()` already composes the workspace
  verb roots.
- Enforcement is specified at BOTH save (reject, Req 5.4(d)) and load
  (skip-with-WARN, Req 5.7).

The change-log, ROADMAP, and TCR all carry the "NOT Req 8.10" correction too. No
finding; the previously-HIGH misattribution is resolved.

### 1(c) Post-save live refresh via ACTIVE_ALIAS_STORE -- seam exists, extension honestly stated

The grounding noted the "reuse the reload seam" claim was overstated because
`reload_active_alias_store_for_locale` -> `seed_and_overlay_for_locale` reads
ONLY the per-locale overlay and would DROP a user-store edit. The revised docs
state this correctly: Req 5.6 and design 9.5 both say the seam SHALL be EXTENDED
(not merely reused) so one re-seed re-applies BOTH overlays, and a save never
drops the per-locale layer. I VERIFIED the live `reload_active_alias_store_for_locale`
calls `seed_and_overlay_for_locale`, which only applies identity base +
`i18n/<locale>/` -- so the "must be extended" statement is accurate and the
re-seed-must-not-drop-per-locale hazard is explicitly guarded by a required unit
test (design 9.8, Task 7.4.4). This is additive; it does not reshape a published
type. No finding.

Conclusion for Area 1: the one new mechanism is well-specified, additive, and
NOT a framework change by the framework-conformance definition. Correctly flagged
for owner sign-off at the CONFIRM step (design 9.9, three owner-confirm points).

---

## Area 2 -- CR-NR-112 coexistence (no double expansion; worked example exact)

- `command_template` belongs to the ALIAS ENTRY (design 9.2 `AliasRowEdit`,
  9.4 `UserAliasRowToml`); a menu option stores the ROOT (CR-NR-111). The two
  axes are kept in separate files in v1 (Req 5.11). No contradiction.
- No double-expansion: design 9.7 states the editor is purely authoring +
  persistence and adds NO expansion/dispatch; invocation runs the single
  CR-NR-112 pre-pass once. Consistent with command-semantics Req 14.7/14.8
  (single pure-text substitution, dispatched through the one front door).
- Req 5.8 worked example EXACTLY matches the finalised CR-NR-112 expansion. I
  verified byte-for-byte against command-semantics Req 14.9: alias `ECOB`, root
  `EDIT`, template `FFWB.COBOL.MASTER(&1)`, `ecob helloW` ->
  `EDIT FFWB.COBOL.MASTER(helloW)`. Identical.
- `&*` = "tokens after the highest referenced &N" (command-semantics Req 14.3) is
  owned by CR-NR-112; the Alias Editor does not restate or contradict it.

The grounding's second HIGH finding -- an unstated hard dependency on
unimplemented CR-NR-112 -- is now resolved and is the strongest part of the
revision. `command_template` exists NOWHERE in source (I re-verified: zero
matches). The docs now carry an explicit "HARD BUILD-ORDER PREREQUISITE" in
Req 5 (preamble), design 9.0, Task 7 preamble + per-subtask CR-NR-112-LANDED
gates on 7.2 / 7.3 (template field) / 7.6.2, the change-log, and ROADMAP. The
non-template subset is correctly identified as buildable first. No finding.

---

## Area 3 -- Framework conformance

- Registration: `WorkspaceKind::CommandConfigurator` + `editor = "aliases"`
  (Req 5.2, design 9.6), mirroring Theme/Menus/Keys/Kinds. No new WorkspaceKind.
  VERIFIED this is the real sibling pattern (grounding confirmed the descriptor
  arms; I did not re-verify).
- The grounding's MEDIUM "under-specified wiring" finding is resolved: design 9.6
  and Task 7.5 now enumerate ALL seven sibling wiring points (new `TabKind` +
  `KindTag`, `BuiltinKind` mapping, `session_manager` non-persist arms, tab ctor,
  descriptor + reconstruct arms, dedicated `render_body.rs` central-panel arm,
  shell panel field), and explicitly note that "no new WorkspaceKind" does NOT
  mean "no new runtime TabKind". This matches the sibling shape.
- Focus contract: Req 5.10 and design 9.3/9.8 use the single
  `WorkspaceContext::render -> InteriorFocus` path with a stable first-control
  `egui::Id` and a MANDATORY full-shell egui_kittest first-Tab test (Task 7.6.1).
  Conforms to workspace-conformance.md.
- No parallel mechanism: open verb `ALIASES` resolves through the single front
  door as a `CustomWorkspace` target; `navigate_to` in place; persists as the
  existing `CustomWorkspace { CommandConfigurator }` descriptor -- no new
  dispatcher, no new persistence format (design 9.6, 9.9). Conforms.

No finding.

---

## Area 4 -- Scope separation (v1 = user store only)

Req 5.11 and design 9.1/9.5 confine v1 to the User_Alias_Store at
`<User_Data_Dir>/FileForgeWorkbench/aliases/user.toml`; the shipped per-locale
`i18n/<locale>/aliases/` tree is explicitly out of scope and never read or
written by the editor. No criterion reads or writes the per-locale tree. The save
path targets a user-data path (never `resolve_i18n_root`). Task 7.7.1 is an
explicit scope-guard verification. The re-seed-drops-per-locale hazard (the one
place the user store and per-locale overlay share a `LocaleAliasStore`) is
covered by the required Task 7.4.4 / 9.8 layering test. No finding.

---

## Area 5 -- EARS quality, numbering, TCR mapping, ASCII

- Requirement 5 is the next integer after the existing Req 1-4 in the same file;
  no numbering collision. VERIFIED.
- TCR: the CR-NR-113 section has EXACTLY 12 rows, labelled Req 5.1 through 5.12,
  1:1 with the 12 criteria, all status RED/NOT-COVERED. VERIFIED.
- Task cross-refs: every Task 7 subtask carries a `Validates: Requirement 5.x`;
  all 12 criteria are covered across the subtasks (5.1 in 7.1.1/7.2/7.3.1; 5.6/5.7
  in 7.1.2/7.4.3; 5.8 in 7.6.2; 5.10 in 7.3.3/7.6.1; 5.11 in 7.7.1; etc.).
- ASCII: Req 5, design Section 9, Task 7, change-log, TCR rows use plain ASCII for
  structural text (`&1`, `&*`, `->`, `FFWB.COBOL.MASTER()` are ASCII). Req 5.12
  correctly scopes ASCII to structural keys and permits UTF-8 for alias VALUES
  (user data), consistent with documentation-charset.md. No prohibited
  typographic characters observed.
- EARS form: most criteria are WHEN...SHALL; a few (5.2, 5.3, 5.10, 5.11, 5.12)
  are capability/invariant statements ("THE Alias_Editor SHALL be a Custom
  Workspace..."), which is acceptable and matches the sibling Req 1-4 style. Each
  is testable.

---

## Findings

1. NIT -- Req 5.4 bundles a lot into one criterion.
   Location: `requirements.md` Requirement 5, criterion 5.4.
   Detail: 5.4 states four distinct save-time rejections (empty surface, empty
   canonical, within-environment surface->two-roots collision, built-in-root
   shadow) PLUS serialise + write + "file unchanged on failure" semantics in a
   single criterion. It is testable (Task 7.4.4 maps each rejection), but a 1:1
   test-to-criterion map would be cleaner if split into sub-criteria
   (5.4a..5.4d). Not blocking.
   Fix (optional): split 5.4 into lettered sub-criteria, one rejection each, with
   the serialise/write/unchanged-on-failure as a trailing clause.

2. NIT -- `AliasEnvironment` module home is slightly ambiguous across sections.
   Location: `design.md` Section 9.2 vs 9.4.
   Detail: 9.2 introduces `pub enum AliasEnvironment` next to the panel-side
   `AliasRowEdit`/`AliasEditorState`, while 9.4 declares `UserAliasRowToml`
   (which also uses `AliasEnvironment`) "alongside `AliasRowToml` in
   `alias_reader.rs`". The enum is therefore shared between the panel module and
   `alias_reader.rs`, but neither section states where it is DEFINED or that it is
   `pub(crate)` for cross-module use. An implementer could reasonably place it in
   either module. Not a behavioural ambiguity.
   Fix (optional): add one sentence naming the enum's home module (e.g. defined in
   `alias_reader.rs` as `pub(crate)` and re-used by the panel state) so the two
   sections agree.

3. NIT -- Req 5.7 restates the no-shadow rule and the cross-layer rule in one
   dense paragraph.
   Location: `requirements.md` Requirement 5, criterion 5.7.
   Detail: 5.7 carries the third-overlay precedence, the per-row environment
   routing, the no-shadow invariant, AND the SAME-root/DIFFERENT-root cross-layer
   rule in a single criterion. It is correct and consistent with design 9.5, but
   it is doing the work of three testable statements. Not blocking (the design and
   TCR row break it down).
   Fix (optional): consider splitting the cross-layer collision behaviour into its
   own sub-criterion for cleaner test mapping.

No HIGH findings. No MEDIUM findings.

---

## Verified Assumptions

- `builtin_ffcmd_roots()` exists, is `pub(crate)`, and folds in
  `command_config::BUILTIN_WORKSPACE_VERB_ROOTS` -- the no-shadow set the design
  cites. (alias_reader.rs:136, command_config/mod.rs:167)
- `ACTIVE_ALIAS_STORE`, `publish_active_alias_store`, and
  `reload_active_alias_store_for_locale` all exist and work as described -- the
  refresh seam is real. (alias_reader.rs:293/301/317)
- `reload_active_alias_store_for_locale` rebuilds via `seed_and_overlay_for_locale`,
  which applies ONLY identity base + `i18n/<locale>/aliases/` and reads NO user
  store -- so the design's "must be EXTENDED" claim is accurate and honest.
- `apply_locale_overlay_keyed` already implements SAME-canonical -> update,
  DIFFERENT-canonical -> `Collision` with `self` untouched -- the cross-layer
  "same-root no-op / different-root reject, retain existing" rule in design 9.5 is
  the EXISTING discipline, not new behaviour. (environment.rs:305)
- `command_template` exists NOWHERE in `.rs` source -- the HARD CR-NR-112 build
  dependency is real and correctly stated as blocking Req 5.1/5.3/5.8.
- command-semantics Req 14.9 worked example is byte-for-byte identical to Alias
  Editor Req 5.8 (`ECOB`/`EDIT`/`FFWB.COBOL.MASTER(&1)` -> `ecob helloW` ->
  `EDIT FFWB.COBOL.MASTER(helloW)`).
- Requirement 5 numbering does not collide with Req 1-4; TCR has exactly 12 rows
  mapping 1:1 to the 12 criteria; Task 7 subtasks cross-reference the criteria.

## Unverified / Wrong Assumptions

- None material. The grounding's two previously-HIGH concerns (8.10 misattribution;
  unstated CR-NR-112 dependency) were assumptions about an EARLIER draft; the
  authored docs under review have already corrected both, which I confirmed
  against live source.
- Not independently re-verified (relied on grounding, which confirmed them): the
  exact shape of each sibling editor's seven wiring points in
  `tab_state.rs` / `workspace_kind/mod.rs` / `session_manager.rs` /
  `tab_state_ctors.rs` / `nav_stack.rs` / `nav_reconstruct.rs` / `render_body.rs` /
  `construct.rs`. Grounding verified these against the Menus Editor; design 9.6 and
  Task 7.5 enumerate them. No doubt remained that warranted re-verification.
