# Gap Analysis
# EI-1 Output -- EARS Requirements vs Existing Specs

**Status:** EI-1 complete -- analysis only, no requirements.md files modified
**Method:** Each existing requirements.md was read in full and mapped against the
EARS source criteria. Classifications follow the EI-2 definitions:
- COVERED: existing criterion matches the EARS criterion
- PARTIAL: existing criterion partially covers the EARS criterion
- NEW: no existing criterion covers the EARS criterion
- OUT-OF-SCOPE: requires z/OS-specific hardware not emulable on desktop

---

## Area 1: ISPF Edit Session and Profile
**EARS source:** ispf-ears edit session lifecycle, profile and modes files
**Existing spec:** edit-operations/requirements.md (15 requirements, 80+ criteria)

### Summary
The edit-operations spec is comprehensive. It covers insert/overstrike modes,
delete operations, line manipulation, selection model, multi-caret, rectangular
selection, clipboard integration, transaction recording, save operations, BOUNDS,
and command-driven dispatch. The ISPF EARS criteria for edit session lifecycle
and profile are largely covered.

### Criterion-level findings

| EARS ID | Description | Classification | Existing coverage |
|---------|-------------|----------------|-------------------|
| Edit session open | WHEN EDIT command issued, editor opens dataset | COVERED | edit-operations Req 12 (Save), startup-and-session Req 14.6 |
| Edit session close (END/SAVE) | END saves and exits, CANCEL discards | COVERED | navigation-commands Req 11 (SAVE/CANCEL/END delegation) |
| Edit session close (CANCEL) | CANCEL exits without saving | COVERED | navigation-commands Req 11.2 |
| Insert mode default | Editor starts in insert mode | COVERED | edit-operations Req 1.4 |
| Overstrike mode toggle | Insert key toggles insert/overstrike | COVERED | edit-operations Req 3.3 |
| Mode indicator in status bar | INSERT/OVERSTRIKE shown in status bar | COVERED | edit-operations Req 3.4, menu-and-statusbar Req 6.3 |
| CAPS mode (uppercase on input) | CAPS primary command forces uppercase | NEW | No criterion in edit-operations |
| HEX mode toggle | HEX ON/OFF switches hex display | COVERED | hex-display Req 1 |
| NULLS mode | NULLS ON/OFF controls null character handling | NEW | No criterion in any spec |
| TABS mode | TABS command displays tab ruler | COVERED | tabs-and-mask Req 1 |
| PROFILE command | PROFILE displays/changes edit profile settings | NEW | No criterion -- profile persistence not specified |
| Profile persistence across sessions | Edit profile settings persist | PARTIAL | configuration-system covers settings persistence but no ISPF profile concept |
| RECOVERY mode | RECOVERY ON enables periodic save | PARTIAL | undo-redo-transactions Req 8 covers recovery files but not RECOVERY command |
| AUTONUM mode | AUTONUM ON enables auto line numbering | PARTIAL | sequence-numbers Req 6.7 covers NUMBER ON but not AUTONUM alias |
| STATS mode | STATS ON shows member statistics | NEW | No criterion |
| IMACRO setting | Initial macro run on edit session open | NEW | No criterion |
| PACK mode | PACK ON enables data compression | OUT-OF-SCOPE | z/OS-specific DASD packing |
| Number mode (NUM ON/OFF) | NUM controls sequence number display | PARTIAL | sequence-numbers covers NUMBER SHOW but not NUM alias |
| HILITE setting | HILITE controls syntax highlighting | PARTIAL | syntax-highlighting covers highlighting but not HILITE command |
| LOCK setting | LOCK prevents profile changes | NEW | No criterion |

---

## Area 2: ISPF Line Commands
**EARS source:** ispf-ears line commands file; tso-ears/tso-commands.md TSO-EDIT-3
**Existing spec:** line-commands/requirements.md (14 requirements)

### Summary
The line-commands spec is thorough. D/Dn/DD, I/In, R/Rn/RR, C/CC, M/MM, A/B,
X/Xn/XX, T/TT, U/UU, >/>/>> , </</<<, bounds-aware shift, block pairing,
compatibility validation, and pending state management are all defined.

### Criterion-level findings

| EARS ID | Description | Classification | Existing coverage |
|---------|-------------|----------------|-------------------|
| D/Dn/DD delete | Delete line(s) | COVERED | line-commands Req 1 |
| I/In insert | Insert blank line(s) | COVERED | line-commands Req 2 |
| R/Rn/RR repeat | Duplicate line(s) | COVERED | line-commands Req 3 |
| C/CC copy source | Mark copy source | COVERED | line-commands Req 4 |
| M/MM move source | Mark move source | COVERED | line-commands Req 5 |
| A/B target | Mark insertion target | COVERED | line-commands Req 6 |
| X/Xn/XX exclude | Exclude line(s) from view | COVERED | line-commands Req 7 |
| T/TT tag | Tag line(s) | COVERED | line-commands Req 8 |
| U/UU untag | Untag line(s) | COVERED | line-commands Req 8 |
| >/>/>> shift right | Shift content right | COVERED | line-commands Req 9 |
| </</<<  shift left | Shift content left | COVERED | line-commands Req 10 |
| )/)) bounds shift right | Shift within bounds right | COVERED | line-commands Req 11 |
| (/(( bounds shift left | Shift within bounds left | COVERED | line-commands Req 11 |
| O overlay | Overlay lines (ISPF O line command) | NEW | No criterion in line-commands |
| W copy to clipboard | Copy line to clipboard (ISPF W) | NEW | No criterion |
| S show/unexclude | Show excluded line (ISPF S) | PARTIAL | exclude-show-filter covers SHOW command but not S line command |
| F first/last | First/last line of excluded block | NEW | No criterion |
| L label | Assign label to line | NEW | No criterion |
| ] shift right one | Shift right by 1 (ISPF ]) | NEW | No criterion -- ] not in line-commands spec |
| TSO-EDIT-3 line commands | d/i/r/c/m/a/b/dd/cc/mm block forms | COVERED | line-commands Req 1-8 |

