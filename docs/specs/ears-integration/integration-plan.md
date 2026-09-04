# Integration Plan
# EI-4 Output -- Ordered Plan for Integrating NEW/PARTIAL EARS Requirements

**Status:** EI-4 in progress
**Input:** coverage-classification.md (EI-2), incomplete-work-audit.md (EI-3)
**Purpose:** Assign every NEW and PARTIAL EARS criterion to a sub-project and
phase label (BW onwards), define the ordered phase sequence, list PARTIAL
criteria as change requests, document OUT-OF-SCOPE rationale, and list
DEFERRED (P3) requirements.

**Gate:** This plan must be approved by the user before EI-5 begins.
No requirements.md files are modified during EI-4.

---

## Section 1: Constraints from EI-3 (Incomplete Work Audit)

The following constraints govern phase sequencing:

1. BV.1 (CatalogLocation refactor) has no dependencies -- complete first.
2. BS.8-BS.15 (Wave 3-4) depend on BS.7 (done). Sequence: BV.1 then BS.8+.
3. BU.2-BU.9 depend on BS.4 (done) for basic path; recommend after BS.8.
4. ff-vfs tasks (Req 9-12) must be created before BS.8 begins.
5. EI-5 batches B01-B11 (P1) are independent of BS Wave 3-4 -- parallel stream.
6. EI-5 batches B12-B16 (P2) follow B01-B11.
7. B11 FFW-JES is split into B11a (panel framework + job queues) and
   B11b (filter/sort/search + SET commands P1) to keep gate size manageable.

---

## Section 2: Phase Label Assignment (BW onwards)

The last assigned phase in project-master is BV.
New EARS-derived phases begin at BW.

| Phase | Batch | Sub-project | Priority | NEW | PAR | Notes |
|-------|-------|-------------|----------|----:|----:|-------|
| BW | B01 | edit-operations | P1 | 11 | 4 | CAPS, NULLS, PROFILE, SUBMIT, CREATE, REPLACE, BROWSE, VIEW, nested EDIT, COMPARE, LOCK, STATS |
| BX | B03 | line-commands | P1 | 6 | 1 | O, W, S(lc), F, L, ] |
| BY | B04 | sequence-numbers | P1 | 0 | 2 | AUTONUM alias, NUM alias |
| BZ | B07 | menu-and-statusbar | P1/P2 | 10 | 4 | SCROLL ===> field, fastpath, split screen, list panel LOCATE, FTSO chrome, scroll amounts |
| CA | B08 | startup-and-session | P1 | 7 | 1 | session timestamps, LOGOFF, TIME command, STATUS routing |
| CB | B10 | command-semantics | P1/P2 | 17 | 1 | ALLOCATE through STATUS, EDIT routing, FTSO extensions |
| CC | B11a | FFW-JES (P1 core) | P1 | 20 | 6 | Panel framework, NP column, action chars, main panel, job queue columns, PREFIX/OWNER/DEST/SORT/FIND/LOCATE |
| CD | B11b | FFW-JES (P1 extended) | P1 | 14 | 3 | ST panel, SDSF-SCROLL, SET ACTION/MAIN/ROWNUM/WHO/QUERY AUTH, PERSIST-1 |
| CE | B12 | undo-redo-transactions | P2 | 2 | 1 | SETUNDO command, RECOVERY command |
| CF | B13 | syntax-highlighting | P2 | 3 | 2 | HILITE ON/OFF, HILITE LOGIC, HILITE PAREN, HILITE FIND |
| CG | B14 | lua-macro-engine | P2 | 24 | 2 | ISREDIT, ISPEXEC, IMACRO, LINENUM, CURSOR, REXX-1 through REXX-4, FFCMD |
| CH | B15 | FFW-JES (P2) | P2 | 16 | 4 | Overtype fields, help system, log panels, system panels, browse/print, SET P2 commands |
| CI | B16 | command-semantics (P2) | P2 | 5 | 0 | OUTPUT, CANCEL(job), SEND, PROFILE(cmd), PRINTDS |

Skipped batches (no new or partial criteria -- no gate required):
- B02 find-and-replace: 0 NEW, 0 PAR
- B05 hex-display: 0 NEW, 0 PAR
- B06 tabs-and-mask: 0 NEW, 0 PAR
- B09 function-keys-and-history: 0 NEW, 0 PAR

---

## Section 3: NEW P1 Requirements by Sub-project

### Phase BW -- edit-operations (B01, P1)

Target spec: `docs/specs/edit-operations/requirements.md`
New criteria to add (11 NEW + 4 PAR = 15 criteria):

