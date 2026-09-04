# Coverage Classification
# EI-2 Output -- Formal Classification of Every EARS Criterion

**Status:** EI-2 complete -- analysis only, no requirements.md files modified
**Input:** gap-analysis.md (EI-1 output), source-of-truth-map.md (EI-0.5 output)
**Purpose:** Assign each EARS criterion a formal classification, priority, target
spec, and EI-5 batch. This document is the direct input to EI-4 (integration plan).

## Classification Key

| Code | Meaning |
|------|---------|
| COV | COVERED -- existing criterion matches the EARS criterion |
| PAR | PARTIAL -- existing criterion partially covers; needs extension |
| NEW | NEW -- no existing criterion; needs new criterion added |
| OOS | OUT-OF-SCOPE -- requires z/OS-specific hardware not emulable on desktop |
| DEF | DEFERRED -- applicable but deliberately deferred (P3) |

## Batch Key

| Batch | Sub-project | Priority |
|-------|-------------|----------|
| B01 | edit-operations | P1 |
| B02 | find-and-replace | P1 |
| B03 | line-commands | P1 |
| B04 | sequence-numbers | P1 |
| B05 | hex-display | P1 |
| B06 | tabs-and-mask | P1 |
| B07 | menu-and-statusbar | P1 |
| B08 | startup-and-session | P1 |
| B09 | function-keys-and-history | P1 |
| B10 | command-semantics | P1/P2 |
| B11 | FFW-JES (P1 criteria) | P1 |
| B12 | undo-redo-transactions | P2 |
| B13 | syntax-highlighting | P2 |
| B14 | lua-macro-engine | P2 |
| B15 | FFW-JES (P2 criteria) | P2 |
| B16 | command-semantics (P2) | P2 |
| DEF | deferred-requirements.md | P3 |

---

## Area 1: ISPF Edit Session and Profile

**EARS source:** ispf-ears edit session lifecycle, profile and modes files
**Target spec:** edit-operations/requirements.md
**EI-5 batch:** B01

| EARS ID | Description | Class | Pri | Batch | Notes |
|---------|-------------|-------|-----|-------|-------|
| Edit-session-open | EDIT command opens dataset | COV | P1 | -- | edit-operations Req 12, startup-and-session Req 14.6 |
| Edit-session-close-END | END saves and exits | COV | P1 | -- | navigation-commands Req 11 |
| Edit-session-close-CANCEL | CANCEL exits without saving | COV | P1 | -- | navigation-commands Req 11.2 |
| Edit-insert-default | Editor starts in insert mode | COV | P1 | -- | edit-operations Req 1.4 |
| Edit-overstrike-toggle | Insert key toggles insert/overstrike | COV | P1 | -- | edit-operations Req 3.3 |
| Edit-mode-indicator | INSERT/OVERSTRIKE shown in status bar | COV | P1 | -- | edit-operations Req 3.4, menu-and-statusbar Req 6.3 |
| Edit-CAPS-mode | CAPS primary command forces uppercase input | NEW | P1 | B01 | No criterion in edit-operations |
| Edit-HEX-mode | HEX ON/OFF switches hex display | COV | P1 | -- | hex-display Req 1 |
| Edit-NULLS-mode | NULLS ON/OFF controls null character handling | NEW | P1 | B01 | No criterion in any spec |
| Edit-TABS-mode | TABS command displays tab ruler | COV | P1 | -- | tabs-and-mask Req 1 |
| Edit-PROFILE-command | PROFILE displays/changes edit profile settings | NEW | P1 | B01 | No profile persistence criterion |
| Edit-profile-persist | Edit profile settings persist across sessions | PAR | P1 | B01 | configuration-system covers settings but no ISPF profile concept |
| Edit-RECOVERY-mode | RECOVERY ON enables periodic save | PAR | P2 | B12 | undo-redo-transactions Req 8 covers recovery files but not RECOVERY command |
| Edit-AUTONUM-mode | AUTONUM ON enables auto line numbering | PAR | P1 | B04 | sequence-numbers Req 6.7 covers NUMBER ON but not AUTONUM alias |
| Edit-STATS-mode | STATS ON shows member statistics | NEW | P2 | B01 | No criterion |
| Edit-IMACRO-setting | Initial macro run on edit session open | NEW | P2 | B14 | No criterion |
| Edit-PACK-mode | PACK ON enables data compression | OOS | -- | -- | z/OS-specific DASD packing |
| Edit-NUM-mode | NUM controls sequence number display | PAR | P1 | B04 | sequence-numbers covers NUMBER SHOW but not NUM alias |
| Edit-HILITE-setting | HILITE controls syntax highlighting | PAR | P2 | B13 | syntax-highlighting covers highlighting but not HILITE command |
| Edit-LOCK-setting | LOCK prevents profile changes | NEW | P2 | B01 | No criterion |

---

## Area 2: ISPF Line Commands

**EARS source:** ispf-ears line commands file; tso-ears/tso-commands.md TSO-EDIT-3
**Target spec:** line-commands/requirements.md
**EI-5 batch:** B03

