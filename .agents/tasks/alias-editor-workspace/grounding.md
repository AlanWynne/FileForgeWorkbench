# Grounding Note -- CR-NR-113 Alias Editor Workspace Context (design-review prep)

Documentation-only grounding pass. NO source code written, NO source file edited.
Every load-bearing claim in the authored CR-NR-113 gate docs
(command-configurator Requirement 5, design Section 9, Task 7, change-log, TCR,
ROADMAP) was checked against the LIVE source in the current worktree. Verdicts
are VERIFIED-CORRECT / WRONG / UNVERIFIED with the file + evidence inspected.

Docs read: command-configurator requirements.md (Req 5.1-5.12), design.md
(Section 9), tasks.md (Task 7.1-7.7), status/change-log.md (CR-NR-113), quality
TCR.md (12 rows), ROADMAP CR-NR-113 bullet, plus the step-b findings.md and
gate-summary.md (for grounding only).

Live source read: crates/ff-desktop/src/shell/alias_reader.rs,
shell/menus_editor.rs, shell/nav_stack.rs, shell/nav_reconstruct.rs,
shell/render_body.rs (central-panel match), shell/render.rs
(render_workspace_context), shell/workspace_context.rs (WorkspaceContext +
InteriorFocus), shell/locale_resolver.rs (rewrite_verb_to_root pre-pass),
tab_state.rs / tab_state_ctors.rs / session_manager.rs / workspace_kind/mod.rs
(the per-editor TabKind/KindTag/BuiltinKind wiring), and the real
command-framework Requirement 8.10 + command-semantics Requirement 14.

---

## AREA 1 -- THE ONE GENUINELY NEW MECHANISM (user store + serialiser + refresh)

### 1(a) User-store load precedence -- VERIFIED-CORRECT (store is new) / PARTIAL on precedence wording

Claim (design 9.5, Req 5.7): the resolver loads the User_Alias_Store "layered on
top of the identity base + per-locale overlay"; a user alias must not shadow a
built-in root; collisions rejected.

Evidence -- `shell/alias_reader.rs`:
- `LocaleAliasStore` today has exactly three layering steps: `seed` (identity
  base: `AliasTable::ffedit_english` + `AliasTable::identity_for(ffcmd_roots)`),
  then `apply_overlay_from_dir` reads `i18n/<locale>/aliases/<env>.toml` and
  loads via `AliasCatalogue::load_into_keyed`. There is NO user-store layer
  today. So the user-store load IS genuinely net-new -- VERIFIED-CORRECT that it
  does not exist and must be built.
- The per-environment overlay seam (`load_into_keyed` ->
  `apply_locale_overlay_keyed`) rejects a within-environment surface->two-roots
  collision and retains the Identity_Base on rejection (test
  `reader_rejects_within_environment_collision_and_retains_base`). So "reuse the
  overlay's collision discipline" (Req 5.4) is VERIFIED-CORRECT and feasible.

PARTIAL / AMBIGUITY (likely finding): the design describes the user store as a
THIRD overlay applied through the SAME `apply_*_keyed` seam, but that seam loads
rows into a SINGLE per-environment `AliasTable`. The design does not say WHICH
environment (FFCMD? FFEDIT? both?) a user alias targets. `ECOB -> EDIT` would be
an FFEDIT verb (EDIT is an FFEDIT root); a bare workbench shorthand could be
FFCMD. The precedence prose ("layered on top of ... per-locale overlay") is
internally consistent as an ordering statement, but the store-to-environment
MAPPING is UNSPECIFIED and is a real design gap. The precedence relative to a
per-locale surface is also only described as "coexist, not override" (9.5), but
the loader's collision rule REJECTS a surface already mapped differently -- so a
user alias whose surface equals a per-locale surface mapped to a DIFFERENT root
would be REJECTED, not "coexist". The "coexist" wording vs the reject-on-collision
mechanism is a latent contradiction.

Verdict: store-is-new = VERIFIED-CORRECT; precedence ORDERING = VERIFIED-CORRECT
as written; store-to-environment mapping + "coexist vs reject" = UNVERIFIED /
internally inconsistent (likely MEDIUM finding).