---

## Area 3: ISPF Primary Commands (FIND, CHANGE, LOCATE, SORT, EXCLUDE, SHOW, RESET)
**EARS source:** ispf-ears primary commands and find/change files
**Existing spec:** command-semantics, find-and-replace, navigation-commands, exclude-show-filter

### Summary
Core primary commands are well covered. FIND/RFIND/CHANGE/RCHANGE with
literal/regex/hex modes, scope modifiers, column ranges, and bounds are fully
specified. LOCATE, SORT, COLS, BOUNDS, EXCLUDE/SHOW/RESET are all defined.

### Criterion-level findings

| EARS ID | Description | Classification | Existing coverage |
|---------|-------------|----------------|-------------------|
| FIND literal | FIND 'text' with direction/scope modifiers | COVERED | find-and-replace Req 1-2 |
| FIND hex | FIND X'hex' | COVERED | find-and-replace Req 3 |
| FIND regex | FIND REGEX 'pattern' | COVERED | find-and-replace Req 4 |
| RFIND | Repeat previous FIND | COVERED | find-and-replace Req 5 |
| CHANGE literal | CHANGE 'old' 'new' with modifiers | COVERED | find-and-replace Req 6-7 |
| CHANGE regex | CHANGE REGEX with group substitution | COVERED | find-and-replace Req 8 |
| RCHANGE | Repeat previous CHANGE | COVERED | find-and-replace Req 9 |
| LOCATE line number | LOCATE n scrolls to line n | COVERED | navigation-commands Req 1 |
| LOCATE label | LOCATE label scrolls to named label | COVERED | navigation-commands Req 1.3 |
| SORT | Sort lines by column key | COVERED | navigation-commands Req 2 |
| EXCLUDE | Exclude lines matching pattern | COVERED | exclude-show-filter spec |
| SHOW/INCLUDE | Show excluded lines matching pattern | COVERED | exclude-show-filter spec |
| RESET | Clear display artifacts and find state | COVERED | command-semantics Req 6, find-and-replace Req 16 |
| COPY primary command | Copy lines to position | COVERED | navigation-commands Req 14 |
| MOVE primary command | Move lines to position | COVERED | navigation-commands Req 15 |
| DELETE primary command | Delete lines by scope | COVERED | navigation-commands Req 13 |
| SUBMIT from editor | Submit current buffer as job | NEW | No criterion in command-semantics for SUBMIT from editor context |
| SAVE primary command | Save file | COVERED | navigation-commands Req 11.1 |
| CANCEL primary command | Exit without saving | COVERED | navigation-commands Req 11.2 |
| END primary command | Save and exit | COVERED | navigation-commands Req 11.3 |
| UNDO primary command | Undo last edit | COVERED | navigation-commands Req 17 |
| REDO primary command | Redo last undo | COVERED | navigation-commands Req 17 |
| MACRO/EXEC/RUN | Invoke macro | COVERED | navigation-commands Req 16 |
| HILITE command | Toggle syntax highlighting | PARTIAL | syntax-highlighting covers highlighting but no HILITE command criterion |
| SETUNDO command | Control undo settings | NEW | No criterion |
| COMPARE command | Compare with another dataset | NEW | No criterion in command-semantics |
| CREATE command | Create new dataset from lines | NEW | No criterion |
| REPLACE command | Replace dataset content | NEW | No criterion |
| EDIT command (from editor) | Open another dataset for edit | NEW | No criterion for nested EDIT |
| BROWSE command | Open dataset for browse | NEW | No criterion in command-semantics |
| VIEW command | Open dataset for view | NEW | No criterion |

---

## Area 4: ISPF Edit Recovery and Undo
**EARS source:** ispf-ears recovery and undo file
**Existing spec:** undo-redo-transactions/requirements.md (18 requirements)

### Summary
The undo-redo-transactions spec is extremely comprehensive -- it covers undo
stack, redo stack, transaction boundaries, coalescing, save point, dirty flag,
bulk transactions, recovery files, selection history, non-undoable operations,
per-document undo, tentative actions, container actions, logical record identity,
UNDO/REDO commands, history validation, and scrap stack. ISPF recovery/undo
criteria are well covered.

### Criterion-level findings

| EARS ID | Description | Classification | Existing coverage |
|---------|-------------|----------------|-------------------|
| UNDO command | Undo last edit | COVERED | undo-redo-transactions Req 4, 15 |
| UNDO n | Undo n operations | COVERED | undo-redo-transactions Req 4.6, 15.6 |
| REDO command | Redo last undo | COVERED | undo-redo-transactions Req 4, 15 |
| Recovery file creation | Periodic save of undo state | COVERED | undo-redo-transactions Req 8 |
| Recovery file restore on open | Offer recovery on next open | COVERED | undo-redo-transactions Req 8.4-8.5 |
| RECOVERY ON/OFF command | Enable/disable recovery | PARTIAL | Req 8.2 covers interval=0 disables but no RECOVERY command |
| Save point tracking | Dirty flag based on save point | COVERED | undo-redo-transactions Req 5 |
| Modified line markers | Visual indicator for changed lines | COVERED | undo-redo-transactions Req 5.8, edit-operations Req 11.6 |
| SETUNDO command | Configure undo settings | NEW | No SETUNDO command criterion |

