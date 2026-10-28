# FORWARD PLAN -- READ THIS FIRST

This is the single authoritative entry point for the CORE-STABILISATION window.
A new context should read THIS document before any RESUME-*.md, before
`current-work.md`, and before diving into the ROADMAP backlog. It states where we
are now and the owner-confirmed phased sequence to take the application from a
stripped bare core to a solid, signed-off core -- and only THEN begin extensions.

Authority for this window: see "Authority and relationship to the ROADMAP" at the
end. In short: for the core-stabilisation window FORWARD-PLAN.md owns the PHASE
ORDERING; `docs/project-management/ROADMAP.md` remains the control tower for the
longer backlog and owns the finalise-core TASK DETAIL that Phase 1 points at.

ASCII-only (per `.kiro/steering/documentation-charset.md`). Terminology: the
former "plugin" concept is now "extension" (gradual in-place rename, per
`.kiro/steering/extension-terminology.md`).

---

## CURRENT REALITY (verified)

- CORE STRIPPED TO A BARE CORE. The parked domains (Editor, File navigator, VFS
  providers, Catalog/IDCAMS, Volume) were UNWIRED from the `ffwb` binary but kept
  as workspace members, to re-enter later as EXTENSIONS. Detail:
  `RESUME-ffdesktop-simplification.md` (the single-front-door dispatch + the
  wiring standard that all new wiring must follow).

- POM TRIMMED TO THE GENUINELY-CORE SET: `0 Settings / 3 Help / X` (the former
  `1 Catalogs` and `2 Files` options were removed from the default POM, but their
  `CATALOGS` / `FILES` commands and Contexts remain reachable if TYPED -- Option
  B). This is CR-CH-073: DONE in code, review APPROVED, but UNCOMMITTED and
  awaiting the owner's full gate. (The change-log HEADER still reads "PENDING
  OWNER APPROVAL"; that header is stale -- the decision line, the gate-approved
  artifact, and the implemented code all say APPROVED/DONE.)

- EXTENSION FRAMEWORK REDESIGNED AND RENAMED (plugin -> extension). The extension
  integration contract is specified in `docs/specs/extension-integration/`
  (consolidating the former `plugin-architecture` + `plugin-manager-ui`). Its two
  additive framework extension points (CR-CH-074) are owner-APPROVED but NOT yet
  built:
  1. a runtime-extensible root registry + classifier (an extension can register a
     new command ROOT at enable time, honoured by `resolve_target`), and
  2. a single identifier-keyed `TabKind::Extension { id }` dispatch arm (all
     extension Contexts dispatched through the existing `render_workspace_context`
     single focus-latch path).
  The extension-host implementation itself (live registry, Extension Manager live
  list, migrating Catalogs/Files into the first real extension) is still gated and
  belongs to Phase 5+.

- SCRM and HELP are ALREADY BUILT and core-wired in code (not vision -- shipping
  crates). SCRM (`ff-scrm` + `ff-screen-model`) has `SNAPSHOT`/`CAPTURE` verbs on
  the single command path and a `[REPLAY]` WorkspaceContext. HELP (`ff-help`) has
  a `[HELP]` WorkspaceContext reached by F1 / `HELP`, with model-layer navigation
  built. The forward work on both is SURFACING and UPGRADE, not greenfield build
  -- see Phases 2 and 3 for the precise build-vs-reuse split.

---

## THE PHASED FORWARD PLAN

Owner-confirmed sequencing. Phases 2 and 3 can run in PARALLEL. Extensions
(Phase 5+) do NOT begin until Phase 4 (full-core TCR) is signed off.

Everything below MUST build ON the existing framework -- single command dispatch
(`resolve_target` -> `dispatch_command_target`), menu-name resolution,
WorkspaceContext single focus-latch path, WorkspaceDescriptor persistence -- per
`.kiro/steering/framework-conformance.md` and `.kiro/steering/wiring-standard.md`.
No parallel dispatcher. No second navigation stack. Nothing here may contradict
the extension-integration spec.

### Phase 1 -- CORE / POM finalisation

