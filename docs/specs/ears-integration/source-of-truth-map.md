# Source of Truth Map
# EI-0.5 Output -- Authoritative Source for Every EARS Requirement Area

**Status:** EI-0 analysis -- no requirements.md files modified
**Purpose:** For each EARS requirement area, this document records:
- the authoritative EARS source file and requirement IDs
- the existing spec that owns that area
- the relevant MiniX/FTSO design section (if any)
- the resolution: what happens in EI-5

This map is the input to EI-1 (gap analysis). EI-1 reads the existing
requirements.md files and maps them against the EARS criteria listed here.

---

## ISPF Edit Session and Profile

| Field | Detail |
|-------|--------|
| Authoritative EARS source | `ispf-ears/` -- edit session lifecycle, profile and modes files |
| Existing spec | `edit-operations/requirements.md` |
| MiniX/FTSO input | Section 10B (EDIT command), Section 18 (EDIT opens FFWB editor) |
| MiniX/FTSO resolution | ALREADY COVERED -- EDIT command routes to existing editor |
| EI-5 batch | Batch 1: edit-operations |
| Priority | P1 |

---

## ISPF Line Commands

| Field | Detail |
|-------|--------|
| Authoritative EARS source | `ispf-ears/` -- line commands file; also `tso-ears/tso-commands.md` TSO-EDIT-3 |
| Existing spec | `line-commands/requirements.md` |
| MiniX/FTSO input | None directly |
| MiniX/FTSO resolution | N/A |
| EI-5 batch | Batch 3: line-commands |
| Priority | P1 |

---

## ISPF Primary Commands (FIND, CHANGE, LOCATE, SORT, EXCLUDE, SHOW, RESET, BOUNDS, COLS)

| Field | Detail |
|-------|--------|
| Authoritative EARS source | `ispf-ears/` -- primary commands, find/change files |
| Existing spec | `command-semantics/requirements.md`, `find-and-replace/requirements.md`, `navigation-commands/requirements.md`, `exclude-show-filter/requirements.md` |
| MiniX/FTSO input | Section 10B (EDIT subcommands) |
| MiniX/FTSO resolution | ALREADY COVERED |
| EI-5 batch | Batch 2: find-and-replace; Batch 1: edit-operations |
| Priority | P1 |

---

## ISPF Edit Recovery and Undo

| Field | Detail |
|-------|--------|
| Authoritative EARS source | `ispf-ears/` -- recovery and undo file |
| Existing spec | `undo-redo-transactions/requirements.md` |
| MiniX/FTSO input | None |
| MiniX/FTSO resolution | N/A |
| EI-5 batch | Batch 12: undo-redo-transactions |
| Priority | P2 |

---

## ISPF Syntax Highlighting (HILITE)

| Field | Detail |
|-------|--------|
| Authoritative EARS source | `ispf-ears/` -- syntax highlighting file |
| Existing spec | `syntax-highlighting/requirements.md` |
| MiniX/FTSO input | None |
| MiniX/FTSO resolution | N/A |
| EI-5 batch | Batch 13: syntax-highlighting |
| Priority | P2 |

---

## ISPF Boundaries, Tabs, and Masks

| Field | Detail |
|-------|--------|
| Authoritative EARS source | `ispf-ears/` -- boundaries, tabs, masks file |
| Existing spec | `tabs-and-mask/requirements.md` |
| MiniX/FTSO input | None |
| MiniX/FTSO resolution | N/A |
| EI-5 batch | Batch 6: tabs-and-mask |
| Priority | P1 |

---

## ISPF Sequence Numbers

| Field | Detail |
|-------|--------|
| Authoritative EARS source | `ispf-ears/` -- sequence numbers file |
| Existing spec | `sequence-numbers/requirements.md` |
| MiniX/FTSO input | None |
| MiniX/FTSO resolution | N/A |
| EI-5 batch | Batch 4: sequence-numbers |
| Priority | P1 |

---

## ISPF Hex Display

| Field | Detail |
|-------|--------|
| Authoritative EARS source | `ispf-ears/` -- hex display file |
| Existing spec | `hex-display/requirements.md` |
| MiniX/FTSO input | Section 9.7 (SET HEX) |
| MiniX/FTSO resolution | ALREADY COVERED -- hex display is in existing spec; SET HEX is a SDSF command in FFW-JES |
| EI-5 batch | Batch 5: hex-display |
| Priority | P1 |