| EARS ID | Type | Description |
|---------|------|-------------|
| Edit-CAPS-mode | NEW | CAPS primary command forces uppercase input |
| Edit-NULLS-mode | NEW | NULLS ON/OFF controls null character handling |
| Edit-PROFILE-command | NEW | PROFILE displays/changes edit profile settings |
| Edit-STATS-mode | NEW | STATS ON shows member statistics (P2 -- gate with BW) |
| Edit-LOCK-setting | NEW | LOCK prevents profile changes (P2 -- gate with BW) |
| PC-SUBMIT | NEW | SUBMIT primary command submits current buffer as job |
| PC-CREATE | NEW | CREATE primary command creates new dataset from lines |
| PC-REPLACE | NEW | REPLACE primary command replaces dataset content |
| PC-EDIT-nested | NEW | EDIT primary command opens another dataset from editor |
| PC-BROWSE | NEW | BROWSE primary command opens dataset for browse |
| PC-VIEW | NEW | VIEW primary command opens dataset for view |
| PC-COMPARE | NEW | COMPARE primary command compares with another dataset (P2) |
| Edit-profile-persist | PAR | Edit profile settings persist across sessions |
| Edit-AUTONUM-mode | PAR | AUTONUM ON alias for NUMBER ON (also in BY) |
| Edit-NUM-mode | PAR | NUM alias for NUMBER command (also in BY) |
| Edit-HILITE-setting | PAR | HILITE controls syntax highlighting (also in CF) |

Note: STATS, LOCK, COMPARE are P2 but grouped into BW to avoid a separate
single-criterion gate. Edit-AUTONUM and Edit-NUM are also addressed in BY
(sequence-numbers); BW adds the edit-operations perspective.

---

### Phase BX -- line-commands (B03, P1)

Target spec: `docs/specs/line-commands/requirements.md`
New criteria to add (6 NEW + 1 PAR = 7 criteria):

| EARS ID | Type | Description |
|---------|------|-------------|
| LC-O | NEW | Overlay lines (ISPF O line command) |
| LC-W | NEW | Copy line to clipboard (ISPF W) |
| LC-F | NEW | First/last line of excluded block (ISPF F) |
| LC-L | NEW | Assign label to line (ISPF L) |
| LC-bracket-right | NEW | Shift right by 1 column (ISPF ]) |
| LC-S | PAR | Show/unexclude excluded line (ISPF S line command) |

Note: LC-S extends exclude-show-filter; the line-commands spec gets the
S line command criterion; exclude-show-filter already covers the SHOW command.

---

### Phase BY -- sequence-numbers (B04, P1)

Target spec: `docs/specs/sequence-numbers/requirements.md`
Partial criteria to extend (0 NEW + 2 PAR = 2 criteria):

| EARS ID | Type | Description |
|---------|------|-------------|
| SN-AUTONUM | PAR | AUTONUM ON as alias for NUMBER ON |
| SN-NUM-alias | PAR | NUM as alias for NUMBER command |

Note: Both are alias additions to existing criteria. No new requirements
section needed -- extend Req 6.7 and Req 8 respectively.

---

### Phase BZ -- menu-and-statusbar (B07, P1/P2)

Target spec: `docs/specs/menu-and-statusbar/requirements.md`
New criteria to add (10 NEW + 4 PAR = 14 criteria):

| EARS ID | Type | Pri | Description |
|---------|------|-----|-------------|
| ISPF-1.6 | NEW | P1 | SCROLL ===> field adjacent to command field |
| ISPF-2.3 | NEW | P1 | Fastpath notation (3.1) navigates to nested option |
| ISPF-3.1 | NEW | P2 | PF2 split screen at cursor |
| ISPF-3.2 | NEW | P2 | PF9 swap between split screen halves |
| ISPF-3.3 | NEW | P2 | Each split screen half operates independently |
| ISPF-3.4 | NEW | P2 | Unsplit by pressing END (PF3) |
| ISPF-4.2 | NEW | P1 | LOCATE nearest alphabetic match on list panel |
| ISPF-4.3 | NEW | P1 | LOCATE accepts partial names on list panel |
| TSO-4.3 | NEW | P1 | SCROLL ===> field retains scroll amount |
| FTSO-chrome | NEW | P2 | ISPF Option 6 panel chrome (FTSO READY prompt, MAXCC display) |
| FTSO-show-equiv | NEW | P2 | Show equivalent FTSO command for GUI operations |
| ISPF-1.2 | PAR | P1 | Data entry panel with ===> fields |
| ISPF-1.3 | PAR | P1 | List panel with NP column |
| ISPF-4.1 | PAR | P1 | LOCATE on list panel scrolls to matching item |
| TSO-4.2 | PAR | P1 | Scroll amounts HALF/CSR/MAX/DATA in addition to PAGE/n |

