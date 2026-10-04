# FFWB Command Environments -- Vision and Catalogue

STATUS: VISION / REFERENCE. This document captures the FULL intended set of FFWB
Command Environments and the mainframe pattern they follow. It is deliberately
AHEAD of implementation: only a small subset is built now (see "Implementation
scope today"). The rest are documented so the environment framework is designed
to ACCOMMODATE them and so each future environment has a predefined name and home
-- NOT so they are built ahead of their consuming contexts.

Owner direction: "put these into a document, but we will not implement most of
these -- only what is relevant right now."

Plain ASCII only.

---

## The principle

A defining characteristic of the IBM mainframe ecosystem is that it is NOT one
giant command set: it is many specialized command environments, each with its
own commands, syntax, objects, and mental model. A user switches INTO an
environment based on what they are doing (REXX `ADDRESS`).

FFWB deliberately adopts the same pattern (CR-CH-053): a named Command
Environment owns the commands relevant to its context; the active context selects
the active environment; the always-present base environment is the fallback; an
explicit address targets a specific environment regardless of context.

## IBM mainframe command environments (reference)

| Environment | Purpose | Typical commands |
|-------------|---------|------------------|
| TSO | Interactive OS shell | ALLOC, LISTCAT, DELETE, RENAME |
| ISPF Primary Option Menu | Application launcher | option selections |
| ISPF Editor | Dataset editing | SAVE, CANCEL, FIND, CHANGE, LOCATE |
| ISPF Browse | Read-only viewing | FIND, LOCATE |
| SDSF | JES spool management | ST, DA, H, O |
| JES2 / JES3 | Job Entry System | submit/manage batch jobs |
| JCL | Batch execution language | JOB, EXEC, DD |
| IDCAMS | Dataset/catalog admin | DEFINE, DELETE, ALTER, LISTCAT |
| DB2 SPUFI | SQL execution | SELECT, INSERT, UPDATE |
| DB2 DSN | DB2 administration | BIND, REBIND, RUN |
| CICS CEMT | Online transaction control | INQUIRE, SET |
| RACF | Security administration | ADDUSER, PERMIT |
| IPCS | Dump analysis | VERBX, SUMMARY |
| REXX | Scripting | ADDRESS TSO, ADDRESS ISPEXEC |
| DFSORT / ICETOOL | Data transformation | SORT, INCLUDE, OMIT |
| SMP/E | Software installation | APPLY, ACCEPT |
| NetView | Network management | DISPLAY, VARY |
| OMEGAMON | Performance monitoring | monitoring commands |

## FFWB environment catalogue (the FF* naming convention)

Authentic `FF*` names mapped one-for-one onto the environments experienced
mainframe developers instinctively expect.

| IBM | FFWB env | Purpose | Example commands | FFWB home (spec / crate) |
|-----|----------|---------|------------------|--------------------------|
| TSO | **FFCMD** | General workbench command env (global ops, navigation, session) | OPEN, EDIT, BROWSE, CLOSE, HELP, EXIT, FILES, CONFIG, START, SWAP | EXISTS: `resolve_target` / `ShellTargetResolver` (the shell front door) |
| ISPF Editor | **FFEDIT** | Text editing (COMMAND line) | SAVE, CANCEL, UNDO, REDO, FIND, CHANGE, LOCATE | EXISTS as ladder arms (`commands_ladder_b2.rs`) -- to become the FFEDIT environment |
| ISPF Editor (prefix area) | **FFLINE** | Line commands (prefix gutter) | D, DD, M, MM, C, CC, A, B, R, RR, repeat counts | EXISTS: prefix-area intake -> `ff-command-semantics` Command_Engine. A SIBLING of FFEDIT, both active when an editor is focused. SEPARATE intake (prefix area, NOT the command line); not routed through the command-line front door. Rarely addressed by macros. Cooperates with FFEDIT (C/CC mark a source, A/B a destination for a COPY/MOVE) over the shared active TabState. |
| ISPF Browse | **FFBROWSE** | Read-only viewer | FIND, LOCATE, HEX ON, WRAP OFF | FUTURE (hex-display, line-wrap-toggle, custom-file-viewers specs) |
| IDCAMS | **FFAMS** | Dataset/catalog admin | LISTCAT, ALLOCATE, DELETE, RENAME, COPY, ATTRIB, VERSION | FUTURE (idcams-emulator, dataset-allocator, virtual-catalog-manager specs) |
| SDSF / JES2 | **FFJES** | Job management | STATUS, HOLD, RELEASE, PURGE, CANCEL, OUTPUT | FUTURE (jes-emulator spec) |
| JCL | **FFJOB** | Job definition | SUBMIT, VALIDATE, EXPAND | FUTURE (jes-emulator / jcl-resolver specs) |
| DB2 SPUFI/DSN | **FFSQL** | Database / SQL | CONNECT, RUN, EXPLAIN, EXPORT, SELECT ... | FUTURE (database-tool spec) |
| CICS | **FFCICS** | Dialog/application env | INSTALL APP, START TRANS, STOP TRANS, TRACE, TEST PANEL | FUTURE (dialog-processing vision) |
| TSO dataset | **FFVFS** | Virtual file system / storage | LIST, MOUNT, UNMOUNT, EXPORT, IMPORT | FUTURE (virtual-file-system spec) |
| SMP/E + RACF + console | **FFADMIN** | Workbench administration | USER LIST, ROLE LIST, PLUGIN LIST/INSTALL, CONFIG SHOW | FUTURE (plugin-manager-ui, configuration-system specs) |
| IPCS + CEDF | **FFDEBUG** | Debug env | BREAK, TRACE, STACK, DUMP | FUTURE (automated-dialog-testing; panel/dialog debugging) |
| OMEGAMON | **FFMON** | Monitoring env | CPU, MEMORY, JOBS, THREADS, CACHE | FUTURE |
| PDS member mgmt | **FFLIB** | Library/repository env | LIST LIB, COPY MEMBER, MOVE MEMBER, COMPARE MEMBER (REQ/ADR/DESIGN/TEST/DOC libraries) | FUTURE (document-driven architecture) |

Owner's recommended CORE set for the eventual product: FFCMD, FFEDIT, FFBROWSE,
FFAMS, FFJES, FFSQL, FFCICS, FFDEBUG.

## Implementation scope TODAY (what CR-CH-053 actually builds)

ONLY the environment FRAMEWORK plus the TWO environments that already exist in
the running app:

- **FFCMD** -- already real: it IS `resolve_target` / `ShellTargetResolver` (the
  shell/workbench front door). CR-CH-053 names it FFCMD and treats it as the
  always-present base environment. No rewrite.
- **FFEDIT** -- the editor-action COMMAND-LINE verbs that B080 correctly left on
  the shell ladder (LOCATE/FIND/CHANGE/RFIND/RCHANGE/EXCLUDE/SHOW/INCLUDE/RESET/
  SORT/TOP/BOTTOM/UP/DOWN/LEFT/RIGHT/CAPS/NULLS/STATS/LOCK/PROFILE/HILITE/SCROLL).
  CR-CH-053 gives them their proper home as the FFEDIT environment, migrated out
  of the command-line front door's shared ladder.
- **FFLINE** -- the prefix-area line commands (D/DD/M/MM/C/CC/A/B/R/RR + repeat
  counts). MODELED now as a separate SIBLING environment of FFEDIT; its intake
  (prefix gutter -> Command_Engine) ALREADY works and is NOT rebuilt now and NOT
  routed through the command-line front door. CR-CH-053 only NAMES it as the
  FFLINE environment so the architecture is coherent and a future
  `ADDRESS FFLINE` macro target has a defined home. No line-command code change
  in phase 1.

EVERYTHING ELSE in the catalogue (FFBROWSE, FFAMS, FFJES, FFJOB, FFSQL, FFCICS,
FFVFS, FFADMIN, FFDEBUG, FFMON, FFLIB) is NOT built now. Each is registered only
when its consuming context/subsystem is actually built (e.g. FFSQL arrives with
the database tool; FFJES with the JES emulator; FFAMS with the IDCAMS emulator).
The framework is designed so adding one is: define the context's TabKind, map
that kind -> its environment in the active-env derivation, and register the
environment's resolver. No front-door change.

## Naming + addressing

- Canonical FFWB names are the `FF*` set above. Mainframe aliases map onto them
  so a macro author can address by the familiar name: `TSO` -> FFCMD,
  `ISREDIT` -> FFEDIT (and future `IDCAMS` -> FFAMS, `SDSF` -> FFJES, etc.).
  Reconciles with the already-specified lua-macro-engine ADDRESS host
  environments (Req 11.11-11.14).
- Addressing follows REXX `ADDRESS`: active environment first, explicit address
  overrides, "already in that environment -> address redundant".

## Why document the full set now

Naming and shape the whole landscape ONCE so: (1) the environment framework is
proven to scale to ~12 environments, not just 2; (2) every future subsystem has
a predefined environment name and a known integration recipe; (3) we avoid a
piecemeal, inconsistent command surface. We BUILD only FFCMD + FFEDIT now;
the rest is the roadmap the framework must not preclude.