### 1(b) No-shadow-built-in-root rule + Requirement 8.10 alignment -- WRONG (misattribution)

Claim (Req 5.4, 5.7, design 9.4/9.5): a user alias must not shadow a reserved
built-in root, "mirroring the Target_Resolution shadowing rule, command-framework
Requirement 8.10", enforced at save (reject) and load (skip-with-WARN).

Evidence -- the REAL command-framework Requirement 8.10 (read in
docs/specs/command-framework/requirements.md, CR-CH-025): it governs the
RESOLUTION-CHAIN STAGE ORDER inside `resolve_target` -- a built-in command /
Command_ID (stage 2) beats a same-named Menu_Name (stage 3) beats a same-named
Macro (stage 4), so a user "CANNOT override a built-in command by creating a menu
or macro". It says NOTHING about aliases, and aliases are NOT a stage of that
chain.

Why this matters -- `shell/locale_resolver.rs` + `shell/dispatch.rs`: aliases are
rewritten in the `rewrite_verb_to_root` PRE-PASS that runs BEFORE
`resolve_target`. So an alias whose surface equals a built-in verb would be
rewritten AWAY before the 8.10 chain is ever consulted -- the alias layer can in
principle shadow a built-in, which is the OPPOSITE of what 8.10 constrains. Req
8.10 is therefore the WRONG authority for the alias no-shadow rule, and citing it
as if it already forbids alias shadowing is a misattribution.

The no-shadow INTENT is reasonable and the save-reject + load-skip-with-WARN
mechanism is implementable, but it is a NEW rule this CR introduces, not an
existing one it inherits. The docs present it as derived from 8.10; it is not.

Verdict: WRONG as attributed (the 8.10 citation does not support the claim).
Likely HIGH/MEDIUM finding: either re-ground the rule as a new CR-NR-113
invariant (and state WHERE "reserved built-in root" is defined -- see the
`builtin_ffcmd_roots()` / `BUILTIN_WORKSPACE_VERB_ROOTS` set in alias_reader.rs
and command_config), or justify it against the pre-pass, not against 8.10.

### 1(c) ACTIVE_ALIAS_STORE post-save live refresh seam -- VERIFIED-CORRECT (seam exists) / PARTIAL (seam does not load the user store)

Claim (Req 5.6, design 9.4/9.5): after save, refresh the live `LocaleAliasStore`
in place "reusing the existing `ACTIVE_ALIAS_STORE` reload seam" so the alias is
live without restart.

Evidence -- `shell/alias_reader.rs`:
- `ACTIVE_ALIAS_STORE` (a process-global `Mutex<Option<Arc<Mutex<LocaleAliasStore>>>>`),
  `publish_active_alias_store`, and `reload_active_alias_store_for_locale` ALL
  exist and work as described -- the seam is REAL. VERIFIED-CORRECT that a
  publish/reload mechanism exists.

PARTIAL (likely finding): `reload_active_alias_store_for_locale` rebuilds the
store via `LocaleAliasStore::seed_and_overlay_for_locale`, which overlays ONLY
`i18n/<locale>/aliases/` -- it does NOT read any user store (the user store does
not exist yet). So the seam as written would NOT pick up a user-store edit on
reload. "Reuse the seam" is only true AFTER `seed_and_overlay_for_locale` (and/or
the reload closure) is EXTENDED to also apply the user-store overlay. The design
says "reuse" but omits that the seam itself must change. This is additive and
feasible, but the claim that the refresh is a pure reuse is incomplete.

Verdict: seam exists = VERIFIED-CORRECT; "pure reuse, no change to the seam" =
PARTIAL/overstated (the reload path must be extended to load the user store).

---

## AREA 2 -- CR-NR-112 COEXISTENCE (no double expansion; worked example exact)

Claim (Req 5.8, design 9.7): the parameter template belongs to the alias entry;
the editor adds no expansion/dispatch; `ecob helloW` -> `EDIT
FFWB.COBOL.MASTER(helloW)` exactly per CR-NR-112; no CR-NR-111 contradiction.

Evidence:
- command-semantics Req 14.1-14.10 read directly: the 4-field record (root,
  alias, localisation key, parameter template) and the owner-confirmed `&*` =
  "argument tokens AFTER the highest referenced &N" semantics (Req 14.3, Glossary
  Rest_Placeholder) are present verbatim.
