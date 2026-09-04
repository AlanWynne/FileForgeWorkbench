# TSO/E Session and Logon Emulation -- EARS Requirements

Source documents: ikjb300 (TSO/E Primer), ikjb700 (TSO/E Command Reference).

Priority: P1 -- Core Emulation.
Sub-project mapping: startup-and-session (primary), menu-and-statusbar (secondary).

---

## Requirement TSO-1: Session Startup

WHEN the user launches FileForge Workbench,
THE workbench SHALL present a session startup experience analogous to TSO/E logon,
including a user identity context, session timestamp, and a READY-equivalent prompt or Primary Option Menu.

Criteria:
- 1.1 WHEN the workbench starts, THE shell SHALL display the Primary Option Menu (POM) as the default landing panel.
- 1.2 WHEN the workbench starts, THE shell SHALL record a session start timestamp visible in the status bar.
- 1.3 WHEN the user exits the workbench, THE shell SHALL record a session end timestamp and display a logoff confirmation analogous to "YOURID LOGGED OFF TSO".
- 1.4 THE workbench SHALL support a LOGOFF command that terminates the session and closes the application.

---

## Requirement TSO-2: READY Prompt and Line Mode

WHEN the user is at the command line,
THE workbench SHALL accept TSO/E-style commands typed directly,
analogous to the TSO/E READY prompt.

Criteria:
- 2.1 WHEN the user types a command in the Command ===> field and presses Enter, THE workbench SHALL execute the command.
- 2.2 WHEN a command is not found, THE workbench SHALL display a message equivalent to "COMMAND FOR NOT FOUND".
- 2.3 THE workbench SHALL support the HELP command to display available commands and their syntax.
- 2.4 THE workbench SHALL support the TIME command to display the current date and time.
- 2.5 THE workbench SHALL support the STATUS command to display the status of submitted jobs.

---

## Requirement TSO-3: PF Key Definitions

WHEN the user is on any panel,
THE workbench SHALL support 24 configurable program function (PF) keys.

Criteria:
- 3.1 THE workbench SHALL provide default PF key assignments: PF1=HELP, PF2=SPLIT, PF3=END, PF4=RETURN, PF5=RFIND, PF6=RCHANGE, PF7=UP, PF8=DOWN, PF9=SWAP, PF10=LEFT, PF11=RIGHT, PF12=RETRIEVE.
- 3.2 THE user SHALL be able to view current PF key assignments by entering the KEYS command.
- 3.3 THE user SHALL be able to toggle PF key display at the bottom of the screen with the PFSHOW command.
- 3.4 THE user SHALL be able to change PF key assignments via the Key Configuration dialog.
- 3.5 PF key assignments SHALL persist across sessions.

---

## Requirement TSO-4: Scrolling

WHEN a panel contains more data than fits on screen,
THE workbench SHALL support scrolling in all four directions.

Criteria:
- 4.1 THE workbench SHALL support UP, DOWN, LEFT, RIGHT scroll commands.
- 4.2 THE workbench SHALL support scroll amounts: PAGE (full screen), HALF (half screen), CSR (to cursor), MAX (to beginning or end), DATA (full page minus one line), and a numeric count.
- 4.3 THE SCROLL field SHALL retain its value between scroll operations.
- 4.4 THE workbench SHALL support TOP and BOTTOM commands to jump to the first and last line of data.
