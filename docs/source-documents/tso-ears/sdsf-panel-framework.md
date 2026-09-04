# SDSF Panel Framework, SET Commands, and Session Persistence -- EARS Requirements

Source documents: ikja100 (SDSF User Guide), ikja300 (SDSF Operation and Customization).

Priority: P1 (SDSF-1, SDSF-2, SDSF-4, SDSF-SET-1, SDSF-SET-8, SDSF-SET-9, SDSF-SET-12, SDSF-PERSIST-1) /
          P2 (SDSF-3, SDSF-5, SDSF-SET-2 through SDSF-SET-11) /
          P3 (SDSF-PERSIST-2).
Sub-project mapping: FFW-JES (primary), menu-and-statusbar, configuration-system,
                     function-keys-and-history, layout-and-docking, startup-and-session (secondary).

---

## Section A: Panel Framework

### Requirement SDSF-1: Panel Layout

WHEN an SDSF-style panel is displayed,
THE workbench SHALL render the standard SDSF panel layout.

Criteria:
- 1.1 THE panel SHALL display an action bar at the top with pull-down menus: Display, Filter, View, Print, Options, Search, Help.
- 1.2 THE panel SHALL display a title line showing the panel name, system name, and line range (e.g., "LINE 1-18 (72)").
- 1.3 THE panel SHALL display a message area to the right of the title line for short error and confirmation messages.
- 1.4 THE panel SHALL display a COMMAND INPUT ===> field at the bottom.
- 1.5 THE panel SHALL display a SCROLL ===> field adjacent to the COMMAND field.
- 1.6 THE panel SHALL display filter information lines below the COMMAND field (PREFIX=, DEST=, OWNER=, SYSNAME=).
- 1.7 THE data area SHALL display tabular data with a fixed NP (iNPut) column at the left that does not scroll.
- 1.8 THE first data column (fixed field) SHALL remain visible when the user scrolls right.

### Requirement SDSF-2: Action Characters (NP Column)

WHEN the user types an action character in the NP column of a tabular panel,
THE workbench SHALL execute the corresponding action against that row.

Criteria:
- 2.1 THE workbench SHALL support the following universal action characters on job panels: S (Browse/Select), ? (Job Data Sets), C (Cancel), H (Hold), A (Release), P (Purge), D (Display), E (Edit/Restart), J (Start), W (Spin).
- 2.2 THE user SHALL be able to display valid action characters for a panel by entering SET ACTION or typing ./ in the NP column.
- 2.3 THE user SHALL be able to repeat the previous action character using = in the NP column.
- 2.4 THE user SHALL be able to apply an action to a block of rows using // on the first and last rows with the action character on any row in between.
- 2.5 THE user SHALL be able to issue action characters from the command line using the syntax: "rows action-character" (e.g., "2 C" to cancel row 2).
- 2.6 WHEN SET ROWNUM is active, THE panel SHALL display row numbers in the NP column area.

### Requirement SDSF-3: Overtype Fields

WHEN a column is overtypeable,
THE workbench SHALL allow the user to change its value by typing over it.

Criteria:
- 3.1 THE workbench SHALL visually distinguish overtypeable fields from read-only fields (e.g., by colour or indicator).
- 3.2 WHEN the user types a new value over an overtypeable field and presses Enter, THE workbench SHALL apply the change.
- 3.3 THE user SHALL be able to overtype values from the command line using the syntax: "rows column-title=value".
- 3.4 WHEN a column has multiple related values, THE user SHALL be able to enter + in the column to open an Overtype Extension pop-up showing all related fields.

### Requirement SDSF-4: Main Panel and MGRP

WHEN the user invokes SDSF (enters =S or the SDSF command),
THE workbench SHALL display the SDSF main panel.