- Req 14.9 worked example is byte-for-byte the same as Alias Editor Req 5.8:
  `ECOB` / root `EDIT` / template `FFWB.COBOL.MASTER(&1)`, `ecob helloW` ->
  `EDIT FFWB.COBOL.MASTER(helloW)`. VERIFIED-CORRECT, exact match.
- No double-expansion: the render path reads only the display surface
  (`menu_option_command_surfaces` in locale_resolver.rs, reads
  `display_surface_for_canonical`), and execution runs the pre-pass once per seam
  (dispatch.rs / commands_menu.rs). Single-pass is Req 14.7. VERIFIED-CORRECT
  that the editor introduces no second expansion.

CRITICAL DEPENDENCY (likely HIGH finding -- not a contradiction, an ordering gap):
the `command_template` field DOES NOT EXIST in live source. A repo-wide search
for `command_template` returns NOTHING; `AliasRowToml` in alias_reader.rs carries
only `surface`, `canonical`, `localisation_key`; the pre-pass
`rewrite_verb_to_root` does verb rewrite + raw-arg re-append ONLY (no `&N`/`&*`
substitution). CR-NR-112 is itself "GATE AUTHORED / PENDING DESIGN REVIEW", i.e.
UNIMPLEMENTED. Therefore CR-NR-113 has a HARD BUILD-ORDER dependency on CR-NR-112:
Req 5.1 (`command_template` field), 5.3 (edit the template), 5.8 (template
expansion) cannot be implemented until CR-NR-112 lands. The docs say "builds ON
CR-NR-112" but do not state this as a blocking prerequisite.

Verdict: worked example + no-double-expansion = VERIFIED-CORRECT; the
unstated hard dependency on unimplemented CR-NR-112 = likely HIGH finding.

CR-NR-111 coexistence: VERIFIED-CORRECT. CR-NR-111 stores the ROOT on a menu
option and derives the localised surface at render; the template lives on the
alias entry. v1 scope keeps the two in separate files (Req 5.11). No contradiction
found with command-semantics Req 14, command-framework Req 19, localization Req 12.

---

## AREA 3 -- FRAMEWORK CONFORMANCE (live-code verification)

### Registration pattern -- VERIFIED-CORRECT in principle, UNDER-SPECIFIED in the docs

Claim (Req 5.2, design 9.6): register as `WorkspaceKind::CommandConfigurator` +
`editor = "aliases"`, mirroring Theme/Menus/Keys/Kinds; no new WorkspaceKind, no
new dispatcher.

Evidence -- `shell/nav_stack.rs` + `shell/nav_reconstruct.rs`: Theme/Menus/Keys/
Kinds editors DO all reuse `WorkspaceKind::CommandConfigurator` with a distinct
`editor` marker param (`"theme"|"menus"|"keys"|"kinds"`);
`descriptor_for_current_context` emits it and `reconstruct_custom` routes on it.
So "no new WorkspaceKind" is VERIFIED-CORRECT.

GAP (likely MEDIUM finding): each sibling editor is NOT only a descriptor arm. In
live source each one ALSO has:
- a runtime `TabKind` variant AND a `KindTag` variant (tab_state.rs:
  `TabKind::MenusEditor` -> `KindTag::MenusEditor`);
- a `BuiltinKind` mapping (workspace_kind/mod.rs: `KindTag::MenusEditor ->
  BuiltinKind::Menus`);
- a non-persist arm in session_manager.rs (`KindTag::MenusEditor => None` plus the
  two match lists);
- a tab constructor (tab_state_ctors.rs `menus_editor(...)`);
- a DEDICATED central-panel arm in shell/render_body.rs doing the `mem::take ->
  render_workspace_context -> put back -> apply_*_action` swap;
- a shell-held panel field (construct.rs `menus_editor_panel: ...`).

