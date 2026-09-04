# Deferred Requirements (P3)
# EI-6 Output -- Requirements Deliberately Outside Current Release Scope

**Status:** Recorded as DEFERRED -- not added to any requirements.md
**Source:** EI-4 integration-plan.md Section 7
**Rule:** These requirements SHALL NOT be added to any sub-project requirements.md
until explicitly promoted by a new requirements gate. They are recorded here
for traceability only.

---

## How to Promote a Deferred Requirement

1. Open a new requirements gate (new-requirements-gate.md rule).
2. Log a CR-NR entry in docs/status/change-log.md.
3. Add the criterion to the relevant sub-project requirements.md.
4. Add a NOT COVERED row to docs/quality/TCR.md.
5. Add tasks to the sub-project tasks.md and project-master/tasks.md.
6. Remove the criterion from this file once promoted.

---

## Group 1 -- REXX Data Stack (REXX-5.x) -- 7 criteria

**Rationale:** REXX data stack is an advanced REXX feature. The P2 REXX
execution bridge (Phase CG) covers the core execution model. Data stack
support requires a more complete REXX runtime and is deferred until the
P2 REXX bridge is proven in production.

**Target sub-project when promoted:** `lua-macro-engine`

| EARS ID | Description |
|---------|-------------|
| REXX-5.1 | PUSH -- add to top of data stack |
| REXX-5.2 | QUEUE -- add to bottom of data stack |
| REXX-5.3 | PULL -- remove from top of data stack |
| REXX-5.4 | QUEUED -- return stack element count |
| REXX-5.5 | MAKEBUF -- create new buffer on stack |
| REXX-5.6 | DROPBUF -- remove buffer from stack |
| REXX-5.7 | NEWSTACK/DELSTACK -- private stack management |

---

## Group 2 -- SDSF Advanced JES Panels (SDSF-JES-1 through SDSF-JES-4) -- 4 criteria

**Rationale:** These panels require deep JES2/JES3 internals knowledge and
WLM integration. They are beyond the scope of the initial SDSF emulation.
Prerequisite: Phase CC and CD (SDSF P1 core) must be fully implemented first.

**Target sub-project when promoted:** `FFW-JES`

| EARS ID | Description |
|---------|-------------|
| SDSF-JES-1 | MAS panel -- multi-access spool |
| SDSF-JES-2 | JG panel -- job group |
| SDSF-JES-3 | SRVC panel -- WLM service class |
| SDSF-JES-4 | SE panel -- scheduling environment |

---

## Group 3 -- SDSF REXX Interface (SDSF-REXX-1 through SDSF-REXX-7) -- 7 criteria

**Rationale:** The SDSF REXX interface depends on both the P2 REXX bridge
(Phase CG) and the P2 SDSF panels (Phase CH) being complete and proven.
It is a P3 integration layer on top of two P2 features.

**Target sub-project when promoted:** `FFW-JES` (primary), `lua-macro-engine` (secondary)

| EARS ID | Description |
|---------|-------------|
| SDSF-REXX-1 | ISFCALLS -- enable/disable SDSF REXX interface |
| SDSF-REXX-2 | ISFEXEC -- execute SDSF command from REXX |
| SDSF-REXX-3 | ISFACT -- perform action on SDSF rows |
| SDSF-REXX-4 | ISFBROWSE -- browse SDSF output from REXX |
| SDSF-REXX-5 | ISFSLASH -- issue slash command from REXX |
| SDSF-REXX-6 | ISFGET -- retrieve SDSF variable values |
| SDSF-REXX-7 | ISFLOG -- write to SDSF log from REXX |

---

## Group 4 -- Out-of-Scope Requirements (never to be promoted) -- 2 criteria

These are documented for traceability but are permanently out of scope.
They SHALL NOT be promoted regardless of future project direction.

| EARS ID | Description | Rationale |
|---------|-------------|-----------|
| Edit-PACK-mode | PACK ON enables data compression | z/OS-specific DASD packing mechanism. No equivalent on desktop filesystems. |
| PERSIST-2 | Special DDNames (ISFMIGNB/ISFMIGXB/ISFMIGNP) | z/OS-specific DDName mechanism for SDSF migration control. No equivalent in the FileForge VFS model. |

---

## Summary

| Group | Count | Status |
|-------|------:|--------|
| REXX Data Stack (REXX-5.x) | 7 | DEFERRED -- promote after Phase CG proven |
| SDSF Advanced JES Panels | 4 | DEFERRED -- promote after Phases CC/CD proven |
| SDSF REXX Interface | 7 | DEFERRED -- promote after Phases CG + CH proven |
| Out-of-scope (permanent) | 2 | NEVER PROMOTE |
| **Total** | **20** | |

Note: EI-4 integration-plan.md listed 27 deferred criteria. After deduplication
(REXX-2.3/2.4 unified with EM-ISPEXEC/EM-ISREDIT in Phase CG, and SET-1/SET-9
unified with SDSF-2.2/2.6 in Phase CC), the effective unique deferred count is 20.