---

## ISPF Edit Macros

| Field | Detail |
|-------|--------|
| Authoritative EARS source | `ispf-ears/` -- edit macros file |
| Existing spec | `lua-macro-engine/requirements.md` |
| MiniX/FTSO input | Section 15 (scripting strategy Phase 2 -- Lua) |
| MiniX/FTSO resolution | ALREADY COVERED -- Lua is the macro engine; ISPF macro compatibility is an EXTENDS item |
| EI-5 batch | Batch 14: lua-macro-engine |
| Priority | P2 |

---

## ISPF POM and Panel Navigation

| Field | Detail |
|-------|--------|
| Authoritative EARS source | `tso-ears/ispf-panel-navigation.md` ISPF-1 through ISPF-5; `tso-ears/tso-session-and-logon.md` TSO-1 through TSO-4 |
| Existing spec | `menu-and-statusbar/requirements.md`, `startup-and-session/requirements.md` |
| MiniX/FTSO input | Section 8.1 (FTSO terminal panel, ISPF Option 6 panel chrome), Section 14 (sessions, SPLIT/SWAP) |
| MiniX/FTSO resolution | ALREADY COVERED (POM, navigation) + EXTENDS (ISPF Option 6 panel chrome for command shell tab) |
| EI-5 batch | Batch 7: menu-and-statusbar; Batch 8: startup-and-session |
| Priority | P1 (ISPF-1, ISPF-2, ISPF-4, ISPF-5); P2 (ISPF-3 split screen) |

---

## TSO Session Startup and Logon

| Field | Detail |
|-------|--------|
| Authoritative EARS source | `tso-ears/tso-session-and-logon.md` TSO-1 through TSO-4 |
| Existing spec | `startup-and-session/requirements.md` |
| MiniX/FTSO input | Section 8.1 (FTSO READY prompt), Section 14 (session state) |
| MiniX/FTSO resolution | ALREADY COVERED (session startup, POM) + EXTENDS (LOGOFF command, session timestamp display, FTSO READY prompt style) |
| EI-5 batch | Batch 8: startup-and-session |
| Priority | P1 |

---

## TSO PF Keys and RETRIEVE

| Field | Detail |
|-------|--------|
| Authoritative EARS source | `tso-ears/tso-session-and-logon.md` TSO-3; `tso-ears/ispf-panel-navigation.md` ISPF-5 |
| Existing spec | `function-keys-and-history/requirements.md` |
| MiniX/FTSO input | Section 8.1 (PF key bar in FTSO panel) |
| MiniX/FTSO resolution | ALREADY COVERED |
| EI-5 batch | Batch 9: function-keys-and-history |
| Priority | P1 |

---

## TSO Scrolling

| Field | Detail |
|-------|--------|
| Authoritative EARS source | `tso-ears/tso-session-and-logon.md` TSO-4 |
| Existing spec | `viewport-and-scrolling/requirements.md`, `navigation-commands/requirements.md` |
| MiniX/FTSO input | None |
| MiniX/FTSO resolution | N/A |
| EI-5 batch | Batch 7: menu-and-statusbar (SCROLL field) |
| Priority | P1 |

---

## TSO Dataset Commands (ALLOCATE, FREE, DELETE, RENAME, LISTCAT, LISTDS, LISTALC)

| Field | Detail |
|-------|--------|
| Authoritative EARS source | `tso-ears/tso-commands.md` TSO-CMD-1 through TSO-CMD-7 |
| Existing spec | `command-semantics/requirements.md` (primary), `dataset-catalog/requirements.md`, `dataset-allocator/requirements.md` |
| MiniX/FTSO input | Section 10B (catalogue/dataset commands), Section 11 (dataset reference parsing), Section 9.5 (MiniX catalogue/dataset services) |
| MiniX/FTSO resolution | EARS-DEFINED -- TSO-CMD-1 through TSO-CMD-7 are the authoritative source. MiniX/FTSO examples are illustrative. |
| EI-5 batch | Batch 10: command-semantics (P1 commands) |
| Priority | P1 |

---

## TSO Job Commands (SUBMIT, STATUS)