Note: Split screen (ISPF-3.x) is P2 but grouped into BZ to keep all
panel navigation criteria together. The split screen criteria are
significant -- they may require a separate design.md section.

---

### Phase CA -- startup-and-session (B08, P1)

Target spec: `docs/specs/startup-and-session/requirements.md`
New criteria to add (7 NEW + 1 PAR = 8 criteria):

| EARS ID | Type | Description |
|---------|------|-------------|
| TSO-1.2 | NEW | Session start timestamp shown in status bar |
| TSO-1.3 | NEW | Session end timestamp and logoff message |
| TSO-1.4 / LOGOFF | NEW | LOGOFF command terminates session |
| TSO-2.4 | NEW | TIME command displays current date and time |
| TSO-2.5 | PAR | STATUS command routes to FFW-JES job status |
| Session-start-ts | NEW | Session start/end timestamp display (FTSO EXTENDS) |
| LOGOFF-cmd | NEW | LOGOFF command (FTSO EXTENDS -- same as TSO-1.4, unified) |

Note: TSO-1.4 and the FTSO EXTENDS LOGOFF item are the same criterion --
they will be written as a single criterion. Similarly TSO-1.2/1.3 and the
FTSO session timestamp item are unified. Effective new criteria: 5 NEW + 1 PAR.

---

### Phase CB -- command-semantics (B10, P1/P2)

Target spec: `docs/specs/command-semantics/requirements.md`
New criteria to add (17 NEW + 1 PAR = 18 criteria):

| EARS ID | Type | Pri | Description |
|---------|------|-----|-------------|
| TSO-CMD-1 | NEW | P1 | ALLOCATE command |
| TSO-CMD-2 | NEW | P1 | FREE command |
| TSO-CMD-3 | NEW | P1 | DELETE command (dataset/member) |
| TSO-CMD-4 | NEW | P1 | RENAME command |
| TSO-CMD-5 | NEW | P1 | LISTCAT command |
| TSO-CMD-6 | NEW | P1 | LISTDS command |
| TSO-CMD-7 | NEW | P1 | LISTALC command |
| TSO-CMD-8 | NEW | P1 | SUBMIT command |
| TSO-CMD-9 | NEW | P1 | STATUS command |
| TSO-EDIT-1 | PAR | P1 | EDIT command routing (extend existing criterion) |
| FTSO-operand-parse | NEW | P1 | TSO-style operand parsing (positional + keyword) |
| FTSO-prefix | NEW | P1 | Dataset prefix per session (SET PREFIX) |
| FTSO-continuation | NEW | P2 | Command continuation character (trailing backslash) |
| FTSO-ds-uri | NEW | P2 | ds:// URI scheme for dataset references |
| FTSO-ns-conflict | NEW | P2 | Namespace conflict resolution for plugin commands |
| FTSO-capability | NEW | P2 | Capability model (per-command capability declarations) |
| FTSO-secret | NEW | P2 | Secret operand handling (redaction from history/logs) |
| FTSO-audit | NEW | P2 | Structured audit events |
| FTSO-fuzz | NEW | P2 | Fuzz-testable parser requirement |

