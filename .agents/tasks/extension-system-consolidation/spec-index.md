# Extension Integration Spec -- Authored Files Index

Pointer for the reviewer and owner. This effort is DOCUMENTATION-ONLY (no source
changes, no commit). The extension integration specification -- the keystone
contract defining how an extension integrates into FFWB through the existing
framework seams -- was authored from the consolidation report.

STATUS: OWNER-APPROVED with two required revisions now APPLIED (D2 coordination
corrected to Option B; D5 "Extension command contribution to FFCMD" section
added). Decisions D1-D4 confirmed by the owner; D5 mechanism confirmed. Next:
hand to the design-review step.

## Authored (new) files

- `docs/specs/extension-integration/requirements.md` -- EARS acceptance criteria
  (Req 1-10 + NFRs + out-of-scope) for the extension integration contract.
- `docs/specs/extension-integration/design.md` -- the capability-to-seam mapping,
  the `ff-desktop` extension host, the four decision points D1-D4 with
  recommendations, testability, and the staged D1 migration plan.
- `docs/specs/extension-integration/tasks.md` -- gated implementation task list
  (tasks 1-13), TDD, `(Req X)` cross-references.

## Changed (existing) files

- `docs/project-management/ROADMAP.md` -- added a KEYSTONE PREREQUISITE note at
  the end of Section 5 pointing to the new spec and summarizing D1-D4.

## Source of truth consumed

- `.agents/tasks/extension-system-consolidation/consolidation.md` -- the
  read-only inventory this spec was authored from.

## Decision points (owner-confirmed)

- D1 -- plugin -> extension rename. ACCEPTED (final, standing decision per
  `.kiro/steering/extension-terminology.md`; no decline fallback). Spec authored
  in the "extension" vocabulary; spec-folder renames happen when touched. Staged
  Waves A-D.
- D2 -- Catalogs/Files return as an extension. CONFIRMED. Coordination note
  CORRECTED to the approved OPTION B of workflow `wf_cd193669ad21dc14` (default
  POM menu options removed; `CATALOGS`/`FILES` commands + Contexts KEPT reachable
  if typed, NOT torn down). The extension RE-HOMES them (re-attaches menu options,
  owns Contexts/Kinds) without a teardown/rebuild. Req 9.1/9.3 reconciled.
- D3 -- Extension Manager load model. CONFIRMED in-tree feature-gated registry;
  discovery seam authored so a future dynamic-load model can slot in; the rebuild
  tradeoff is accepted.
- D4 -- SCRM stays core. CONFIRMED; menu/menu-bar surfacing is a separate effort,
  out of scope here.
- D5 -- Extension command contribution to FFCMD. ADDED, then REWORKED after an
  owner correction verified against the code (EnvironmentRegistry
  contains_root/root_commands, user_aliases AliasEnvironment/UserAliasRowToml, the
  Alias Editor, builtin_workspace_target_for). Corrected model: ALIAS is a
  first-class binding (surface -> environment + root + template) but cannot create
  a capability -- it can only target an already-valid root. The load-bearing
  dynamic-add is a RUNTIME-EXTENSIBLE root registry + classifier (option (b)): the
  host registers an extension's new root(s) + classification at enable and removes
  them at disable; aliases and resolve_target then operate unchanged. This is
  FLAGGED as an owner-confirmed FRAMEWORK CHANGE (design D5.3, Req 11.5) touching
  EnvironmentRegistry::root_commands/contains_root and the classifier, with the
  blast radius recorded (Alias Editor dropdown, revalidation, filter, alias
  validation all already call those functions). In-buffer (environment-owned)
  verbs need no runtime-extensible CE-object registry for in-scope work. Canonical
  EDIT -> Editor Context walk-through worked end to end. Contribution is a
  load-time programmatic API (apply_capability), NOT a user-typed REGISTER verb;
  user and extension aliases coexist. Requirement 11 rewritten accordingly; Req 3,
  Req 10.3 updated to reference the runtime-extensible registry as the second
  owner-confirmed framework extension point.

## Status

Revisions applied and internally consistent. Package approved; ready to hand to
the design-review step.
