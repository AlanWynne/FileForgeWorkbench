# ISPF-Style Panel Navigation -- EARS Requirements

Source documents: ikjb300 (TSO/E Primer), ikja100 (SDSF User Guide).

Priority: P1 (ISPF-1, ISPF-2, ISPF-4, ISPF-5) / P2 (ISPF-3).
Sub-project mapping: menu-and-statusbar (primary), navigation-commands, function-keys-and-history (secondary).

---

## Requirement ISPF-1: Panel Types

THE workbench SHALL support four panel types analogous to ISPF/PDF:
data entry panels, menu panels, list panels, and edit panels.

Criteria:
- 1.1 A menu panel SHALL display a list of numbered or lettered options and accept an OPTION ===> input field.
- 1.2 A data entry panel SHALL display labelled input fields with ===> arrows and accept typed values.
- 1.3 A list panel SHALL display rows of items with an action field (NP column) to the left of each row.
- 1.4 An edit panel SHALL display file content with line numbers and a COMMAND ===> field.
- 1.5 ALL panels SHALL display a COMMAND ===> field at the bottom (or top) of the screen.
- 1.6 ALL panels SHALL display a SCROLL ===> field adjacent to the COMMAND field.

---

## Requirement ISPF-2: Panel Hierarchy and Navigation

WHEN the user navigates between panels,
THE workbench SHALL maintain a panel hierarchy analogous to ISPF.

Criteria:
- 2.1 THE user SHALL be able to return to the previous panel by pressing PF3 (END command).
- 2.2 THE user SHALL be able to return to the Primary Option Menu from any panel by pressing PF4 (RETURN command) or entering =0 through =9.
- 2.3 THE user SHALL be able to navigate directly to a nested option using fastpath notation (e.g., 3.1 on the OPTION line).
- 2.4 THE user SHALL be able to jump from one option to another using =option notation (e.g., =2 from within option 3).
- 2.5 THE user SHALL be able to exit the workbench from any menu panel by entering X or =X.

---

## Requirement ISPF-3: Split Screen

WHEN the user presses PF2 (SPLIT),
THE workbench SHALL divide the display into two independent panels.

Criteria:
- 3.1 THE user SHALL be able to split the screen at the cursor position.
- 3.2 THE user SHALL be able to swap between the two halves using PF9 (SWAP).
- 3.3 EACH half of the split screen SHALL operate independently.
- 3.4 THE user SHALL be able to unsplit the screen by pressing PF3 (END) in one half until only one panel remains.

---

## Requirement ISPF-4: LOCATE Command

WHEN the user enters LOCATE (or L) followed by a name on a list panel,
THE workbench SHALL scroll the list to display the matching item at the top.

Criteria:
- 4.1 WHEN the item exists, THE list SHALL scroll to position it at the top of the visible area.
- 4.2 WHEN the item does not exist, THE list SHALL scroll to the nearest alphabetically adjacent item.
- 4.3 THE LOCATE command SHALL accept partial names.

---

## Requirement ISPF-5: RETRIEVE Command

WHEN the user presses PF12 (RETRIEVE),
THE workbench SHALL recall the previously entered command into the COMMAND field.

Criteria:
- 5.1 THE workbench SHALL maintain a command history of at least the last 20 commands entered.
- 5.2 EACH press of PF12 SHALL cycle backward through the command history.
- 5.3 Command history SHALL persist within a session.