The CR-NR-113 design (9.6) and Task 7.5.1 mention ONLY the `nav_stack.rs`
descriptor arm + the `nav_reconstruct.rs` reconstruct arm. They DO NOT enumerate
the new `TabKind::AliasEditor` / `KindTag::AliasEditor`, the `BuiltinKind`
mapping, the session_manager non-persist arms, the ctor, the render_body.rs arm,
or the shell panel field. Those are REQUIRED to match the sibling pattern the
design claims to copy "one-to-one". The wiring is under-specified; an implementer
following Task 7 literally would miss them. (Note: "no new WorkspaceKind" is true;
"no new KindTag/TabKind" is NOT claimed and would be false -- a new runtime
TabKind IS needed.)

Verdict: pattern correct, "one-to-one copy" claim VERIFIED-CORRECT for the shape,
but the task/design enumeration of the required new variants is INCOMPLETE.

### Focus contract -- VERIFIED-CORRECT

Claim (Req 5.10, design 9.3/9.8): interior focus via the single
`WorkspaceContext::render -> InteriorFocus` path, stable `egui::Id` on the first
control, full-shell egui_kittest first-Tab test.

Evidence -- `shell/workspace_context.rs`: `WorkspaceContext::render(&mut self,
ui, &mut ShellServices) -> InteriorFocus` and `InteriorFocus::single(id)` exist
exactly as described; `shell/render.rs::render_workspace_context` +
`apply_interior_focus` are the single latch path; render_body.rs shows every
migrated editor dispatched through it. `menus_editor_panel` already implements
`WorkspaceContext` and reports the selector combo as first / Save-As field as
last. VERIFIED-CORRECT. The mandatory first-Tab egui_kittest test
(workspace-conformance.md) is correctly required by Req 5.10 and Task 7.6.1.

### No parallel mechanism -- VERIFIED-CORRECT

The open verb goes through the single front door (`rewrite_verb_to_root` ->
`resolve_target` -> `dispatch_command_target`); `open_alias_editor` would
`navigate_to` in place like `open_menus_editor` (verified in menus_editor.rs).
No second dispatcher, no second nav stack, no new session-persistence FORMAT (the
Context persists as the existing `CustomWorkspace { CommandConfigurator }`
descriptor). VERIFIED-CORRECT. The ONE new mechanism (user store + load path) is
correctly flagged as additive, not a framework reshape.

---

## AREA 4 -- SCOPE SEPARATION FROM CR-NR-111 (v1 = user store only)

Claim (Req 5.11, design 9.1/9.5): v1 edits the User_Alias_Store ONLY; the
per-locale `i18n/<locale>/aliases/` tree is not read or written by the editor.

Evidence: no authored criterion reads or writes the per-locale tree. The save
path (design 9.4) writes `<User_Data_Dir>/FileForgeWorkbench/aliases/user.toml`
via `create_dir_all` + `fs::write` -- a user-data path, never the shipped `i18n/`
root resolved by `resolve_i18n_root` (alias_reader.rs). The load path (9.5) adds
a user-store overlay distinct from `apply_overlay_from_dir`'s `i18n/<locale>/`
read. Task 7.7.1 is an explicit scope-guard verification. VERIFIED-CORRECT.

One watch-item (NIT): the live REFRESH reuses the SAME `LocaleAliasStore` that
ALSO holds the per-locale overlay, so the implementation must take care that
re-seeding on save does not drop the per-locale overlay (the reload path
re-reads `i18n/`, so this is fine IF the user-store load is folded into the same
re-seed; see 1(c)). Not a scope breach, but an implementation hazard worth a
design note.

---

## AREA 5 -- EARS QUALITY, NUMBERING, TCR MAPPING, ASCII

- Testable / WHEN...SHALL form: Req 5.1-5.12 are mostly well-formed. 5.1, 5.4,
  5.6, 5.7, 5.8, 5.9 use WHEN/THE...SHALL. A few are capability statements
  ("THE Alias_Editor SHALL be a Custom Workspace ...", 5.2/5.3/5.10/5.11/5.12)
  rather than event-driven WHEN clauses -- acceptable (state invariants), matches
  the style of the sibling Req 1-4 in the same file. No ambiguity blocking a test.
- Req 5.4 is DENSE (four distinct rejections + serialise + write + failure-leaves-
  file-unchanged in one criterion). Testable but would map more cleanly as
  sub-criteria; minor (NIT).
