# Gate Summary -- Menu-Kind Localisation (CR-NR-111)

The requirements gate for the owner's "menu-kinds localisation redesign" is
AUTHORED end to end (requirements -> design -> design review -> tasks -> coverage
-> roadmap). This summary is the handoff for the gate's CONFIRM step: it names the
CR, every document produced or edited, the key decisions made, the
framework-delta position, and the open decisions the owner must confirm before any
code is written.

## CR identity

- **CR id**: CR-NR-111 -- "Menu Kinds localise FULLY: mnemonic + localised command
  alias + description all follow the active locale (first-prize auto-conversion;
  second-prize per-language saved menu)".
- **Classification**: NEW REQUIREMENT (the DELTA on top of the in-progress
  CR-NR-109). Logged in `docs/status/change-log.md`.
- **Status**: GATE AUTHORED / PENDING OWNER APPROVAL. Design review verdict
  APPROVED (0 HIGH, 0 MEDIUM, 2 NIT). No code written (this is a docs-only gate).

## Documents produced / edited

Change log
- `docs/status/change-log.md` -- CR-NR-111 entry (authored earlier; Status line
  updated by this step to GATE AUTHORED / PENDING OWNER APPROVAL).

Investigation + design handoff + review (under `.agents/tasks/menu-kind-localisation/`)
- `findings.md` -- read-only investigation (storage-shape crux, framework-delta
  question, existing seams). Read-only input to this gate.
- `design-files.md` -- handoff note naming every design.md/requirements.md touched.
- `design-review.md` -- APPROVED verdict (0 HIGH, 0 MEDIUM, 2 NIT).
- `gate-summary.md` -- this file.

Requirements (EARS criteria -- authored by the requirements step)
- `docs/specs/localization/requirements.md` -- NEW Requirement 12 (12.1-12.10),
  the GOVERNING criteria; plus the Out-of-Scope note for the per-locale menu DATA.
- `docs/specs/menu-workspace/requirements.md` -- NEW Requirement 21 (21.1-21.8;
  21.8 added per review finding 4: displayed mnemonic is display-only).
- `docs/specs/menu-and-statusbar/requirements.md` -- NEW Requirement 20 (20.1-20.3).
- `docs/specs/command-framework/requirements.md` -- NEW Requirement 18 (18.1-18.3,
  routing-invariant decision record).
- `docs/specs/command-semantics/requirements.md` -- NEW Requirement 13 (13.1-13.5,
  behaviour-preserving-routing decision record).
- `docs/specs/workspace-kinds/requirements.md` -- NEW Requirement 9 (9.1-9.3,
  no Kinds-registry change).

Design
- `docs/specs/localization/design.md` -- GOVERNING design section "Menu Kinds
  localise FULLY ... (CR-NR-111, Requirement 12)" plus the iteration-2 and
  iteration-3 review-response logs.
- `docs/specs/menu-workspace/design.md` -- render-model + second-prize + mnemonic
  REALISED CONTRACT delta.
- `docs/specs/menu-and-statusbar/design.md` -- no-new-dispatch-path guarantee delta.

Tasks (authored/edited by THIS step)
- `docs/specs/localization/tasks.md` -- NEW Phase 8 (Tasks 16-22): the GOVERNING
  implementation plan (mnemonic seam, shared-renderer change, stored-ROOT proof,
  first-prize, second-prize, chrome invariance, coverage/doc update).
- `docs/specs/menu-workspace/tasks.md` -- NEW Phase (menu-kind-localisation) Task
  49: menu-workspace-specific slices cross-referenced to the governing plan.
- `docs/specs/menu-and-statusbar/tasks.md` -- NEW Phase Task 38: verification +
  coverage for the menu-row chrome guarantee.
- `docs/specs/command-framework/tasks.md` -- NEW Phase Task 31: verification +
  coverage for the routing-invariant decision record (no new code).
- `docs/specs/command-semantics/tasks.md` -- NEW Phase Task 29: verification +
  coverage for the behaviour-preserving decision record (no new code).

