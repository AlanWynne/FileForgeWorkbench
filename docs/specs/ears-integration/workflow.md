# EARS Integration Workflow
# Merging ISPF and TSO/SDSF Source Requirements into FileForge Workbench

## Purpose

This document is the master workflow and task list for integrating the EARS
requirements extracted from IBM manuals into the FileForge Workbench formal
requirements system. It governs the entire process from analysis through to
implementation planning.

No source file outside `docs/` may be created or modified until the analysis
and gate phases defined here are complete.

---

## Source Material

| Folder | Files | Requirement IDs |
|--------|-------|-----------------|
| `docs/source-documents/ispf-ears/` | 12 files | ISPF edit session, profile, line commands, primary commands, find/change, recovery/undo, syntax highlighting, boundaries/tabs/masks, sequence numbers, POM/navigation, macros, hex display |
| `docs/source-documents/tso-ears/` | 10 files | TSO session/logon, ISPF panel navigation, TSO commands (ALLOCATE through PRINTDS), SDSF panel framework, SDSF job queue panels, SDSF filter/sort/search, SDSF log/system panels, SDSF browse/print, REXX scripting, SDSF REXX interface |

---

## Sub-Project Mapping

The EARS requirements map to these existing FileForge Workbench sub-projects:

| EARS Source Area | Primary Sub-Project | Secondary Sub-Projects |
|-----------------|---------------------|------------------------|
| ISPF edit session lifecycle | `edit-operations` | `startup-and-session` |
| ISPF edit profile and modes | `edit-operations` | `configuration-system` |
| ISPF line commands | `line-commands` | `edit-operations` |
| ISPF primary commands | `command-semantics` | `find-and-replace`, `navigation-commands` |
| ISPF find/change/search strings | `find-and-replace` | `command-semantics` |
| ISPF edit recovery and undo | `undo-redo-transactions` | `edit-operations` |
| ISPF syntax highlighting (HILITE) | `syntax-highlighting` | `language-service` |
| ISPF boundaries, tabs, masks | `tabs-and-mask` | `edit-operations`, `viewport-and-scrolling` |
| ISPF sequence numbers | `sequence-numbers` | `edit-operations` |
| ISPF POM and navigation | `menu-and-statusbar` | `navigation-commands`, `function-keys-and-history` |
| ISPF edit macros | `lua-macro-engine` | `command-framework`, `line-commands` |
| ISPF hex display | `hex-display` | `edit-operations` |
| TSO session and logon | `startup-and-session` | `menu-and-statusbar` |
| ISPF panel navigation (TSO) | `menu-and-statusbar` | `navigation-commands`, `function-keys-and-history` |
| TSO commands (ALLOCATE-PRINTDS) | `command-semantics` | `dataset-catalog`, `dataset-allocator`, `FFW-JES` |
| SDSF panel framework | `FFW-JES` | `menu-and-statusbar`, `layout-and-docking` |
| SDSF job queue panels | `FFW-JES` | `dataset-catalog`, `virtual-file-system` |
| SDSF filter/sort/search | `FFW-JES` | `record-selection-criteria`, `exclude-show-filter`, `find-and-replace` |
| SDSF log and system panels | `FFW-JES` | `logging-subsystem`, `menu-and-statusbar` |
| SDSF browse and print | `FFW-JES` | `custom-file-viewers`, `file-operations` |
| REXX scripting | `lua-macro-engine` | `command-framework`, `workflow-engine` |
| SDSF REXX interface | `FFW-JES` | `lua-macro-engine` |

---

## Priority Classification (from source documents)

### P1 -- Core (must have for mainframe workstation experience)
- All ISPF edit session, profile, line commands, primary commands, find/change
- ISPF POM and navigation, sequence numbers, hex display, boundaries/tabs/masks
- TSO session startup, PF keys, scrolling
- ISPF panel types and hierarchy, LOCATE, RETRIEVE
- SDSF panel layout, action characters, main panel, job queue panels (I/O/H/ST/DA)
- SDSF PREFIX/OWNER/DEST/FILTER/SORT, FIND, LOCATE
- SDSF system log, user log, browse job output
- TSO commands: ALLOCATE, FREE, DELETE, RENAME, LISTCAT, LISTDS, LISTALC, SUBMIT, STATUS
- TSO EDIT command and subcommands
- SDSF session persistence