| Field | Detail |
|-------|--------|
| Authoritative EARS source | `tso-ears/tso-commands.md` TSO-CMD-8 through TSO-CMD-9 |
| Existing spec | `command-semantics/requirements.md` (primary), `FFW-JES/requirements.md` |
| MiniX/FTSO input | Section 10G (JES/spool commands: SUBMIT, JES STATUS) |
| MiniX/FTSO resolution | EARS-DEFINED -- TSO-CMD-8 and TSO-CMD-9 are authoritative. MiniX/FTSO examples are illustrative. |
| EI-5 batch | Batch 10: command-semantics (P1 commands) |
| Priority | P1 |

---

## TSO Output Management Commands (OUTPUT, CANCEL, SEND, PROFILE, PRINTDS)

| Field | Detail |
|-------|--------|
| Authoritative EARS source | `tso-ears/tso-commands.md` TSO-CMD-10 through TSO-CMD-14 |
| Existing spec | `command-semantics/requirements.md` (primary), `FFW-JES/requirements.md` |
| MiniX/FTSO input | Section 10G (JES/spool commands) |
| MiniX/FTSO resolution | EARS-DEFINED -- TSO-CMD-10 through TSO-CMD-14 are authoritative. |
| EI-5 batch | Batch 16: command-semantics (P2 commands) |
| Priority | P2 |

---

## TSO EDIT Command and Subcommands

| Field | Detail |
|-------|--------|
| Authoritative EARS source | `tso-ears/tso-commands.md` TSO-EDIT-1 through TSO-EDIT-3 |
| Existing spec | `edit-operations/requirements.md`, `command-semantics/requirements.md` |
| MiniX/FTSO input | Section 10B (EDIT command), Section 18 (EDIT opens FFWB editor) |
| MiniX/FTSO resolution | ALREADY COVERED (EDIT opens editor) + EARS-DEFINED (EDIT subcommands: FIND, CHANGE, DELETE, INSERT, COPY, MOVE, SAVE, END, CANCEL, TOP, BOTTOM, UP, DOWN, SUBMIT, RENUM, UNNUM, PROFILE, VERIFY, TABSET) |
| EI-5 batch | Batch 1: edit-operations; Batch 10: command-semantics |
| Priority | P1 |

---

## SDSF Panel Framework and Layout

| Field | Detail |
|-------|--------|
| Authoritative EARS source | `tso-ears/sdsf-panel-framework.md` SDSF-1 through SDSF-5 |
| Existing spec | `FFW-JES/requirements.md` Req 9 (Job Monitor Panel) |
| MiniX/FTSO input | Section 10G (SDSF command), Section 18 (SDSF opens spool monitor) |
| MiniX/FTSO resolution | EARS-DEFINED -- SDSF-1 through SDSF-5 are authoritative for panel layout, action characters, main panel, help system. MiniX/FTSO confirms SDSF is a panel launcher (Decision D5 in reconciliation). |
| EI-5 batch | Batch 11: FFW-JES (P1) |
| Priority | P1 (SDSF-1, SDSF-2, SDSF-4); P2 (SDSF-3, SDSF-5) |

---

## SDSF SET Commands

| Field | Detail |
|-------|--------|
| Authoritative EARS source | `tso-ears/sdsf-panel-framework.md` SDSF-SET-1 through SDSF-SET-13 |
| Existing spec | `FFW-JES/requirements.md` (to be extended), `configuration-system/requirements.md` |
| MiniX/FTSO input | Section 17.2 (confirmation policies -- maps to SET CONFIRM) |
| MiniX/FTSO resolution | EARS-DEFINED -- SET commands are authoritative from SDSF-SET-1 through SDSF-SET-13. MiniX/FTSO confirmation policy aligns with SDSF-SET-3. |
| EI-5 batch | Batch 11: FFW-JES (P1: SET-1, SET-8, SET-9, SET-12); Batch 15: FFW-JES (P2: SET-2 through SET-11) |
| Priority | P1 (SET-1, SET-8, SET-9, SET-12); P2 (SET-2 through SET-11) |

---

## SDSF Session Persistence

| Field | Detail |
|-------|--------|
| Authoritative EARS source | `tso-ears/sdsf-panel-framework.md` SDSF-PERSIST-1 through SDSF-PERSIST-2 |
| Existing spec | `startup-and-session/requirements.md`, `FFW-JES/requirements.md` |
| MiniX/FTSO input | Section 14 (session state persistence) |
| MiniX/FTSO resolution | EARS-DEFINED -- SDSF-PERSIST-1 is authoritative. MiniX/FTSO session state aligns. |
| EI-5 batch | Batch 11: FFW-JES (P1: PERSIST-1); P3: PERSIST-2 (special DDNames -- deferred) |
| Priority | P1 (PERSIST-1); P3 (PERSIST-2) |