---

## Area 5: ISPF Syntax Highlighting (HILITE)
**EARS source:** ispf-ears syntax highlighting file
**Existing spec:** syntax-highlighting/requirements.md (15 requirements)

### Summary
The syntax-highlighting spec is comprehensive -- lexer trait, style assignment,
incremental re-highlighting, demand-driven styling, keyword matching, comment
detection, sub-styles, fold-level assignment, idle-time styling, property-based
configuration, GUI independence, theme integration, lexer lifecycle, style context,
and display-line-mapping integration are all defined.

### Criterion-level findings

| EARS ID | Description | Classification | Existing coverage |
|---------|-------------|----------------|-------------------|
| HILITE ON/OFF command | Toggle syntax highlighting | NEW | No HILITE command criterion -- highlighting is always on |
| HILITE LOGIC | Highlight logical operators | NEW | No criterion for HILITE LOGIC mode |
| HILITE FIND | Highlight find matches | PARTIAL | find-and-replace Req 15 covers highlight-all but not HILITE FIND command |
| HILITE PAREN | Highlight matching parentheses | NEW | No criterion |
| Keyword highlighting | Keywords in distinct colour | COVERED | syntax-highlighting Req 5 |
| Comment highlighting | Comments in distinct colour | COVERED | syntax-highlighting Req 6 |
| String highlighting | String literals highlighted | COVERED | syntax-highlighting Req 5 (keyword sets) |
| Language detection | Language detected from file type | COVERED | syntax-highlighting Req 13 |
| Incremental re-highlight | Only changed region re-highlighted | COVERED | syntax-highlighting Req 3 |
| Custom colour configuration | User-configurable highlight colours | COVERED | syntax-highlighting Req 12, theme-and-appearance |

---

## Area 6: ISPF Boundaries, Tabs, and Masks
**EARS source:** ispf-ears boundaries, tabs, masks file
**Existing spec:** tabs-and-mask/requirements.md (18 requirements)

### Summary
The tabs-and-mask spec is comprehensive. TABS display/configure, MASK display/edit,
MASK OFF, default tab stops, language profile integration, Tab key behaviour,
RESET interaction, and display artifact compatibility are all defined.

### Criterion-level findings

| EARS ID | Description | Classification | Existing coverage |
|---------|-------------|----------------|-------------------|
| TABS primary command | Display tab ruler | COVERED | tabs-and-mask Req 1 |
| TABS col1 col2 ... | Set tab stop positions | COVERED | tabs-and-mask Req 2 |
| TABS line command | Insert tab ruler at line | COVERED | tabs-and-mask Req 3 |
| MASK primary command | Display insert mask | COVERED | tabs-and-mask Req 6 |
| MASK OFF | Clear insert mask | COVERED | tabs-and-mask Req 7 |
| MASK line command | Insert mask at line | COVERED | tabs-and-mask Req 8 |
| Mask applied to I/In | New lines pre-filled with mask | COVERED | tabs-and-mask Req 9 |
| Default tab stops from config | Global default tab stops | COVERED | tabs-and-mask Req 4 |
| Language profile tab stops | Per-language default tab stops | COVERED | tabs-and-mask Req 4.3 |
| Tab key advances to tab stop | Tab key uses tab stop list | COVERED | tabs-and-mask Req 5 |
| BOUNDS command | Set column boundaries | COVERED | navigation-commands Req 5 |
| BNDS line command | Insert bounds display line | COVERED | navigation-commands Req 5.3 |
| COLS command | Display column ruler | COVERED | navigation-commands Req 4 |

---

## Area 7: ISPF Sequence Numbers
**EARS source:** ispf-ears sequence numbers file
**Existing spec:** sequence-numbers/requirements.md (14 requirements)

### Summary
The sequence-numbers spec is comprehensive. Auto-detection, auto-strip on open,
UNNUM command, NUMBER command, NUMBER SHOW, undo integration, BOUNDS interaction,
SAVE interaction, and configuration are all defined.

### Criterion-level findings

| EARS ID | Description | Classification | Existing coverage |
|---------|-------------|----------------|-------------------|
| Auto-detect sequence numbers | Detect on file open | COVERED | sequence-numbers Req 2 |
| Auto-strip on open | Remove sequence numbers before display | COVERED | sequence-numbers Req 3 |
| UNNUM command | Explicit strip | COVERED | sequence-numbers Req 5 |
| NUMBER command | Write sequence numbers | COVERED | sequence-numbers Req 6 |
| NUMBER SHOW | Display overlay without modifying buffer | COVERED | sequence-numbers Req 8 |
| NUMBER ON/OFF | Auto-numbering mode | COVERED | sequence-numbers Req 6.7-6.8 |
| COBOL sequence columns 1-6, 73-80 | Language profile definition | COVERED | sequence-numbers Req 1.5 |
| FORTRAN sequence columns | Language profile definition | COVERED | sequence-numbers Req 1.6 |
| JCL sequence columns | Language profile definition | COVERED | sequence-numbers Req 1.7 |
| Undo UNNUM/NUMBER | Single-step undo | COVERED | sequence-numbers Req 9 |

---

## Area 8: ISPF Hex Display
**EARS source:** ispf-ears hex display file
**Existing spec:** hex-display/requirements.md (16 requirements)

### Summary
The hex-display spec is comprehensive. HEX ON/OFF/toggle, three-pane layout,
configurable bytes-per-row, hex editing, hex search integration, cursor sync,
undo/redo, modified byte indicators, scrolling, binary file handling, hex dump
export, goto offset, uppercase/lowercase digits, FileForge integration, and
session state are all defined.