Criteria:
- 4.1 THE main panel SHALL list all available SDSF commands with their name, description, group, and availability status.
- 4.2 THE main panel SHALL organise commands into groups: Jobs, Output, JES, Log, Memory, Network, OMVS, Program, Security, Sysplex, System, WLM, Devices, Measure.
- 4.3 THE user SHALL be able to select a command from the main panel using the S action character.
- 4.4 THE user SHALL be able to set the main panel to display as a grouped list (MGRP) using SET MAIN GROUP.
- 4.5 THE MGRP panel SHALL display command groups that can be expanded or collapsed.
- 4.6 THE user SHALL be able to return to the main panel from any SDSF panel by entering the MENU command.

### Requirement SDSF-5: Help System

WHEN the user presses PF1 or enters HELP,
THE workbench SHALL display context-sensitive help for the current panel.

Criteria:
- 5.1 THE help panel SHALL display a scrollable description of the current panel's purpose, commands, and action characters.
- 5.2 THE user SHALL be able to search help content using the SEARCH command.
- 5.3 THE user SHALL be able to view help for action characters using the ACTH command.
- 5.4 THE user SHALL be able to view help for column names using the COLH command.
- 5.5 THE user SHALL be able to view help for commands using the CMDH command.

---

## Section B: SET Commands

### Requirement SDSF-SET-1: SET ACTION

WHEN the user enters SET ACTION,
THE workbench SHALL display the valid action characters for the current panel.

Criteria:
- 1.1 SET ACTION (or SET ACTION LONG) SHALL display action characters with descriptions.
- 1.2 SET ACTION SHORT SHALL display action characters without descriptions.
- 1.3 SET ACTION OFF SHALL stop displaying action characters.
- 1.4 The ACTION= line SHALL appear below the filter information lines.

### Requirement SDSF-SET-2: SET BCOLOR

WHEN the user enters SET BCOLOR ON or OFF,
THE workbench SHALL enable or disable color and highlighting on browse panels.

Criteria:
- 2.1 WHEN SET BCOLOR ON, THE ULOG, CK, and health check browse panels SHALL display messages with color and highlighting based on severity.
- 2.2 WHEN SET BCOLOR OFF, browse panels SHALL display without color differentiation.
- 2.3 The BCOLOR setting SHALL persist across sessions.

### Requirement SDSF-SET-3: SET CONFIRM

WHEN the user enters SET CONFIRM ON or OFF,
THE workbench SHALL control whether confirmation dialogs appear for destructive actions.

Criteria:
- 3.1 WHEN SET CONFIRM ON, THE workbench SHALL display a confirmation pop-up before executing cancel, purge, or delete actions.
- 3.2 WHEN SET CONFIRM OFF, destructive actions SHALL execute without confirmation.

### Requirement SDSF-SET-4: SET CURSOR

WHEN the user enters SET CURSOR,
THE workbench SHALL control cursor positioning behavior on panels.

Criteria:
- 4.1 SET CURSOR CMDLINE SHALL position the cursor on the COMMAND field when a panel is displayed.
- 4.2 SET CURSOR DATA SHALL position the cursor in the data area when a panel is displayed.

### Requirement SDSF-SET-5: SET DATE

WHEN the user enters SET DATE format,
THE workbench SHALL control the date display format on panels.

Criteria:
- 5.1 THE SET DATE command SHALL support formats: MDY, DMY, YMD, JUL (Julian).
- 5.2 The date format setting SHALL apply to all date columns on all panels.

### Requirement SDSF-SET-6: SET DELAY

WHEN the user enters SET DELAY n,
THE workbench SHALL set the automatic refresh interval for panels.

Criteria:
- 6.1 THE SET DELAY command SHALL accept a value in seconds.
- 6.2 WHEN SET DELAY 0, automatic refresh SHALL be disabled.
- 6.3 The delay setting SHALL persist for the session.

### Requirement SDSF-SET-7: SET HEX

WHEN the user enters SET HEX ON or OFF,
THE workbench SHALL toggle hexadecimal display of column values.

Criteria:
- 7.1 WHEN SET HEX ON, column values SHALL be displayed in hexadecimal format.
- 7.2 WHEN SET HEX OFF, column values SHALL be displayed in character format.