---

## SDSF Job Queue Panels (I, O, H, ST, DA)

| Field | Detail |
|-------|--------|
| Authoritative EARS source | `tso-ears/sdsf-job-queue-panels.md` SDSF-JQ-1 through SDSF-JQ-7 |
| Existing spec | `FFW-JES/requirements.md` Req 3, 5, 9 |
| MiniX/FTSO input | Section 10G (JES STATUS, JES CANCEL, JES HOLD, JES RELEASE, SPOOL LIST, SPOOL VIEW) |
| MiniX/FTSO resolution | EARS-DEFINED -- SDSF-JQ-1 through SDSF-JQ-7 are authoritative. MiniX/FTSO command examples are illustrative. |
| EI-5 batch | Batch 11: FFW-JES (P1) |
| Priority | P1 |

---

## SDSF Filter, Sort, and Search

| Field | Detail |
|-------|--------|
| Authoritative EARS source | `tso-ears/sdsf-filter-sort-search.md` SDSF-FILTER-1 through SDSF-FILTER-7, SDSF-SCROLL-1 through SDSF-SCROLL-5 |
| Existing spec | `FFW-JES/requirements.md` Req 9.4-9.6, `record-selection-criteria/requirements.md`, `exclude-show-filter/requirements.md` |
| MiniX/FTSO input | None directly |
| MiniX/FTSO resolution | EARS-DEFINED |
| EI-5 batch | Batch 11: FFW-JES (P1) |
| Priority | P1 |

---

## SDSF Log and System Panels

| Field | Detail |
|-------|--------|
| Authoritative EARS source | `tso-ears/sdsf-log-and-system-panels.md` SDSF-LOG-1 through SDSF-LOG-4, SDSF-SYS-1 through SDSF-SYS-5 |
| Existing spec | `FFW-JES/requirements.md`, `logging-subsystem/requirements.md` |
| MiniX/FTSO input | None directly |
| MiniX/FTSO resolution | EARS-DEFINED |
| EI-5 batch | Batch 15: FFW-JES (P2) |
| Priority | P2 |

---

## SDSF Browse and Print

| Field | Detail |
|-------|--------|
| Authoritative EARS source | `tso-ears/sdsf-browse-and-print.md` SDSF-BROWSE-1 through SDSF-BROWSE-4 |
| Existing spec | `FFW-JES/requirements.md` Req 7, `custom-file-viewers/requirements.md` |
| MiniX/FTSO input | Section 10B (BROWSE command), Section 18 (BROWSE opens read-only view) |
| MiniX/FTSO resolution | EARS-DEFINED -- SDSF-BROWSE-1 through SDSF-BROWSE-4 are authoritative. MiniX/FTSO BROWSE command aligns. |
| EI-5 batch | Batch 15: FFW-JES (P2) |
| Priority | P2 |

---

## REXX Scripting

| Field | Detail |
|-------|--------|
| Authoritative EARS source | `tso-ears/rexx-and-sdsf-rexx.md` REXX-1 through REXX-5 |
| Existing spec | `lua-macro-engine/requirements.md` (scripting bridge), `command-framework/requirements.md` Req 6 (scripting bridge) |
| MiniX/FTSO input | Section 15 (scripting strategy Phase 3 -- REXX/CLIST compatibility) |
| MiniX/FTSO resolution | EARS-DEFINED -- REXX-1 through REXX-5 define what REXX execution must do. MiniX/FTSO Phase 3 strategy aligns. The Lua engine is the bridge (REXX-1.1 maps REXX execs to Lua execution). |
| EI-5 batch | Batch 14: lua-macro-engine (P2) |
| Priority | P2 |

---

## SDSF JES/WLM Panels (MAS, JG, SRVC, SE)

| Field | Detail |
|-------|--------|
| Authoritative EARS source | `tso-ears/rexx-and-sdsf-rexx.md` SDSF-JES-1 through SDSF-JES-4 |
| Existing spec | `FFW-JES/requirements.md` |
| MiniX/FTSO input | None directly |
| MiniX/FTSO resolution | EARS-DEFINED |
| EI-5 batch | P3 -- deferred (MAS, JG, SRVC, SE are advanced JES panels) |
| Priority | P3 |

---