### Criterion-level findings

| EARS ID | Description | Classification | Existing coverage |
|---------|-------------|----------------|-------------------|
| HEX ON/OFF command | Toggle hex display | COVERED | hex-display Req 1 |
| Three-pane layout | Offset + hex + ASCII | COVERED | hex-display Req 2 |
| Hex editing | Overwrite bytes in hex pane | COVERED | hex-display Req 4 |
| FIND X'hex' | Search for hex byte sequence | COVERED | hex-display Req 5, find-and-replace Req 3 |
| Cursor sync between panes | Hex and ASCII panes stay in sync | COVERED | hex-display Req 6 |
| HEX display in ISPF edit | Hex display within edit session | COVERED | hex-display Req 16 |
| Hex display of record content | Show raw bytes of record | COVERED | hex-display Req 10 |


---

## Area 9: ISPF Edit Macros
**EARS source:** ispf-ears edit macros file
**Existing spec:** lua-macro-engine/requirements.md (10 requirements)

### Summary
The lua-macro-engine spec covers Lua runtime embedding, editor API surface,
event hook system, per-buffer state, MACRO/EXEC/RUN commands, error handling,
security modes, auto-reload, directory scanning, and debugging. ISPF macro
concepts map to Lua with some gaps around ISPF-specific macro API calls.

### Criterion-level findings

| EARS ID | Description | Classification | Existing coverage |
|---------|-------------|----------------|-------------------|
| MACRO command | Invoke macro by name | COVERED | lua-macro-engine Req 5.1 |
| EXEC command | Execute inline expression | COVERED | lua-macro-engine Req 5.2 |
| RUN command | Run macro file by path | COVERED | lua-macro-engine Req 5.3 |
| editor.lines() API | Get line count | COVERED | lua-macro-engine Req 2.2 |
| editor.get_line() API | Get line content | COVERED | lua-macro-engine Req 2.3 |
| editor.set_line() API | Set line content | COVERED | lua-macro-engine Req 2.4 |
| editor.command() API | Dispatch primary command | COVERED | lua-macro-engine Req 2.8 |
| OnOpen/OnSave hooks | File lifecycle event hooks | COVERED | lua-macro-engine Req 3.1 |
| ISREDIT host command environment | ISPF Edit macro calls from REXX | NEW | No criterion -- ISREDIT environment not defined |
| ISPEXEC host command environment | ISPF service calls from REXX | NEW | No criterion -- ISPEXEC environment not defined |
| Macro undo as single transaction | Macro changes undone atomically | COVERED | lua-macro-engine Req 5.4 |
| IMACRO initial macro | Macro run on edit session open | NEW | No criterion for IMACRO setting |
| LINENUM function | Get/set line number in macro | NEW | No criterion |
| CURSOR function | Get/set cursor position in macro | PARTIAL | lua-macro-engine Req 2.9 covers cursor_line/cursor_col |

---

## Area 10: ISPF POM and Panel Navigation
**EARS source:** tso-ears/ispf-panel-navigation.md ISPF-1 through ISPF-5
**Existing spec:** menu-and-statusbar/requirements.md, startup-and-session/requirements.md

### Summary
The POM and panel navigation are well covered. The startup-and-session spec
defines the full POM with 9 options, tab container, context menus, and
navigation commands. The menu-and-statusbar spec covers the command field,
status bar, tab-order focus cycle, title line, and detachable windows.

### Criterion-level findings

| EARS ID | Description | Classification | Existing coverage |
|---------|-------------|----------------|-------------------|
| ISPF-1.1 Menu panel with OPTION ===> | Menu panel with option input | COVERED | startup-and-session Req 14 |
| ISPF-1.2 Data entry panel with ===> fields | Data entry panel format | PARTIAL | No formal data entry panel spec -- dialogs exist but no EARS-style criterion |
| ISPF-1.3 List panel with NP column | List panel with action field | PARTIAL | FFW-JES Req 9 covers job list panel but no general list panel criterion |
| ISPF-1.4 Edit panel with COMMAND ===> | Edit panel format | COVERED | menu-and-statusbar Req 9, startup-and-session Req 13 |
| ISPF-1.5 COMMAND ===> on all panels | Command field on every panel | COVERED | menu-and-statusbar Req 9 |
| ISPF-1.6 SCROLL ===> field | Scroll field adjacent to command | NEW | No criterion for SCROLL ===> field |
| ISPF-2.1 PF3 returns to previous panel | END returns to previous context | COVERED | function-keys-and-history Req 17.1 |
| ISPF-2.2 PF4 returns to POM | RETURN goes to POM | COVERED | function-keys-and-history Req 17.3 |
| ISPF-2.3 Fastpath notation (3.1) | Navigate to nested option directly | NEW | No criterion for fastpath notation |
| ISPF-2.4 =option jump notation | Jump between options with =2 etc | COVERED | startup-and-session Req 19.1-19.2 |
| ISPF-2.5 X or =X exits | Exit from any panel | COVERED | startup-and-session Req 14.12 |
| ISPF-3.1 PF2 split screen | Split screen at cursor | NEW | No criterion -- split screen not implemented |
| ISPF-3.2 PF9 swap between halves | Swap split screen halves | NEW | No criterion |
| ISPF-3.3 Independent halves | Each half operates independently | NEW | No criterion |
| ISPF-3.4 Unsplit with PF3 | Unsplit by pressing END | NEW | No criterion |
| ISPF-4.1 LOCATE on list panel | Scroll list to matching item | PARTIAL | navigation-commands Req 1 covers LOCATE for editor but not list panels |
| ISPF-4.2 LOCATE nearest match | Scroll to nearest alphabetic item | NEW | No criterion for list panel LOCATE |
| ISPF-4.3 LOCATE partial names | Accept partial name | NEW | No criterion |
| ISPF-5.1 Command history 20 entries | Maintain at least 20 commands | COVERED | function-keys-and-history Req 9 (default 200) |
| ISPF-5.2 PF12 cycles backward | Each PF12 goes one step back | COVERED | function-keys-and-history Req 5 |
| ISPF-5.3 History persists in session | History available within session | COVERED | function-keys-and-history Req 6 |

