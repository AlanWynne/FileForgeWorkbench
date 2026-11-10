# Menu-snapshot layout/fidelity bugs (owner-reported 2026-11-01)

The menu (POM) ScreenModel does not match the real ISPF screen. Owner gave a
current-vs-expected mock. Three bugs, all in the menu snapshot model/layout
(crates/ff-desktop/src/screen_snapshot.rs + the ff-screen-model renderer).

## Bug 1 -- Command line not rendered
The `Command ===>` input line is missing from the snapshot. The model DOES carry
`command_line` (menu_workspace_screen_model sets model.command_line), but the
render does not emit a `Command ===> ____...` row. Expected: the first body row
(under the title border) is `Command ===> ` followed by the input area.

## Bug 2 -- Superfluous header block (the Phase 2A "chrome" is wrong for menus)
menu_workspace_screen_model_full adds header_lines = window_title + menu_bar
joined + tab_header ("[POM]") + title_line. In the capture this prints as a junk
block at the top:
    FileForge Workbench
    FFWB Settings...   Help...   SCRM...     <- the menu BAR echoed as text
    [POM]
    FileForge Workbench -- Primary Option Menu   <- title duplicated
Expected output has NONE of this -- just the title in the frame border, then the
command line, then the options. REMOVE the header-chrome block for the menu
snapshot (the window title / menu-bar echo / tab tag / duplicated title line).
This supersedes Phase 2A Req 3.9 "header chrome" for menus -- it was the wrong
model; the owner does not want the window chrome / menu bar / tab tag echoed into
the menu capture.

## Bug 3 -- Layout should be TWO-COLUMN (options left, calendar right), not stacked
Current: options listed, then calendar stacked BELOW as footer lines, then PF
keys. Expected: ISPF two-column body -- options on the LEFT, the calendar on the
RIGHT beside them, and under the calendar a small block:
    Time              : 12:58
    Day of Year       :   305
then blank rows, then the PF-key footer line at the bottom (inside the border).
The calendar columns must align. Also the option ROW FORMAT differs:
  expected: `0 Settings : FFWB Settings and Client Parameters (Settings)`
  current:  `0 . . . : FFWB Settings and Client Parameters (Settings)`
i.e. key + short name + " : " + description (command), not key + " . . . : ".

## Expected layout (owner mock, authoritative)
Title in the top border. Then inside the frame:
  Command ===> ____...
  0 Settings : <desc> (<cmd>)                         November 2026
  3 Help     : <desc> (<cmd>)                   Su  Mo  Tu  We  Th  Fr  Sa
  S SCRM     : <desc> (<cmd>)                  * 1   2   3 ...
  X X        : <desc> (<cmd>)                    8   9  10 ...
                                                15 ...
                                                22 ...
                                                29  30
                                                Time          : 12:58
                                                Day of Year   :   305
  (blank rows)
  Base | F1 Help | F2 Detach | ... | F12 Retrieve

## Scope / sequencing note
These are the menu ScreenModel layout -- independent of the Word-monospace issue
(which is PARKED as a known limitation, see below). Fix these on scrm-2e; they
matter more than the Word font and are about faithful capture content.

The Config-panel bugs in .agents/tasks/config-snapshot-bugs.md are a related but
separate Context provider (Config), still pending after this.