### P2 -- Enhanced (should have)
- ISPF edit recovery/undo, syntax highlighting, edit macros
- ISPF split screen
- SDSF overtype fields, help system, ARRANGE, SET DISPLAY
- SDSF log/scroll commands (LOG, NEXT/PREV, SNAPSHOT)
- SDSF system info panels (SYS, DASH, INIT, JC, SP)
- SDSF SET commands (BCOLOR through SCREEN)
- SDSF browse settings, print, show columns
- TSO commands: OUTPUT, CANCEL, SEND, PROFILE, PRINTDS
- REXX scripting (REXX-1 through REXX-5)

### P3 -- Advanced (nice to have)
- SDSF JES/WLM panels (MAS, JG, SRVC, SE)
- SDSF REXX interface (ISFCALLS, ISFEXEC, ISFACT, ISFBROWSE, ISFSLASH, ISFGET, ISFLOG)
- SDSF special DDNames

---

## Source Documents Under Rationalisation

In addition to the EARS source files, the following design document must be
rationalised before any requirements.md files are modified:

| Document | Nature | Status |
|----------|--------|--------|
| `docs/source-documents/FileForgeWorkbench_MiniX_FTSO_Command_Environment_Design.md` | Architecture proposal -- ISPF Option 6-style command shell (FTSO) and portable mainframe service layer (MiniX). Written in isolation before the TSO-EARS files were available. | Requires rationalisation against EARS ground truth and existing specs before use. |

This document is an input to Phase EI-0. It is NOT authoritative. The TSO-EARS
files and the existing committed specs take precedence wherever they conflict.

---

## Workflow Phases

### Phase EI-0: MiniX/FTSO Rationalisation -- Resolve Before All Other Phases

**Goal:** Reconcile the MiniX/FTSO design proposal against the TSO-EARS ground
truth and the existing committed specs. Produce a single resolution map that
defines what the design document contributes, what it duplicates, and what it
contradicts. This map becomes the input to EI-1 through EI-5.

**Why this must come first:**
The MiniX/FTSO document was written without knowledge of the TSO-EARS files.
Several of its proposals are already covered by existing specs
(`command-framework`, `shell-command`, `FFW-JES`, `lua-macro-engine`). If EI-1
through EI-5 proceed without resolving these overlaps, requirements will be
duplicated or contradicted across multiple specs.

**Key conflicts to resolve:**

| MiniX/FTSO proposal | Existing coverage | Resolution needed |
|---------------------|-------------------|-------------------|
| FTSO command shell (interactive terminal, history, sessions, PF keys) | `shell-command` Req 1-18 already defines SHELL/TSO interactive terminal, output panel, async execution, cancellation, security | Decide: extend `shell-command` or create `ftso-command-shell` sub-project |
| MiniX service layer (catalogue, dataset, GDG, VSAM, JES, spool, security, audit) | `ff-dscatalog`, `ff-dsalloc`, `ff-jes`, `ff-vfs`, `ff-lua`, `ff-command`, `ff-workflow` already exist | MiniX is an integration label, not a new crate -- confirm no new sub-project needed |
| FTSO command dispatcher and registry (CommandDescriptor, metadata-driven) | `command-framework` Req 1-7 already defines Command_Registry, Command_Dispatch, metadata, scripting bridge, history | Decide: FTSO dispatcher IS `ff-command`; no duplication |
| FTSO command categories A-J (LISTCAT, ALLOC, SUBMIT, etc.) | TSO-EARS files define authoritative behaviour for these commands | EARS files are ground truth; FTSO categories are organisational only |
| HOST command (explicit OS boundary) | `shell-command` Req 1-2 defines SHELL/TSO with `shell.mode` security control | HOST and SHELL/TSO are the same concept -- unify naming |
| REXX scripting (3-phase strategy: FFCMD -> embedded -> REXX compat) | TSO-EARS REXX-1 through REXX-5 define concrete REXX execution requirements; `lua-macro-engine` is the scripting bridge | EARS files define what REXX must do; scripting strategy must align |
| SDSF commands (SDSF, SPOOL VIEW, JES STATUS) | TSO-EARS SDSF files define authoritative SDSF behaviour; `FFW-JES` is the target spec | EARS files are ground truth; FTSO SDSF proposals are superseded |
| MiniX naming (risk of confusion with MINIX OS) | No existing spec uses this name | Use as internal architecture label only -- never user-facing |