---

## Area 11: TSO Session Startup and Logon
**EARS source:** tso-ears/tso-session-and-logon.md TSO-1 through TSO-4
**Existing spec:** startup-and-session/requirements.md

### Summary
Session startup is well covered. The startup-and-session spec defines the full
startup sequence, POM as landing panel, session persistence, and exit sequence.
Some TSO-specific concepts (LOGOFF command, session timestamp, READY prompt) are
not yet specified.

### Criterion-level findings

| EARS ID | Description | Classification | Existing coverage |
|---------|-------------|----------------|-------------------|
| TSO-1.1 POM as default landing panel | POM shown on startup | COVERED | startup-and-session Req 14.1 |
| TSO-1.2 Session start timestamp | Timestamp shown in status bar | NEW | No criterion for session timestamp display |
| TSO-1.3 Session end timestamp + logoff message | Logoff confirmation on exit | NEW | No criterion for logoff message |
| TSO-1.4 LOGOFF command | LOGOFF terminates session | NEW | No criterion -- EXIT/=X exist but not LOGOFF |
| TSO-2.1 Command field accepts TSO commands | Command ===> accepts commands | COVERED | menu-and-statusbar Req 9, command-semantics Req 8 |
| TSO-2.2 Command not found message | Error for unknown command | COVERED | command-semantics Req 1.4 |
| TSO-2.3 HELP command | HELP displays available commands | COVERED | command-semantics Req 7 |
| TSO-2.4 TIME command | Display current date and time | NEW | No criterion for TIME command |
| TSO-2.5 STATUS command | Display job status | PARTIAL | FFW-JES covers job status but no STATUS command in command-semantics |
| TSO-3.1 Default PF key assignments | PF1=HELP, PF3=END, PF7=UP, etc. | COVERED | function-keys-and-history Req 15 |
| TSO-3.2 KEYS command | View PF key assignments | COVERED | function-keys-and-history Req 20.1 |
| TSO-3.3 PFSHOW command | Toggle PF key display | COVERED | function-keys-and-history Req 12 |
| TSO-3.4 Change PF key assignments | Edit via Key Configuration dialog | COVERED | function-keys-and-history Req 20 |
| TSO-3.5 PF key assignments persist | Persist across sessions | COVERED | function-keys-and-history Req 6 |
| TSO-4.1 UP/DOWN/LEFT/RIGHT scroll | Scroll commands | COVERED | navigation-commands Req 3 |
| TSO-4.2 Scroll amounts PAGE/HALF/CSR/MAX/DATA/n | Scroll amount modifiers | PARTIAL | navigation-commands Req 3 covers UP n and DOWN n but not HALF/CSR/MAX/DATA modifiers |
| TSO-4.3 SCROLL field retains value | Scroll amount persists | NEW | No criterion for SCROLL ===> field persistence |
| TSO-4.4 TOP/BOTTOM commands | Jump to first/last line | COVERED | navigation-commands Req 3.9-3.10 |

---

## Area 12: TSO Dataset Commands (P1: ALLOCATE through STATUS)
**EARS source:** tso-ears/tso-commands.md TSO-CMD-1 through TSO-CMD-9
**Existing spec:** command-semantics/requirements.md, dataset-catalog, dataset-allocator

### Summary
The existing command-semantics spec does not define TSO dataset commands.
The dataset-catalog and dataset-allocator specs define the underlying services
but not the command-line interface. All TSO dataset commands are NEW criteria
for command-semantics.

### Criterion-level findings

| EARS ID | Description | Classification | Existing coverage |
|---------|-------------|----------------|-------------------|
| TSO-CMD-1 ALLOCATE | Allocate dataset with full operand set | NEW | No command-semantics criterion |
| TSO-CMD-2 FREE | Release allocated dataset | NEW | No command-semantics criterion |
| TSO-CMD-3 DELETE | Delete dataset or member | NEW | No command-semantics criterion (edit DELETE is different) |
| TSO-CMD-4 RENAME | Rename dataset or member | NEW | No command-semantics criterion |
| TSO-CMD-5 LISTCAT | List catalog entries | NEW | No command-semantics criterion |
| TSO-CMD-6 LISTDS | Display dataset attributes | NEW | No command-semantics criterion |
| TSO-CMD-7 LISTALC | List allocated datasets | NEW | No command-semantics criterion |
| TSO-CMD-8 SUBMIT | Submit job for batch execution | NEW | No command-semantics criterion |
| TSO-CMD-9 STATUS | Display job status | NEW | No command-semantics criterion |
| TSO-EDIT-1 EDIT command | Open dataset for editing | PARTIAL | startup-and-session covers EDIT command routing but no formal criterion |
| TSO-EDIT-2 EDIT subcommands | FIND/CHANGE/DELETE/INSERT/etc in EDIT | COVERED | find-and-replace, line-commands, navigation-commands cover these |
| TSO-EDIT-3 Line commands in EDIT | d/i/r/c/m/a/b line commands | COVERED | line-commands Req 1-8 |

---