### Requirement SDSF-SET-8: SET MAIN

WHEN the user enters SET MAIN,
THE workbench SHALL set the default main panel displayed on entry to SDSF.

Criteria:
- 8.1 SET MAIN TABLE SHALL set the tabular command list as the default main panel.
- 8.2 SET MAIN DASH SHALL set the Dashboard panel as the default main panel.
- 8.3 SET MAIN GROUP SHALL set the MGRP grouped panel as the default main panel.
- 8.4 The SET MAIN setting SHALL persist across sessions.

### Requirement SDSF-SET-9: SET ROWNUM

WHEN the user enters SET ROWNUM ON,
THE workbench SHALL display row numbers in the NP column area.

Criteria:
- 9.1 WHEN SET ROWNUM ON, each row SHALL display a sequential number in the NP area.
- 9.2 Row numbers SHALL enable command-line action character syntax (e.g., "2 C").
- 9.3 SET ROWNUM OFF SHALL hide row numbers.

### Requirement SDSF-SET-10: SET SCHARS

WHEN the user enters SET SCHARS,
THE workbench SHALL set the wildcard characters used for pattern matching.

Criteria:
- 10.1 THE default search characters SHALL be * (any string) and % (any single character).
- 10.2 THE user SHALL be able to redefine these characters via SET SCHARS.

### Requirement SDSF-SET-11: SET SCREEN

WHEN the user enters SET SCREEN,
THE workbench SHALL control the color scheme used to distinguish field types.

Criteria:
- 11.1 THE workbench SHALL use distinct visual indicators for: not active/not overtypeable, active/not overtypeable, not active/overtypeable, active/overtypeable.
- 11.2 THE user SHALL be able to configure these visual indicators.

### Requirement SDSF-SET-12: WHO Command

WHEN the user enters the WHO command,
THE workbench SHALL display the current user's session information.

Criteria:
- 12.1 THE WHO command SHALL display: user ID, logon procedure name, terminal ID, group index, group name, MVS version, JES version, SDSF version, ISPF version, server name, JES name, member name, JES type, system name, sysplex name.
- 12.2 THE WHO command SHALL be accessible from any tabular panel.
- 12.3 THE WHO command SHALL be accessible from the View menu.

### Requirement SDSF-SET-13: QUERY AUTH Command

WHEN the user enters QUERY AUTH,
THE workbench SHALL display the list of SDSF commands the current user is authorized to use.

Criteria:
- 13.1 THE QUERY AUTH command SHALL list all authorized commands.
- 13.2 THE QUERY AUTH LONG command SHALL include JES dependency information for each command.

---

## Section C: Session Persistence

### Requirement SDSF-PERSIST-1: Save Session Settings

WHEN the user exits SDSF,
THE workbench SHALL save the current session settings.

Criteria:
- 1.1 THE workbench SHALL save: SET ACTION setting, SET BCOLOR setting, SET BROWSE setting, SET CONFIRM setting, SET CURSOR setting, SET DATE format, SET DELAY value, SET MAIN setting, SET ROWNUM setting, SET SCHARS values, SET SCREEN colors, SCROLL amount, active filters per panel, column arrangements per panel.
- 1.2 Session settings SHALL be restored when the user next opens SDSF.
- 1.3 THE workbench SHALL support saving settings to the z/OS UNIX file system profile as an alternative to the ISPF profile (SET PTRACE).
- 1.4 THE workbench SHALL support the SNAP command to save the current panel state.

### Requirement SDSF-PERSIST-2: Special DDNames

WHEN the workbench initializes SDSF,
THE workbench SHALL support SDSF special DDNames for customization.

Criteria:
- 2.1 THE workbench SHALL support ISFMIGNB to disable color and highlighting on browse panels.
- 2.2 THE workbench SHALL support ISFMIGXB to enable color and highlighting on browse panels.
- 2.3 THE workbench SHALL support ISFMIGNP to disable file system profiles when running under TSO.