## SDSF REXX Interface (ISFCALLS, ISFEXEC, ISFACT, ISFBROWSE, ISFSLASH, ISFGET, ISFLOG)

| Field | Detail |
|-------|--------|
| Authoritative EARS source | `tso-ears/rexx-and-sdsf-rexx.md` SDSF-REXX-1 through SDSF-REXX-7 |
| Existing spec | `FFW-JES/requirements.md`, `lua-macro-engine/requirements.md` |
| MiniX/FTSO input | None directly |
| MiniX/FTSO resolution | EARS-DEFINED |
| EI-5 batch | P3 -- deferred (SDSF REXX interface is advanced automation) |
| Priority | P3 |

---

## EXTENDS Items (from MiniX/FTSO -- not in EARS files, not yet in any spec)

These items were identified in `minix-ftso-reconciliation.md` as genuine
extensions. They are not gated now. They are added to existing specs during
the relevant EI-5 batch.

| Item | Target spec | EI-5 batch | Priority |
|------|-------------|------------|----------|
| ISPF Option 6 panel chrome (title line, COMMAND ===> field, SCROLL ===> field, "FTSO READY" prompt, MAXCC display) | `shell-command` | Batch 10 | P1 |
| TSO-style operand parsing (positional + keyword operands: DSNAME(), RECFM(), LRECL()) | `command-semantics` | Batch 10 | P1 |
| Command continuation character (trailing backslash) | `command-semantics` | Batch 10 | P2 |
| ds:// URI scheme for dataset references | `virtual-file-system` or `command-semantics` | Batch 10 | P2 |
| Dataset prefix per session (SET PREFIX) | `command-semantics` | Batch 10 | P1 |
| FFCMD command files (Phase 1 scripting -- sequential execution, variables, conditionals) | `lua-macro-engine` | Batch 14 | P2 |
| Namespace conflict resolution for plugin commands (CORE:LISTCAT vs IDCAMS:LISTCAT) | `command-framework` | Batch 10 | P2 |
| Capability model (fine-grained per-command capability declarations) | `command-framework` | Batch 10 | P2 |
| Secret operand handling (redaction from history and logs) | `command-framework` | Batch 10 | P2 |
| Structured audit events (timestamp, session, principal, command, resources, correlation ID) | `command-framework` | Batch 10 | P2 |
| "Show equivalent FTSO command" for GUI operations | `shell-command` or `menu-and-statusbar` | Batch 7 | P2 |
| Fuzz-testable parser requirement | `command-semantics` | Batch 10 | P2 |
| LOGOFF command | `startup-and-session` | Batch 8 | P1 |
| Session start/end timestamp display | `startup-and-session` | Batch 8 | P1 |

---

## Deferred Items (P3 -- never gated, go to deferred-requirements.md)

| Item | Source | Reason |
|------|--------|--------|
| Record-aware inter-command pipelines | MiniX/FTSO Section 12 | No EARS file requires this; no TSO/ISPF equivalent |
| SDSF JES/WLM panels (MAS, JG, SRVC, SE) | SDSF-JES-1 through SDSF-JES-4 | Advanced JES panels, not core workstation experience |
| SDSF REXX interface (ISFCALLS through ISFLOG) | SDSF-REXX-1 through SDSF-REXX-7 | Advanced automation, depends on full REXX engine |
| SDSF special DDNames (ISFMIGNB, ISFMIGXB, ISFMIGNP) | SDSF-PERSIST-2 | z/OS-specific customisation mechanism |
| REXX/CLIST compatibility adapter (Phase 3 scripting) | REXX-1 through REXX-5 (partial) | Depends on full REXX engine; Lua bridge covers P2 |
| ERI/RISL/PSL/AI provider integration | MiniX/FTSO Section 15.4 | Future capability, no EARS source |

---

## Reading Guide for EI-1

When executing EI-1 (gap analysis), read each existing requirements.md file
and map its criteria against the EARS requirement IDs listed in this document.

For each EARS requirement ID, determine:
- COVERED: an existing criterion in requirements.md matches the EARS criterion
- PARTIAL: an existing criterion partially covers the EARS criterion
- NEW: no existing criterion covers the EARS criterion
- OUT-OF-SCOPE: the EARS criterion requires z/OS-specific hardware or behaviour
  that cannot be emulated on a desktop (e.g., real RACF, 3270 data streams)

The EXTENDS items in the table above are treated as NEW criteria in EI-2
(they have no existing coverage in any requirements.md).
