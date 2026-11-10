# Design Review -- Parameterised Command Aliases (CR-NR-112)

Fresh, context-free review of the documentation-only requirements gate for
parameterised command aliases. Reviewed docs:

- Grounding: `.agents/tasks/parameterised-aliases/findings.md`,
  `.agents/tasks/parameterised-aliases/gate-summary.md`
- command-semantics: `requirements.md` (Req 14), `design.md` (Section:
  Parameterised Command Aliases), `tasks.md` (Phase parameterised-aliases,
  Tasks 30-33)
- command-framework: `requirements.md` (Req 19), `design.md` (Section:
  Parameterised Alias Expansion in the Pre-Pass), `tasks.md` (Phase
  parameterised-aliases, Tasks 32-35)
- change-log: `docs/status/change-log.md` (CR-NR-112)
- Coexistence: `docs/specs/localization/{design.md,requirements.md}` (CR-NR-111)

Claims about the live source were verified by reading the actual code, not by
trusting the docs. Those results are in the "Verified / Unverified Assumptions"
sections at the end.

---

## Findings

Each finding is HIGH, MEDIUM, or NIT. Verdict is mechanical: any HIGH or MEDIUM
means CHANGES_REQUESTED.

### 1. NIT -- Extra-arg append silently drops unreferenced tokens below the highest `&N`

**Where:** command-semantics `design.md`, "The substitution algorithm", the
extra-arg append step `if template has no "&*" and tokens.len() > highest:
surplus = tokens[highest..].join(" ")`; and Requirement 14.5.

**Problem:** The append slices `tokens[highest..]`, where `highest` is the
highest *referenced* 1-based index. If a template references a non-contiguous
subset of positions -- e.g. a template that uses only `&2` (never `&1`) -- then
token index 0 (the first supplied arg) is neither substituted (no `&1` in the
template) nor appended (append begins at index `highest`=2). That first token is
silently discarded, which contradicts the stated intent of 14.5 ("no supplied
token is silently discarded"). The worked examples all use contiguous `&1`
(or `&1..&N`) so the case is never exercised, but the rule as written does not
hold for the general template.

**Fix:** Either (a) state explicitly in 14.5 and the design that the append
covers only tokens *beyond the highest referenced index* and that any
lower-indexed token not referenced by the template is intentionally dropped
(name it a recorded limitation, like the quoting limitation), or (b) change the
rule to append "tokens not consumed by any `&N`" so no supplied token is lost.
Given the owner's "no supplied token silently discarded" phrasing, (a) needs a
one-line caveat; (b) is a slightly larger algorithm change. Recommend (a): add a
sentence to the design algorithm note and to 14.5's rationale.

### 2. NIT -- Menu-option whose stored command IS a parameterised-alias surface is unspecified

**Where:** command-framework `design.md`, "Menu-composition finding"; Req 19.6;
gate-summary "Open decisions 3".

**Problem:** The docs correctly scope OUT "feeding the command-line field into a
menu-option activation". But `activate_menu_option` calls
`rewrite_verb_to_root(option.command)` unconditionally (verified in source,
`commands_menu.rs:186`). If a menu option's stored command is itself a
parameterised-alias SURFACE (e.g. an option literally stores `ECOB`), the
pre-pass will expand it with an EMPTY argument list on a menu click, yielding
`EDIT FFWB.COBOL.MASTER()`. The docs say arg-supply-from-menu is out of scope,
but they do not state what happens when the option's command is a template alias
with no args: silent empty-arg expansion. This is a behaviour that falls out of
the design but is never named, so an implementer/tester has no criterion for it.

**Fix:** Add one sentence to Req 19.6 (or the menu-composition design section):
"A menu option whose stored command is itself a template alias expands with the
empty argument list (the menu click supplies no residue), producing the
empty-substitution form per command-semantics Requirement 14.4; this is a
consequence of the single pre-pass and is expected." Alternatively state that
menu options are not expected to store template-alias surfaces. Either way, name
the outcome so it is testable.

### 3. NIT -- `&*` mixed-with-`&N` repetition is flagged "open for owner" but encoded as settled in criteria

**Where:** gate-summary "Open decisions 1"; command-semantics `design.md` note
on the last table row (`&1 TO &*` -> `MOVE a TO a b c`); Requirement 14.3.

**Problem:** 14.3 defines `&*` as "all remaining argument tokens" (the whole
argument string), and the design explicitly documents that mixing `&1` and `&*`
repeats tokens by design. The gate-summary simultaneously lists this as an "open
decision for the owner" (whether `&*` should instead mean "tokens after the
highest referenced positional"). The requirement text is written as settled while
the gate-summary treats it as unconfirmed. This is a presentation inconsistency,
not a spec contradiction -- the criteria are internally consistent. But a reviewer
cannot tell whether 14.3 is final.

**Fix:** This is acceptable to leave to owner confirmation at the gate approval
step (it is a genuine owner question, not a doc defect). Keep 14.3 as the
proposed default but add an explicit "(proposed; owner to confirm `&*` semantics
-- see gate-summary open decision 1)" marker on 14.3 so the criterion is not
mistaken for owner-confirmed. No change to the algorithm is required.

