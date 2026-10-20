# Current Work Dashboard

Use this page first to see what the project is working on. Keep the status
summary short and link to the detailed task list.

## Status key

| Status | Meaning |
|--------|---------|
| `ACTIVE` | Work is currently being implemented |
| `NEXT` | Ready to start after the active item |
| `BLOCKED` | Cannot proceed until a dependency or decision is resolved |
| `DONE` | Implemented and validated |
| `DEFERRED` | Deliberately outside the current release |
| `SUPERSEDED` | Replaced by a later decision; see the linked source |

## Work areas

| Area | Status | Current focus | Detailed tracking |
|------|--------|---------------|-------------------|
| Phase BR -- Requirements Maintenance | DONE | CA-01/CA-02 annotations, FFW-JES rename, B009/CR-NR-035 housekeeping | [project-master tasks](../project-management/project-master/tasks.md) |
| Phase BT -- Cross-File Search and Replace | DONE | GlobalReplaceEngine, Replace UI, search history | [global-search tasks](../specs/global-search/tasks.md) |
| Phase BS-A -- Workspace Model | DONE | WorkspaceState, session persistence, lifecycle commands, root management | [workspace-model tasks](../specs/workspace-model/tasks.md) |
| Phase BS-B -- Command Palette | DONE | Fuzzy engine, palette state, rendering, Ctrl+Shift+P activation | [command-palette tasks](../specs/command-palette/tasks.md) |
| Phase BS-C -- Global Search | DONE | ff-global-search crate, Search Results panel, Ctrl+Shift+F, GSEARCH command | [global-search tasks](../specs/global-search/tasks.md) |
| Phase CO -- Accessibility, Plugin Manager UI, Notification System | DONE | All 7 deliverables complete | [project-master tasks](../project-management/project-master/tasks.md) |
| Phase CP -- Batch Command Execution | DONE | All 11 deliverables complete | [project-master tasks](../project-management/project-master/tasks.md) |
| Phase W.5 -- Generic ToolchainPlugin Trait Validation | DONE | MockToolchain test double, trait audit, CI constraint | [compiler-toolchain-integration tasks](../specs/compiler-toolchain-integration/tasks.md) |
| Phase CQ -- Enterprise Features | DONE | audit-logging, settings export/import, locked config keys | [project-master tasks](../project-management/project-master/tasks.md) |
| Phase CR -- OS Theme Follow + Macro Library | DONE | OS dark/light follow, Macro Library panel | [project-master tasks](../project-management/project-master/tasks.md) |
| Phase CS -- Test Warning Cleanup | DONE | All cargo test warnings eliminated across workspace | [project-master tasks](../project-management/project-master/tasks.md) |
| Phase CT -- Workbench/Workspace/Context Terminology | DONE | All 7 tasks complete -- 69 specs updated | [project-master tasks](../project-management/project-master/tasks.md) |
| Phase CU -- Menu Workspace Pattern | DONE | All 8 implementation tasks complete -- TabKind, loader, hot-reload, render, dispatch, defaults | [project-master tasks](../project-management/project-master/tasks.md) |
| Phase CV -- POM Redesign | SUPERSEDED | Menu Workspace pattern merged into `main`; Settings menu reworked under CR-CH-025. Forward CORE path now driven by ROADMAP Section 6a, not CV/CW task numbering | [ROADMAP](../project-management/ROADMAP.md) |
| Phase CW -- Settings Menu | SUPERSEDED | Settings now opens as a real Menu_Workspace via keyword-less Menu_Name resolution (CR-CH-025, see bug B032 FIXED); flat browser moved to `CONFIG [<namespace>]` | [ROADMAP](../project-management/ROADMAP.md) |
| Phase CX -- Named Workspaces + KEYS + SPLIT | DONE | Spec + implementation complete (CX.1-CX.7) | [project-master tasks](../project-management/project-master/tasks.md) |
| Phase CZ -- FFTest Script Suite | DONE | All 9 deliverables complete | [project-master tasks](../project-management/project-master/tasks.md) |
| Deferred connectors | DEFERRED | Network, FTP/SFTP, mainframe, and cloud connectors | [connector specs](../specs/) |

## Before starting work

1. Choose one feature from the table above.
2. Read its `requirements.md` and `design.md`.
3. Update its `tasks.md` before implementing.
4. Record any new source material in
   [`../requirements/source-register.md`](../requirements/source-register.md).
5. Update this dashboard when the active focus changes.

## Active work item

**Authoritative source:** the single active phase and the ordered forward plan
live in [`../project-management/ROADMAP.md`](../project-management/ROADMAP.md)
(the control tower, Sections 6 and 6a). This dashboard summarises it; when the
two disagree, the ROADMAP wins. The earlier "Phase CV-impl / POM Redesign" focus
recorded here has been SUPERSEDED by the 2026-10-11 CORE re-sequencing.

**Current focus:** Phase CORE STABILISATION + TEST. Get the six CORE capabilities
working and sign off `../project-management/core-acceptance-test-plan.md` before
resuming plugin work. The re-sequenced CORE forward path (ROADMAP Section 6a) is:

1. **Localisation Phase 2** (CR-NR-103 L10N.4) -- full UI-string extraction +
   Settings locale picker. Pulled to the FRONT (owner decision) because it is a
   large, mechanical, cross-cutting change; doing it first means every panel built
   afterward is written localisation-aware. Starting point: the WIP branch
   `feature/localization-phase2`. Run in its own worktree.