Coverage (workflow.md section 3 step 5)
- `docs/quality/TCR.md` -- NEW section "CR-NR-111 -- Menu Kinds localise FULLY"
  with 22 NOT-COVERED (RED) rows, one per new EARS criterion across all six specs
  (localization 12.1-12.10 incl. 12.7a/12.7b, menu-workspace 21.1-21.8,
  menu-and-statusbar 20.1-20.3, command-framework 18.1-18.3, command-semantics
  13.1-13.5, workspace-kinds 9.1-9.3).

Roadmap / master tracking (TASK C)
- `docs/project-management/ROADMAP.md` -- CR-NR-111 added as a sibling sub-bullet
  under Section 6a step 1 (beside CR-NR-109), in the same style.

## Storage-shape decision made

STORE THE ROOT COMMAND (Option A) as the Locale_Independent_Stored_Value. The
displayed command alias, mnemonic, and description are DERIVED at render time from
the active-locale catalogues + per-locale alias DATA. Option B (store a
localisation key) was considered and REJECTED (it duplicates per-menu what the
alias catalogue already holds per-locale, needs a new `MenuOption` field, and only
satisfies Req 11.9 via indirection). Consequence: NO migration of existing saved
menus and NO new `MenuOption` field; existing files render byte-identically at
`ui.locale = "en"` and auto-convert when per-locale DATA exists.

The one genuinely new seam: the MNEMONIC localises via a new catalogue key
`menu-<menu>-opt-<command>-key` resolved by `localised_option_mnemonic` =
`t_or(key, &option.key)` (Option ii), mirroring the existing description seam.
Deriving the mnemonic from the command surface (Option i) is REJECTED and must not
be implemented.

## Framework-delta position

NO framework delta flagged for owner confirmation. The owner's hard constraint
("this should not affect how commands are routed") is already satisfied by the
EXISTING pre-pass `rewrite_verb_to_root` -> `AliasTable::canonical_verb` that runs
BEFORE `ff_command::resolve_target` on BOTH the command-line and menu-selection
seams (wired by CR-NR-109). CR-NR-111 reshapes NEITHER the `AliasTable` entry model
NOR the dispatch classifier, so it is NOT a framework change and needs no framework
confirmation beyond this gate. Moving the alias lookup INSIDE `resolve_target`
WOULD be a framework change and is explicitly REJECTED/avoided.

## OPEN DECISIONS for the owner at the CONFIRM step

1. **Approve the gate as a whole** (requirements + design + tasks + coverage +
   roadmap) so implementation (localization Phase 8) may begin. Until approved, no
   source file is touched.
2. **Confirm the storage-shape decision**: store the ROOT (Option A), with the
   mnemonic as a dedicated per-locale catalogue key (Option ii). This is the one
   substantive design choice; confirm it is the intended "first prize".
3. **Confirm first-prize is the primary path and second-prize is an opt-in
   fallback**: the POM and built-in menus localise by first-prize auto-conversion
   ONLY (no `menus/<L>/pom.toml` is wired; built-ins stay code-only, CR-CH-021).
   The optional per-language saved-menu FILE override applies only to the two
   user-menu reconstruct seams (`reconstruct_settings_menu`,
   `reconstruct_named_menu`). Confirm this scoping is acceptable.
4. **Acknowledge the DATA dependency**: the reason a localised POM still shows
   English is MISSING per-locale alias/mnemonic/description DATA for the built-in
   roots (e.g. the `af` surfaces for Settings/Catalogs/Files/Help/X and the
   Settings roots). Authoring that DATA is a SEPARATE gated deliverable (out of
   ASCII scope, cross-referenced in Req 12 Out of Scope); CR-NR-111 delivers the
   CODE contract only. Confirm the DATA authoring is tracked separately.
5. **Note the two review NITs** (non-blocking, recorded for the implementer): (a)
   spell out that a USER menu's "Identity_Base definition" in Req 12.7 is the base
   `menus/<name>.toml`; (b) name the exact active-locale accessor the two
   reconstruct seams read. Neither blocks approval.

## Not done (by design)

- No source code changed; no remaining workflow steps restructured.
- No implementation started; all 22 coverage rows remain NOT COVERED (RED) until
  the owner approves and localization Phase 8 lands the tests.