**Tasks:**

- [x] EI-0.1 Read `shell-command/requirements.md` and `command-framework/requirements.md` in full -- confirm overlap with FTSO shell and dispatcher proposals
- [x] EI-0.2 Produce `docs/specs/ears-integration/minix-ftso-reconciliation.md` -- a resolution table covering every section of the MiniX/FTSO document: ALREADY COVERED / EXTENDS / CONFLICTS / GENUINELY NEW
- [x] EI-0.3 Decide the FTSO sub-project question: extend `shell-command` -- no new sub-project (Decision D2 in reconciliation)
- [x] EI-0.4 Decide the MiniX naming question: internal-label-only -- no new sub-project (Decision D1 in reconciliation)
- [x] EI-0.5 Produce `docs/specs/ears-integration/source-of-truth-map.md` -- for each EARS requirement area: authoritative source (EARS file), existing spec coverage, MiniX/FTSO design input (section), resolution
- [x] EI-0.6 Present reconciliation and source-of-truth map to user for approval before EI-1 begins

**Output:**
- `docs/specs/ears-integration/minix-ftso-reconciliation.md`
- `docs/specs/ears-integration/source-of-truth-map.md`

**Gate:** EI-0.6 must be approved before EI-1 starts. No requirements.md files
are modified during EI-0.

---

### Phase EI-1: Gap Analysis -- What Already Exists

**Goal:** For each EARS requirement area, determine what is already covered by
existing requirements.md files vs what is genuinely new.

**Tasks:**

- [x] EI-1.1 Read `edit-operations/requirements.md` -- map against ISPF edit session (01), profile (02), line commands (03), primary commands (04)
- [x] EI-1.2 Read `find-and-replace/requirements.md` -- map against ISPF find/change (05)
- [x] EI-1.3 Read `undo-redo-transactions/requirements.md` -- map against ISPF recovery/undo (06)
- [x] EI-1.4 Read `syntax-highlighting/requirements.md` -- map against ISPF HILITE (07)
- [x] EI-1.5 Read `tabs-and-mask/requirements.md` -- map against ISPF boundaries/tabs/masks (08)
- [x] EI-1.6 Read `sequence-numbers/requirements.md` -- map against ISPF sequence numbers (09)
- [x] EI-1.7 Read `menu-and-statusbar/requirements.md` -- map against ISPF POM/navigation (10) and TSO panel navigation
- [x] EI-1.8 Read `lua-macro-engine/requirements.md` -- map against ISPF macros (11) and REXX scripting
- [x] EI-1.9 Read `hex-display/requirements.md` -- map against ISPF hex display (12)
- [x] EI-1.10 Read `startup-and-session/requirements.md` -- map against TSO session/logon
- [x] EI-1.11 Read `function-keys-and-history/requirements.md` -- map against TSO PF keys and RETRIEVE
- [x] EI-1.12 Read `FFW-JES/requirements.md` -- map against all SDSF requirements
- [x] EI-1.13 Read `command-semantics/requirements.md` -- map against TSO commands
- [x] EI-1.14 Read `line-commands/requirements.md` -- map against ISPF line commands (03)
- [x] EI-1.15 Read `navigation-commands/requirements.md` -- map against ISPF LOCATE/scroll

**Output:** `docs/specs/ears-integration/gap-analysis.md`

---

### Phase EI-2: Coverage Classification

**Goal:** Classify every EARS requirement as one of:
- COVERED -- already in requirements.md with equivalent criterion
- PARTIAL -- partially covered, needs extension
- NEW -- not covered at all, needs new criterion
- OUT-OF-SCOPE -- not applicable to a desktop tool (e.g. z/OS-specific hardware)
- DEFERRED -- applicable but deliberately deferred (P3 items)

**Tasks:**