2. **Volume UI** (CR-CH-057 / CR-NR-105 VM.6-VM.8 / RC.C.10) -- Volume management
   WorkspaceContext, Define_Volume_Dialog, DEFINE VOLUME / vary / ALTER via the
   single command path, volume picker in catalog creation, IDCAMS VOLUMES().
3. **Volume in the "Reset Bare" POM**, then **Catalog UI rework**
   (catalog-as-pure-locator), then **ff-vfs Volume/Catalog UI-path wiring**, then
   **File Navigator dataset/volume duality** (CR-NR-060 Slice B).
4. **Editor windowed-record F3-F5** (CR-CH-058 WRF.4-6) -- runs in PARALLEL with
   steps 2-3 (depends only on F1/F2, already merged). Starting point: the WIP
   branch `feature/windowed-record-f3`.
5. **CORE #4 editor test pass against real datasets** -- the convergence /
   sign-off milestone (needs the navigator opening dataset/volume files AND the
   editor finished).

Plugin work (Utilities, mainframe emulation, File Formatter, Database, Job
Monitor) stays DEFERRED until CORE is signed off.

### Repository state (verified 2026-10-16)

- **`main`** is the single source of truth, tip `8ada2dc`, 1 commit ahead of
  `origin/main` (unpushed). The project was relocated to
  `c:\workspace\apps\FileForgeWorkbench` -- see
  [`../../.agents/POST-REORG-VERIFY.md`](../../.agents/POST-REORG-VERIFY.md).
- **Two unmerged WIP branches** (git-confirmed `--no-merged main`):
  `feature/windowed-record-f3` and `feature/localization-phase2` (both
  "pre-stall checkpoint" commits). These are the starting points for steps 1 and
  4 above. (The ROADMAP's "all non-main branches 0 ahead" line predates these.)
- **Gate status:** last fully clean FULL gate was 2026-10-11 (9866 tests, 0
  failed). The first gate run after the relocation reported 3 failures in
  `ff-governance-tests::architecture_compliance` (`all_governed_crates_exist`,
  `dataset_allocator_has_no_idcams_dependency`, `vfs_has_no_domain_dependencies`).
  Root cause: a STALE cached `.rlib` from the old `c:\workspace\VSC\...` path --
  `env!("CARGO_MANIFEST_DIR")` was baked into the pre-move artifact, so the tests
  asserted the dead path. NOT a code regression. Resolved by
  `cargo clean -p ff-governance-tests` + rebuild (all 8 pass). A fresh FULL gate
  from `main` should be run to re-confirm green in the new location before new
  work begins.

### Resume-from-reorg checklist (do before new feature work)

- [ ] Run a fresh FULL gate from `main`; confirm 9866/0 in the new location.
- [ ] `git worktree prune` to clear the dead old-path worktree registrations.
- [ ] Delete the stray `git\` PortableGit folder at the repo root (binary cruft).
- [ ] Resolve the uncommitted `.kiro/steering/` changes (commit or revert the
      in-flight steering refactor).

### Phase CQ -- Enterprise Features (next)

| Deliverable | Spec | Status |
|-------------|------|--------|
| Requirements gate | configuration-system | [x] CQ.1 |
| Audit logging implementation | Structured audit trail | [x] CQ.2 |
| Settings export/import | Save/load user config as portable TOML | [x] CQ.3 |
| Locked config keys | Admin-enforced settings | [x] CQ.4 |
| Integration tests + TCR | All new criteria | [x] CQ.5 |

### Phase CR -- OS Theme Follow + Macro Library Management (next)

| Deliverable | Spec | Status |
|-------------|------|--------|
| Requirements gate | theme-and-appearance Req 16, lua-macro-engine Req 12 | [x] CR.1 |
| OS dark/light mode follow | theme.follow_os config key + frame detection | [x] CR.2 |
| Macro Library panel | TabKind::MacroLibrary, MACROS/=6, list/run/edit/delete | [x] CR.3 |
| Integration tests + TCR | All new criteria | [x] CR.4 |

### Phase CO -- Accessibility, Plugin Manager UI, and Notification System -- COMPLETE

| Deliverable | Spec | Status |
|-------------|------|--------|
| Requirements gate (3 new sub-projects) | accessibility, plugin-manager-ui, notification-system | [x] CO.1-CO.3 |
| Accessibility implementation | WCAG AA, keyboard-only, focus indicators | [x] CO.4 |
| Plugin Manager UI | POM option 8 panel | [x] CO.5 |
| Notification System | Toast + event log | [x] CO.6 |
| Integration tests + TCR | All new criteria | [x] CO.7 |

### Phase CP -- Batch Command Execution -- COMPLETE

**Test count at completion:** 731 passing, 0 failures.

### Phase BS -- Productivity Core -- COMPLETE

| Sub-project | Spec | Tasks | Status |
|-------------|------|-------|--------|
| Workspace Model | [workspace-model](../specs/workspace-model/requirements.md) | BS-A.1 to BS-A.6 | DONE |
| Command Palette | [command-palette](../specs/command-palette/requirements.md) | BS-B.1 to BS-B.4 | DONE |
| Global Search | [global-search](../specs/global-search/requirements.md) | BS-C.1 to BS-C.5 | DONE |

**Test count at completion (Phase BS):** 655 passing, 0 failures.

### Suggested next steps

- Review open bugs in [bugs.md](bugs.md) for a bug-fix sprint.
- Review [change-log.md](change-log.md) for any pending change requests.
- Plan the next feature phase based on the [executive assessment](../reviews/requirements-review/executive-assessment.md).

## Project records

- [Bug register](bugs.md)
- [Change log](change-log.md)