Goal: lock the trimmed bare core as the stable baseline.

- FIRST CONCRETE STEP: the owner runs the full gate and COMMITS CR-CH-073. It is
  DONE in code and review-APPROVED but currently UNCOMMITTED; committing + gating
  it is the gate between "stripped in the working tree" and "locked baseline".
  Once committed, correct the stale "PENDING OWNER APPROVAL" change-log header.
- DECLARE THE TRIMMED POM THE STABLE BARE-CORE BASELINE: default POM =
  `0 Settings / 3 Help / X` only. `CATALOGS` / `FILES` stay typeable (Option B);
  no default POM option for them until they return as the first extension
  (Phase 5+).
- FINALISE-CORE TASK DETAIL: the editor/dataset/volume/localisation cluster that
  makes the six core capabilities pass is owned by ROADMAP Section 6a (steps 1-8:
  localisation extraction -> Volume UI -> catalog/navigator rework -> editor
  F3-F5 -> CORE #4 editor test pass). FORWARD-PLAN does NOT restate that ordering;
  it DEFERS to ROADMAP 6a for it. See `RESUME-l10n-phase2.md` (ACTIVE),
  `RESUME-volume-model.md` (ACTIVE-deferred), `RESUME-dataset-rationalisation.md`
  (DONE engine).

### Phase 2 -- SCRM wiring (runs in PARALLEL with Phase 3)

Goal: SURFACE already-built SCRM. This is surfacing, NOT building -- SCRM is built
and core-wired (verified): `ff-scrm` + `ff-screen-model` are in the core set, the
`SNAPSHOT` and `CAPTURE START/STOP/SCREEN/STATUS/LIST/PURGE/REPLAY` verbs are
dispatched through the single front door, and the `[REPLAY]` WorkspaceContext is
fully wired (KindTag/TabKind/WorkspaceKind descriptor/open path/render dispatch).

Scope of the wiring work (placement RESOLVED -- decision 1):
- Surface SCRM as a MENU-BAR ENTRY (NOT a bare-core POM option, NOT typed-only).
  The bare-core POM stays exactly `0 Settings / 3 Help / X` (CR-CH-073 -- do not
  re-expand it), but SCRM is core functionality the owner wants discoverable, so
  it belongs on the menu bar. Typed access continues to work regardless.
- The menu-bar entry's `command` value is the EXISTING `SNAPSHOT` / `CAPTURE`
  verb, so a click routes through `resolve_target` -> `dispatch_command_target`
  exactly like typing it. Menu content goes in a `menus/*.toml` / the code-only
  `DEFAULT_*_TOML`; NO new verb, NO bespoke intercept, NO second dispatcher (per
  `.kiro/steering/wiring-standard.md`).
- Deferred SCRM add-ons (digital signature, protected-PDF evidence, DIDL export,
  conditional-capture-rule shell wiring, replay filter UI, off-frame async write)
  are discrete future slices; none blocks core SCRM surfacing. Bookkeeping: close
  SCRM.7 TCR rows.

Detail / history: `RESUME-scrm-and-ff-desktop-refactor.md` (DONE for the engine +
decomposition waves). If the "built" claim is ever found not to hold on a given
component, scope that component as a build and flag it; the current verified state
is "built, needs surfacing".

### Phase 3 -- HELP system, CORE scope (runs in PARALLEL with Phase 2)

Goal: a CORE help viewer with rich Markdown + inter-file navigation, help authored
for the CORE feature set only, and a defined seam for extensions to contribute
their own help later. This is MORE than wiring: part reuse, part build, part
content authoring.

Build-vs-reuse per component (verified):

- Help-viewer Context -- REUSE as-is. `ff-desktop` `help_context.rs`
  (`HelpContextPanel` over `ff_help::HelpPanelModel`) is a correct
  WorkspaceContext with a stable first-interior id, dispatched through the single
  focus-latch path; `[HELP]` tab, F1 + `HELP` routing DONE (CR-NR-097). The rich
  renderer plugs INTO this frame.
- Markdown RENDERER in the Help Context -- NEEDS BUILDING (upgrade). The Help body
  currently renders RAW Markdown in a monospace label. RESOLVED (decision 2):
  adopt `egui_commonmark` (the in-tree engine, used by the ffmdx app) as the rich
  renderer in the Help Context; CR-NR-100 is the vehicle for the Help-Context
  integration. REUSE that engine; BUILD the thin integration that swaps the
  monospace label for the rich renderer. DO NOT compose the CR-NR-101
  markdown-rendering STYLE config in the core window -- Phase 3 is scoped to a
  WORKING rich renderer + navigation + core content, NOT theming. CR-NR-101
  styling is a DEFERRED follow-up, not a Phase-3 dependency.
- Inter-file (inter-topic) NAVIGATION -- PARTLY BUILT. The model layer is built
  and tested (`ff-help` `navigation.rs` NavigationStack back/forward;
  `help_panel.rs::follow_link(&TopicKey)`); shell Back + cross-reference
  activation is DONE (CR-NR-097). What is NOT yet present is CLICKABLE inline links
  inside the RENDERED body (the body is non-interactive monospace today). REUSE
  `NavigationStack` + `follow_link`; BUILD the rendered-link-click -> TopicKey ->
  `follow_link` bridge (depends on the rich renderer above).
- Core help CONTENT -- NEEDS AUTHORING (data, not code). The engine already
  resolves `cmd:*`/`line:*`/`mode:*`/`feature:*` Topic_Keys and shows a
  not-yet-available message for unauthored topics. RESOLVED (decision 3) -- author
  `.help.md` Markdown (ASCII per documentation-charset) for the CORE surface ONLY:
  - the bare-core POM commands: `SETTINGS`, `HELP`, and the terminate/`X` family;
  - the always-present core Contexts/commands: the shell command line,
    navigation (RETURN / END), the Settings / Config surface, SCRM `SNAPSHOT` /
    `CAPTURE` (now core per decision 1), and the Help system itself.
  SET ASIDE (defer) all help for the parked/extension domains -- Editor, File
  navigator, VFS, Catalog/IDCAMS, Volume -- each ships its OWN help when it
  returns as an extension. DO NOT author help for the typed-but-not-core
  `FILES` / `CATALOGS` now.
- Extension-help seam -- DESIGNED AND BUILT at the engine layer; live wiring
  DEFERRED to Phase 5+. `ff-help` `plugin_help.rs` already provides
  `HelpPluginBridge` (`register_topic` at an extension's initialize,
  `deregister_all` at shutdown; extension topics override file topics). So "each
  extension ships its own help" is already expressible. What remains is exposing
  `HelpPluginBridge` through the extension `PluginContext` of the live extension
  host (the `docs/specs/extension-integration/` contract) -- that is extension-host
  work, correctly sequenced into Phase 5+, not core.

References: `docs/specs/context-help/`, `docs/specs/markdown-rendering/`,
`docs/specs/extension-integration/`.

### Phase 4 -- CORE acceptance / full TCR (the GATE to extensions)

Goal: prove the solid core with a full-core TCR pass. This is the hard gate
between "solid core" and "start extensions" -- Phase 5+ does not begin until this
is signed off.

- The sign-off INSTRUMENT is the EXISTING
  `docs/project-management/core-acceptance-test-plan.md` (its Groups 1-9). Do NOT
  invent a parallel sign-off checklist; drive that document's walk.
- EXTEND that document before sign-off, in its own format, because SCRM and Help
  are now core:
  - ADD a new SCRM acceptance group (there is none today).
  - ADD a new HELP acceptance group. NOTE the existing Help-related rows
    (7.1 / 7.1a / 7.1b, etc.) are NOT a standalone group -- they sit INSIDE
    Group 7 "Function keys" and only cover F1-launches-Help behaviour. RESOLVED
    (decision 4): SPLIT OUT a new dedicated Help acceptance group for the new
    rich Markdown renderer, inter-file navigation, and the authored core help
    content; Help is now a first-class core subsystem, so it warrants its own
    group rather than riding under "Function keys". Leave the existing
    F1-launches-Help row where it is under Group 7 (it is a function-key
    behaviour); the NEW rich-renderer / navigation / core-content rows form the
    new Help group.
- Account for the `[B]` blocked rows gated on PENDING-GATE CRs (e.g. CR-NR-068
  Create/Edit-Menu dialog, CR-NR-062 context-menu command parity) as part of
  finalise-core; CORE is DONE only when every REQUIRED row passes.

### Phase 5+ -- EXTENSIONS (only after Phase 4 sign-off)

Forward backlog, sequenced AFTER core sign-off. Keep the core frozen while it is
being signed off (do NOT fold these into the core window).

- Build the extension host: implement CR-CH-074's two APPROVED additive framework
  points -- the runtime-extensible root registry / classifier, and the single
  `TabKind::Extension { id }` dispatch arm through `render_workspace_context`.
- Stand up the Extension Manager live registry (live `PluginRegistry` /
  `PlatformServices` wiring, production service-trait impls, the live extension
  list).
- Migrate Catalogs/Files into the FIRST real extension (extension-integration
  spec Req 9), returning them as extension-contributed commands/Contexts/menu
  options through the framework seams.
- Wire the extension-help seam live (expose `HelpPluginBridge` through the
  extension `PluginContext`), so each extension ships its own help.
- Thereafter, the ROADMAP Section 5 plugin/extension build order (Utilities ->
  mainframe emulation -> POSIX -> File Formatter -> Database -> Job Monitor), each
  built and signed off before the next.

---

## Detail documents (DETAIL, not competing authority)

These are the detail layer. They do not override this plan for the core window;
where a RESUME file and this plan disagree on SEQUENCE, this plan wins for the
core window. Each RESUME file now carries a one-line status banner at its top.

- `docs/project-management/ROADMAP.md` -- the control tower (see authority note
  below). Owns the finalise-core task detail (Section 6a) that Phase 1 points at,
  and the longer post-core backlog (Section 5).
- `docs/project-management/core-acceptance-test-plan.md` -- the Phase 4 sign-off
  instrument.
- `RESUME-ffdesktop-simplification.md` -- single-front-door dispatch + wiring
  standard foundation (Phase 1 context).
- `RESUME-scrm-and-ff-desktop-refactor.md` -- SCRM engine + decomposition
  (Phase 2 detail).
- `RESUME-l10n-phase2.md` -- localisation Phase 2, ACTIVE (ROADMAP 6a step 1,
  part of Phase 1 finalise-core).
- `RESUME-volume-model.md` -- Volume model, requirements approved, code deferred
  into dataset rationalisation (ROADMAP 6a steps 2-6, part of Phase 1).
- `RESUME-dataset-rationalisation.md` -- dataset ENGINE DONE/merged (Phase 1
  dependency for the editor test pass).
- `RESUME-decomp-wave6.md`, `RESUME-decomp-wave7.md` -- ff-desktop decomposition,
  DONE/historical.
- `RESUME-menus-editor-typing-bug.md` -- a bug-fix resume (B054), not a
  forward-plan driver.
- `docs/specs/extension-integration/` -- the extension contract that Phase 5+
  builds; the extension-help seam referenced in Phase 3.

---

## Authority and relationship to the ROADMAP

For the CORE-STABILISATION window, THIS document (FORWARD-PLAN.md) is the
authority on PHASE ORDERING and is the first-read entry point. It owns the
five-phase sequence: (1) finalise core, (2) SCRM surfacing, (3) HELP build-out,
(4) full-core TCR sign-off, (5+) extensions.

`docs/project-management/ROADMAP.md` remains the CONTROL TOWER for the product as
a whole -- the CORE-vs-extension split, the six core capabilities, the longer
post-core backlog (Section 5), and the finalise-core TASK DETAIL (Section 6a).

Who wins where they overlap, for this window:

- Phase ORDERING (the five-phase sequence, and the fact that SCRM surfacing and
  the HELP build-out are explicit core steps): FORWARD-PLAN wins. The ROADMAP does
  not currently sequence SCRM surfacing or a Help build-out as core steps; this
  plan adds them.
- Finalise-core TASK DETAIL inside Phase 1 (the editor/dataset/volume/localisation
  ordering and dependencies): ROADMAP Section 6a wins. FORWARD-PLAN defers to it
  and does not restate it.
- Phase 4 sign-off CONTENT: `core-acceptance-test-plan.md` wins as the instrument;
  FORWARD-PLAN only directs that it be EXTENDED with SCRM + rich-Help rows.

This plan must NOT silently contradict the ROADMAP. The intended reconciliation is
that ROADMAP Section 6 points AT this FORWARD-PLAN as the active-phase ordering,
keeping the control tower's single-source-of-truth role intact while this document
drives the core-stabilisation window. (That ROADMAP Section 6 pointer edit is made
as part of this effort -- resolved decision 5.)

---

## Resolved decisions (owner-confirmed)

All seven decisions below are RESOLVED and folded into the phases above. They are
recorded here as the decision log; none is open.

1. SCRM surfacing placement: RESOLVED -- MENU-BAR ENTRY (not a bare-core POM
   option, not typed-only). The POM stays `0 Settings / 3 Help / X`; SCRM is
   surfaced on the menu bar invoking the existing `SNAPSHOT` / `CAPTURE` verbs via
   the single command path. Typed access still works. (See Phase 2.)
2. HELP renderer approach: RESOLVED -- adopt `egui_commonmark` in the Help Context;
   CR-NR-100 is the vehicle for the integration + the rendered-link-click ->
   TopicKey -> `follow_link` bridge. CR-NR-101 markdown-rendering STYLE config is
   DEFERRED (not a Phase-3 dependency). (See Phase 3.)
3. HELP core content scope: RESOLVED -- author help for the CORE surface only
   (bare-core POM commands SETTINGS / HELP / terminate-X; the shell command line;
   navigation RETURN / END; the Settings / Config surface; SCRM SNAPSHOT /
   CAPTURE; the Help system itself). Non-core / parked-domain help (Editor, File
   navigator, VFS, Catalog/IDCAMS, Volume) is DEFERRED to each owning extension;
   no help authored for typed-but-not-core FILES / CATALOGS now. (See Phase 3.)
4. Phase-4 TCR instrument: RESOLVED -- drive the existing
   `core-acceptance-test-plan.md` (Groups 1-9); no parallel checklist. EXTEND it
   before sign-off with a new SCRM acceptance group AND a new HELP acceptance
   group. NOTE the existing Help-related rows (7.1 / 7.1a / 7.1b, etc.) are NOT a
   standalone group -- they sit inside Group 7 "Function keys" and cover only
   F1-launches-Help. The new rich-renderer / navigation / authored-core-content
   rows SPLIT OUT into a new dedicated Help group (Help is now a first-class core
   subsystem); the existing F1-launches-Help row stays under Group 7. The `[B]`
   blocked PENDING-GATE rows ARE in scope of finalise-core; CORE is DONE only when
   every REQUIRED row passes. (See Phase 4.)
5. ROADMAP reconciliation: RESOLVED -- FORWARD-PLAN.md is the first-read
   active-phase ordering doc for the core-stabilisation window; ROADMAP Section 6
   points AT it (a pointer edit, made as part of this effort), keeping the ROADMAP
   the control tower for the backlog + finalise-core detail (6a); `current-work.md`
   reduced to a thin pointer. (See "Authority and relationship to the ROADMAP".)
6. CR-CH-073 commit/gate: RESOLVED -- Phase 1 step 1. The owner runs the
   authoritative full gate and commits; once committed, the stale "PENDING OWNER
   APPROVAL" change-log header is corrected. (See Phase 1.)
7. CR-CH-074 build timing: RESOLVED -- the two approved extension framework points
   (runtime-extensible root registry / classifier; the single
   `TabKind::Extension` dispatch arm) are built in PHASE 5+, NOT folded into the
   core window. The core stays frozen during sign-off. (See Phase 5+.)