---

## No HIGH or MEDIUM findings

All ten required coverage points (below) are present, grounded, and internally
consistent. The three findings above are all NIT (presentation / edge-case
documentation), none blocking.

---

## Coverage of the ten required checks

1. **Four-field alias record + TOML schema + back-compat.** COVERED. Req 14.1
   enumerates the four parts (root, alias name, optional localisation key,
   optional parameter template). command-semantics `design.md` shows the additive
   `command_template: Option<String>` on `AliasEntry` and the `#[serde(default)]`
   `AliasRowToml` field with a concrete TOML example. Back-compat (absent/empty =
   pure verb rewrite, byte-identical) is Req 14.6 and is stated in both designs.
   Grounded: the existing `AliasEntry` three-field shape and the
   `i18n/<locale>/aliases/<env>.toml` schema are confirmed against source.

2. **Placeholders &1..&N and &*; substitution algorithm + tokenisation interaction.**
   COVERED. Req 14.3 defines `&N` positional and `&*` rest. The design gives a
   concrete left-to-right, longest-digit-run algorithm, and ties tokenisation to
   `str::split_whitespace` of the `split_verb_arg` remainder -- consistent with
   the verified single-whitespace front-door split. Req 14.10 records the
   one-token-per-`&N` (no quoted multi-word) limitation.

3. **No-arg -> EMPTY substitution; extra-arg residue append.** COVERED. Req 14.4
   (empty string for an unsupplied `&N`, no alias-layer error, resolved command
   owns the result). Req 14.5 (surplus tokens appended, reusing the verified
   verb-only residual pass-through). See Finding 1 for the one non-contiguous
   edge case that is under-specified (NIT, not blocking).

4. **Single-pass v1 + forward-compatible guards.** COVERED. Req 14.7 and
   command-framework Req 19.5 state single-pass; the command-framework design
   documents the max-depth limit and visited-set cycle guard as the additive
   future path, explicitly a separate gated decision.

5. **Expansion in the existing pre-pass before resolve_target, both seams; no
   framework change.** COVERED and VERIFIED. The design grounds the pre-pass as
   `rewrite_verb_to_root` with exactly two call sites, both before
   `resolve_target`: typed (`dispatch.rs:87`) and menu
   (`commands_menu.rs:186`) -- both confirmed in source. Req 19.1/19.2 keep
   `resolve_target` / `dispatch_command_target` / `CommandTarget` / nav stack
   unchanged. command-framework design carries the explicit "NO framework change
   is required" statement.

6. **Menu composition residue-append VERIFIED and matched to reality.** COVERED
   and VERIFIED. The findings' claim is correct: residue-append exists ONLY on
   the function-key seam (`dispatch_key_command` merges `format!("{} {}",
   command, field)`), NOT on a menu-option click. The design and Req 19.6 state
   this honestly and scope menu-option arg-supply OUT. See Finding 2 for the one
   unnamed consequence (NIT).

7. **Coexistence with CR-NR-111; no double-expansion; no contradiction.** COVERED
   and VERIFIED. CR-NR-111 stores the ROOT on a menu option and derives the
   display surface at render time (confirmed in localization design.md,
   "store the ROOT command" decision). The template belongs to the alias entry
   (execution axis); render path never expands a template. Alias lookup is by
   `surface`, not `canonical` (verified in `environment.rs:canonical_verb`), so a
   menu storing root `EDIT` does not match the `ECOB` template entry -- the
   two-axis argument holds. No contradiction with CR-NR-111 Req 12.