## Area 13: TSO Dataset Commands (P2: OUTPUT through PRINTDS)
**EARS source:** tso-ears/tso-commands.md TSO-CMD-10 through TSO-CMD-14
**Existing spec:** command-semantics/requirements.md, FFW-JES

### Criterion-level findings

| EARS ID | Description | Classification | Existing coverage |
|---------|-------------|----------------|-------------------|
| TSO-CMD-10 OUTPUT | Display/manage job output | NEW | No command-semantics criterion |
| TSO-CMD-11 CANCEL | Cancel batch job | NEW | No command-semantics criterion (edit CANCEL is different) |
| TSO-CMD-12 SEND | Send message to user | NEW | No command-semantics criterion |
| TSO-CMD-13 PROFILE | Display/change user profile | NEW | No command-semantics criterion |
| TSO-CMD-14 PRINTDS | Print dataset | NEW | No command-semantics criterion |


---

## Area 14: SDSF Panel Framework and SET Commands
**EARS source:** tso-ears/sdsf-panel-framework.md SDSF-1 through SDSF-5, SET-1 through SET-13
**Existing spec:** FFW-JES/requirements.md

### Summary
The FFW-JES spec covers job submission, queue management, initiator pool,
active job monitoring, completion/failure/cancellation, job logs/SYSOUT,
retained output, Job Monitor Panel, hold/release, dataset catalog integration,
Job/Dataset APIs, command integration, provider abstraction, and async execution.
SDSF panel framework criteria are largely NEW -- the FFW-JES spec was written
before the SDSF EARS files were available.

### Criterion-level findings

| EARS ID | Description | Classification | Existing coverage |
|---------|-------------|----------------|-------------------|
| SDSF-1.1 Action bar with pull-down menus | Display/Filter/View/Print/Options/Search/Help | NEW | No criterion in FFW-JES |
| SDSF-1.2 Title line with panel name and line range | Panel title showing LINE n-m (total) | NEW | No criterion |
| SDSF-1.3 Message area in title line | Short error/confirmation messages | PARTIAL | FFW-JES Req 9 covers status but not SDSF-style message area |
| SDSF-1.4 COMMAND INPUT ===> field | Command field at bottom | PARTIAL | FFW-JES has command integration but no COMMAND INPUT ===> field criterion |
| SDSF-1.5 SCROLL ===> field | Scroll field adjacent to command | NEW | No criterion |
| SDSF-1.6 Filter information lines | PREFIX=/DEST=/OWNER=/SYSNAME= display | NEW | No criterion |
| SDSF-1.7 NP column (fixed, non-scrolling) | Action character input column | NEW | No criterion |
| SDSF-1.8 First data column fixed | First column stays visible on scroll | NEW | No criterion |
| SDSF-2.1 Action characters S/?/C/H/A/P/D/E/J/W | Universal action characters | PARTIAL | FFW-JES Req 9.10 covers context menu actions but not NP column action chars |
| SDSF-2.2 SET ACTION display | Show valid action characters | NEW | No criterion |
| SDSF-2.3 = repeats previous action | Repeat last action character | NEW | No criterion |
| SDSF-2.4 // block action | Apply action to block of rows | NEW | No criterion |
| SDSF-2.5 Command-line action syntax | "2 C" to cancel row 2 | NEW | No criterion |
| SDSF-2.6 SET ROWNUM row numbers | Row numbers in NP area | NEW | No criterion |
| SDSF-3.1 Overtypeable fields | Visual distinction for editable fields | NEW | No criterion |
| SDSF-3.2 Overtype to change value | Type over field to change it | NEW | No criterion |
| SDSF-3.3 Command-line overtype syntax | "rows column=value" | NEW | No criterion |
| SDSF-3.4 Overtype Extension pop-up | + opens related fields pop-up | NEW | No criterion |
| SDSF-4.1 Main panel command list | All SDSF commands with name/desc/group | NEW | No criterion |
| SDSF-4.2 Command groups | Jobs/Output/JES/Log/Memory/etc groups | NEW | No criterion |
| SDSF-4.3 S action on main panel | Select command from main panel | NEW | No criterion |
| SDSF-4.4 SET MAIN GROUP | Set grouped main panel | NEW | No criterion |
| SDSF-4.5 MGRP expandable groups | Expandable/collapsible command groups | NEW | No criterion |
| SDSF-4.6 MENU command | Return to main panel | NEW | No criterion |
| SDSF-5.1 Context-sensitive help | PF1/HELP shows panel help | PARTIAL | context-help spec covers help but not SDSF-specific panel help |
| SDSF-5.2 SEARCH in help | Search help content | NEW | No criterion |
| SDSF-5.3 ACTH action character help | Help for action characters | NEW | No criterion |
| SDSF-5.4 COLH column help | Help for column names | NEW | No criterion |
| SDSF-5.5 CMDH command help | Help for commands | NEW | No criterion |
| SET-1 SET ACTION | Display valid action characters | NEW | No criterion |
| SET-2 SET BCOLOR | Color on browse panels | NEW | No criterion |
| SET-3 SET CONFIRM | Confirmation for destructive actions | NEW | No criterion |
| SET-4 SET CURSOR | Cursor positioning on panel display | NEW | No criterion |
| SET-5 SET DATE | Date display format | NEW | No criterion |
| SET-6 SET DELAY | Auto-refresh interval | PARTIAL | FFW-JES Req 9.7 covers refresh interval but not SET DELAY command |
| SET-7 SET HEX | Hex display of column values | NEW | No criterion (different from editor HEX mode) |
| SET-8 SET MAIN | Default main panel | NEW | No criterion |
| SET-9 SET ROWNUM | Row numbers in NP area | NEW | No criterion |
| SET-10 SET SCHARS | Wildcard characters | NEW | No criterion |
| SET-11 SET SCREEN | Color scheme for field types | NEW | No criterion |
| SET-12 WHO command | Display session information | NEW | No criterion |
| SET-13 QUERY AUTH | Display authorized commands | NEW | No criterion |
| PERSIST-1 Save session settings | Persist SET settings across sessions | PARTIAL | startup-and-session covers session persistence but not SDSF-specific settings |
| PERSIST-2 Special DDNames | ISFMIGNB/ISFMIGXB/ISFMIGNP | OUT-OF-SCOPE | z/OS-specific DDName mechanism |

