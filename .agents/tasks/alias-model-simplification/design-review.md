# Design Review -- CR-CH-072 Simplified Single-Layer Command-Alias + Menu-Localisation Model

Reviewer: design-review subagent (fresh read, docs-only gate).
Scope: judge the authored/amended gate documents only. No builds, no tests, no
source changes expected (none by design). Verdict is mechanical: any HIGH or
MEDIUM finding => CHANGES_REQUESTED; zero => APPROVED.

## What I read

- `.agents/tasks/alias-model-simplification/gate-summary.md`
- `docs/status/change-log.md` -- CR-CH-072 entry (and CR-CH-071 context; last-id check)
- `docs/specs/localization/requirements.md` -- NEW Req 13; Req 11/12 amendment notes
- `docs/specs/localization/design.md` -- NEW CR-CH-072 design section + supersession table
- `docs/specs/command-environments/requirements.md` -- NEW Req 2b; Req 6a amendment note
- `docs/specs/command-environments/design.md` -- NEW CR-CH-072 resolution-path section
- `docs/specs/command-configurator/requirements.md` -- NEW Req 6; Req 5.24/5.11 supersession
- `docs/specs/command-semantics/requirements.md` -- Req 14.1 amendment note
- `docs/specs/command-framework/requirements.md` -- Req 18.1 amendment note
- `docs/specs/localization/tasks.md` -- NEW Task group 23-30
- `docs/specs/command-environments/tasks.md` -- NEW Task group 32 + coverage table
- `docs/quality/TCR.md` -- NEW Phase (alias-model-simplification) NOT-COVERED rows
- `docs/project-management/ROADMAP.md` -- CR-CH-072 roadmap entry

Source cross-checks (feasibility of the "reuse the existing front door" claim):

- `crates/ff-desktop/src/shell/locale_resolver.rs` -- `rewrite_verb_to_root` pre-pass EXISTS.
- `crates/ff-desktop/src/shell/alias_reader.rs` -- `AliasRowToml` carries the optional
  `localisation_key` + `command_template` fields the design proposes to drop/keep.
- `crates/ff-command/src/command_target.rs` -- `resolve_target` EXISTS as the FFCMD classifier.

## Verification of the 10 mandated checks

### 1. ONE LAYER (surface-in-row; .ftl for descriptions/titles retained) -- SATISFIED
localization Req 13.1 removes the `localisation_key -> verb-*` command-surface
indirection and makes the In_Row_Surface authoritative with "exactly ONE lookup
(surface -> root), not three". Req 13.3 explicitly keeps DESCRIPTION/TITLE `.ftl`
localisation unchanged. The design "The unified AliasEntry shape" drops
`localisation_key`, keeps `surface`/`canonical`/`command_template`/`environment`.
Source confirms `localisation_key` is a real optional field today, so "drop it"
is accurate, not hypothetical.

### 2. SINGLE 2-LEVEL RESOLUTION PATH, reuses the existing pre-pass -- SATISFIED
command-environments Req 2b.1/2b.2 states exactly two levels (active env alias ->
active roots -> reject -> FFCMD alias -> FFCMD roots). Req 2b.5 forbids a second
dispatcher and keeps `CommandTarget`/`resolve_target` unchanged. Both design
sections tie the path to the existing `rewrite_verb_to_root` pre-pass before
`resolve_target`, re-pointed at the in-row surface. Verified the pre-pass and
`resolve_target` exist in source. No parallel resolution path is introduced.

### 3. MENU OPTION REFERENCES AN ALIAS (owner reservation #3) -- SATISFIED
localization Req 13.4 makes a menu option reference an alias so render AND dispatch
use the one path and the render-time reverse-derivation is retired. The mechanism
is stated and justified in the design ("How a menu option references an alias"):
store the ROOT + mark the option "resolve-as-alias"; render resolves the
active-locale surface by the option's root, dispatch resolves the root; rationale
= persisted identity stays the root (no saved-menu migration), no new field/id.
The alternative (store alias id/surface-key) is explicitly considered and rejected.

### 4. ONE LOCALE-AWARE STORE, SHIPPED + USER-EDITABLE -- SATISFIED
localization Req 13.5/13.6 define the Locale_Aware_Alias_Store = shipped read-only
seed overlaid by the User_Locale_Override_Store, user override wins, every row
customisable, seed never written. command-configurator Req 6.1-6.6 makes the Alias
Editor the single in-app place to view/customise the active-locale EFFECTIVE
(merged) table and explicitly SUPERSEDES CR-NR-113 Req 5.11's user-store-only scope
(also noted inline at Req 5.24). Precedence is stated (override wins; a shared
surface is a customisation, not a collision -- Req 6.6).

### 5. OBSERVABLE BEHAVIOUR PRESERVED -- SATISFIED
localization Req 13.9 enumerates the af POM/Settings surfaces rendering fully in
af via surface-in-row, roots dispatching unchanged, two-level resolution holding.
Req 13.8 retains the CR-NR-112 `&1..&N`/`&*` template byte-for-byte. configurator
Req 6.5 preserves the CR-CH-071 ISPF Save/End/Cancel/Return/Refresh semantics over
the override store. Mnemonics stay English (CR-NR-111 Req 12.3 UNCHANGED, stated in
both the change-log and the design "What is NOT changing"). Routing/root-sets
(CR-CH-069/070 + =FILES fix) listed unchanged in design "What is NOT changing".

### 6. SUPERSESSION TABLE COMPLETE -- SATISFIED
`localization/design.md` carries an explicit "Migration / supersession table" with
a disposition and behaviour-preservation note for every prior criterion:
CR-NR-109 (11.1/11.2 superseded; 11.3/11.4/11.6/11.11-13 retained), CR-NR-111
(12.2/12.4 amended; 12.3 unchanged), CR-NR-112 (14.1 amended; 14.2-14.4 retained),
CR-NR-113 (5.11 superseded; 5.6/5.7 superseded), CR-CH-071 (Req 20a preserved).
All four named prior CRs plus CR-CH-071 are accounted for. The gate-summary carries
a matching condensed table.