- Numbering: Requirement 5 is the next integer after the existing Req 1-4 in
  command-configurator/requirements.md -- NO collision. VERIFIED-CORRECT.
- TCR mapping: quality/TCR.md CR-NR-113 section has exactly 12 rows labelled Req
  5.1 through 5.12, 1:1 with the 12 criteria, all status RED / NOT-COVERED.
  VERIFIED-CORRECT.
- Task cross-refs: Task 7 subtasks each carry a `Validates: Requirement 5.x`
  (e.g. 7.1.2 -> 5.7, 7.4.3 -> 5.4/5.5/5.6/5.7, 7.6.1 -> 5.10, 7.6.2 -> 5.8).
  Coverage of all 12 criteria is present across the subtasks. VERIFIED-CORRECT,
  with the caveat that the task list omits the TabKind/render_body wiring noted
  in Area 3 (a completeness gap, not a cross-ref gap).
- ASCII: the authored Req 5, design Section 9, Task 7, change-log entry, and TCR
  rows use plain ASCII for structural text; `&1`, `&*`, `->`, `FFWB.COBOL.MASTER()`
  are ASCII. Req 5.12 correctly scopes ASCII to structural keys and allows UTF-8
  for alias VALUES (user data), consistent with documentation-charset.md. No
  prohibited typographic characters observed in the authored text. VERIFIED-CORRECT.

---

## MOST LIKELY REVIEW FINDINGS (surfaced by this grounding)

1. [HIGH] Misattributed shadowing rule. The no-shadow-built-in-root rule (Req 5.4,
   5.7) is grounded on command-framework Req 8.10, but the REAL 8.10 is a
   resolve_target STAGE-ORDER rule (built-in > menu > macro) that does not cover
   aliases at all. Aliases rewrite in the PRE-PASS before 8.10 is consulted, so
   8.10 neither forbids nor supports alias shadowing. Re-ground the rule as a NEW
   CR-NR-113 invariant enforced in the pre-pass, and define "reserved built-in
   root" against the real set (builtin_ffcmd_roots / BUILTIN_WORKSPACE_VERB_ROOTS).

2. [HIGH] Unstated hard dependency on unimplemented CR-NR-112. `command_template`
   exists nowhere in live source; `AliasRowToml` has 3 fields; the pre-pass does
   no `&N`/`&*` substitution. Req 5.1/5.3/5.8 cannot be built until CR-NR-112
   (itself only gate-authored) lands. The docs say "builds ON CR-NR-112" but must
   state this as a blocking prerequisite / sequencing constraint.

3. [MEDIUM] Under-specified Context wiring. The design claims a one-to-one copy of
   the Menus Editor but Task 7 / design 9.6 enumerate only the nav_stack +
   nav_reconstruct arms. Every sibling editor ALSO needs a new TabKind + KindTag
   variant, a BuiltinKind mapping, session_manager non-persist arms, a tab ctor, a
   dedicated render_body.rs central-panel arm, and a shell panel field. Add these
   to the task list, and correct any implication that "no new Kind" means no new
   runtime TabKind (a new TabKind IS required).

4. [MEDIUM] "Reuse the ACTIVE_ALIAS_STORE reload seam" is incomplete. The seam
   exists, but reload_active_alias_store_for_locale /
   seed_and_overlay_for_locale read ONLY i18n/<locale>/aliases/ and would not pick
   up a user-store edit. The seam/reseed path must be EXTENDED to apply the
   user-store overlay; the design should say so rather than imply pure reuse.

5. [MEDIUM] User-store-to-environment mapping unspecified + "coexist vs reject"
   tension. The design does not say which AliasTable environment(s) a user alias
   loads into (FFCMD vs FFEDIT), and the loader's collision rule REJECTS a
   surface already mapped to a different root, which contradicts the "a user alias
   may coexist with (not override) a per-locale surface" wording (9.5). Clarify
   the mapping and reconcile coexist-vs-reject.

6. [NIT] Req 5.4 bundles four rejections + serialise + write + failure semantics
   in one criterion; consider splitting for cleaner 1:1 test mapping.

7. [NIT] Re-seed-on-save must not drop the per-locale overlay held in the same
   LocaleAliasStore (Area 4 watch-item); add a design note.