- [x] EI-2.1 Classify all 12 ISPF-EARS files (approx 200 criteria)
- [x] EI-2.2 Classify all TSO session + panel navigation criteria (TSO-1 through ISPF-5)
- [x] EI-2.3 Classify all TSO command criteria (TSO-CMD-1 through TSO-EDIT-3)
- [x] EI-2.4 Classify all SDSF panel framework criteria (SDSF-1 through SDSF-5, SET commands, persistence)
- [x] EI-2.5 Classify all SDSF job queue panel criteria (SDSF-JQ-1 through SDSF-JQ-7)
- [x] EI-2.6 Classify all SDSF filter/sort/search criteria
- [x] EI-2.7 Classify all SDSF log/system panel criteria
- [x] EI-2.8 Classify all SDSF browse/print criteria
- [x] EI-2.9 Classify all REXX and SDSF REXX criteria

**Output:** `docs/specs/ears-integration/coverage-classification.md`

---

### Phase EI-3: Incomplete Work Audit

**Goal:** Identify all pending/incomplete work in the current task lists and
requirements that needs to be reorganised before new requirements are added.

**Tasks:**

- [x] EI-3.1 List all `[ ]` tasks in project-master/tasks.md (BS.8-BS.15, BU.1-BU.9, BV.1)
- [x] EI-3.2 For each pending phase, read its sub-project tasks.md and confirm task list is current
- [x] EI-3.3 Identify any requirements in existing specs that have no corresponding tasks (gaps in tasks.md)
- [x] EI-3.4 Identify any tasks that reference requirements that no longer exist or have been renumbered
- [x] EI-3.5 Identify any TCR.md rows that are NOT COVERED but have no corresponding task

**Output:** `docs/specs/ears-integration/incomplete-work-audit.md`

---

### Phase EI-4: Integration Plan

**Goal:** Produce a concrete plan for which NEW/PARTIAL requirements go into
which sub-project requirements.md files, in what order, and what new phases
they create in project-master/tasks.md.

**Rules:**
- P1 NEW requirements get their own gate and phase immediately
- P2 NEW requirements get their own gate and phase, sequenced after P1
- P3 requirements are logged as DEFERRED in a new `deferred-requirements.md`
- OUT-OF-SCOPE requirements are documented with rationale
- PARTIAL requirements are handled as CHANGE REQUESTs to existing criteria

**Tasks:**

- [x] EI-4.1 Group all NEW P1 requirements by sub-project; assign phase labels (BW onwards)
- [x] EI-4.2 Group all NEW P2 requirements by sub-project; assign phase labels
- [x] EI-4.3 Group all PARTIAL requirements by sub-project; list as change requests
- [x] EI-4.4 List all OUT-OF-SCOPE requirements with rationale
- [x] EI-4.5 List all DEFERRED (P3) requirements
- [x] EI-4.6 Produce ordered phase sequence: which phases depend on which

**Output:** `docs/specs/ears-integration/integration-plan.md`

---

### Phase EI-5: Gate Execution (per sub-project batch)

**Goal:** Execute the full requirements gate for each batch of new requirements,
sub-project by sub-project. Each batch follows the standard gate sequence:

```
1. requirements.md updated (new criteria appended)
2. design.md updated (if architectural changes needed)
3. tasks.md updated (new tasks appended)
4. project-master/tasks.md updated (new phase added)
5. TCR.md updated (NOT COVERED rows added)
6. change-log.md updated
```

**Batches (to be executed in order after EI-4 is approved):**