---

## Area 15: SDSF Job Queue Panels, Filter/Sort, Log/System, Browse/Print
**EARS source:** sdsf-job-queue-panels.md, sdsf-filter-sort-search.md, sdsf-log-and-system-panels.md, sdsf-browse-and-print.md
**Existing spec:** FFW-JES/requirements.md

### Summary
The FFW-JES spec covers the Job Monitor Panel with tabbed sub-panels for
Input Queue, Active Jobs, Held Jobs, Output/Completed, Failed, and Cancelled.
It covers filtering, sorting, and context menu actions. The SDSF EARS files
define more specific panel layouts, column definitions, and commands.

### Criterion-level findings

| EARS ID | Description | Classification | Existing coverage |
|---------|-------------|----------------|-------------------|
| SDSF-JQ-1 Input queue panel (I) | Jobs awaiting execution | COVERED | FFW-JES Req 9.1 (Input Queue sub-panel) |
| SDSF-JQ-2 Output queue panel (O) | Completed job output | COVERED | FFW-JES Req 9.1 (Output/Completed sub-panel) |
| SDSF-JQ-3 Held queue panel (H) | Held jobs | COVERED | FFW-JES Req 9.1 (Held Jobs sub-panel) |
| SDSF-JQ-4 Status panel (ST) | All jobs with status | PARTIAL | FFW-JES Req 9 covers status display but no dedicated ST panel |
| SDSF-JQ-5 Active jobs panel (DA) | Currently executing jobs | COVERED | FFW-JES Req 9.1 (Active Jobs sub-panel) |
| SDSF-JQ-6 Column definitions | JOBNAME/JOBID/OWNER/STATUS/etc columns | PARTIAL | FFW-JES Req 9 covers sortable columns but no specific column definitions |
| SDSF-JQ-7 PREFIX/OWNER/DEST filters | Filter by prefix, owner, destination | PARTIAL | FFW-JES Req 9.4 covers filtering but not PREFIX/OWNER/DEST specific fields |
| SDSF-FILTER-1 PREFIX filter | Filter by job name prefix | NEW | No specific PREFIX filter criterion |
| SDSF-FILTER-2 OWNER filter | Filter by job owner | NEW | No specific OWNER filter criterion |
| SDSF-FILTER-3 DEST filter | Filter by output destination | NEW | No criterion |
| SDSF-FILTER-4 FILTER command | Advanced filter expression | NEW | No criterion |
| SDSF-FILTER-5 SORT command | Sort panel columns | PARTIAL | FFW-JES Req 9 covers sortable columns but no SORT command |
| SDSF-FILTER-6 FIND command | Search within panel | NEW | No criterion |
| SDSF-FILTER-7 LOCATE command | Scroll to matching row | NEW | No criterion |
| SDSF-SCROLL-1 through SDSF-SCROLL-5 | Scroll commands in SDSF panels | PARTIAL | navigation-commands covers scroll but not SDSF panel scroll |
| SDSF-LOG-1 System log panel | Display system log | NEW | No criterion |
| SDSF-LOG-2 User log panel (ULOG) | Display user log | NEW | No criterion |
| SDSF-LOG-3 LOG command | Navigate log | NEW | No criterion |
| SDSF-LOG-4 NEXT/PREV/SNAPSHOT | Log navigation commands | NEW | No criterion |
| SDSF-SYS-1 SYS panel | System information | NEW | No criterion |
| SDSF-SYS-2 DASH panel | Dashboard | NEW | No criterion |
| SDSF-SYS-3 INIT panel | Initiator information | NEW | No criterion |
| SDSF-SYS-4 JC panel | Job class information | NEW | No criterion |
| SDSF-SYS-5 SP panel | Spool information | NEW | No criterion |
| SDSF-BROWSE-1 Browse job output | Open job output for browsing | COVERED | FFW-JES Req 7 (JobLogViewerPanel) |
| SDSF-BROWSE-2 Browse settings | Configure browse display | NEW | No criterion |
| SDSF-BROWSE-3 Print job output | Print spool output | NEW | No criterion |
| SDSF-BROWSE-4 Show columns | Display column information | NEW | No criterion |

---

## Area 16: REXX Scripting and SDSF REXX Interface
**EARS source:** tso-ears/rexx-and-sdsf-rexx.md REXX-1 through REXX-5, SDSF-JES-1 through SDSF-JES-4, SDSF-REXX-1 through SDSF-REXX-7
**Existing spec:** lua-macro-engine/requirements.md

### Summary
The lua-macro-engine spec covers Lua scripting comprehensively. REXX execution
is mapped to Lua as the scripting bridge (REXX-1.1). REXX external functions
(LISTDSI, OUTTRAP, SYSDSN, etc.) and EXECIO are NEW criteria. The SDSF REXX
interface (ISFCALLS, ISFEXEC, etc.) is P3 deferred.

### Criterion-level findings