Note: P2 FTSO EXTENDS items are grouped into CB to avoid a separate gate.
The P2 items (continuation, ds://, namespace, capability, secret, audit, fuzz)
are lower priority but architecturally related to the P1 command parsing work.

---

### Phase CC -- FFW-JES P1 core (B11a, P1)

Target spec: `docs/specs/FFW-JES/requirements.md`
New criteria to add (20 NEW + 6 PAR = 26 criteria):

Panel framework core:

| EARS ID | Type | Description |
|---------|------|-------------|
| SDSF-1.1 | NEW | Action bar with pull-down menus |
| SDSF-1.2 | NEW | Title line with panel name and line range |
| SDSF-1.5 | NEW | SCROLL ===> field adjacent to command |
| SDSF-1.6 | NEW | Filter information lines (PREFIX=/DEST=/OWNER=) |
| SDSF-1.7 | NEW | NP column (fixed, non-scrolling action input) |
| SDSF-1.8 | NEW | First data column fixed on horizontal scroll |
| SDSF-2.2 | NEW | SET ACTION displays valid action characters |
| SDSF-2.3 | NEW | = repeats previous action character |
| SDSF-2.4 | NEW | // block action applies to block of rows |
| SDSF-2.5 | NEW | Command-line action syntax ("2 C" to cancel row 2) |
| SDSF-2.6 | NEW | SET ROWNUM shows row numbers in NP area |
| SDSF-4.1 | NEW | Main panel command list with name/desc/group |
| SDSF-4.2 | NEW | Command groups (Jobs/Output/JES/Log/Memory/etc.) |
| SDSF-4.3 | NEW | S action on main panel selects command |
| SDSF-4.4 | NEW | SET MAIN GROUP sets grouped main panel |
| SDSF-4.5 | NEW | MGRP expandable/collapsible command groups |
| SDSF-4.6 | NEW | MENU command returns to main panel |
| SDSF-FILTER-1 | NEW | PREFIX filter -- filter by job name prefix |
| SDSF-FILTER-2 | NEW | OWNER filter -- filter by job owner |
| SDSF-FILTER-3 | NEW | DEST filter -- filter by output destination |
| SDSF-1.3 | PAR | Message area in title line |
| SDSF-1.4 | PAR | COMMAND INPUT ===> field |
| SDSF-2.1 | PAR | Action characters S/?/C/H/A/P/D/E/J/W via NP column |
| SDSF-JQ-6 | PAR | Column definitions (JOBNAME/JOBID/OWNER/STATUS/etc.) |
| SDSF-JQ-7 | PAR | PREFIX/OWNER/DEST filter fields |
| SDSF-FILTER-5 | PAR | SORT command for panel columns |

---

### Phase CD -- FFW-JES P1 extended (B11b, P1)

Target spec: `docs/specs/FFW-JES/requirements.md`
New criteria to add (14 NEW + 3 PAR = 17 criteria):

| EARS ID | Type | Description |
|---------|------|-------------|
| SDSF-JQ-4 | PAR | Status panel (ST) -- all jobs with status |
| SDSF-FILTER-4 | NEW | FILTER command -- advanced filter expression |
| SDSF-FILTER-6 | NEW | FIND command -- search within panel |
| SDSF-FILTER-7 | NEW | LOCATE command -- scroll to matching row |
| SDSF-SCROLL-1-5 | PAR | Scroll commands in SDSF panels |
| SET-1 | NEW | SET ACTION -- display valid action characters |
| SET-8 | NEW | SET MAIN -- default main panel |
| SET-9 | NEW | SET ROWNUM -- row numbers in NP area |
| SET-12 | NEW | WHO command -- display session information |
| SET-13 | NEW | QUERY AUTH -- display authorized commands |
| PERSIST-1 | PAR | Save SDSF SET settings across sessions |

Note: SET-1 and SET-9 overlap with SDSF-2.2 and SDSF-2.6 from CC --
they will be unified into single criteria in the requirements.md update.
Effective unique criteria: approximately 14 NEW + 3 PAR after deduplication.

---

## Section 4: NEW P2 Requirements by Sub-project

### Phase CE -- undo-redo-transactions (B12, P2)

Target spec: `docs/specs/undo-redo-transactions/requirements.md`
New criteria to add (2 NEW + 1 PAR = 3 criteria):

| EARS ID | Type | Description |
|---------|------|-------------|
| RU-SETUNDO | NEW | SETUNDO command to configure undo settings |
| PC-SETUNDO | NEW | SETUNDO primary command (same as above -- unified) |
| RU-RECOVERY-command | PAR | RECOVERY ON/OFF command |

Note: RU-SETUNDO and PC-SETUNDO are the same criterion from two EARS areas.
They will be written as a single criterion. Effective: 1 NEW + 1 PAR.

---

### Phase CF -- syntax-highlighting (B13, P2)

Target spec: `docs/specs/syntax-highlighting/requirements.md`
New criteria to add (3 NEW + 2 PAR = 5 criteria):

| EARS ID | Type | Description |
|---------|------|-------------|
| SH-HILITE-toggle | NEW | HILITE ON/OFF command |
| SH-HILITE-LOGIC | NEW | HILITE LOGIC highlights logical operators |
| SH-HILITE-PAREN | NEW | HILITE PAREN highlights matching parentheses |
| SH-HILITE-FIND | PAR | HILITE FIND highlights find matches |
| PC-HILITE | PAR | HILITE primary command (extends existing) |

Note: PC-HILITE and SH-HILITE-toggle are the same concept from two EARS areas.
They will be unified into a single criterion.

---

### Phase CG -- lua-macro-engine (B14, P2)

Target spec: `docs/specs/lua-macro-engine/requirements.md`
New criteria to add (24 NEW + 2 PAR = 26 criteria):

ISPF macro API:

| EARS ID | Type | Description |
|---------|------|-------------|
| EM-ISREDIT | NEW | ISREDIT host command environment |
| EM-ISPEXEC | NEW | ISPEXEC host command environment |
| EM-IMACRO | NEW | IMACRO initial macro on edit session open |
| Edit-IMACRO-setting | NEW | IMACRO setting in edit profile |
| EM-LINENUM | NEW | LINENUM function get/set line number |
| EM-CURSOR | PAR | CURSOR function get/set cursor position |

REXX execution bridge:

| EARS ID | Type | Description |
|---------|------|-------------|
| REXX-1.1 | NEW | Execute REXX exec from PDS (SYSEXEC/SYSPROC) |
| REXX-1.2 | NEW | EXEC command -- run exec explicitly |
| REXX-1.3 | NEW | Implicit exec invocation by member name |
| REXX-1.4 | NEW | % prefix for exec to reduce search time |
| REXX-1.5 | NEW | Pass arguments to exec via EXEC command |
| REXX-2.1 | NEW | TSO host command environment |
| REXX-2.2 | NEW | ADDRESS environment-name changes host env |
| REXX-2.3 | NEW | ISPEXEC environment for ISPF service calls |
| REXX-2.4 | NEW | ISREDIT environment for ISPF Edit macro calls |
| REXX-2.5 | NEW | RC special variable -- return code from host command |
| REXX-3.1 | NEW | LISTDSI function |
| REXX-3.2 | NEW | MSG function |
| REXX-3.3 | NEW | MVSVAR function |
| REXX-3.4 | NEW | OUTTRAP function |
| REXX-3.5 | NEW | PROMPT function |
| REXX-3.6 | NEW | SYSDSN function |
| REXX-3.7 | NEW | SYSVAR function |
| REXX-3.8 | NEW | USERID function |
| REXX-4.1 | NEW | EXECIO DISKR |
| REXX-4.2 | NEW | EXECIO DISKW |
| REXX-4.3 | NEW | EXECIO FINIS |
| REXX-4.4 | NEW | EXECIO SKIP |
| REXX-4.5 | NEW | EXECIO return codes |
| FFCMD | NEW | FFCMD command files (Phase 1 scripting) |
| EM-CURSOR | PAR | CURSOR function (already listed above) |

Note: REXX-2.3/REXX-2.4 overlap with EM-ISPEXEC/EM-ISREDIT -- unified.
Effective unique criteria after deduplication: approximately 22 NEW + 2 PAR.

---

### Phase CH -- FFW-JES P2 (B15, P2)

Target spec: `docs/specs/FFW-JES/requirements.md`
New criteria to add (16 NEW + 4 PAR = 20 criteria):

| EARS ID | Type | Description |
|---------|------|-------------|
| SDSF-3.1 | NEW | Overtypeable fields visually distinct |
| SDSF-3.2 | NEW | Overtype field to change value |
| SDSF-3.3 | NEW | Command-line overtype syntax |
| SDSF-3.4 | NEW | Overtype Extension pop-up |
| SDSF-5.2 | NEW | SEARCH in help content |
| SDSF-5.3 | NEW | ACTH action character help |
| SDSF-5.4 | NEW | COLH column help |
| SDSF-5.5 | NEW | CMDH command help |
| SET-2 | NEW | SET BCOLOR |
| SET-3 | NEW | SET CONFIRM |
| SET-4 | NEW | SET CURSOR |
| SET-5 | NEW | SET DATE |
| SET-7 | NEW | SET HEX |
| SET-10 | NEW | SET SCHARS |
| SET-11 | NEW | SET SCREEN |
| SDSF-LOG-1 | NEW | System log panel |
| SDSF-LOG-2 | NEW | User log panel (ULOG) |
| SDSF-LOG-3 | NEW | LOG command |
| SDSF-LOG-4 | NEW | NEXT/PREV/SNAPSHOT log navigation |
| SDSF-SYS-1 | NEW | SYS panel |
| SDSF-SYS-2 | NEW | DASH panel |
| SDSF-SYS-3 | NEW | INIT panel |
| SDSF-SYS-4 | NEW | JC panel |
| SDSF-SYS-5 | NEW | SP panel |
| SDSF-BROWSE-2 | NEW | Browse settings |
| SDSF-BROWSE-3 | NEW | Print job output |
| SDSF-BROWSE-4 | NEW | Show columns |
| SDSF-5.1 | PAR | Context-sensitive help (SDSF-specific) |
| SET-6 | PAR | SET DELAY command |

Note: CH is a large batch (27+ criteria). Consider splitting into CH-a
(overtype + help) and CH-b (log/system panels + browse/print + SET P2)
at gate time if the gate becomes unwieldy.

---

### Phase CI -- command-semantics P2 (B16, P2)

Target spec: `docs/specs/command-semantics/requirements.md`
New criteria to add (5 NEW + 0 PAR = 5 criteria):

| EARS ID | Type | Description |
|---------|------|-------------|
| TSO-CMD-10 | NEW | OUTPUT command |
| TSO-CMD-11 | NEW | CANCEL command (batch job) |
| TSO-CMD-12 | NEW | SEND command |
| TSO-CMD-13 | NEW | PROFILE command |
| TSO-CMD-14 | NEW | PRINTDS command |

---

## Section 5: PARTIAL Criteria -- Change Requests

These 33 PARTIAL criteria require extensions to existing requirements.md
criteria rather than new criteria. Each is a CHANGE REQUEST to the owning spec.

| EARS ID | Target spec | Existing criterion | Change needed |
|---------|-------------|-------------------|---------------|
| Edit-profile-persist | edit-operations | configuration-system Req (settings) | Add ISPF profile concept and persistence |
| Edit-AUTONUM-mode | sequence-numbers | Req 6.7 (NUMBER ON) | Add AUTONUM as alias |
| Edit-NUM-mode | sequence-numbers | Req 8 (NUMBER SHOW) | Add NUM as alias |
| Edit-RECOVERY-mode | undo-redo-transactions | Req 8.2 (interval=0) | Add RECOVERY ON/OFF command |
| Edit-HILITE-setting | syntax-highlighting | Req (highlighting on) | Add HILITE command to toggle |
| PC-HILITE | syntax-highlighting | Req (highlighting) | Add HILITE primary command criterion |
| SH-HILITE-FIND | find-and-replace | Req 15 (highlight-all) | Add HILITE FIND command |
| RU-RECOVERY-command | undo-redo-transactions | Req 8.2 | Add RECOVERY ON/OFF command |
| EM-CURSOR | lua-macro-engine | Req 2.9 (cursor_line/col) | Extend to cover CURSOR function API |
| ISPF-1.2 | menu-and-statusbar | (dialogs exist) | Add data entry panel formal criterion |
| ISPF-1.3 | menu-and-statusbar | FFW-JES Req 9 | Add general list panel criterion |
| ISPF-4.1 | menu-and-statusbar | navigation-commands Req 1 | Extend LOCATE to cover list panels |
| TSO-4.2 | menu-and-statusbar | navigation-commands Req 3 | Add HALF/CSR/MAX/DATA scroll amounts |
| TSO-2.5 | startup-and-session | FFW-JES (job status) | Add STATUS command routing criterion |
| TSO-EDIT-1 | command-semantics | startup-and-session (EDIT routing) | Add formal EDIT command criterion |
| SDSF-1.3 | FFW-JES | Req 9 (status) | Add SDSF-style message area criterion |
| SDSF-1.4 | FFW-JES | (command integration) | Add COMMAND INPUT ===> field criterion |
| SDSF-2.1 | FFW-JES | Req 9.10 (context menu) | Extend to NP column action characters |
| SDSF-JQ-4 | FFW-JES | Req 9 (status display) | Add dedicated ST panel criterion |
| SDSF-JQ-6 | FFW-JES | Req 9 (sortable columns) | Add specific column definitions |
| SDSF-JQ-7 | FFW-JES | Req 9.4 (filtering) | Add PREFIX/OWNER/DEST specific fields |
| SDSF-FILTER-5 | FFW-JES | Req 9 (sortable columns) | Add SORT command criterion |
| SDSF-SCROLL-1-5 | FFW-JES | navigation-commands (scroll) | Add SDSF panel scroll criterion |
| SDSF-5.1 | FFW-JES | context-help spec | Add SDSF-specific panel help criterion |
| SET-6 | FFW-JES | Req 9.7 (refresh interval) | Add SET DELAY command criterion |
| PERSIST-1 | FFW-JES | startup-and-session (persistence) | Add SDSF-specific settings persistence |

---

## Section 6: OUT-OF-SCOPE Requirements

These 2 criteria are not applicable to a desktop tool and will never be
added to any requirements.md. They are documented here for traceability.

| EARS ID | Description | Rationale |
|---------|-------------|-----------|
| Edit-PACK-mode | PACK ON enables data compression | z/OS-specific DASD packing mechanism. No equivalent on desktop filesystems. The NativeFileProvider and SqliteRecordProvider have no concept of DASD block packing. |
| PERSIST-2 | Special DDNames (ISFMIGNB/ISFMIGXB/ISFMIGNP) | z/OS-specific DDName mechanism for SDSF migration control. No equivalent in the FileForge VFS model. |

---

## Section 7: DEFERRED (P3) Requirements

These 27 criteria are applicable in principle but deliberately deferred.
They will be recorded in `docs/specs/ears-integration/deferred-requirements.md`
(to be created in EI-5 or EI-6) and will NOT be added to any requirements.md
until explicitly promoted.

### REXX Data Stack (REXX-5.x) -- 7 criteria

| EARS ID | Description |
|---------|-------------|
| REXX-5.1 | PUSH -- add to top of data stack |
| REXX-5.2 | QUEUE -- add to bottom of data stack |
| REXX-5.3 | PULL -- remove from top of data stack |
| REXX-5.4 | QUEUED -- return stack element count |
| REXX-5.5 | MAKEBUF -- create new buffer on stack |
| REXX-5.6 | DROPBUF -- remove buffer from stack |
| REXX-5.7 | NEWSTACK/DELSTACK -- private stack management |

Rationale: REXX data stack is an advanced REXX feature. The P2 REXX
execution bridge (CG) covers the core execution model. Data stack
support requires a more complete REXX runtime and is deferred until
the P2 REXX bridge is proven.

### SDSF Advanced JES Panels (SDSF-JES-1 through SDSF-JES-4) -- 4 criteria

| EARS ID | Description |
|---------|-------------|
| SDSF-JES-1 | MAS panel -- multi-access spool |
| SDSF-JES-2 | JG panel -- job group |
| SDSF-JES-3 | SRVC panel -- WLM service class |
| SDSF-JES-4 | SE panel -- scheduling environment |

Rationale: These panels require deep JES2/JES3 internals knowledge and
WLM integration. They are beyond the scope of the initial SDSF emulation.

### SDSF REXX Interface (SDSF-REXX-1 through SDSF-REXX-7) -- 7 criteria

| EARS ID | Description |
|---------|-------------|
| SDSF-REXX-1 | ISFCALLS -- enable/disable SDSF REXX interface |
| SDSF-REXX-2 | ISFEXEC -- execute SDSF command from REXX |
| SDSF-REXX-3 | ISFACT -- perform action on SDSF rows |
| SDSF-REXX-4 | ISFBROWSE -- browse SDSF output from REXX |
| SDSF-REXX-5 | ISFSLASH -- issue slash command from REXX |
| SDSF-REXX-6 | ISFGET -- retrieve SDSF variable values |
| SDSF-REXX-7 | ISFLOG -- write to SDSF log from REXX |

Rationale: The SDSF REXX interface depends on both the P2 REXX bridge
(CG) and the P2 SDSF panels (CH) being complete. It is a P3 integration
layer on top of two P2 features.

---

## Section 8: Ordered Phase Sequence

The complete ordered sequence of all pending and new phases, with dependencies.

### Pre-EARS pending work (must not be blocked by EI-5)

```
BV.1  -- CatalogLocation refactor (no deps, start immediately)
  |
  +-- BS.8  -- Staged transaction protocol (depends on BS.7 done)
       |
       +-- BS.9  -- Integrity/backup/restore
            |
            +-- BS.10 -- Audit trail + migrations
                 |
                 +-- BS.11 -- Security hardening
                      |
                      +-- BS.12 -- Master/user catalogue hierarchy
                           |
                           +-- BS.13 -- Record-oriented editor integration
                                |
                                +-- BS.14 -- Non-functional validation
                                     |
                                     +-- BS.15 -- design.md update
                                          |
                                          +-- ff-vfs tasks (new -- Req 9-12)
                                               |
                                               +-- BU.2-BU.9 (SQLite integration)
```

### EARS P1 phases (parallel to BS Wave 3-4, no dependency)

```
BW  -- edit-operations P1 (no deps on BS)
BX  -- line-commands P1 (no deps)
BY  -- sequence-numbers P1 (no deps)
BZ  -- menu-and-statusbar P1/P2 (no deps)
CA  -- startup-and-session P1 (no deps)
CB  -- command-semantics P1/P2 (no deps on BS; TSO commands delegate to ff-dscatalog API)
CC  -- FFW-JES P1 core (no deps on BS)
CD  -- FFW-JES P1 extended (depends on CC)
```

### EARS P2 phases (follow P1 phases)

```
CE  -- undo-redo-transactions P2 (follows BW)
CF  -- syntax-highlighting P2 (no deps)
CG  -- lua-macro-engine P2 (follows BW, CB)
CH  -- FFW-JES P2 (follows CC, CD)
CI  -- command-semantics P2 (follows CB)
```

### Full recommended execution order

```
1.  BV.1   (immediate -- pure refactor, no gate)
2.  BW     (EI-5 Batch 1 -- edit-operations P1)
3.  BX     (EI-5 Batch 3 -- line-commands P1)
4.  BY     (EI-5 Batch 4 -- sequence-numbers P1, alias extensions only)
5.  BZ     (EI-5 Batch 7 -- menu-and-statusbar P1/P2)
6.  CA     (EI-5 Batch 8 -- startup-and-session P1)
7.  CB     (EI-5 Batch 10 -- command-semantics P1/P2)
8.  CC     (EI-5 Batch 11a -- FFW-JES P1 core)
9.  CD     (EI-5 Batch 11b -- FFW-JES P1 extended, depends on CC)
    [BS Wave 3-4 runs in parallel: BS.8 -> BS.9 -> ... -> BS.15 -> ff-vfs -> BU]
10. CE     (EI-5 Batch 12 -- undo-redo-transactions P2)
11. CF     (EI-5 Batch 13 -- syntax-highlighting P2)
12. CG     (EI-5 Batch 14 -- lua-macro-engine P2)
13. CH     (EI-5 Batch 15 -- FFW-JES P2)
14. CI     (EI-5 Batch 16 -- command-semantics P2)
```

Skipped in EI-5 (no gate required):
- B02 find-and-replace (0 new/partial)
- B05 hex-display (0 new/partial)
- B06 tabs-and-mask (0 new/partial)
- B09 function-keys-and-history (0 new/partial)

---

## Section 9: ff-vfs Task Gap (EI-3 Finding)

The EI-3 audit identified that ff-vfs Req 9-12 have TCR rows marked NOT COVERED
but no corresponding tasks in virtual-file-system/tasks.md.

These tasks must be created before BS.8 begins. They are not an EI-5 batch
(the requirements already exist) -- they are a task gap fix.

| Requirement | Description | Depends on |
|-------------|-------------|------------|
| ff-vfs Req 9.1-9.5 | StorageProvider trait in ff-vfs | BS.15 |
| ff-vfs Req 10.1-10.6 | POSIX files as native objects | BS.15 |
| ff-vfs Req 11.1-11.5 | VFS staged transaction protocol | BS.15 |
| ff-vfs Req 12.1-12.5 | workspace.backup/restore/reconcile/diagnose | BS.15 |

Action: Add a new task group to virtual-file-system/tasks.md (Tasks 5.x)
covering these four requirements. This is a standalone gate (no new
requirements needed -- requirements already exist) that should be executed
before or alongside BS.8 planning.

---

## Section 10: project-master/tasks.md Updates Required (EI-6)

After EI-5 gate execution, project-master/tasks.md will need the following
new phase entries added (EI-6 task):

| Phase | One-line description |
|-------|---------------------|
| BW | edit-operations: CAPS/NULLS/PROFILE/SUBMIT/CREATE/REPLACE/BROWSE/VIEW/nested EDIT |
| BX | line-commands: O/W/F/L/]/S line commands |
| BY | sequence-numbers: AUTONUM and NUM alias extensions |
| BZ | menu-and-statusbar: SCROLL field, fastpath, split screen, list panel LOCATE |
| CA | startup-and-session: session timestamps, LOGOFF, TIME command |
| CB | command-semantics: ALLOCATE through STATUS + FTSO operand parsing |
| CC | FFW-JES P1 core: panel framework, NP column, action chars, main panel |
| CD | FFW-JES P1 extended: ST panel, filter/sort/find/locate, SET P1 commands |
| CE | undo-redo-transactions: SETUNDO command, RECOVERY ON/OFF |
| CF | syntax-highlighting: HILITE ON/OFF/LOGIC/PAREN/FIND |
| CG | lua-macro-engine: ISREDIT/ISPEXEC/IMACRO, REXX bridge, FFCMD |
| CH | FFW-JES P2: overtype, help, log/system panels, browse/print, SET P2 |
| CI | command-semantics P2: OUTPUT/CANCEL/SEND/PROFILE/PRINTDS |

---

## Summary

| Category | Count |
|----------|------:|
| New phases assigned (BW-CI) | 13 |
| Skipped batches (no gate needed) | 4 (B02, B05, B06, B09) |
| NEW criteria to be added | 136 (across 13 phases) |
| PARTIAL criteria (change requests) | 33 (distributed across phases) |
| OUT-OF-SCOPE criteria | 2 (documented, never added) |
| DEFERRED P3 criteria | 27 (to deferred-requirements.md) |
| Pre-EARS pending phases | 3 (BS Wave 3-4, BU, BV) |
| ff-vfs task gap | 1 (4 requirements, ~20 tasks to create) |