- [x] EI-5.1 Batch 1: edit-operations (ISPF edit session, profile, line commands -- P1 gaps)
- [x] EI-5.2 Batch 2: find-and-replace (ISPF find/change string types -- P1 gaps) -- skipped: 0 new/partial criteria
- [x] EI-5.3 Batch 3: line-commands (ISPF line command completeness -- P1 gaps)
- [x] EI-5.4 Batch 4: sequence-numbers (ISPF sequence number completeness -- P1 gaps)
- [x] EI-5.5 Batch 5: hex-display (ISPF hex display completeness -- P1 gaps) -- skipped: 0 new/partial criteria
- [x] EI-5.6 Batch 6: tabs-and-mask (ISPF boundaries/tabs/masks -- P1 gaps) -- skipped: 0 new/partial criteria
- [x] EI-5.7 Batch 7: menu-and-statusbar (ISPF POM/navigation + TSO panel types -- P1 gaps)
- [x] EI-5.8 Batch 8: startup-and-session (TSO session startup -- P1 gaps)
- [x] EI-5.9 Batch 9: function-keys-and-history (TSO PF keys, RETRIEVE -- P1 gaps) -- skipped: 0 new/partial criteria
- [x] EI-5.10 Batch 10: command-semantics (TSO commands P1: ALLOCATE through STATUS)
- [x] EI-5.11 Batch 11: FFW-JES (SDSF panel framework, job queues, filter/sort -- P1)
- [x] EI-5.12 Batch 12: undo-redo-transactions (ISPF recovery/undo -- P2)
- [x] EI-5.13 Batch 13: syntax-highlighting (ISPF HILITE -- P2)
- [x] EI-5.14 Batch 14: lua-macro-engine (ISPF macros + REXX -- P2)
- [x] EI-5.15 Batch 15: FFW-JES (SDSF system panels, browse, print, SET commands -- P2)
- [x] EI-5.16 Batch 16: command-semantics (TSO commands P2: OUTPUT through PRINTDS)

---

### Phase EI-6: Reorganised Master Task List

**Goal:** After all gates are executed, produce a clean reorganised
project-master/tasks.md that:
- Marks all completed phases correctly
- Lists all pending phases (BS.8-BS.15, BU, BV) in correct dependency order
- Lists all new phases (BW onwards) from EI-5 in priority order
- Has an accurate summary table

**Tasks:**

- [x] EI-6.1 Confirm BS.8-BS.15 dependency order is still correct given new requirements
- [x] EI-6.2 Confirm BU and BV are still correctly scoped
- [x] EI-6.3 Insert new phases BW onwards in correct sequence
- [x] EI-6.4 Update summary table counts
- [x] EI-6.5 Add `deferred-requirements.md` reference to project-master

---

## Execution Order

```
EI-0 (MiniX/FTSO rationalisation) -- reconcile design doc against EARS and existing specs
    |
    v
[USER APPROVAL OF EI-0 BEFORE PROCEEDING]
    |
    v
EI-1 (gap analysis) -- read all existing requirements.md files
    |
    v
EI-2 (coverage classification) -- classify every EARS criterion
    |
    v
EI-3 (incomplete work audit) -- audit current pending work
    |
    v
EI-4 (integration plan) -- produce ordered plan
    |
    v
[USER APPROVAL OF EI-4 BEFORE PROCEEDING]
    |
    v
EI-5 (gate execution, batch by batch) -- update requirements, design, tasks, TCR
    |
    v
EI-6 (reorganised master task list)
```

---

## Constraints

- EI-0 is analysis and decision only -- no requirements.md files are modified
- EI-0 must be approved before EI-1 starts
- EI-1 through EI-4 are analysis only -- no requirements.md files are modified
- EI-5 batches are executed one at a time, each requiring user approval before the next
- P3 (DEFERRED) requirements are never gated -- they go into deferred-requirements.md only
- OUT-OF-SCOPE requirements are documented but never added to any requirements.md
- The existing pending phases (BS.8-BS.15, BU, BV) are NOT blocked by this workflow --
  they can proceed in parallel; EI-5 batches are sequenced to avoid conflicts
- The MiniX/FTSO design document is an input to EI-0 only -- it is never directly
  transcribed into requirements.md; all requirements must trace to EARS source files
  or existing committed specs

---

## Estimated Scale

| Area | EARS criteria count | Estimated NEW criteria | Estimated PARTIAL |
|------|--------------------|-----------------------|-------------------|
| ISPF edit (01-06) | ~120 | ~40 | ~30 |
| ISPF display (07-12) | ~80 | ~20 | ~20 |
| TSO session + navigation | ~30 | ~5 | ~15 |
| TSO commands | ~60 | ~30 | ~10 |
| SDSF framework + queues | ~60 | ~50 | ~5 |
| SDSF filter/log/browse | ~50 | ~40 | ~5 |
| REXX + SDSF REXX | ~50 | ~30 | ~5 |
| **Total** | **~450** | **~215** | **~90** |

This is a large integration. The workflow is designed to be executed
incrementally -- one phase at a time -- rather than all at once.
