# Forward-Plan Investigation (DOCS-ONLY, read-only verification)

Date: this investigation pass. No source or docs were changed. All findings are
grounded in files read in the live tree; code citations are file-relative to the
workspace root `c:\workspace\apps\FileForgeWorkbench`.

---

## (A) Current reality summary

The application has been STRIPPED to a bare core (milestone core-001): parked
domains (Editor, File navigator, VFS providers, Catalog/IDCAMS, Volume) were
UNWIRED from the `ffwb` binary but kept as workspace members, to re-enter later
as EXTENSIONS (the former "plugin" concept, renamed per the standing owner
decision). The default POM has been trimmed to the genuinely-core set `0
Settings / 3 Help / X` (CR-CH-073, DONE in code, review APPROVED, uncommitted
pending the owner's full gate). The extension framework contract has been
specified and its two additive framework extension points are owner-APPROVED but
NOT yet built (CR-CH-074). SCRM (Screen Snapshot + Collection/Replay) and the
context-sensitive HELP subsystem are BOTH already built and core-wired in code
(crates `ff-scrm`/`ff-screen-model` and `ff-help`), each with its own
WorkspaceContext (`[REPLAY]` and `[HELP]`) reached through the single command
front door. The authoritative governing doc is `docs/project-management/
ROADMAP.md` (the control tower), whose Section 6a forward path is the current
plan; the owner now wants a re-sequenced forward plan (finalise core -> SCRM ->
Help -> full-core TCR sign-off -> extensions) that must RECONCILE with and point
to the ROADMAP + `core-acceptance-test-plan.md` rather than contradict them.

---

## (B) RESUME-file inventory (docs/status/)

| File | Real status | Role in the forward plan |
|------|-------------|--------------------------|
| `current-work.md` | STALE / SUPERSEDED (dated 2026-10-16) | Pre-reorg dashboard; its "Active work item" points at ROADMAP 6a but the surrounding tables predate strip-to-core, the extension rename, and CR-CH-073/074. Should be rewritten to point at the new forward plan. See (C). |
| `RESUME-scrm-and-ff-desktop-refactor.md` | DONE (SCRM engine + Waves 0-3 gate-verified) | SCRM is BUILT + core-wired; remaining is SURFACING (POM/menu entry) + a few deferred add-ons. Confirms (F). |
| `RESUME-decomp-wave6.md` | DONE (Waves 1-6 owner-verified, merged) | ff-desktop decomposition into aspect crates; historical. No forward action beyond the CR-NR-099 "wire editor onto aspect crates when editor testing begins" note. |
| `RESUME-decomp-wave7.md` | DONE / SUPERSEDED (final decomposition wave; ROADMAP 6a says decomposition merged) | Last decomposition wave (files_panel). Historical. |
| `RESUME-ffdesktop-simplification.md` | DONE for Phases 1-2 + B080 dispatch-unify (owner-verified full gate clean); a few low-priority follow-ups | Establishes the single-front-door dispatch + the wiring-standard steering. Foundational context for how SCRM/Help/extensions must wire. |
| `RESUME-l10n-phase2.md` | ACTIVE (Localisation Phase 2: Tasks 7-8 merged; Task 9 NEXT, Task 10 capstone later) | ROADMAP 6a step 1. Large cross-cutting string extraction; pulled to the FRONT so later panels are written localisation-aware. Partially overlaps "finalise core". |
| `RESUME-volume-model.md` | ACTIVE-but-DEFERRED (requirements gate authored + owner-approved; code deferred into dataset rationalisation / RC.C.10) | ROADMAP 6a steps 2-6 (Volume UI + catalog/navigator rework). Part of "finalise core". |
| `RESUME-dataset-rationalisation.md` | DONE (RC.A..RC.B.8 committed + pushed, full-gate clean; record-aware MAINFRAME SAVE wired with one documented live-handle limitation) | Dataset stack ENGINE is finished and merged. The remaining editor-against-datasets test pass (CORE #4) depends on it. |
| `RESUME-menus-editor-typing-bug.md` | DONE pending close-out (B054 fixed + Tab-accessibility rework committed; awaiting owner's in-app Tab confirmation) | A bug-fix resume; not a forward-plan driver. Close-out + a noted follow-up bug (per-frame theme.active validation storm) remain. |

Note: `current-work.md` is the only RESUME-class doc that is genuinely stale and
misleading as an entry point; the rest are accurate historical/active records.

---

## (C) current-work.md staleness assessment

CONFIRMED STALE. Evidence and omissions:

- It is explicitly dated "verified 2026-10-16" and its work-area table is a
  pre-reorg phase ledger (Phases BR..CZ) that does not reflect the current state.
- Its own "Active work item" block DOES already defer to ROADMAP Sections 6/6a
  and says "when the two disagree, the ROADMAP wins" -- so it is secondary by its
  own admission.
- It OMITS every pillar of the new direction:
  - strip-to-core (core-001) and the parked-domain model.
  - the plugin -> EXTENSION rename and the Extension Manager.
  - CR-CH-073 (POM trim to `0/3/X`).
  - CR-CH-074 (the two approved extension framework points).
  - SCRM surfacing and the HELP build-out as explicit forward steps.
- It still references a "Resume-from-reorg checklist" and an old git tip
  (`8ada2dc`) that the later dataset-rationalisation resume supersedes (RC.* were
  since committed + pushed).

Recommendation: the NEW forward plan should become the first-read document, and
`current-work.md` should be reduced to a thin pointer at it (and at the ROADMAP),
or retired. Do not treat its tables as current.

---

## (D) CR-CH-073 and CR-CH-074 state

### CR-CH-073 -- Trim the core default POM (remove Catalogs + Files)
- STATE: DONE in code; review APPROVED; UNCOMMITTED pending the owner's full
  gate. (The change-log HEADER still reads "GATE AUTHORED / PENDING OWNER
  APPROVAL", but that header is stale: the DECISION line in the same entry records
  "Option B CONFIRMED, APPROVED", the gate-approved artifact exists, the code is
  implemented, and CR-CH-074's "Related" note calls it "DONE, review APPROVED,
  uncommitted pending owner full gate".)
- CODE EVIDENCE (verified): `crates/ff-desktop/src/menu_workspace/defaults.rs`
  `DEFAULT_POM_TOML` now contains exactly three options -- `0 Settings`,
  `3 Help`, `X` (X has `show_in_menu_bar = false`). The former `1 Catalogs` and
  `2 Files` blocks are gone. The test `default_pom_toml_has_recovery_baseline_options`
  carries the comment "Catalogs/Files removed from the barebones core".
- OPTION B honoured: the `FILES`/`CATALOGS` commands and their Contexts remain
  reachable if TYPED (no change to `command_config/mod.rs`, `workspace_kind`,
  `TabKind`/`KindTag`, or descriptor reconstruction). Regression guards
  `equals_files_jump_path_still_opens_file_explorer` and
  `bare_files_still_opens_file_explorer_via_front_door` are kept GREEN.
- ARTIFACTS: `.agents/tasks/strip-core-pom-trim/{gate.md, gate-approved.json,
  implementation.md, review.md}`. Review verdict: APPROVED, no blocking findings.
- NOTE on stale reference: `consolidation.md` section 8 still describes the POM
  as listing `1 Catalogs` / `2 Files` -- that report predates the CR-CH-073
  implementation and is now out of date against the live `defaults.rs`.

### CR-CH-074 -- Extension Integration contract: two additive framework points
- STATE: APPROVED by the owner; implementation DEFERRED (owner was restarting the
  session to add steering rules + hooks). NOT yet built. No source changed by the
  approval entry.
- THE TWO APPROVED ADDITIVE POINTS:
  1. Runtime-extensible root registry + classifier: make
     `EnvironmentRegistry::root_commands`/`contains_root` and the
     `builtin_workspace_target_for`/`resolve_target` classifier runtime-extensible
     so an extension's `apply_capability` can register a new command ROOT (with its
     `CommandTarget` classification + owning environment) at enable time, honoured
     by resolve_target, root enumeration, and the alias layer while active;
     `revoke_capability` removes it at disable. Compiled built-in roots stay
     authoritative and first (first-wins). This is the dynamic command-contribution
     mechanism (an ALIAS cannot CREATE a capability).
  2. Single identifier-keyed extension-Context dispatch arm: ONE additive
     `TabKind::Extension { id }` variant and ONE match arm dispatching ALL
     extension Contexts by identifier through the existing `render_workspace_context`
     single focus-latch path (owned-panel swap). Does NOT change resolve_target, the
     per-tab nav stack, the InteriorFocus type, or the focus-latch mechanism.
- STILL GATED SEPARATELY (not covered by this approval): the extension-host
  implementation itself (live `PluginRegistry` + `PlatformServices` wiring,
  production service-trait impls, the Extension Manager live list, and migrating
  Catalogs/Files into the FIRST real extension -- Req 9).
- Linked spec: `docs/specs/extension-integration/requirements.md` Req 5.3, 10.3,
  11.5, 11.6, 11.8; design.md D5.3.

### Other recent relevant CRs (context)
- A dense cluster of PENDING-GATE / in-progress localisation + alias CRs
  (CR-NR-109/111/112/113, CR-CH-066/067/069/070/071/072) sits in ROADMAP 6a step 1
  (localisation). These are the alias/menu-localisation stream; most are docs-gate
  authored or in-progress, not core-blocking for SCRM/Help.
- CR-NR-097 (context-help) is DONE (see (G)). CR-NR-098 (SCRM) is DONE (see (F)).
- CR-NR-100 is referenced as the (not-yet-done) "render help via the Markdown
  viewer" enhancement -- directly relevant to the Help build-out (see (G)).
- CR-NR-101 (markdown-rendering config) and CR-NR-102 (ffmdx-app) are drafted,
  gate pending.

---

## (E) FORWARD-PLAN vs ROADMAP vs core-acceptance-test-plan -- authority relationship

Three documents, three distinct roles; they must form a hierarchy, not compete:

1. `docs/project-management/ROADMAP.md` -- the CONTROL TOWER. It is self-described
   as "the ONE document to read" for product/core/plugin split, status, and the
   active phase. Section 2 fixes CORE-before-PLUGIN; Section 3 lists the six core
   capabilities; Section 5 fixes the post-core plugin build order; Section 6/6a is
   the current active forward path.
2. `docs/project-management/core-acceptance-test-plan.md` -- the SIGN-OFF GATE for
   CORE. It is a living, repeatable MANUAL end-to-end regression pass organised
   into Groups 1-9 that map onto the six core capabilities plus function keys,
   theme, and RESET BARE. CORE is "DONE only when every REQUIRED row passes".
3. The NEW FORWARD-PLAN (to be authored) -- the re-sequenced, first-read ORDERING
   the owner asked for: (1) finalise stripped core, (2) wire in SCRM, (3) build/wire
   HELP, (4) full-core TCR sign-off, (5) THEN extensions.

Where they overlap / who wins:

- The FORWARD-PLAN and ROADMAP 6a overlap heavily on "finalise core". The ROADMAP
  6a steps (localisation extraction -> Volume UI -> catalog/navigator rework ->
  editor F3-F5 -> CORE #4 editor test pass) ARE the detail of the owner's step 1.
  The FORWARD-PLAN must RECONCILE with and POINT TO ROADMAP 6a for that detail, not
  restate or contradict it. For the core-stabilisation window, treat ROADMAP 6a as
  the authoritative task ordering for the editor/dataset/volume cluster; the
  FORWARD-PLAN adds the SCRM-surfacing and HELP-build-out steps the ROADMAP does
  not currently sequence as explicit core steps.
- Two reconciliations the FORWARD-PLAN must make explicit:
  (a) SCRM: ROADMAP treats SCRM as DONE/merged (SCRM.0-6; SCRM.7 = TCR
  bookkeeping) and does NOT list SCRM surfacing as a core step. The owner's "wire
  in SCRM" is therefore a SMALL SURFACING step (POM/menu-bar entry), not a build.
  The FORWARD-PLAN should say so and add it as a named core step (and note
  consolidation.md section 7 already flags exactly this gap).
  (b) HELP: ROADMAP does NOT list a Help build-out as a core step at all, yet the
  owner wants a markdown renderer + inter-file navigation + core-only help +
  per-extension help seam. The FORWARD-PLAN introduces Help as a NEW explicit core
  step; it must be reconciled with the already-DONE context-help (CR-NR-097) and
  the markdown-rendering spec (CR-NR-100/101) so it is framed as "upgrade the
  existing Help Context's renderer + author core help content", not "build Help
  from scratch". See (G).

- Phase 4 (full-core TCR sign-off) reconciles with the EXISTING
  core-acceptance-test-plan by BEING that document's walk. The FORWARD-PLAN should
  NOT invent a parallel sign-off list; it should drive the Groups 1-9 pass and,
  because SCRM and Help are now core, note that the plan currently has:
    - a Help group (Group 7 rows 7.1/7.1a/7.1b exist, CR-NR-097 DONE), but
    - NO SCRM group at all.
  So Phase 4 must ADD a SCRM acceptance group (and Help rows for the new rich
  renderer + core help content) to core-acceptance-test-plan.md before sign-off,
  in that document's own format. Many Group 2/3 rows are currently `[B]` blocked on
  PENDING-GATE CRs (Create/Edit-Menu dialog CR-NR-068, context-menu command parity
  CR-NR-062, etc.) and gate CORE sign-off -- the FORWARD-PLAN must account for them
  as part of "finalise core".

Recommended resolution: the FORWARD-PLAN becomes the top-of-stack ORDERING doc
that (i) owns the five-phase sequence, (ii) defers to ROADMAP 6a for the
finalise-core task detail, (iii) drives core-acceptance-test-plan.md as the Phase
4 sign-off instrument and extends it with SCRM + rich-Help rows, and (iv) never
restates a decision already owned by ROADMAP/the test plan. Update ROADMAP Section
6 to point AT the FORWARD-PLAN as the active-phase detail, keeping the control
tower's single-source-of-truth role intact.

---

## (F) SCRM verified state + remaining surfacing scope

VERDICT: the consolidation claim holds. SCRM is BUILT and core-wired; the forward
work is SURFACING, not building. File evidence:

- Engine crates present and in the core set: `crates/ff-scrm/` (model, persist,
  replay, rules, export, evidence, pdf, pdf_protected, error) and
  `crates/ff-screen-model/`. Both declared in `crates/ff-desktop/Cargo.toml`
  (per consolidation.md section 7 and the SCRM resume).
- SNAPSHOT + CAPTURE verbs dispatched through the single front door:
  `crates/ff-desktop/src/shell/commands_ladder_b.rs` -- `SNAPSHOT [...]` ->
  `self.handle_snapshot(arg)`; `CAPTURE START/STOP/SCREEN/STATUS/LIST/PURGE/REPLAY`
  -> `self.handle_capture(rest)`. Comments cite screen-snapshot-scrm Req 7/8/9.1/
  11.1 and note routing "through the single handle_command path like every other
  built-in verb".
- Shell session owner: `crates/ff-desktop/src/scrm_session.rs` (`ScrmSession`,
  capture + auto-capture). POM ScreenProvider: `crates/ff-desktop/src/
  screen_snapshot.rs`.
- The REPLAY Context exists as a WorkspaceContext: `crates/ff-desktop/src/
  scrm_viewer_panel.rs` (`ScrmViewerState` with First/Prev/Next/Last + selectable
  text + stable first-interior id; `impl WorkspaceContext`). The tab is titled
  `"[REPLAY]"` -- `crates/ff-desktop/src/tab_state_ctors.rs::scrm_viewer` ->
  `base_tab!(... TabKind::ScrmViewer, "[REPLAY]" ...)`. Full Context wiring is
  present: `KindTag::ScrmViewer`/`TabKind::ScrmViewer` (tab_state.rs),
  `WorkspaceKind::ScrmViewer` descriptor mapping (session_manager.rs),
  `open_scrm_viewer_tab` (tab_manager.rs), render dispatch (render_body.rs).

REMAINING SURFACING SCOPE (the "wire in SCRM" step):
- There is NO POM option and NO menu-bar entry for SNAPSHOT/CAPTURE today; they
  are reachable only by TYPING the verbs (confirmed by consolidation.md section 7:
  "SNAPSHOT/CAPTURE are currently reached via the command ladder, not yet via a
  menu-bar / POM menu option").
- Per `.kiro/steering/wiring-standard.md`, surfacing MUST be done by invoking the
  EXISTING verbs through the single command path: add a POM option and/or a
  menu-bar entry whose `command` value is the existing SNAPSHOT/CAPTURE verb (menu
  content in a `menus/*.toml` / the code-only `DEFAULT_*_TOML`; no new verb, no
  bespoke intercept, no second dispatcher). A toolbar/menu click routes through
  `resolve_target` -> `dispatch_command_target` exactly like typing it.
- Decision for the owner: SNAPSHOT/CAPTURE are global (FFCMD) commands with no
  dedicated Context, so a POM option in the bare core must be a deliberate choice
  (mirrors the Catalogs/Files POM debate -- is SCRM a bare-core POM option, or only
  a menu-bar entry, or only typed?). Flag in (H).
- Deferred SCRM add-ons (recorded as non-gate-blocking in the SCRM resume): Req
  20.4a digital signature; Req 14.5 evidence-as-protected-PDF; Req 15.4 DIDL
  state-transition export; Req 9.2-9.4/9.6-9.9 conditional-capture-rule shell
  wiring; Req 15.3 DIDL replay filter UI; Req 18.3 off-frame async capture write.
  These are discrete future slices; none blocks core SCRM surfacing.
- Bookkeeping: SCRM.7 (TCR rows) per ROADMAP 6a.

---

## (G) HELP verified state, build-vs-reuse per component, and the extension-help seam

HELP is MORE than wiring, but much of the foundation already exists. The context-
sensitive help subsystem (CR-NR-097) is DONE and core-wired; the GAP against the
owner's scope is the RICH MARKDOWN RENDERER in the Help Context and the authoring
of core help content. Component-by-component, with file evidence:

### Component 1 -- Markdown RENDERER in the Help Context: NEEDS BUILDING (upgrade)
- The Help Context currently renders the topic body as RAW MARKDOWN in MONOSPACE,
  not rich rendered Markdown. Evidence: `crates/ff-desktop/src/help_context.rs`
  render body: `ui.label(egui::RichText::new(topic.body()).monospace());` with the
  comment "A richer Markdown renderer is a later enhancement; the model carries raw
  Markdown by design (content is data, not code)."
- A REAL rich egui Markdown renderer ALREADY EXISTS in the tree but is NOT used by
  Help: `ff-mdx-app` draws Markdown via `egui_commonmark::CommonMarkViewer`
  (per `docs/specs/markdown-rendering/requirements.md` Introduction), and
  `crates/ff-md-viewer/` renders Markdown-to-HTML (consumed by ff-html-export /
  ff-pdf-export). So the renderer is REUSABLE: wire the Help Context body to
  `egui_commonmark` (or ff-md-viewer's model) instead of a monospace label. This is
  the substance CR-NR-100 names ("render help via the Markdown viewer") and what
  the markdown-rendering spec's `context-help` "Related" row anticipates.
- BUILD-VS-REUSE: REUSE the existing egui_commonmark/ff-md-viewer engine; BUILD
  the thin Help-Context integration (swap the monospace label for the rich
  renderer) and, if desired, compose the CR-NR-101 markdown-rendering style config.

### Component 2 -- Inter-file (inter-topic) NAVIGATION: PARTLY BUILT
- The model-layer navigation IS built: `crates/ff-help/src/navigation.rs`
  (`NavigationStack` back/forward history, tested) and
  `crates/ff-help/src/help_panel.rs::follow_link(&TopicKey)` ("Follow a
  cross-reference link to another topic"). The shell has Back navigation and
  cross-reference activation (core-acceptance-test-plan Group 7 row 7.1a asserts
  "activate a cross-reference link, then Back"; CR-NR-097 marked DONE/AUTOMATED).
- What is NOT yet present is CLICKABLE inline links inside the RENDERED body,
  because the body is currently non-interactive monospace text. Inter-topic links
  today resolve via `follow_link(TopicKey)` driven by the model, not by clicking a
  rendered Markdown hyperlink. Once Component 1 (rich renderer) lands, the
  Markdown link-click must be wired to `follow_link`/`show_topic` so a link in the
  rendered page navigates between help files. BUILD: the link-click -> TopicKey ->
  follow_link bridge in the Help Context render path.
- BUILD-VS-REUSE: REUSE `NavigationStack` + `follow_link` (model navigation);
  BUILD the rendered-link-to-topic click bridge (depends on Component 1).

### Component 3 -- Help-viewer Context: BUILT and reusable
- `crates/ff-desktop/src/help_context.rs` (`HelpContextPanel` wrapping
  `ff_help::HelpPanelModel`) is a correct WorkspaceContext with a stable first
  interior id (`help_search_field_id`), dispatched through the single focus-latch
  path. Shell glue: `crates/ff-desktop/src/shell/help.rs` (`open_help`, index +
  function-key dynamic topics, miss-handling), `commands_ladder_a.rs` (`HELP` /
  `HELP <arg>` verbs), `[HELP]` tab title, `KindTag::HelpContext`. F1 + `HELP`
  routing is DONE (CR-NR-097). This Context is the frame the rich renderer plugs
  into -- REUSE as-is, upgrade only the body render.
- `ff-help` is a mature subsystem (lib.rs modules: topic_key, topic, registry,
  context_detector, content_loader, content_parser, help_panel, navigation,
  search, plugin_help, dynamic_content, commands, menu, config). Content format is
  `.help.md` Markdown parsed by content_parser. REUSE.

### Component 4 (owner scope) -- help built AROUND CORE FUNCTIONALITY only
- This is a CONTENT-authoring task, not an engine task: author `.help.md` topics
  for the core commands/contexts (and set non-core help aside). The engine already
  resolves `cmd:*`/`line:*`/`mode:*`/`feature:*` Topic_Keys and shows an
  "unrecognised/not-yet-available topic" index message for unauthored topics
  (core-acceptance-test-plan rows 7.1b; context-help Req 18.4/19.1 DONE). So the
  forward step is: enumerate core topics, author their Markdown, and leave
  non-core topics to each extension. BUILD: core help CONTENT (data, per
  documentation-charset ASCII rules), not code.

### The EXTENSION-HELP seam -- ALREADY DESIGNED AND BUILT at the engine layer
- `crates/ff-help/src/plugin_help.rs` provides `HelpPluginBridge` with
  `register_topic(plugin_id, TopicKey, title, markdown_content)` (called at an
  extension's `initialize`) and `deregister_all(plugin_id)` (at `shutdown`), over
  a shared `Arc<HelpTopicRegistry>`. Topics carry `TopicSource::Plugin { plugin_id }`
  and a plugin topic OVERRIDES a file-based topic of the same key (tested:
  `plugin_topic_overrides_file_based`). So "each extension ships its own help" is
  already expressible: an extension registers its Markdown help topics through this
  bridge during its lifecycle.
- What remains is the CONNECTION to the live extension host: per
  `consolidation.md` section 9, ff-desktop does NOT yet construct a live
  `PluginRegistry`/`PlatformServices`, so `HelpPluginBridge` has no production
  caller yet. Exposing `HelpPluginBridge` through the extension `PluginContext`
  (the extension-integration contract, `docs/specs/extension-integration/`) is the
  work that ties per-extension help into the extension host -- and that belongs to
  PHASE 5 (extensions), AFTER core. The seam EXISTS; wiring it live is extension-
  host work, correctly sequenced last.
- References for the authoring step: `docs/specs/context-help/` (Req 18/19,
  CR-NR-097), `docs/specs/markdown-rendering/` (CR-NR-100/101 rich render + style
  config), `docs/specs/extension-integration/` (the PluginContext seam that will
  expose `HelpPluginBridge`).

HELP summary: Help-viewer Context = BUILT/REUSE; model navigation + cross-ref =
BUILT/REUSE; rich Markdown renderer in Help = NEEDS BUILDING (reuse egui_commonmark/
ff-md-viewer, build the Help-Context integration + rendered-link click bridge);
core help CONTENT = NEEDS AUTHORING; extension-help seam = BUILT at engine layer,
live wiring deferred to the extensions phase.

---

## (H) Open decision points for the owner

1. SCRM surfacing placement: should SNAPSHOT/CAPTURE appear as a bare-core POM
   option, a menu-bar entry only, or stay typed-only? (Mirrors the Catalogs/Files
   POM-trim philosophy -- what genuinely belongs in the bare-core POM.) Whatever is
   chosen must invoke the EXISTING verbs via the single command path
   (wiring-standard), adding no new verb or dispatcher.
2. HELP renderer approach: adopt `egui_commonmark` directly in the Help Context,
   or route through `ff-md-viewer`, and whether to compose the CR-NR-101
   markdown-rendering style config now or later. Confirm CR-NR-100 scope (render
   help via the Markdown viewer) as the vehicle.
3. HELP content scope: confirm the exact set of "core" help topics to author now
   (which commands/contexts count as core) and that all non-core help is deferred
   to its owning extension.
4. Phase-4 TCR instrument: confirm the FORWARD-PLAN drives the EXISTING
   `core-acceptance-test-plan.md` (Groups 1-9) as the sign-off, and approve ADDING
   a new SCRM acceptance group + rich-Help rows to that document (rather than a
   parallel checklist). Note the many `[B]` blocked rows gated on PENDING-GATE CRs
   (e.g. CR-NR-068 Create/Edit-Menu dialog, CR-NR-062 context-menu command parity)
   that must be resolved for CORE sign-off -- confirm these are in scope of
   "finalise core".
5. ROADMAP reconciliation: approve making the new FORWARD-PLAN the first-read
   active-phase ordering doc that POINTS AT ROADMAP 6a for finalise-core detail,
   and reducing `current-work.md` to a pointer (or retiring it).
6. CR-CH-073 commit/gate: it is implemented + review-APPROVED but UNCOMMITTED and
   the change-log header still reads "PENDING OWNER APPROVAL". Confirm the owner
   runs the full gate and commits so the stale header can be corrected and the
   trimmed POM is locked in before the forward plan proceeds.
7. CR-CH-074 build timing: the two extension framework points are APPROVED but
   unbuilt; confirm they are built in PHASE 5 (extensions), not folded into the
   core-stabilisation window, so the core stays frozen while being signed off.

---

## Primary sources read (verification trail)

- `docs/status/current-work.md`, and all `RESUME-*.md` in `docs/status/`.
- `docs/status/change-log.md` (CR-CH-073, CR-CH-074, and neighbouring CRs).
- `docs/project-management/ROADMAP.md` (Sections 1-8, incl. 6a).
- `docs/project-management/core-acceptance-test-plan.md` (Groups 1-9; Group 7
  Help rows; absence of a SCRM group).
- `docs/specs/extension-integration/requirements.md` (referenced via change-log +
  consolidation) and `docs/specs/markdown-rendering/requirements.md`.
- `.agents/tasks/extension-system-consolidation/consolidation.md` (esp. sections
  7 SCRM and 9 keystone contract / extension-help gap).
- `.agents/tasks/strip-core-pom-trim/{gate-approved.json, review.md}`.
- Code: `crates/ff-desktop/src/menu_workspace/defaults.rs`,
  `crates/ff-desktop/src/help_context.rs`,
  `crates/ff-desktop/src/scrm_viewer_panel.rs`,
  `crates/ff-desktop/src/tab_state_ctors.rs`,
  `crates/ff-desktop/src/shell/{commands_ladder_a.rs, commands_ladder_b.rs,
  help.rs}`,
  `crates/ff-help/src/{lib.rs, plugin_help.rs, navigation.rs, help_panel.rs}`,
  and the crate listings for `ff-help` / `ff-md-viewer` / `ff-scrm` /
  `ff-screen-model`.
- `.kiro/steering/wiring-standard.md` (surfacing-via-single-command-path rule).

Content from steering, specs, and status docs was rephrased/summarised for
compliance with licensing restrictions; see the originals for authoritative
wording.
