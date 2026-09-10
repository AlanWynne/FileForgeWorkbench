# Analysis Record: startup-and-session (W3.9)

- **Wave**: 3 (Shell, commands, menus, session)
- **Backing crate**: `ff-session`
- **Spec files**: requirements.md (564 lines, 16 requirements -- numbered
  1-11, 14, 13, 19, 20, 21; 12/15-18 renumbered away, appended out of order),
  tasks.md (68 sub-tasks, all `[x]`), design.md present
- **Analysed**: Wave 3 pass

---

## 1. Split candidacy -- WATCH (pre-flagged), owner-gated

The checklist pre-flag "split candidate (388)" is a stale count; the spec is 564
lines / 16 reqs. Assessment against the 2-of-4 rule:

- Requirement volume: 16 reqs, 564 lines. WELL above thresholds. (signal 1)
- Responsibilities: SEVEN named concerns bundled -- startup sequence ordering (Req 1),
  config-loading orchestration (Req 2), user-data-dir init (Req 3), session
  persist/restore (Req 4/5/8/21), CLI args (Req 6), empty-startup (Req 7), exit
  sequence (Req 9), crash recovery (Req 10), graceful degradation (Req 11). There
  are two natural clusters: (a) LIFECYCLE (startup/exit/crash-recovery/degradation/
  CLI) and (b) SESSION STATE persistence/restore (session_state 529, session_file,
  session_restore, window_geometry, workspace descriptors, recent_files). (signal 2)
- Crates: single crate `ff-session` (15 files); deps ff-config + ff-logging only.
- Cohesion: medium -- lifecycle and session-state share the startup flow but are
  distinct concerns (session_state.rs alone is 529 lines).

Meets 2-of-4 (volume + separable responsibilities). Recorded PA-SPLIT-011 (owner-gated,
MEDIUM): consider splitting `session-state` (the persisted SessionState/Workspace-
Descriptor model + restore) from `startup-lifecycle` (startup ordering / config
orchestration / exit / crash-recovery / degradation / CLI). The 529-line
session_state.rs is the natural extraction. Owner decision; not forced.

### Source file size violation (PA-STD-040)

`session_state.rs` = 529 non-test lines, over the 400 cap (the persisted
SessionState + WorkspaceDescriptor + TabState model). Split feeds PA-SPLIT-011.
REFACTOR, no gate. Recorded PA-STD-040.

---

## 2. Cross-unit consistency

### PA-WATCH-012 (catalog-persistence layers) -- REFINED, three-layer separation

The W2.2 watch (do `[virtual_catalogs]` (VCM) and `[catalog].mounted_catalogs`
(ff-dscatalog) compete?) is REFINED by W3.9: ff-session adds a THIRD, DISTINCT
persistence layer -- it persists WHICH WORKSPACES/TABS are open via
`WorkspaceDescriptor` + `PersistedTabKind` (including `VirtualCatalogManager` (Files
Panel) and `FileExplorerPanel` (POM option 2) tab kinds). This is a CLEAN
separation of concern, NOT a fourth competing catalog store:
- ff-session persists TAB/WORKSPACE identity (which panels are open + their
  geometry/selection) via descriptors.
- VCM `[virtual_catalogs]` persists the catalog REGISTRY (names/types/paths).
- ff-dscatalog `[catalog].mounted_catalogs` persists mainframe MOUNT runtime state.

Restore flow: ff-session restores a VCM tab descriptor -> VCM restores its registry
-> ff-dscatalog remounts. Three layers, one direction. Recorded PA-WATCH-012 REFINED:
the three are complementary (tab vs registry vs mount), but confirm the RESTORE
ORDERING at startup (Req 5 session restore + Req 2 config orchestration) does not
double-restore or race the catalog registries. Coordination, not a conflict.

### Scope-creep reqs (13/14/19/20/21) -- persistence vs functionality split

Like menu-and-statusbar (PA-WATCH-020), ff-session's spec carries CR-appended reqs
that touch other units' functionality: Req 14 (ISPF POM + Tabbed Window Container),
Req 13 (Desktop Shell Editor Interactions), Req 19 (File Explorer Context POM
Option 2), Req 20 (TSO Session Lifecycle: LOGOFF/TIME/STATUS routing), Req 21
(Descriptor-Based Persistence of Visible Workspaces). UNLIKE menu-and-statusbar,
ff-session DOES have relevant code (18 refs) -- but the boundary is: ff-session owns
the PERSISTENCE of these workspace kinds (descriptors), while the FUNCTIONALITY is
owned elsewhere (POM/File-Explorer = virtual-catalog-manager W2.2 + menu-workspace
W3.5; TSO lifecycle = shell/command-semantics). Recorded PA-WATCH-023 (LOW):
confirm Req 13/14/19/20 describe ff-session's PERSISTENCE/ROUTING slice only (not
duplicate functionality); the POM/File-Explorer/TSO functionality lives in the
owning units. Persistence (Req 21 descriptors) is legitimately ff-session's.

### Config-loading orchestration (Req 2) -- ff-session drives ff-config init

Req 2: ff-session ORCHESTRATES configuration loading (the ordered startup flow).
It deps ff-config and owns the init SEQUENCE (which layers load when). Consistent
with configuration-system (W0.3) as the config OWNER; ff-session is the startup
ORCHESTRATOR. Clean (this is the two-phase-init pattern noted in W0.5 logging).

### Public types and ownership

- SessionState, WorkspaceDescriptor, PersistedTabKind, TabState, LayoutSnapshot,
  WindowGeometryState, RecentFiles, DescriptorParams, crash-recovery + degraded-mode
  types -- sole-owned by `ff-session`. No duplication.