8. **Worked ecob/helloW example as a concrete acceptance scenario.** COVERED.
   Req 14.9 (and 14.2) state `ecob helloW` -> `EDIT FFWB.COBOL.MASTER(helloW)`;
   command-framework Req 19.1 and the full-shell test
   `full_shell_ecob_opens_edit_with_member` carry it as a scenario.

9. **tasks.md conforms to task-format.md; EARS numbering no collision.** COVERED.
   All task checkboxes are `[ ]`/`[x]`, every task line has a title, and tasks
   cross-reference criteria via "Validates:" / "Covers:". command-semantics Req
   14 (after Req 13) and command-framework Req 19 (after Req 18) are each the
   unique highest integer -- no collision with CR-NR-111 (Req 13 / Req 18).

10. **ASCII compliance; next-free CR-NR id.** COVERED and VERIFIED. The newly
    authored CR-NR-112 sections in all four spec files and the change-log entry
    scan clean against `[^\x00-\x7F\u2500-\u257F]` (pre-existing non-ASCII in
    OTHER sections of these files is out of this gate's scope). CR-NR-112 is the
    next free id (highest existing CR-NR-111, confirmed in change-log).

---

## Verified Assumptions (checked against live source)

- `AliasEntry` is today `(surface, canonical, localisation_key)` with NO template
  field -- `crates/ff-desktop/src/shell/environment.rs`. The design's additive
  field is a true delta.
- `AliasTable::canonical_verb` matches by `surface` (case-insensitive) and
  returns the canonical verb only -- `environment.rs`. The design's claim that a
  template-aware entry lookup is a new addition is accurate, and the
  lookup-by-surface fact is what makes the CR-NR-111 coexistence argument sound.
- `rewrite_verb_to_root` is a single String->String pre-pass in
  `shell/locale_resolver.rs`; today it returns `root` / `format!("{root} {arg}")`
  / unchanged. The design's "extend this one function" plan is grounded.
- Two call sites, both before `resolve_target`: `dispatch.rs:87`
  (`let raw = self.rewrite_verb_to_root(raw);`) and `commands_menu.rs:186`
  (`let root_command = self.rewrite_verb_to_root(command);` inside
  `activate_menu_option`). Both confirmed.
- `split_verb_arg` (`helpers.rs`) splits on the FIRST whitespace and trims the
  remainder; `FFWB.COBOL.MASTER(helloW)` has no internal whitespace, so the
  parenthesised member token survives intact as the argument. The routing proof
  is sound.
- Residue-append exists only on the function-key seam; menu-option click
  dispatches the option command alone. Confirmed via the menu-option path reading
  `option.command` with no `command_text` reference.
- CR-NR-111 localization design.md decision is "store the ROOT command" with
  render-time surface derivation -- confirmed; coexistence claim holds.
- ASCII scan of the new sections: clean.
- CR-NR-112 is the next free CR-NR id; Req 14 (command-semantics) and Req 19
  (command-framework) do not collide with any existing requirement number.

## Unverified / Wrong Assumptions

- **Stale doc comment in `locale_resolver.rs` (NOT a spec defect).** The doc
  comment on `rewrite_verb_to_root` states "the menu-selection entry calls
  `resolve_locale_verb` directly ... for the single-path property", which reads
  as if the menu seam does NOT call `rewrite_verb_to_root`. The actual code
  (`commands_menu.rs:186`) DOES call `rewrite_verb_to_root`. The CR-NR-112
  findings/design correctly describe the real call (two sites, both
  `rewrite_verb_to_root`); the source COMMENT is stale. This does not affect the
  CR-NR-112 docs' correctness but the implementer should not be misled by that
  comment when touching the file. No spec change required.
- No claim in the design was found to be contradicted by source. Every code-level
  assertion the docs make (field shapes, call sites, split behaviour, residue
  seam, lookup-by-surface) was confirmed.

---

## Verdict

Findings: 0 HIGH, 0 MEDIUM, 3 NIT. Per the mechanical rule (HIGH+MEDIUM == 0),
the verdict is **APPROVED**. The three NITs are recommended polish for the owner-
approval step (name the non-contiguous-token edge case, name the menu-option-is-
a-template-alias consequence, mark the `&*` semantics as proposed-pending-owner)
but none block implementation.