### 7. EARS QUALITY -- SATISFIED
Req 13.1-13.9, Req 2b.1-2b.5, and configurator Req 6.1-6.7 are in EARS form
(WHEN/WHERE ... THE ... SHALL ...) and are testable; each maps to a TCR row and a
TDD task. Amendments carry inline supersession notes (loc Req 11.1/11.2, 12.2/12.4;
command-environments Req 6a.1; command-semantics Req 14.1; command-framework
Req 18.1), each flagged "AMENDED/SUPERSEDED by CR-CH-072" with the behaviour-kept
statement. New criterion numbers do not collide: Req 13 is new in localization (12
was the highest), Req 2b is a new lettered insertion in command-environments,
Req 6 is new in command-configurator (5 was the highest).

### 8. CR ID -- SATISFIED
The last id in `docs/status/change-log.md` is CR-CH-071; CR-CH-072 is the next free
id and appears once as a new entry. No collision with any existing id (grep of the
change-log shows CR-CH-072 only in the new entry and its cross-references).

### 9. OPEN DECISIONS surfaced, not finalised -- SATISFIED
All four open decisions are surfaced with a RECOMMENDED default and explicitly left
for the owner CONFIRM step in the change-log ("OPEN DECISIONS (owner CONFIRM step)"),
the gate-summary ("OPEN DECISIONS ... recommend, do not finalise"), the design
("Decision -- OPEN (recommended, for owner CONFIRM)" on the menu-option reference and
the shipped-seed-vs-copy choice), and the ROADMAP. The four: (1) user override store
location/shape; (2) how a menu option references its alias; (3) shipped read-only
seed vs copy-to-user; (4) retain vs collapse CR-NR-112 template. None is unilaterally
finalised in a SHALL criterion -- the criteria state the observable contract while the
specific shape is marked OPEN/recommended.

### 10. DOCS HYGIENE -- SATISFIED (with one NIT, pre-existing and out of scope)
The authored CR-CH-072 content (localization Req 13 + design + tasks, command-
environments Req 2b + design + tasks, command-configurator Req 6, the amendment
notes, the change-log entry, the TCR phase rows) is plain ASCII: targeted greps for
the prohibited em/en-dash, curly-quote, ellipsis, and arrow glyphs returned NO
matches in any of those files. New tasks use only `[ ]` markers with titled lines
(groups 23-30, 32). No source code was written (gate is docs-only; the "Affects"
and tasks describe a SEPARATE post-approval implementation run). The TCR status
emoji are the one file-specific exception the documentation-charset rule explicitly
allows for a coverage report. See NIT-1 for a pre-existing non-ASCII presence in
UNRELATED legacy task files.

## Findings

1. NIT -- `docs/specs/command-framework/tasks.md` and
   `docs/specs/command-semantics/tasks.md` contain non-ASCII glyphs (en-dashes `–`,
   arrows `→`, set/logic symbols `∈`, `⟺`) in already-completed `[x]` tasks and
   property-test prose. These are PRE-EXISTING lines, NOT part of the CR-CH-072
   authored content, and do not touch any CR-CH-072 criterion. They violate
   documentation-charset.md but are out of scope for this gate. Fix (separate
   cleanup, not blocking): replace `–`->`-`, `→`->`->`, `∈`->`in`, `⟺`->`iff`.

2. NIT -- The menu-option-reference mechanism (Req 13.4 / design "resolve-as-alias")
   is the recommended default of OPEN DECISION 2 and is also written into a SHALL
   criterion (Req 13.4) and a task (25.2). This is internally consistent (the
   criterion states the observable contract; the specific storage shape is flagged
   OPEN and "recommended, for owner CONFIRM"), but a reader could briefly read
   Req 13.4 as finalising decision 2. Suggested clarification (non-blocking): add a
   one-line note on Req 13.4 that the stored-value SHAPE is OPEN DECISION 2 pending
   owner CONFIRM, mirroring the design's explicit "Decision -- OPEN" marker, so the
   requirement and the open-decision list cannot be read as contradictory.

## Verified assumptions

- `rewrite_verb_to_root` exists in `shell/locale_resolver.rs` (the pre-pass the
  design re-points) -- confirmed in source.
- `resolve_target` exists in `ff-command/src/command_target.rs` (FFCMD classifier,
  left unchanged) -- confirmed in source.
- `AliasRowToml` today carries optional `localisation_key` and `command_template`
  fields -- confirmed in `shell/alias_reader.rs`, so "drop localisation_key, keep
  command_template" is a real, accurate delta.
- CR-CH-072 is the next free change-log id (last = CR-CH-071) -- confirmed by grep.
- New criterion numbers (loc Req 13, cmd-env Req 2b, cmd-cfg Req 6) do not collide
  with existing numbers in their files -- confirmed.
- The supersession table names all of CR-NR-109/111/112/113 and CR-CH-071 -- confirmed.
- The CR-CH-072 authored markdown is plain ASCII -- confirmed by targeted grep.

## Unverified / wrong assumptions

- None material. The design's "reuses the existing single front door" claim was
  spot-checked against real source (pre-pass + resolve_target both present) rather
  than taken on the document's word; it holds. The two NITs above are the only
  discrepancies, and neither is a HIGH/MEDIUM correctness or feasibility gap.

## Verdict

HIGH findings: 0. MEDIUM findings: 0. NIT findings: 2.
Mechanical verdict: zero HIGH/MEDIUM => APPROVED.