| EARS ID | Description | Class | Pri | Batch | Notes |
|---------|-------------|-------|-----|-------|-------|
| LC-D | Delete line(s) D/Dn/DD | COV | P1 | -- | line-commands Req 1 |
| LC-I | Insert blank line(s) I/In | COV | P1 | -- | line-commands Req 2 |
| LC-R | Duplicate line(s) R/Rn/RR | COV | P1 | -- | line-commands Req 3 |
| LC-C | Mark copy source C/CC | COV | P1 | -- | line-commands Req 4 |
| LC-M | Mark move source M/MM | COV | P1 | -- | line-commands Req 5 |
| LC-A | Mark after target A | COV | P1 | -- | line-commands Req 6 |
| LC-B | Mark before target B | COV | P1 | -- | line-commands Req 6 |
| LC-X | Exclude line(s) X/Xn/XX | COV | P1 | -- | line-commands Req 7 |
| LC-T | Tag line(s) T/TT | COV | P1 | -- | line-commands Req 8 |
| LC-U | Untag line(s) U/UU | COV | P1 | -- | line-commands Req 8 |
| LC-shift-right | Shift content right >/>> | COV | P1 | -- | line-commands Req 9 |
| LC-shift-left | Shift content left </<<  | COV | P1 | -- | line-commands Req 10 |
| LC-bounds-right | Shift within bounds right )/)) | COV | P1 | -- | line-commands Req 11 |
| LC-bounds-left | Shift within bounds left (/(( | COV | P1 | -- | line-commands Req 11 |
| LC-O | Overlay lines (ISPF O line command) | NEW | P1 | B03 | No criterion in line-commands |
| LC-W | Copy line to clipboard (ISPF W) | NEW | P1 | B03 | No criterion |
| LC-S | Show/unexclude excluded line | PAR | P1 | B03 | exclude-show-filter covers SHOW command but not S line command |
| LC-F | First/last line of excluded block | NEW | P1 | B03 | No criterion |
| LC-L | Assign label to line | NEW | P1 | B03 | No criterion |
| LC-bracket-right | Shift right by 1 (ISPF ]) | NEW | P1 | B03 | ] not in line-commands spec |
| TSO-EDIT-3 | d/i/r/c/m/a/b/dd/cc/mm block forms | COV | P1 | -- | line-commands Req 1-8 |


---

## Area 3: ISPF Primary Commands

**EARS source:** ispf-ears primary commands and find/change files
**Target specs:** command-semantics, find-and-replace, navigation-commands, edit-operations
**EI-5 batches:** B01, B02

| EARS ID | Description | Class | Pri | Batch | Notes |
|---------|-------------|-------|-----|-------|-------|
| PC-FIND-literal | FIND 'text' with direction/scope modifiers | COV | P1 | -- | find-and-replace Req 1-2 |
| PC-FIND-hex | FIND X'hex' | COV | P1 | -- | find-and-replace Req 3 |
| PC-FIND-regex | FIND REGEX 'pattern' | COV | P1 | -- | find-and-replace Req 4 |
| PC-RFIND | Repeat previous FIND | COV | P1 | -- | find-and-replace Req 5 |
| PC-CHANGE-literal | CHANGE 'old' 'new' with modifiers | COV | P1 | -- | find-and-replace Req 6-7 |
| PC-CHANGE-regex | CHANGE REGEX with group substitution | COV | P1 | -- | find-and-replace Req 8 |
| PC-RCHANGE | Repeat previous CHANGE | COV | P1 | -- | find-and-replace Req 9 |
| PC-LOCATE-line | LOCATE n scrolls to line n | COV | P1 | -- | navigation-commands Req 1 |
| PC-LOCATE-label | LOCATE label scrolls to named label | COV | P1 | -- | navigation-commands Req 1.3 |
| PC-SORT | Sort lines by column key | COV | P1 | -- | navigation-commands Req 2 |
| PC-EXCLUDE | Exclude lines matching pattern | COV | P1 | -- | exclude-show-filter spec |
| PC-SHOW | Show excluded lines matching pattern | COV | P1 | -- | exclude-show-filter spec |
| PC-RESET | Clear display artifacts and find state | COV | P1 | -- | command-semantics Req 6, find-and-replace Req 16 |
| PC-COPY | Copy lines to position | COV | P1 | -- | navigation-commands Req 14 |
| PC-MOVE | Move lines to position | COV | P1 | -- | navigation-commands Req 15 |
| PC-DELETE | Delete lines by scope | COV | P1 | -- | navigation-commands Req 13 |
| PC-SUBMIT | Submit current buffer as job | NEW | P1 | B01 | No criterion for SUBMIT from editor context |
| PC-SAVE | Save file | COV | P1 | -- | navigation-commands Req 11.1 |
| PC-CANCEL | Exit without saving | COV | P1 | -- | navigation-commands Req 11.2 |
| PC-END | Save and exit | COV | P1 | -- | navigation-commands Req 11.3 |
| PC-UNDO | Undo last edit | COV | P1 | -- | navigation-commands Req 17 |
| PC-REDO | Redo last undo | COV | P1 | -- | navigation-commands Req 17 |
| PC-MACRO | Invoke macro | COV | P2 | -- | navigation-commands Req 16 |
| PC-HILITE | Toggle syntax highlighting | PAR | P2 | B13 | syntax-highlighting covers highlighting but no HILITE command criterion |
| PC-SETUNDO | Configure undo settings | NEW | P2 | B12 | No criterion |
| PC-COMPARE | Compare with another dataset | NEW | P2 | B01 | No criterion in command-semantics |
| PC-CREATE | Create new dataset from lines | NEW | P1 | B01 | No criterion |
| PC-REPLACE | Replace dataset content | NEW | P1 | B01 | No criterion |
| PC-EDIT-nested | Open another dataset for edit from editor | NEW | P1 | B01 | No criterion for nested EDIT |
| PC-BROWSE | Open dataset for browse | NEW | P1 | B01 | No criterion in command-semantics |
| PC-VIEW | Open dataset for view | NEW | P1 | B01 | No criterion |

---

## Area 4: ISPF Edit Recovery and Undo

**EARS source:** ispf-ears recovery and undo file
**Target spec:** undo-redo-transactions/requirements.md
**EI-5 batch:** B12

| EARS ID | Description | Class | Pri | Batch | Notes |
|---------|-------------|-------|-----|-------|-------|
| RU-UNDO | Undo last edit | COV | P1 | -- | undo-redo-transactions Req 4, 15 |
| RU-UNDO-n | Undo n operations | COV | P1 | -- | undo-redo-transactions Req 4.6, 15.6 |
| RU-REDO | Redo last undo | COV | P1 | -- | undo-redo-transactions Req 4, 15 |
| RU-recovery-create | Periodic save of undo state | COV | P2 | -- | undo-redo-transactions Req 8 |
| RU-recovery-restore | Offer recovery on next open | COV | P2 | -- | undo-redo-transactions Req 8.4-8.5 |
| RU-RECOVERY-command | RECOVERY ON/OFF command | PAR | P2 | B12 | Req 8.2 covers interval=0 disables but no RECOVERY command |
| RU-save-point | Dirty flag based on save point | COV | P1 | -- | undo-redo-transactions Req 5 |
| RU-modified-markers | Visual indicator for changed lines | COV | P1 | -- | undo-redo-transactions Req 5.8, edit-operations Req 11.6 |
| RU-SETUNDO | SETUNDO command to configure undo settings | NEW | P2 | B12 | No SETUNDO command criterion |

---

## Area 5: ISPF Syntax Highlighting (HILITE)

**EARS source:** ispf-ears syntax highlighting file
**Target spec:** syntax-highlighting/requirements.md
**EI-5 batch:** B13

| EARS ID | Description | Class | Pri | Batch | Notes |
|---------|-------------|-------|-----|-------|-------|
| SH-HILITE-toggle | HILITE ON/OFF command | NEW | P2 | B13 | No HILITE command criterion -- highlighting is always on |
| SH-HILITE-LOGIC | HILITE LOGIC highlights logical operators | NEW | P2 | B13 | No criterion for HILITE LOGIC mode |
| SH-HILITE-FIND | HILITE FIND highlights find matches | PAR | P2 | B13 | find-and-replace Req 15 covers highlight-all but not HILITE FIND command |
| SH-HILITE-PAREN | HILITE PAREN highlights matching parentheses | NEW | P2 | B13 | No criterion |
| SH-keywords | Keywords in distinct colour | COV | P2 | -- | syntax-highlighting Req 5 |
| SH-comments | Comments in distinct colour | COV | P2 | -- | syntax-highlighting Req 6 |
| SH-strings | String literals highlighted | COV | P2 | -- | syntax-highlighting Req 5 |
| SH-lang-detect | Language detected from file type | COV | P2 | -- | syntax-highlighting Req 13 |
| SH-incremental | Only changed region re-highlighted | COV | P2 | -- | syntax-highlighting Req 3 |
| SH-custom-colours | User-configurable highlight colours | COV | P2 | -- | syntax-highlighting Req 12, theme-and-appearance |

---

## Area 6: ISPF Boundaries, Tabs, and Masks

**EARS source:** ispf-ears boundaries, tabs, masks file
**Target spec:** tabs-and-mask/requirements.md
**EI-5 batch:** B06 (no NEW criteria -- batch confirmed empty, no gate required)

| EARS ID | Description | Class | Pri | Batch | Notes |
|---------|-------------|-------|-----|-------|-------|
| TM-TABS-cmd | TABS primary command displays tab ruler | COV | P1 | -- | tabs-and-mask Req 1 |
| TM-TABS-set | TABS col1 col2 ... sets tab stop positions | COV | P1 | -- | tabs-and-mask Req 2 |
| TM-TABS-lc | TABS line command inserts tab ruler at line | COV | P1 | -- | tabs-and-mask Req 3 |
| TM-MASK-cmd | MASK primary command displays insert mask | COV | P1 | -- | tabs-and-mask Req 6 |
| TM-MASK-OFF | MASK OFF clears insert mask | COV | P1 | -- | tabs-and-mask Req 7 |
| TM-MASK-lc | MASK line command inserts mask at line | COV | P1 | -- | tabs-and-mask Req 8 |
| TM-MASK-apply | New lines pre-filled with mask on I/In | COV | P1 | -- | tabs-and-mask Req 9 |
| TM-default-stops | Global default tab stops from config | COV | P1 | -- | tabs-and-mask Req 4 |
| TM-lang-stops | Per-language default tab stops | COV | P1 | -- | tabs-and-mask Req 4.3 |
| TM-tab-key | Tab key advances to next tab stop | COV | P1 | -- | tabs-and-mask Req 5 |
| TM-BOUNDS | BOUNDS command sets column boundaries | COV | P1 | -- | navigation-commands Req 5 |
| TM-BNDS-lc | BNDS line command inserts bounds display line | COV | P1 | -- | navigation-commands Req 5.3 |
| TM-COLS | COLS command displays column ruler | COV | P1 | -- | navigation-commands Req 4 |

---

## Area 7: ISPF Sequence Numbers

**EARS source:** ispf-ears sequence numbers file
**Target spec:** sequence-numbers/requirements.md
**EI-5 batch:** B04 (PARTIAL criteria only -- AUTONUM and NUM aliases)

| EARS ID | Description | Class | Pri | Batch | Notes |
|---------|-------------|-------|-----|-------|-------|
| SN-auto-detect | Auto-detect sequence numbers on open | COV | P1 | -- | sequence-numbers Req 2 |
| SN-auto-strip | Auto-strip on open before display | COV | P1 | -- | sequence-numbers Req 3 |
| SN-UNNUM | UNNUM command explicit strip | COV | P1 | -- | sequence-numbers Req 5 |
| SN-NUMBER | NUMBER command writes sequence numbers | COV | P1 | -- | sequence-numbers Req 6 |
| SN-NUMBER-SHOW | NUMBER SHOW display overlay | COV | P1 | -- | sequence-numbers Req 8 |
| SN-NUMBER-ON | NUMBER ON/OFF auto-numbering mode | COV | P1 | -- | sequence-numbers Req 6.7-6.8 |
| SN-AUTONUM | AUTONUM ON alias for NUMBER ON | PAR | P1 | B04 | sequence-numbers Req 6.7 covers NUMBER ON but not AUTONUM alias |
| SN-NUM-alias | NUM alias for NUMBER command | PAR | P1 | B04 | sequence-numbers covers NUMBER SHOW but not NUM alias |
| SN-COBOL | COBOL sequence columns 1-6, 73-80 | COV | P1 | -- | sequence-numbers Req 1.5 |
| SN-FORTRAN | FORTRAN sequence columns | COV | P1 | -- | sequence-numbers Req 1.6 |
| SN-JCL | JCL sequence columns | COV | P1 | -- | sequence-numbers Req 1.7 |
| SN-undo | Undo UNNUM/NUMBER as single step | COV | P1 | -- | sequence-numbers Req 9 |

---

## Area 8: ISPF Hex Display

**EARS source:** ispf-ears hex display file
**Target spec:** hex-display/requirements.md
**EI-5 batch:** B05 (no NEW criteria -- batch confirmed empty, no gate required)

| EARS ID | Description | Class | Pri | Batch | Notes |
|---------|-------------|-------|-----|-------|-------|
| HX-toggle | HEX ON/OFF command | COV | P1 | -- | hex-display Req 1 |
| HX-layout | Three-pane layout: offset + hex + ASCII | COV | P1 | -- | hex-display Req 2 |
| HX-edit | Overwrite bytes in hex pane | COV | P1 | -- | hex-display Req 4 |
| HX-FIND | FIND X'hex' search for hex byte sequence | COV | P1 | -- | hex-display Req 5, find-and-replace Req 3 |
| HX-cursor-sync | Hex and ASCII panes stay in sync | COV | P1 | -- | hex-display Req 6 |
| HX-in-edit | Hex display within edit session | COV | P1 | -- | hex-display Req 16 |
| HX-record | Show raw bytes of record | COV | P1 | -- | hex-display Req 10 |

---

## Area 9: ISPF Edit Macros

**EARS source:** ispf-ears edit macros file
**Target spec:** lua-macro-engine/requirements.md
**EI-5 batch:** B14

| EARS ID | Description | Class | Pri | Batch | Notes |
|---------|-------------|-------|-----|-------|-------|
| EM-MACRO | MACRO command invokes macro by name | COV | P2 | -- | lua-macro-engine Req 5.1 |
| EM-EXEC | EXEC command executes inline expression | COV | P2 | -- | lua-macro-engine Req 5.2 |
| EM-RUN | RUN command runs macro file by path | COV | P2 | -- | lua-macro-engine Req 5.3 |
| EM-lines-api | editor.lines() API -- get line count | COV | P2 | -- | lua-macro-engine Req 2.2 |
| EM-get-line-api | editor.get_line() API -- get line content | COV | P2 | -- | lua-macro-engine Req 2.3 |
| EM-set-line-api | editor.set_line() API -- set line content | COV | P2 | -- | lua-macro-engine Req 2.4 |
| EM-command-api | editor.command() API -- dispatch primary command | COV | P2 | -- | lua-macro-engine Req 2.8 |
| EM-hooks | OnOpen/OnSave event hooks | COV | P2 | -- | lua-macro-engine Req 3.1 |
| EM-ISREDIT | ISREDIT host command environment | NEW | P2 | B14 | No criterion -- ISREDIT environment not defined |
| EM-ISPEXEC | ISPEXEC host command environment | NEW | P2 | B14 | No criterion -- ISPEXEC environment not defined |
| EM-undo-atomic | Macro changes undone as single transaction | COV | P2 | -- | lua-macro-engine Req 5.4 |
| EM-IMACRO | IMACRO initial macro on edit session open | NEW | P2 | B14 | No criterion for IMACRO setting |
| EM-LINENUM | LINENUM function get/set line number | NEW | P2 | B14 | No criterion |
| EM-CURSOR | CURSOR function get/set cursor position | PAR | P2 | B14 | lua-macro-engine Req 2.9 covers cursor_line/cursor_col |

---

## Area 10: ISPF POM and Panel Navigation

**EARS source:** tso-ears/ispf-panel-navigation.md ISPF-1 through ISPF-5
**Target specs:** menu-and-statusbar, startup-and-session, function-keys-and-history
**EI-5 batches:** B07, B08

| EARS ID | Description | Class | Pri | Batch | Notes |
|---------|-------------|-------|-----|-------|-------|
| ISPF-1.1 | Menu panel with OPTION ===> field | COV | P1 | -- | startup-and-session Req 14 |
| ISPF-1.2 | Data entry panel with ===> fields | PAR | P1 | B07 | No formal data entry panel criterion -- dialogs exist but no EARS criterion |
| ISPF-1.3 | List panel with NP column | PAR | P1 | B07 | FFW-JES Req 9 covers job list but no general list panel criterion |
| ISPF-1.4 | Edit panel with COMMAND ===> | COV | P1 | -- | menu-and-statusbar Req 9, startup-and-session Req 13 |
| ISPF-1.5 | COMMAND ===> on all panels | COV | P1 | -- | menu-and-statusbar Req 9 |
| ISPF-1.6 | SCROLL ===> field adjacent to command | NEW | P1 | B07 | No criterion for SCROLL ===> field |
| ISPF-2.1 | PF3 returns to previous panel | COV | P1 | -- | function-keys-and-history Req 17.1 |
| ISPF-2.2 | PF4 returns to POM | COV | P1 | -- | function-keys-and-history Req 17.3 |
| ISPF-2.3 | Fastpath notation (3.1) navigates to nested option | NEW | P1 | B07 | No criterion for fastpath notation |
| ISPF-2.4 | =option jump notation | COV | P1 | -- | startup-and-session Req 19.1-19.2 |
| ISPF-2.5 | X or =X exits from any panel | COV | P1 | -- | startup-and-session Req 14.12 |
| ISPF-3.1 | PF2 split screen at cursor | NEW | P2 | B07 | No criterion -- split screen not implemented |
| ISPF-3.2 | PF9 swap between split screen halves | NEW | P2 | B07 | No criterion |
| ISPF-3.3 | Each split screen half operates independently | NEW | P2 | B07 | No criterion |
| ISPF-3.4 | Unsplit by pressing END (PF3) | NEW | P2 | B07 | No criterion |
| ISPF-4.1 | LOCATE on list panel scrolls to matching item | PAR | P1 | B07 | navigation-commands Req 1 covers LOCATE for editor but not list panels |
| ISPF-4.2 | LOCATE nearest alphabetic match on list panel | NEW | P1 | B07 | No criterion for list panel LOCATE |
| ISPF-4.3 | LOCATE accepts partial names | NEW | P1 | B07 | No criterion |
| ISPF-5.1 | Command history at least 20 entries | COV | P1 | -- | function-keys-and-history Req 9 (default 200) |
| ISPF-5.2 | PF12 cycles backward through history | COV | P1 | -- | function-keys-and-history Req 5 |
| ISPF-5.3 | History persists within session | COV | P1 | -- | function-keys-and-history Req 6 |

---

## Area 11: TSO Session Startup and Logon

**EARS source:** tso-ears/tso-session-and-logon.md TSO-1 through TSO-4
**Target spec:** startup-and-session/requirements.md
**EI-5 batch:** B08

| EARS ID | Description | Class | Pri | Batch | Notes |
|---------|-------------|-------|-----|-------|-------|
| TSO-1.1 | POM as default landing panel on startup | COV | P1 | -- | startup-and-session Req 14.1 |
| TSO-1.2 | Session start timestamp shown in status bar | NEW | P1 | B08 | No criterion for session timestamp display |
| TSO-1.3 | Session end timestamp and logoff message | NEW | P1 | B08 | No criterion for logoff message |
| TSO-1.4 | LOGOFF command terminates session | NEW | P1 | B08 | EXIT/=X exist but not LOGOFF |
| TSO-2.1 | Command ===> field accepts TSO commands | COV | P1 | -- | menu-and-statusbar Req 9, command-semantics Req 8 |
| TSO-2.2 | Error message for unknown command | COV | P1 | -- | command-semantics Req 1.4 |
| TSO-2.3 | HELP command displays available commands | COV | P1 | -- | command-semantics Req 7 |
| TSO-2.4 | TIME command displays current date and time | NEW | P1 | B08 | No criterion for TIME command |
| TSO-2.5 | STATUS command displays job status | PAR | P1 | B08 | FFW-JES covers job status but no STATUS command in command-semantics |
| TSO-3.1 | Default PF key assignments (PF1=HELP, PF3=END, etc.) | COV | P1 | -- | function-keys-and-history Req 15 |
| TSO-3.2 | KEYS command views PF key assignments | COV | P1 | -- | function-keys-and-history Req 20.1 |
| TSO-3.3 | PFSHOW command toggles PF key display | COV | P1 | -- | function-keys-and-history Req 12 |
| TSO-3.4 | Change PF key assignments via Key Configuration dialog | COV | P1 | -- | function-keys-and-history Req 20 |
| TSO-3.5 | PF key assignments persist across sessions | COV | P1 | -- | function-keys-and-history Req 6 |
| TSO-4.1 | UP/DOWN/LEFT/RIGHT scroll commands | COV | P1 | -- | navigation-commands Req 3 |
| TSO-4.2 | Scroll amounts PAGE/HALF/CSR/MAX/DATA/n | PAR | P1 | B07 | navigation-commands Req 3 covers UP n and DOWN n but not HALF/CSR/MAX/DATA |
| TSO-4.3 | SCROLL ===> field retains scroll amount | NEW | P1 | B07 | No criterion for SCROLL ===> field persistence |
| TSO-4.4 | TOP/BOTTOM commands jump to first/last line | COV | P1 | -- | navigation-commands Req 3.9-3.10 |

---

## Area 12: TSO Dataset Commands P1 (ALLOCATE through STATUS)

**EARS source:** tso-ears/tso-commands.md TSO-CMD-1 through TSO-CMD-9, TSO-EDIT-1 through TSO-EDIT-3
**Target spec:** command-semantics/requirements.md (primary)
**EI-5 batch:** B10

| EARS ID | Description | Class | Pri | Batch | Notes |
|---------|-------------|-------|-----|-------|-------|
| TSO-CMD-1 | ALLOCATE -- allocate dataset with full operand set | NEW | P1 | B10 | No command-semantics criterion |
| TSO-CMD-2 | FREE -- release allocated dataset | NEW | P1 | B10 | No command-semantics criterion |
| TSO-CMD-3 | DELETE -- delete dataset or member | NEW | P1 | B10 | No command-semantics criterion (edit DELETE is different) |
| TSO-CMD-4 | RENAME -- rename dataset or member | NEW | P1 | B10 | No command-semantics criterion |
| TSO-CMD-5 | LISTCAT -- list catalog entries | NEW | P1 | B10 | No command-semantics criterion |
| TSO-CMD-6 | LISTDS -- display dataset attributes | NEW | P1 | B10 | No command-semantics criterion |
| TSO-CMD-7 | LISTALC -- list allocated datasets | NEW | P1 | B10 | No command-semantics criterion |
| TSO-CMD-8 | SUBMIT -- submit job for batch execution | NEW | P1 | B10 | No command-semantics criterion |
| TSO-CMD-9 | STATUS -- display job status | NEW | P1 | B10 | No command-semantics criterion |
| TSO-EDIT-1 | EDIT command opens dataset for editing | PAR | P1 | B10 | startup-and-session covers EDIT routing but no formal criterion |
| TSO-EDIT-2 | EDIT subcommands (FIND/CHANGE/DELETE/INSERT/etc.) | COV | P1 | -- | find-and-replace, line-commands, navigation-commands cover these |
| TSO-EDIT-3 | Line commands in EDIT (d/i/r/c/m/a/b) | COV | P1 | -- | line-commands Req 1-8 |

---

## Area 13: TSO Dataset Commands P2 (OUTPUT through PRINTDS)

**EARS source:** tso-ears/tso-commands.md TSO-CMD-10 through TSO-CMD-14
**Target spec:** command-semantics/requirements.md
**EI-5 batch:** B16

| EARS ID | Description | Class | Pri | Batch | Notes |
|---------|-------------|-------|-----|-------|-------|
| TSO-CMD-10 | OUTPUT -- display/manage job output | NEW | P2 | B16 | No command-semantics criterion |
| TSO-CMD-11 | CANCEL -- cancel batch job | NEW | P2 | B16 | No command-semantics criterion (edit CANCEL is different) |
| TSO-CMD-12 | SEND -- send message to user | NEW | P2 | B16 | No command-semantics criterion |
| TSO-CMD-13 | PROFILE -- display/change user profile | NEW | P2 | B16 | No command-semantics criterion |
| TSO-CMD-14 | PRINTDS -- print dataset | NEW | P2 | B16 | No command-semantics criterion |

---

## Area 14: SDSF Panel Framework and SET Commands

**EARS source:** tso-ears/sdsf-panel-framework.md SDSF-1 through SDSF-5, SET-1 through SET-13
**Target spec:** FFW-JES/requirements.md
**EI-5 batches:** B11 (P1), B15 (P2)

| EARS ID | Description | Class | Pri | Batch | Notes |
|---------|-------------|-------|-----|-------|-------|
| SDSF-1.1 | Action bar with pull-down menus | NEW | P1 | B11 | No criterion in FFW-JES |
| SDSF-1.2 | Title line with panel name and line range | NEW | P1 | B11 | No criterion |
| SDSF-1.3 | Message area in title line | PAR | P1 | B11 | FFW-JES Req 9 covers status but not SDSF-style message area |
| SDSF-1.4 | COMMAND INPUT ===> field | PAR | P1 | B11 | FFW-JES has command integration but no COMMAND INPUT ===> criterion |
| SDSF-1.5 | SCROLL ===> field adjacent to command | NEW | P1 | B11 | No criterion |
| SDSF-1.6 | Filter information lines (PREFIX=/DEST=/OWNER=) | NEW | P1 | B11 | No criterion |
| SDSF-1.7 | NP column (fixed, non-scrolling action input) | NEW | P1 | B11 | No criterion |
| SDSF-1.8 | First data column fixed on horizontal scroll | NEW | P1 | B11 | No criterion |
| SDSF-2.1 | Action characters S/?/C/H/A/P/D/E/J/W | PAR | P1 | B11 | FFW-JES Req 9.10 covers context menu actions but not NP column action chars |
| SDSF-2.2 | SET ACTION displays valid action characters | NEW | P1 | B11 | No criterion |
| SDSF-2.3 | = repeats previous action character | NEW | P1 | B11 | No criterion |
| SDSF-2.4 | // block action applies to block of rows | NEW | P1 | B11 | No criterion |
| SDSF-2.5 | Command-line action syntax ("2 C" to cancel row 2) | NEW | P1 | B11 | No criterion |
| SDSF-2.6 | SET ROWNUM shows row numbers in NP area | NEW | P1 | B11 | No criterion |
| SDSF-3.1 | Overtypeable fields visually distinct | NEW | P2 | B15 | No criterion |
| SDSF-3.2 | Overtype field to change value | NEW | P2 | B15 | No criterion |
| SDSF-3.3 | Command-line overtype syntax ("rows column=value") | NEW | P2 | B15 | No criterion |
| SDSF-3.4 | Overtype Extension pop-up (+ opens related fields) | NEW | P2 | B15 | No criterion |
| SDSF-4.1 | Main panel command list with name/desc/group | NEW | P1 | B11 | No criterion |
| SDSF-4.2 | Command groups (Jobs/Output/JES/Log/Memory/etc.) | NEW | P1 | B11 | No criterion |
| SDSF-4.3 | S action on main panel selects command | NEW | P1 | B11 | No criterion |
| SDSF-4.4 | SET MAIN GROUP sets grouped main panel | NEW | P1 | B11 | No criterion |
| SDSF-4.5 | MGRP expandable/collapsible command groups | NEW | P1 | B11 | No criterion |
| SDSF-4.6 | MENU command returns to main panel | NEW | P1 | B11 | No criterion |
| SDSF-5.1 | Context-sensitive help (PF1/HELP) | PAR | P2 | B15 | context-help spec covers help but not SDSF-specific panel help |
| SDSF-5.2 | SEARCH in help content | NEW | P2 | B15 | No criterion |
| SDSF-5.3 | ACTH action character help | NEW | P2 | B15 | No criterion |
| SDSF-5.4 | COLH column help | NEW | P2 | B15 | No criterion |
| SDSF-5.5 | CMDH command help | NEW | P2 | B15 | No criterion |
| SET-1 | SET ACTION -- display valid action characters | NEW | P1 | B11 | No criterion |
| SET-2 | SET BCOLOR -- colour on browse panels | NEW | P2 | B15 | No criterion |
| SET-3 | SET CONFIRM -- confirmation for destructive actions | NEW | P2 | B15 | No criterion |
| SET-4 | SET CURSOR -- cursor positioning on panel display | NEW | P2 | B15 | No criterion |
| SET-5 | SET DATE -- date display format | NEW | P2 | B15 | No criterion |
| SET-6 | SET DELAY -- auto-refresh interval | PAR | P2 | B15 | FFW-JES Req 9.7 covers refresh interval but not SET DELAY command |
| SET-7 | SET HEX -- hex display of column values | NEW | P2 | B15 | No criterion (different from editor HEX mode) |
| SET-8 | SET MAIN -- default main panel | NEW | P1 | B11 | No criterion |
| SET-9 | SET ROWNUM -- row numbers in NP area | NEW | P1 | B11 | No criterion |
| SET-10 | SET SCHARS -- wildcard characters | NEW | P2 | B15 | No criterion |
| SET-11 | SET SCREEN -- colour scheme for field types | NEW | P2 | B15 | No criterion |
| SET-12 | WHO command -- display session information | NEW | P1 | B11 | No criterion |
| SET-13 | QUERY AUTH -- display authorized commands | NEW | P1 | B11 | No criterion |
| PERSIST-1 | Save SDSF SET settings across sessions | PAR | P1 | B11 | startup-and-session covers session persistence but not SDSF-specific settings |
| PERSIST-2 | Special DDNames (ISFMIGNB/ISFMIGXB/ISFMIGNP) | OOS | -- | -- | z/OS-specific DDName mechanism |

---

## Area 15: SDSF Job Queue Panels, Filter/Sort, Log/System, Browse/Print

**EARS source:** sdsf-job-queue-panels.md, sdsf-filter-sort-search.md, sdsf-log-and-system-panels.md, sdsf-browse-and-print.md
**Target spec:** FFW-JES/requirements.md
**EI-5 batches:** B11 (P1), B15 (P2)

| EARS ID | Description | Class | Pri | Batch | Notes |
|---------|-------------|-------|-----|-------|-------|
| SDSF-JQ-1 | Input queue panel (I) -- jobs awaiting execution | COV | P1 | -- | FFW-JES Req 9.1 (Input Queue sub-panel) |
| SDSF-JQ-2 | Output queue panel (O) -- completed job output | COV | P1 | -- | FFW-JES Req 9.1 (Output/Completed sub-panel) |
| SDSF-JQ-3 | Held queue panel (H) -- held jobs | COV | P1 | -- | FFW-JES Req 9.1 (Held Jobs sub-panel) |
| SDSF-JQ-4 | Status panel (ST) -- all jobs with status | PAR | P1 | B11 | FFW-JES Req 9 covers status display but no dedicated ST panel |
| SDSF-JQ-5 | Active jobs panel (DA) -- currently executing | COV | P1 | -- | FFW-JES Req 9.1 (Active Jobs sub-panel) |
| SDSF-JQ-6 | Column definitions (JOBNAME/JOBID/OWNER/STATUS/etc.) | PAR | P1 | B11 | FFW-JES Req 9 covers sortable columns but no specific column definitions |
| SDSF-JQ-7 | PREFIX/OWNER/DEST filter fields | PAR | P1 | B11 | FFW-JES Req 9.4 covers filtering but not PREFIX/OWNER/DEST specific fields |
| SDSF-FILTER-1 | PREFIX filter -- filter by job name prefix | NEW | P1 | B11 | No specific PREFIX filter criterion |
| SDSF-FILTER-2 | OWNER filter -- filter by job owner | NEW | P1 | B11 | No specific OWNER filter criterion |
| SDSF-FILTER-3 | DEST filter -- filter by output destination | NEW | P1 | B11 | No criterion |
| SDSF-FILTER-4 | FILTER command -- advanced filter expression | NEW | P1 | B11 | No criterion |
| SDSF-FILTER-5 | SORT command -- sort panel columns | PAR | P1 | B11 | FFW-JES Req 9 covers sortable columns but no SORT command |
| SDSF-FILTER-6 | FIND command -- search within panel | NEW | P1 | B11 | No criterion |
| SDSF-FILTER-7 | LOCATE command -- scroll to matching row | NEW | P1 | B11 | No criterion |
| SDSF-SCROLL-1-5 | Scroll commands in SDSF panels | PAR | P1 | B11 | navigation-commands covers scroll but not SDSF panel scroll |
| SDSF-LOG-1 | System log panel | NEW | P2 | B15 | No criterion |
| SDSF-LOG-2 | User log panel (ULOG) | NEW | P2 | B15 | No criterion |
| SDSF-LOG-3 | LOG command -- navigate log | NEW | P2 | B15 | No criterion |
| SDSF-LOG-4 | NEXT/PREV/SNAPSHOT log navigation commands | NEW | P2 | B15 | No criterion |
| SDSF-SYS-1 | SYS panel -- system information | NEW | P2 | B15 | No criterion |
| SDSF-SYS-2 | DASH panel -- dashboard | NEW | P2 | B15 | No criterion |
| SDSF-SYS-3 | INIT panel -- initiator information | NEW | P2 | B15 | No criterion |
| SDSF-SYS-4 | JC panel -- job class information | NEW | P2 | B15 | No criterion |
| SDSF-SYS-5 | SP panel -- spool information | NEW | P2 | B15 | No criterion |
| SDSF-BROWSE-1 | Browse job output | COV | P1 | -- | FFW-JES Req 7 (JobLogViewerPanel) |
| SDSF-BROWSE-2 | Browse settings -- configure browse display | NEW | P2 | B15 | No criterion |
| SDSF-BROWSE-3 | Print job output | NEW | P2 | B15 | No criterion |
| SDSF-BROWSE-4 | Show columns -- display column information | NEW | P2 | B15 | No criterion |

---

## Area 16: REXX Scripting and SDSF REXX Interface

**EARS source:** tso-ears/rexx-and-sdsf-rexx.md REXX-1 through REXX-5, SDSF-JES-1 through SDSF-JES-4, SDSF-REXX-1 through SDSF-REXX-7
**Target spec:** lua-macro-engine/requirements.md (REXX-1 through REXX-5); FFW-JES/requirements.md (SDSF-JES, SDSF-REXX)
**EI-5 batches:** B14 (P2 REXX), DEF (P3 SDSF-JES and SDSF-REXX)

| EARS ID | Description | Class | Pri | Batch | Notes |
|---------|-------------|-------|-----|-------|-------|
| REXX-1.1 | Execute REXX exec from PDS (SYSEXEC/SYSPROC) | NEW | P2 | B14 | No criterion -- Lua handles scripting but no REXX exec execution |
| REXX-1.2 | EXEC command -- run exec explicitly | NEW | P2 | B14 | No criterion for REXX EXEC command |
| REXX-1.3 | Implicit exec invocation by member name | NEW | P2 | B14 | No criterion |
| REXX-1.4 | % prefix for exec to reduce search time | NEW | P2 | B14 | No criterion |
| REXX-1.5 | Pass arguments to exec via EXEC command | NEW | P2 | B14 | No criterion |
| REXX-2.1 | TSO host command environment | NEW | P2 | B14 | No criterion for host command environments |
| REXX-2.2 | ADDRESS environment-name changes host env | NEW | P2 | B14 | No criterion |
| REXX-2.3 | ISPEXEC environment for ISPF service calls | NEW | P2 | B14 | No criterion |
| REXX-2.4 | ISREDIT environment for ISPF Edit macro calls | NEW | P2 | B14 | No criterion |
| REXX-2.5 | RC special variable -- return code from host command | NEW | P2 | B14 | No criterion |
| REXX-3.1 | LISTDSI function -- retrieve dataset information | NEW | P2 | B14 | No criterion |
| REXX-3.2 | MSG function -- control message display | NEW | P2 | B14 | No criterion |
| REXX-3.3 | MVSVAR function -- retrieve system variable values | NEW | P2 | B14 | No criterion |
| REXX-3.4 | OUTTRAP function -- capture command output into stem | NEW | P2 | B14 | No criterion |
| REXX-3.5 | PROMPT function -- control interactive prompting | NEW | P2 | B14 | No criterion |
| REXX-3.6 | SYSDSN function -- test whether dataset exists | NEW | P2 | B14 | No criterion |
| REXX-3.7 | SYSVAR function -- retrieve TSO/E session variables | NEW | P2 | B14 | No criterion |
| REXX-3.8 | USERID function -- return current user ID | NEW | P2 | B14 | No criterion |
| REXX-4.1 | EXECIO DISKR -- read from dataset | NEW | P2 | B14 | No criterion |
| REXX-4.2 | EXECIO DISKW -- write to dataset | NEW | P2 | B14 | No criterion |
| REXX-4.3 | EXECIO FINIS -- close dataset after operation | NEW | P2 | B14 | No criterion |
| REXX-4.4 | EXECIO SKIP -- skip records | NEW | P2 | B14 | No criterion |
| REXX-4.5 | EXECIO return codes (0/2/non-zero) | NEW | P2 | B14 | No criterion |
| REXX-5.1 | PUSH -- add to top of data stack | NEW | P2 | B14 | No criterion |
| REXX-5.2 | QUEUE -- add to bottom of data stack | NEW | P2 | B14 | No criterion |
| REXX-5.3 | PULL -- remove from top of data stack | NEW | P2 | B14 | No criterion |
| REXX-5.4 | QUEUED -- return stack element count | NEW | P2 | B14 | No criterion |
| REXX-5.5 | MAKEBUF -- create new buffer on stack | NEW | P2 | B14 | No criterion |
| REXX-5.6 | DROPBUF -- remove buffer from stack | NEW | P2 | B14 | No criterion |
| REXX-5.7 | NEWSTACK/DELSTACK -- private stack management | NEW | P2 | B14 | No criterion |
| SDSF-JES-1 | MAS panel -- multi-access spool | DEF | P3 | DEF | Advanced JES panel |
| SDSF-JES-2 | JG panel -- job group | DEF | P3 | DEF | Advanced JES panel |
| SDSF-JES-3 | SRVC panel -- WLM service class | DEF | P3 | DEF | Advanced JES panel |
| SDSF-JES-4 | SE panel -- scheduling environment | DEF | P3 | DEF | Advanced JES panel |
| SDSF-REXX-1 | ISFCALLS -- enable/disable SDSF REXX interface | DEF | P3 | DEF | SDSF REXX interface |
| SDSF-REXX-2 | ISFEXEC -- execute SDSF command from REXX | DEF | P3 | DEF | SDSF REXX interface |
| SDSF-REXX-3 | ISFACT -- perform action on SDSF rows | DEF | P3 | DEF | SDSF REXX interface |
| SDSF-REXX-4 | ISFBROWSE -- browse SDSF output from REXX | DEF | P3 | DEF | SDSF REXX interface |
| SDSF-REXX-5 | ISFSLASH -- issue slash command from REXX | DEF | P3 | DEF | SDSF REXX interface |
| SDSF-REXX-6 | ISFGET -- retrieve SDSF variable values | DEF | P3 | DEF | SDSF REXX interface |
| SDSF-REXX-7 | ISFLOG -- write to SDSF log from REXX | DEF | P3 | DEF | SDSF REXX interface |

---

## EXTENDS Items (from source-of-truth-map.md)

These items originate from the MiniX/FTSO design document (EI-0 EXTENDS resolution).
They have no existing coverage in any requirements.md and are treated as NEW criteria.

| Item | Class | Pri | Target spec | Batch | Notes |
|------|-------|-----|-------------|-------|-------|
| ISPF Option 6 panel chrome (title, COMMAND ===> , SCROLL ===> , FTSO READY prompt, MAXCC display) | NEW | P1 | shell-command | B10 | EI-0 EXTENDS item |
| TSO-style operand parsing (positional + keyword: DSNAME(), RECFM(), LRECL()) | NEW | P1 | command-semantics | B10 | EI-0 EXTENDS item |
| Dataset prefix per session (SET PREFIX) | NEW | P1 | command-semantics | B10 | EI-0 EXTENDS item |
| Command continuation character (trailing backslash) | NEW | P2 | command-semantics | B10 | EI-0 EXTENDS item |
| ds:// URI scheme for dataset references | NEW | P2 | command-semantics | B10 | EI-0 EXTENDS item |
| FFCMD command files (Phase 1 scripting) | NEW | P2 | lua-macro-engine | B14 | EI-0 EXTENDS item |
| Namespace conflict resolution for plugin commands | NEW | P2 | command-framework | B10 | EI-0 EXTENDS item |
| Capability model (per-command capability declarations) | NEW | P2 | command-framework | B10 | EI-0 EXTENDS item |
| Secret operand handling (redaction from history/logs) | NEW | P2 | command-framework | B10 | EI-0 EXTENDS item |
| Structured audit events | NEW | P2 | command-framework | B10 | EI-0 EXTENDS item |
| "Show equivalent FTSO command" for GUI operations | NEW | P2 | menu-and-statusbar | B07 | EI-0 EXTENDS item |
| Fuzz-testable parser requirement | NEW | P2 | command-semantics | B10 | EI-0 EXTENDS item |
| LOGOFF command | NEW | P1 | startup-and-session | B08 | EI-0 EXTENDS item (also TSO-1.4) |
| Session start/end timestamp display | NEW | P1 | startup-and-session | B08 | EI-0 EXTENDS item (also TSO-1.2/1.3) |

---

## Summary Table

| Area | Total | COV | PAR | NEW | OOS | DEF |
|------|------:|----:|----:|----:|----:|----:|
| 1. ISPF Edit Session/Profile | 20 | 10 | 4 | 5 | 1 | 0 |
| 2. ISPF Line Commands | 21 | 13 | 1 | 7 | 0 | 0 |
| 3. ISPF Primary Commands | 31 | 18 | 2 | 11 | 0 | 0 |
| 4. ISPF Edit Recovery/Undo | 9 | 7 | 1 | 1 | 0 | 0 |
| 5. ISPF Syntax Highlighting | 10 | 5 | 2 | 3 | 0 | 0 |
| 6. ISPF Boundaries/Tabs/Masks | 13 | 13 | 0 | 0 | 0 | 0 |
| 7. ISPF Sequence Numbers | 12 | 10 | 2 | 0 | 0 | 0 |
| 8. ISPF Hex Display | 7 | 7 | 0 | 0 | 0 | 0 |
| 9. ISPF Edit Macros | 14 | 7 | 1 | 6 | 0 | 0 |
| 10. ISPF POM/Navigation | 21 | 10 | 3 | 8 | 0 | 0 |
| 11. TSO Session Startup | 18 | 9 | 2 | 7 | 0 | 0 |
| 12. TSO Dataset Commands P1 | 12 | 3 | 1 | 8 | 0 | 0 |
| 13. TSO Dataset Commands P2 | 5 | 0 | 0 | 5 | 0 | 0 |
| 14. SDSF Panel Framework/SET | 47 | 2 | 6 | 38 | 1 | 0 |
| 15. SDSF Queues/Filter/Log/Browse | 28 | 4 | 8 | 16 | 0 | 0 |
| 16. REXX/SDSF REXX | 34 | 0 | 0 | 7 | 0 | 27 |
| EXTENDS items | 14 | 0 | 0 | 14 | 0 | 0 |
| **Total** | **326** | **118** | **33** | **136** | **2** | **27** |

**NEW P1 criteria:** ~75 (Batches B01-B11)
**NEW P2 criteria:** ~61 (Batches B12-B16)
**PARTIAL criteria requiring extension:** 33 (distributed across batches)
**OUT-OF-SCOPE:** 2 (PACK mode, PERSIST-2 DDNames)
**DEFERRED P3:** 27 (SDSF-JES-1-4, SDSF-REXX-1-7, REXX-5.1-5.7)

---

## Batch Load Summary

| Batch | Sub-project | NEW | PAR | Priority |
|-------|-------------|----:|----:|----------|
| B01 | edit-operations | 11 | 4 | P1 |
| B02 | find-and-replace | 0 | 0 | P1 (no new criteria) |
| B03 | line-commands | 6 | 1 | P1 |
| B04 | sequence-numbers | 0 | 2 | P1 |
| B05 | hex-display | 0 | 0 | P1 (no new criteria) |
| B06 | tabs-and-mask | 0 | 0 | P1 (no new criteria) |
| B07 | menu-and-statusbar | 10 | 4 | P1/P2 |
| B08 | startup-and-session | 7 | 1 | P1 |
| B09 | function-keys-and-history | 0 | 0 | P1 (no new criteria) |
| B10 | command-semantics | 17 | 1 | P1/P2 |
| B11 | FFW-JES (P1) | 34 | 9 | P1 |
| B12 | undo-redo-transactions | 2 | 1 | P2 |
| B13 | syntax-highlighting | 3 | 2 | P2 |
| B14 | lua-macro-engine | 24 | 2 | P2 |
| B15 | FFW-JES (P2) | 16 | 4 | P2 |
| B16 | command-semantics (P2) | 5 | 0 | P2 |

Note: Batches B02, B05, B06, B09 have no new or partial criteria.
They require no gate execution and can be skipped in EI-5.