| EARS ID | Description | Classification | Existing coverage |
|---------|-------------|----------------|-------------------|
| REXX-1.1 Execute REXX exec from PDS | Run exec stored in SYSEXEC/SYSPROC | NEW | No criterion -- Lua handles scripting but no REXX exec execution |
| REXX-1.2 EXEC command | Run exec explicitly | NEW | No criterion for REXX EXEC command |
| REXX-1.3 Implicit exec invocation | Type member name to run exec | NEW | No criterion |
| REXX-1.4 % prefix for exec | Reduce search time | NEW | No criterion |
| REXX-1.5 Pass arguments to exec | Arguments via EXEC command | NEW | No criterion |
| REXX-2.1 TSO host command environment | Route commands to workbench processor | NEW | No criterion for host command environments |
| REXX-2.2 ADDRESS environment-name | Change host command environment | NEW | No criterion |
| REXX-2.3 ISPEXEC environment | ISPF service calls | NEW | No criterion |
| REXX-2.4 ISREDIT environment | ISPF Edit macro calls | NEW | No criterion |
| REXX-2.5 RC special variable | Return code from host command | NEW | No criterion |
| REXX-3.1 LISTDSI function | Retrieve dataset information | NEW | No criterion |
| REXX-3.2 MSG function | Control message display | NEW | No criterion |
| REXX-3.3 MVSVAR function | Retrieve system variable values | NEW | No criterion |
| REXX-3.4 OUTTRAP function | Capture command output into stem | NEW | No criterion |
| REXX-3.5 PROMPT function | Control interactive prompting | NEW | No criterion |
| REXX-3.6 SYSDSN function | Test whether dataset exists | NEW | No criterion |
| REXX-3.7 SYSVAR function | Retrieve TSO/E session variables | NEW | No criterion |
| REXX-3.8 USERID function | Return current user ID | NEW | No criterion |
| REXX-4.1 EXECIO DISKR | Read from dataset | NEW | No criterion |
| REXX-4.2 EXECIO DISKW | Write to dataset | NEW | No criterion |
| REXX-4.3 EXECIO FINIS | Close dataset after operation | NEW | No criterion |
| REXX-4.4 EXECIO SKIP | Skip records | NEW | No criterion |
| REXX-4.5 EXECIO return codes | 0/2/non-zero return codes | NEW | No criterion |
| REXX-5.1 PUSH | Add to top of data stack | NEW | No criterion |
| REXX-5.2 QUEUE | Add to bottom of data stack | NEW | No criterion |
| REXX-5.3 PULL | Remove from top of data stack | NEW | No criterion |
| REXX-5.4 QUEUED | Return stack element count | NEW | No criterion |
| REXX-5.5 MAKEBUF | Create new buffer on stack | NEW | No criterion |
| REXX-5.6 DROPBUF | Remove buffer from stack | NEW | No criterion |
| REXX-5.7 NEWSTACK/DELSTACK | Private stack management | NEW | No criterion |
| SDSF-JES-1 through SDSF-JES-4 | MAS/JG/SRVC/SE panels | NEW (P3) | Deferred -- advanced JES panels |
| SDSF-REXX-1 through SDSF-REXX-7 | ISFCALLS/ISFEXEC/ISFACT/etc | NEW (P3) | Deferred -- SDSF REXX interface |

---

## Summary Table

| Area | EARS criteria count | COVERED | PARTIAL | NEW | OUT-OF-SCOPE |
|------|--------------------:|--------:|--------:|----:|-------------:|
| 1. ISPF Edit Session/Profile | 20 | 10 | 4 | 5 | 1 |
| 2. ISPF Line Commands | 21 | 13 | 1 | 7 | 0 |
| 3. ISPF Primary Commands | 29 | 18 | 2 | 9 | 0 |
| 4. ISPF Edit Recovery/Undo | 9 | 7 | 1 | 1 | 0 |
| 5. ISPF Syntax Highlighting | 10 | 5 | 2 | 3 | 0 |
| 6. ISPF Boundaries/Tabs/Masks | 13 | 13 | 0 | 0 | 0 |
| 7. ISPF Sequence Numbers | 10 | 10 | 0 | 0 | 0 |
| 8. ISPF Hex Display | 7 | 7 | 0 | 0 | 0 |
| 9. ISPF Edit Macros | 14 | 7 | 1 | 6 | 0 |
| 10. ISPF POM/Navigation | 21 | 10 | 3 | 8 | 0 |
| 11. TSO Session Startup | 18 | 9 | 2 | 7 | 0 |
| 12. TSO Dataset Commands P1 | 12 | 3 | 1 | 8 | 0 |
| 13. TSO Dataset Commands P2 | 5 | 0 | 0 | 5 | 0 |
| 14. SDSF Panel Framework/SET | 47 | 2 | 6 | 38 | 1 |
| 15. SDSF Queues/Filter/Log/Browse | 28 | 4 | 8 | 16 | 0 |
| 16. REXX/SDSF REXX | 34 | 0 | 0 | 7 | 0 (27 P3) |
| **Total** | **298** | **118** | **31** | **120** | **2 (+27 P3)** |

Note: REXX-5 (data stack) and SDSF-JES/SDSF-REXX criteria (27 total) are
classified as P3 DEFERRED and excluded from the NEW count above.
Actual NEW P1+P2 criteria: approximately 120.
Actual NEW P3 (deferred): approximately 27.

---

## Next Step: EI-2 Coverage Classification

EI-1 is complete. The findings above feed directly into EI-2 where each
criterion is formally classified as COVERED/PARTIAL/NEW/OUT-OF-SCOPE/DEFERRED
and assigned to a specific EI-5 batch.