- 37 fs calls -- LEGITIMATE: ff-session OWNS the on-disk session/recovery/geometry/
  recent-files persistence (it is the persistence layer). Not an FFW-ARCH-001 issue
  (session state is workbench config, not documents; and there is no "session VFS
  provider" -- ff-session is the terminal writer). Note: these are user-data files,
  another (largest) member of the local-store family (PA-WATCH-019 adjacent) but
  legitimately raw.

### Cross-reference integrity

Cross-refs resolve. No dangling refs. Reqs 13/14/19/20 imply cross-unit boundaries
(PA-WATCH-023).

---

## 3. Completeness

Tracking: all 68 sub-tasks `[x]`. Implementation present across all 16 reqs
(startup ordering, config orchestration, user-data-dir, session persist/restore,
CLI, empty-start, window geometry, exit, crash recovery, graceful degradation,
workspace descriptors). Tests in-file. No PA-INCOMPLETE. Complete.

### TCR (adequate)

27 TCR rows for 16 reqs -- adequate. No PA-TCR raised.

---

## 4. Logging audit -- notable gap on a fault-tolerance subsystem

Scan of `crates/ff-session/src` (recursive):

- `ff_logging` / `log_*!`: 0
- `ff-logging` Cargo dep: PRESENT -- 0 uses. DEAD.
- `println!` / `eprintln!`: 0
- `std::fs`: 37 (session/recovery/geometry/recent-files persistence -- legitimate)

This is the MOST NOTABLE dead-ff-logging gap so far: ff-session is the
FAULT-TOLERANCE subsystem -- STARTUP ordering, CRASH RECOVERY (Req 10), GRACEFUL
DEGRADATION (Req 11: "no single corrupt/missing file prevents startup"). Its
error.rs has exactly the events that MUST be logged for a degraded/recovered
startup to be diagnosable: user-data-dir unavailable, session-file corrupt, session
write failed, plugin-init failed, recovery-file scan/corrupt. ALL returned as
`SessionError`, NONE logged. A workbench that silently skips a corrupt file
(graceful degradation) with NO log record is very hard to support. Recorded
PA-LOG-026 (MEDIUM-HIGH): wire ff-logging across the startup/recovery/degradation
paths -- WARN on each skipped/corrupt/missing file (Req 11 degradation events),
ERROR on unrecoverable failures, INFO on the startup-sequence milestones + crash-
recovery actions (Req 10). This is a high-value operational-logging target (second
only to ff-dscatalog PA-LOG-012 in Waves 2-3).

---

## 5. Task revision proposals

- **PA-SPLIT-011 (owner-gated, MEDIUM)**: split `session-state` (SessionState +
  WorkspaceDescriptor + restore, session_state.rs 529) from `startup-lifecycle`
  (startup/config-orchestration/exit/crash-recovery/degradation/CLI). Owner decision.
- **PA-STD-040 (REFACTOR)**: split `session_state.rs` (529). Feeds PA-SPLIT-011.
- **PA-LOG-026 (MEDIUM-HIGH)**: wire ff-logging (dead dep) across startup/crash-
  recovery/graceful-degradation -- WARN on skipped/corrupt/missing files (Req 11),
  ERROR on unrecoverable, INFO on startup milestones + recovery (Req 10). High-value:
  a silently-degrading startup is undebuggable without it. Depends on Phase PA-W0.1.
- **PA-STD-041 (ASCII, runtime strings)**: 89 non-ASCII bytes (9 non-comment) --
  em-dashes inside runtime `#[error]` strings (error.rs, 6+ messages). Replace with
  `--`. REFACTOR, no gate.
- **PA-WATCH-012 (REFINED)**: confirm the startup RESTORE ORDERING across the three
  catalog-persistence layers (ff-session tab descriptors -> VCM registry ->
  ff-dscatalog mounts) does not race/double-restore. Coordination.
- **PA-WATCH-023 (LOW)**: confirm Reqs 13/14/19/20 are ff-session's PERSISTENCE/
  routing slice, not duplicate POM/File-Explorer/TSO functionality (owned by VCM/
  menu-workspace/shell).

No requirement CHANGE proposed; the spec is complete and internally consistent
(modulo the persistence-vs-functionality boundary of Reqs 13/14/19/20).

---

## Summary

startup-and-session (`ff-session`) owns the application lifecycle (startup ordering,
config orchestration, exit, crash recovery, graceful degradation, CLI) AND session
persistence (SessionState + WorkspaceDescriptor + restore). 16 reqs / 564 lines --
a broad spec that is a MEDIUM split candidate (PA-SPLIT-011: session-state vs
startup-lifecycle; session_state.rs 529 over cap PA-STD-040). PA-WATCH-012 is
REFINED as a clean THREE-LAYER separation (ff-session persists WHICH tabs are open;
VCM the catalog registry; ff-dscatalog the mounts) -- confirm restore ordering. The
37 fs calls are LEGITIMATE (ff-session is the terminal session-persistence layer).
The headline is PA-LOG-026 (MEDIUM-HIGH): a FAULT-TOLERANCE subsystem (crash
recovery + graceful degradation) with a DEAD ff-logging dep and rich error-typed
degradation paths that log NOTHING -- a silently-degrading startup is undebuggable;
this is the second-highest-value logging target in Waves 2-3 after ff-dscatalog.
Minor: runtime-string ASCII (PA-STD-041), and a persistence-vs-functionality
scope-boundary watch on Reqs 13/14/19/20 (PA-WATCH-023). Complete tracking (68/68),
adequate TCR (27).
