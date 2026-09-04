# TSO and SDSF EARS Requirements for FileForge Workbench

Source documents parsed: ikja100 (SDSF User Guide), ikja300 (SDSF Operation and Customization), ikjb300 (TSO/E Primer), ikjb700 (TSO/E Command Reference), ikjb800 (TSO/E System Programming Command Reference), ikjc200 (TSO/E REXX User Guide).

These requirements describe what FileForge Workbench SHALL emulate from TSO/E and SDSF to provide a faithful mainframe workstation experience on Windows, Linux, and macOS.

---

## Section 1: TSO/E Session and Logon Emulation

Source: ikjb300 (TSO/E Primer), ikjb700 (TSO/E Command Reference).

### Requirement TSO-1: Session Startup

WHEN the user launches FileForge Workbench,
THE workbench SHALL present a session startup experience analogous to TSO/E logon,
including a user identity context, session timestamp, and a READY-equivalent prompt or Primary Option Menu.

Criteria:
- 1.1 WHEN the workbench starts, THE shell SHALL display the Primary Option Menu (POM) as the default landing panel.
- 1.2 WHEN the workbench starts, THE shell SHALL record a session start timestamp visible in the status bar.
- 1.3 WHEN the user exits the workbench, THE shell SHALL record a session end timestamp and display a logoff confirmation analogous to "YOURID LOGGED OFF TSO".
- 1.4 THE workbench SHALL support a LOGOFF command that terminates the session and closes the application.

### Requirement TSO-2: READY Prompt and Line Mode

WHEN the user is at the command line,
THE workbench SHALL accept TSO/E-style commands typed directly,
analogous to the TSO/E READY prompt.

Criteria:
- 2.1 WHEN the user types a command in the Command ===> field and presses Enter, THE workbench SHALL execute the command.
- 2.2 WHEN a command is not found, THE workbench SHALL display a message equivalent to "COMMAND FOR NOT FOUND".
- 2.3 THE workbench SHALL support the HELP command to display available commands and their syntax.
- 2.4 THE workbench SHALL support the TIME command to display the current date and time.
- 2.5 THE workbench SHALL support the STATUS command to display the status of submitted jobs.

### Requirement TSO-3: PF Key Definitions

WHEN the user is on any panel,
THE workbench SHALL support 24 configurable program function (PF) keys.

Criteria:
- 3.1 THE workbench SHALL provide default PF key assignments: PF1=HELP, PF2=SPLIT, PF3=END, PF4=RETURN, PF5=RFIND, PF6=RCHANGE, PF7=UP, PF8=DOWN, PF9=SWAP, PF10=LEFT, PF11=RIGHT, PF12=RETRIEVE.
- 3.2 THE user SHALL be able to view current PF key assignments by entering the KEYS command.
- 3.3 THE user SHALL be able to toggle PF key display at the bottom of the screen with the PFSHOW command.
- 3.4 THE user SHALL be able to change PF key assignments via the Key Configuration dialog.
- 3.5 PF key assignments SHALL persist across sessions.

### Requirement TSO-4: Scrolling

WHEN a panel contains more data than fits on screen,
THE workbench SHALL support scrolling in all four directions.

Criteria:
- 4.1 THE workbench SHALL support UP, DOWN, LEFT, RIGHT scroll commands.
- 4.2 THE workbench SHALL support scroll amounts: PAGE (full screen), HALF (half screen), CSR (to cursor), MAX (to beginning or end), DATA (full page minus one line), and a numeric count.
- 4.3 THE SCROLL field SHALL retain its value between scroll operations.
- 4.4 THE workbench SHALL support TOP and BOTTOM commands to jump to the first and last line of data.


## Section 2: ISPF-Style Panel Navigation

Source: ikjb300 (TSO/E Primer), ikja100 (SDSF User Guide).

### Requirement ISPF-1: Panel Types

THE workbench SHALL support four panel types analogous to ISPF/PDF:
data entry panels, menu panels, list panels, and edit panels.

Criteria:
- 1.1 A menu panel SHALL display a list of numbered or lettered options and accept an OPTION ===> input field.
- 1.2 A data entry panel SHALL display labelled input fields with ===> arrows and accept typed values.
- 1.3 A list panel SHALL display rows of items with an action field (NP column) to the left of each row.
- 1.4 An edit panel SHALL display file content with line numbers and a COMMAND ===> field.
- 1.5 ALL panels SHALL display a COMMAND ===> field at the bottom (or top) of the screen.
- 1.6 ALL panels SHALL display a SCROLL ===> field adjacent to the COMMAND field.

### Requirement ISPF-2: Panel Hierarchy and Navigation

WHEN the user navigates between panels,
THE workbench SHALL maintain a panel hierarchy analogous to ISPF.

Criteria:
- 2.1 THE user SHALL be able to return to the previous panel by pressing PF3 (END command).
- 2.2 THE user SHALL be able to return to the Primary Option Menu from any panel by pressing PF4 (RETURN command) or entering =0 through =9.
- 2.3 THE user SHALL be able to navigate directly to a nested option using fastpath notation (e.g., 3.1 on the OPTION line).
- 2.4 THE user SHALL be able to jump from one option to another using =option notation (e.g., =2 from within option 3).
- 2.5 THE user SHALL be able to exit the workbench from any menu panel by entering X or =X.

### Requirement ISPF-3: Split Screen

WHEN the user presses PF2 (SPLIT),
THE workbench SHALL divide the display into two independent panels.

Criteria:
- 3.1 THE user SHALL be able to split the screen at the cursor position.
- 3.2 THE user SHALL be able to swap between the two halves using PF9 (SWAP).
- 3.3 EACH half of the split screen SHALL operate independently.
- 3.4 THE user SHALL be able to unsplit the screen by pressing PF3 (END) in one half until only one panel remains.

### Requirement ISPF-4: LOCATE Command

WHEN the user enters LOCATE (or L) followed by a name on a list panel,
THE workbench SHALL scroll the list to display the matching item at the top.

Criteria:
- 4.1 WHEN the item exists, THE list SHALL scroll to position it at the top of the visible area.
- 4.2 WHEN the item does not exist, THE list SHALL scroll to the nearest alphabetically adjacent item.
- 4.3 THE LOCATE command SHALL accept partial names.

### Requirement ISPF-5: RETRIEVE Command

WHEN the user presses PF12 (RETRIEVE),
THE workbench SHALL recall the previously entered command into the COMMAND field.

Criteria:
- 5.1 THE workbench SHALL maintain a command history of at least the last 20 commands entered.
- 5.2 EACH press of PF12 SHALL cycle backward through the command history.
- 5.3 Command history SHALL persist within a session.


## Section 3: SDSF Panel Framework

Source: ikja100 (SDSF User Guide), ikja300 (SDSF Operation and Customization).

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


## Section 4: SDSF Job Queue Panels

Source: ikja100 (SDSF User Guide) - panels I, O, H, ST, DA.

### Requirement SDSF-JQ-1: Input Queue Panel (I)

WHEN the user enters the I command,
THE workbench SHALL display the Input Queue panel showing jobs awaiting execution.

Criteria:
- 1.1 THE Input Queue panel SHALL display columns: NP, JOBNAME, JobID, Owner, Priority, Class, Position, PrtDest, Remote, Node.
- 1.2 THE panel SHALL support filtering by PREFIX, DEST, OWNER, SYSNAME.
- 1.3 THE panel SHALL support action characters: C (Cancel), H (Hold), A (Release), D (Display), S (Browse), ? (Job Data Sets), E (Edit/Restart), J (Start), P (Purge).
- 1.4 THE panel SHALL display arrival time (ARRTIME) and current queue time (CQTIME) columns.
- 1.5 THE panel SHALL display execution start and end times when available.

### Requirement SDSF-JQ-2: Output Queue Panel (O)

WHEN the user enters the O command,
THE workbench SHALL display the Output Queue panel showing jobs with output awaiting printing.

Criteria:
- 2.1 THE Output Queue panel SHALL display columns: NP, JOBNAME, JobID, Owner, Priority, Class, Position, PrtDest, Remote, Node, Max-CC.
- 2.2 THE panel SHALL support action characters: P (Purge), S (Browse output), X (Print), D (Display), H (Hold), A (Release).
- 2.3 THE panel SHALL support the ? action character to display the Job Data Set panel for a job.

### Requirement SDSF-JQ-3: Held Output Panel (H)

WHEN the user enters the H command,
THE workbench SHALL display the Held Output panel showing jobs with held output.

Criteria:
- 3.1 THE Held Output panel SHALL display columns: NP, JOBNAME, JobID, Owner, Priority, Class, Position, PrtDest, Remote, Node, Max-CC, ResGroup.
- 3.2 THE panel SHALL support action characters: A (Release), P (Purge), S (Browse), X (Print), D (Display), ? (Job Data Sets).
- 3.3 THE panel SHALL display execution start and end times.

### Requirement SDSF-JQ-4: Status Panel (ST)

WHEN the user enters the ST command,
THE workbench SHALL display the Status panel showing all jobs in all queues.

Criteria:
- 4.1 THE Status panel SHALL display columns: NP, JOBNAME, JobID, Owner, Priority, Queue, Class, Position, SAff, ASys, Status, Max-CC, ResGroup.
- 4.2 THE panel SHALL support all action characters available on the I, O, and H panels.
- 4.3 THE panel SHALL support the JRL action character to display resource limits for a job.
- 4.4 THE panel SHALL display arrival time, current queue time, execution start time, and execution end time.
- 4.5 THE panel SHALL display LimitsImpact and LimitsRaised indicators.
- 4.6 THE panel SHALL support the JESCANCEL overtypeable column to control JES cancel options.

### Requirement SDSF-JQ-5: Display Active Users Panel (DA)

WHEN the user enters the DA command,
THE workbench SHALL display the Display Active Users panel showing all active address spaces.

Criteria:
- 5.1 THE DA panel SHALL display columns: NP, JOBNAME, StepName, ProcStep, JobID, Owner, Class, Position, DP, Real, Paging, SIO, CPU%, ElapsedTime, OutTime.
- 5.2 THE panel SHALL support action characters: C (Cancel), D (Display), S (Browse), ? (Job Data Sets), JS (Job Steps), JM (Job Memory), JD (Job Devices), JDD (Job DDNames), JT (Job Tasks).
- 5.3 THE panel title line SHALL display system name, paging rate, CPU utilization, and zIIP utilization.
- 5.4 THE panel SHALL support the FJ action character to fetch module information by job name.
- 5.5 THE panel SHALL support the JCM action character to list job common memory objects.
- 5.6 THE panel SHALL support the LE action character to list enclaves.
- 5.7 THE panel SHALL support the LU action character to list user ID information.

### Requirement SDSF-JQ-6: Job Data Set Panel (JDS)

WHEN the user types ? in the NP column of a job on any queue panel,
THE workbench SHALL display the Job Data Set panel for that job.

Criteria:
- 6.1 THE JDS panel SHALL display all SYSOUT data sets for the job with columns: NP, DDName, StepName, ProcStep, DSName, Class, Dest, Copies, Records, Bytes.
- 6.2 THE panel SHALL support action characters: S (Browse), SB (ISPF Browse), SE (ISPF Edit), X (Print), P (Purge).
- 6.3 THE panel SHALL display job name and job ID in the title line.

### Requirement SDSF-JQ-7: Job Step Panel (JS)

WHEN the user types JS in the NP column of a job,
THE workbench SHALL display the Job Step panel for that job.

Criteria:
- 7.1 THE JS panel SHALL display all steps for the job with columns: NP, StepName, ProcStep, Program, StepCC, StartTime, EndTime, CPU, SIO.
- 7.2 THE panel SHALL support the S action character to browse step output.


## Section 5: SDSF Filter, Sort, and Arrange Commands

Source: ikja100 (SDSF User Guide) Chapter 1 and Chapter 9.

### Requirement SDSF-FILTER-1: PREFIX Filter

WHEN the user enters PREFIX(pattern) on any job panel,
THE workbench SHALL filter the display to show only jobs whose names match the pattern.

Criteria:
- 1.1 THE PREFIX filter SHALL support wildcard characters (* for any string, % for any single character).
- 1.2 THE PREFIX filter SHALL be displayed in the filter information line as "PREFIX=pattern".
- 1.3 WHEN PREFIX=* is set, ALL jobs SHALL be displayed regardless of name.
- 1.4 THE PREFIX filter SHALL persist until changed or reset.

### Requirement SDSF-FILTER-2: OWNER Filter

WHEN the user enters OWNER(userid) on any job panel,
THE workbench SHALL filter the display to show only jobs owned by the specified user.

Criteria:
- 2.1 THE OWNER filter SHALL support wildcard characters.
- 2.2 THE OWNER filter SHALL be displayed in the filter information line as "OWNER=userid".
- 2.3 WHEN OWNER=* is set, jobs from ALL owners SHALL be displayed.

### Requirement SDSF-FILTER-3: DEST Filter

WHEN the user enters DEST(destination) on any job panel,
THE workbench SHALL filter the display to show only jobs destined for the specified output destination.

Criteria:
- 3.1 THE DEST filter SHALL support the value ALL to show all destinations.
- 3.2 THE DEST filter SHALL be displayed in the filter information line as "DEST=(destination)".

### Requirement SDSF-FILTER-4: FILTER Command

WHEN the user enters FILTER column operator value,
THE workbench SHALL apply a column-level filter to the current panel.

Criteria:
- 4.1 THE FILTER command SHALL support operators: EQ, NE, GT, LT, GE, LE, CONTAINS, OMIT.
- 4.2 THE FILTER command SHALL support multiple simultaneous filter conditions combined with AND/OR.
- 4.3 THE FILTER command SHALL support wildcard pattern matching using * and %.
- 4.4 THE user SHALL be able to clear all filters using FILTER RESET or RESET.
- 4.5 THE active filter conditions SHALL be displayed in the filter information lines below the COMMAND field.
- 4.6 THE FILTER command SHALL support the SET DISPLAY command to control which columns are shown.

### Requirement SDSF-FILTER-5: SORT Command

WHEN the user enters SORT column [A|D],
THE workbench SHALL sort the panel data by the specified column.

Criteria:
- 5.1 THE SORT command SHALL support ascending (A) and descending (D) sort order.
- 5.2 THE SORT command SHALL support sorting by multiple columns (e.g., SORT JOBNAME A JOBID D).
- 5.3 THE default sort order SHALL be ascending.
- 5.4 THE SORT command SHALL support SET CSORT to set a persistent column sort.

### Requirement SDSF-FILTER-6: ARRANGE Command

WHEN the user enters ARRANGE,
THE workbench SHALL allow the user to reorder, hide, or show columns on the current panel.

Criteria:
- 6.1 THE ARRANGE command SHALL allow columns to be moved left or right.
- 6.2 THE ARRANGE command SHALL allow columns to be hidden (ARRANGE column OFF).
- 6.3 THE ARRANGE command SHALL allow hidden columns to be restored (ARRANGE column ON).
- 6.4 Column arrangements SHALL persist for the session.

### Requirement SDSF-FILTER-7: SET DISPLAY Command

WHEN the user enters SET DISPLAY,
THE workbench SHALL control which columns are visible on the current panel.

Criteria:
- 7.1 THE SET DISPLAY command SHALL support showing the primary column set (SET DISPLAY ON).
- 7.2 THE SET DISPLAY command SHALL support showing the alternate column set (? command or SET DISPLAY ALT).
- 7.3 THE alternate column set SHALL include all primary columns plus additional delayed-access columns.


## Section 6: SDSF Search and Scroll Commands

Source: ikja100 (SDSF User Guide) Chapter 9.

### Requirement SDSF-SCROLL-1: FIND Command

WHEN the user enters FIND string on a browse or log panel,
THE workbench SHALL search for the string and position the display at the first occurrence.

Criteria:
- 1.1 THE FIND command SHALL search forward from the current position by default.
- 1.2 THE FIND command SHALL support FIND string PREV to search backward.
- 1.3 THE FIND command SHALL support FIND string FIRST and FIND string LAST.
- 1.4 THE FIND command SHALL support FIND string NEXT to find the next occurrence.
- 1.5 WHEN the string is not found, THE workbench SHALL display a "string not found" message.
- 1.6 THE RFIND command (PF5) SHALL repeat the last FIND in the same direction.
- 1.7 THE FINDLIM command SHALL set the maximum number of lines to search.

### Requirement SDSF-SCROLL-2: LOCATE Command

WHEN the user enters LOCATE value on a tabular panel,
THE workbench SHALL scroll the panel to position the row matching value at the top.

Criteria:
- 2.1 THE LOCATE command SHALL match against the fixed (first) column of the panel.
- 2.2 THE LOCATE command SHALL support date/time format patterns for time-based columns.
- 2.3 WHEN no exact match exists, THE panel SHALL scroll to the nearest match.

### Requirement SDSF-SCROLL-3: LOG Command

WHEN the user enters LOG on a log panel,
THE workbench SHALL position the display at a specific date/time in the log.

Criteria:
- 3.1 THE LOG command SHALL accept a date/time parameter to position within the system log.
- 3.2 THE LOG command SHALL support relative positioning (e.g., LOG -1H for one hour ago).

### Requirement SDSF-SCROLL-4: NEXT and PREV Commands

WHEN the user enters NEXT or PREV on a log panel,
THE workbench SHALL scroll to the next or previous occurrence of a search string or log record type.

Criteria:
- 4.1 THE NEXT command SHALL scroll forward to the next matching record.
- 4.2 THE PREV command SHALL scroll backward to the previous matching record.
- 4.3 THE NEXT and PREV commands SHALL support filtering by record type.

### Requirement SDSF-SCROLL-5: SNAPSHOT Command

WHEN the user enters SNAPSHOT,
THE workbench SHALL capture the current panel state for comparison or export.

Criteria:
- 5.1 THE SNAPSHOT command SHALL capture the current panel data to a data set or file.
- 5.2 THE captured snapshot SHALL include all visible columns and rows.


## Section 7: SDSF Log Panels

Source: ikja100 (SDSF User Guide) Chapter 1.

### Requirement SDSF-LOG-1: System Log Panel (LOG)

WHEN the user enters the LOG command,
THE workbench SHALL display the system log panel showing system messages.

Criteria:
- 1.1 THE LOG panel SHALL display system messages in chronological order with timestamps.
- 1.2 THE LOG panel SHALL support FIND, NEXT, PREV, and LOG positioning commands.
- 1.3 THE LOG panel SHALL support filtering by message ID, system name, and time range.
- 1.4 THE LOG panel SHALL display record type codes (action, informational, etc.).
- 1.5 THE LOG panel SHALL support color and highlighting based on message severity when SET BCOLOR is ON.
- 1.6 THE LOG panel SHALL support the LOGLIM command to set the maximum number of log records displayed.

### Requirement SDSF-LOG-2: User Log Panel (ULOG)

WHEN the user enters the ULOG command,
THE workbench SHALL display the user log panel showing messages directed to the current user.

Criteria:
- 2.1 THE ULOG panel SHALL display messages sent to the current user session.
- 2.2 THE ULOG panel SHALL support FIND and scroll commands.
- 2.3 THE ULOG panel SHALL support color and highlighting based on message severity.
- 2.4 THE ULOG panel SHALL be accessible from the SDSF main panel.

### Requirement SDSF-LOG-3: System Requests Panel (SR)

WHEN the user enters the SR command,
THE workbench SHALL display the System Requests panel showing outstanding operator messages and WTORs.

Criteria:
- 3.1 THE SR panel SHALL display outstanding action messages, eventual action messages, and messages awaiting replies.
- 3.2 THE SR panel SHALL display the elapsed time since each system request was issued.
- 3.3 THE SR panel SHALL support the D (Display) action character to show message details.

### Requirement SDSF-LOG-4: Event Log Panel (ELOG)

WHEN the user enters the ELOG command,
THE workbench SHALL display the Event Log panel showing key system events.

Criteria:
- 4.1 THE ELOG panel SHALL display important system events with timestamps and event types.
- 4.2 THE ELOG panel SHALL support the LI action character to list JES resource information related to an event.
- 4.3 THE ELOG panel SHALL provide fast access to the system log at the time an event occurred.


## Section 8: SDSF System Information Panels

Source: ikja100 (SDSF User Guide) Chapter 2.

### Requirement SDSF-SYS-1: System Panel (SYS)

WHEN the user enters the SYS command,
THE workbench SHALL display the System panel showing system configuration and resource utilization.

Criteria:
- 1.1 THE SYS panel SHALL display: system name, sysplex name, z/OS level, IPL date/time, IPL volume, CPU utilization, real memory utilization, paging rate, spool utilization.
- 1.2 THE SYS panel SHALL display dedicated memory (DMem), dedicated memory percentage, and system memory in use.
- 1.3 THE SYS panel SHALL display the number of active TSO users, batch jobs, and started tasks.
- 1.4 THE SYS panel SHALL display the current IEASYMS and IEASYS parameters.
- 1.5 THE SYS panel SHALL display the validated boot status.

### Requirement SDSF-SYS-2: Dashboard Panel (DASH)

WHEN the user enters the DASH command or sets SET MAIN DASH,
THE workbench SHALL display the Dashboard panel showing a system overview.

Criteria:
- 2.1 THE DASH panel SHALL display system attributes: sysplex name, system name, z/OS level, system clone, SMFID, JES name, JES node, JES member, LPAR name, IPL volume, IPL date/time, IEASYMS, IEASYMS, CVTVERID, hardware name, CPC node, user ID.
- 2.2 THE DASH panel SHALL display system metrics: CPU%, zIIP%, spool%, SIO rate, auxiliary storage%, real available frames, real%, page rate, system MSU, average MSU, max ASID, free ASID, bad ASID, TSO users, batch jobs, WTORs, HVComm%.
- 2.3 THE DASH panel SHALL be configurable via SET DASH.
- 2.4 THE DASH panel SHALL be settable as the default main panel via SET MAIN DASH.

### Requirement SDSF-SYS-3: Initiator Panel (INIT)

WHEN the user enters the INIT command,
THE workbench SHALL display the Initiator panel showing all active initiators.

Criteria:
- 3.1 THE INIT panel SHALL display columns: NP, InitName, Status, Class, JobName, JobID, StepName, ProcStep.
- 3.2 THE panel SHALL support action characters: D (Display), S (Browse), JD (Job Devices), JDD (Job DDNames), JM (Job Memory).
- 3.3 THE panel SHALL support overtypeable Class column to change the job class an initiator accepts.

### Requirement SDSF-SYS-4: Job Class Panel (JC)

WHEN the user enters the JC command,
THE workbench SHALL display the Job Class panel showing all defined job classes.

Criteria:
- 4.1 THE JC panel SHALL display columns: NP, Class, Active, MaxActive, Priority, Description, ProcName, QAff.
- 4.2 THE panel SHALL support overtypeable columns: Active, MaxActive, Priority, Description, ProcName, QAff, JESCancel.
- 4.3 THE panel SHALL support the JRL action character to display resource limits for a job class.
- 4.4 THE panel SHALL support the I action character to display Job Class Members (JCM panel).

### Requirement SDSF-SYS-5: Spool Volumes Panel (SP)

WHEN the user enters the SP command,
THE workbench SHALL display the Spool Volumes panel showing spool volume utilization.

Criteria:
- 5.1 THE SP panel SHALL display columns: NP, VolSer, DevType, Status, TotalTracks, UsedTracks, FreeTracks, UsedPct.
- 5.2 THE panel SHALL support the LH action character to list resource history for a spool volume.
- 5.3 THE panel SHALL support the LV action character to list data sets on a spool volume.
- 5.4 THE panel SHALL support the LVT action character to display the VTOC for a spool volume.


## Section 9: SDSF SET Commands and Session Settings

Source: ikja100 (SDSF User Guide) Chapter 9.

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


## Section 10: SDSF Browse and Print

Source: ikja100 (SDSF User Guide) Chapter 1.

### Requirement SDSF-BROWSE-1: Browse Job Output

WHEN the user types S in the NP column of a job,
THE workbench SHALL open the job output in a browse viewer.

Criteria:
- 1.1 THE browse viewer SHALL display the job output in line-mode format.
- 1.2 THE browse viewer SHALL support FIND, RFIND, UP, DOWN, LEFT, RIGHT, TOP, BOTTOM scroll commands.
- 1.3 THE browse viewer SHALL support SET HEX to toggle hexadecimal display.
- 1.4 THE user SHALL be able to open job output in ISPF Browse (SB action), ISPF Edit (SE action), or ISPF View (SV action).
- 1.5 THE user SHALL be able to browse a specific output data set using the Sn action character (where n is the data set sequence number).

### Requirement SDSF-BROWSE-2: Browse Session Settings

WHEN the user configures browse behavior,
THE workbench SHALL support SET BROWSE settings.

Criteria:
- 2.1 SET BROWSE ISPF SHALL cause the S action to invoke ISPF Browse instead of SDSF browse.
- 2.2 SET BROWSE SDSF SHALL cause the S action to use the SDSF line-mode browser.
- 2.3 The browse setting SHALL persist across sessions.

### Requirement SDSF-BROWSE-3: Print from SDSF Panels

WHEN the user invokes print from an SDSF panel,
THE workbench SHALL support printing panel content to a file or output destination.

Criteria:
- 3.1 THE PRINT command SHALL support printing the current panel to a data set, SYSOUT, a file, or a DDNAME.
- 3.2 THE PRINT command SHALL support printing a tabular panel with all visible columns.
- 3.3 THE PRINT command SHALL support PRINT CLOSE to close the print data set.
- 3.4 THE PRINT command SHALL support PRINT OPEN to open a print data set before printing.

### Requirement SDSF-BROWSE-4: Show All Column Values

WHEN the user types / (slash) in the NP column of a row,
THE workbench SHALL display a pop-up showing all column values for that row.

Criteria:
- 4.1 THE Show Columns pop-up SHALL display all columns and their values in a scrollable list.
- 4.2 THE pop-up SHALL include an option to show all columns (including blank values) or only columns with values.
- 4.3 THE pop-up SHALL include an option to format values using the panel column width or maximum width.


## Section 11: TSO/E Command Emulation

Source: ikjb700 (TSO/E Command Reference).

### Requirement TSO-CMD-1: ALLOCATE Command

WHEN the user enters the ALLOCATE command,
THE workbench SHALL allocate a dataset or file to the current session.

Criteria:
- 1.1 THE ALLOCATE command SHALL support DATASET/DSNAME, FILE/DDNAME, and DUMMY operands.
- 1.2 THE ALLOCATE command SHALL support disposition operands: OLD, SHR, MOD, NEW, SYSOUT.
- 1.3 THE ALLOCATE command SHALL support space operands: SPACE, TRACKS, CYLINDERS, BLOCK, BLKSIZE, AVBLOCK, AVGREC.
- 1.4 THE ALLOCATE command SHALL support DCB operands: RECFM, LRECL, BLKSIZE, DSORG, BUFNO, BUFL.
- 1.5 THE ALLOCATE command SHALL support the LIKE operand to copy attributes from an existing dataset.
- 1.6 THE ALLOCATE command SHALL support the REUSE operand to reallocate an already-allocated file.
- 1.7 THE ALLOCATE command SHALL support the DSNTYPE operand: LIBRARY, PDS, HFS, LARGE, BASIC, EXTREQ, EXTPREF.
- 1.8 THE ALLOCATE command SHALL support the RELEASE operand to free unused space on close.
- 1.9 THE ALLOCATE command SHALL support the KEEP, DELETE, CATALOG, UNCATALOG disposition operands.

### Requirement TSO-CMD-2: FREE Command

WHEN the user enters the FREE command,
THE workbench SHALL release a previously allocated dataset or file.

Criteria:
- 2.1 THE FREE command SHALL support FILE/DDNAME and DATASET/DSNAME operands.
- 2.2 THE FREE command SHALL support the ATTRLIST operand to delete an attribute list.
- 2.3 THE FREE command SHALL support the HOLD and NOHOLD operands for SYSOUT datasets.
- 2.4 THE FREE command SHALL support the SPIN operand to control when SYSOUT is made available.

### Requirement TSO-CMD-3: DELETE Command

WHEN the user enters the DELETE command,
THE workbench SHALL delete a dataset or member.

Criteria:
- 3.1 THE DELETE command SHALL accept a dataset name or list of dataset names.
- 3.2 THE DELETE command SHALL support the MEMBER operand to delete specific PDS members.
- 3.3 THE DELETE command SHALL support the PURGE operand to delete datasets regardless of expiration date.
- 3.4 THE DELETE command SHALL support the SCRATCH operand to physically remove the dataset.

### Requirement TSO-CMD-4: RENAME Command

WHEN the user enters the RENAME command,
THE workbench SHALL rename a dataset or PDS member.

Criteria:
- 4.1 THE RENAME command SHALL accept old-name and new-name positional operands.
- 4.2 THE RENAME command SHALL support renaming PDS members.
- 4.3 THE RENAME command SHALL support creating an alias for a PDS member.

### Requirement TSO-CMD-5: LISTCAT Command

WHEN the user enters the LISTCAT command,
THE workbench SHALL list catalog entries.

Criteria:
- 5.1 THE LISTCAT command SHALL list all cataloged datasets matching the specified criteria.
- 5.2 THE LISTCAT command SHALL support the ENTRIES operand to list specific entries.
- 5.3 THE LISTCAT command SHALL support the LEVEL operand to list entries at a specific qualifier level.
- 5.4 THE LISTCAT command SHALL support the ALL operand to display all catalog attributes.

### Requirement TSO-CMD-6: LISTDS Command

WHEN the user enters the LISTDS command,
THE workbench SHALL display attributes of one or more datasets.

Criteria:
- 6.1 THE LISTDS command SHALL display: RECFM, LRECL, BLKSIZE, DSORG, volume serial, creation date, expiration date.
- 6.2 THE LISTDS command SHALL support the MEMBERS operand to list PDS members.
- 6.3 THE LISTDS command SHALL support the STATUS operand to display allocation status.
- 6.4 THE LISTDS command SHALL support the HISTORY operand to display dataset history.

### Requirement TSO-CMD-7: LISTALC Command

WHEN the user enters the LISTALC command,
THE workbench SHALL list all datasets currently allocated to the session.

Criteria:
- 7.1 THE LISTALC command SHALL display the file name (DDNAME), dataset name, and disposition for each allocated dataset.
- 7.2 THE LISTALC command SHALL support the STATUS operand to show allocation status details.
- 7.3 THE LISTALC command SHALL support the HISTORY operand to show allocation history.

### Requirement TSO-CMD-8: SUBMIT Command

WHEN the user enters the SUBMIT command,
THE workbench SHALL submit a job for batch execution.

Criteria:
- 8.1 THE SUBMIT command SHALL accept a dataset name containing JCL or FFJCL.
- 8.2 THE SUBMIT command SHALL support the NOTIFY operand to send a message when the job completes.
- 8.3 THE SUBMIT command SHALL support the HOLD operand to submit the job in held status.
- 8.4 THE SUBMIT command SHALL support the CLASS operand to override the job class.
- 8.5 WHEN a job is submitted successfully, THE workbench SHALL display the assigned job ID.

### Requirement TSO-CMD-9: STATUS Command

WHEN the user enters the STATUS command,
THE workbench SHALL display the status of submitted jobs.

Criteria:
- 9.1 THE STATUS command SHALL display the job name, job ID, and current status for each job.
- 9.2 THE STATUS command SHALL support a job name operand to filter results.
- 9.3 THE STATUS command SHALL display the job class, priority, and queue position.

### Requirement TSO-CMD-10: OUTPUT Command

WHEN the user enters the OUTPUT command,
THE workbench SHALL display or manage job output.

Criteria:
- 10.1 THE OUTPUT command SHALL display job output at the terminal.
- 10.2 THE OUTPUT command SHALL support the DELETE operand to delete job output.
- 10.3 THE OUTPUT command SHALL support the CLASS operand to change the output class.
- 10.4 THE OUTPUT command SHALL support the HOLD and NOHOLD operands.

### Requirement TSO-CMD-11: CANCEL Command

WHEN the user enters the CANCEL command,
THE workbench SHALL cancel a submitted batch job.

Criteria:
- 11.1 THE CANCEL command SHALL accept a job name or job ID operand.
- 11.2 THE CANCEL command SHALL support the DUMP operand to request a storage dump on cancellation.
- 11.3 WHEN the job is successfully cancelled, THE workbench SHALL display a confirmation message.

### Requirement TSO-CMD-12: SEND Command

WHEN the user enters the SEND command,
THE workbench SHALL send a message to another user or the operator.

Criteria:
- 12.1 THE SEND command SHALL accept a message text and USER operand.
- 12.2 THE SEND command SHALL support the LOGON operand to deliver the message at next logon.
- 12.3 THE SEND command SHALL support sending to multiple users.
- 12.4 THE message text SHALL be limited to 115 characters.

### Requirement TSO-CMD-13: PROFILE Command

WHEN the user enters the PROFILE command,
THE workbench SHALL display or change the user profile settings.

Criteria:
- 13.1 THE PROFILE command SHALL display current settings: prefix, language, message mode, prompt mode, character set.
- 13.2 THE PROFILE command SHALL support the PREFIX operand to set the default dataset name prefix.
- 13.3 THE PROFILE command SHALL support the NOPREFIX operand to disable the default prefix.
- 13.4 THE PROFILE command SHALL support the MSGID operand to control message ID display.
- 13.5 THE PROFILE command SHALL support the PROMPT and NOPROMPT operands.

### Requirement TSO-CMD-14: PRINTDS Command

WHEN the user enters the PRINTDS command,
THE workbench SHALL print a dataset to a system printer or file.

Criteria:
- 14.1 THE PRINTDS command SHALL accept a DATASET operand specifying the dataset to print.
- 14.2 THE PRINTDS command SHALL support the DEST operand to specify the print destination.
- 14.3 THE PRINTDS command SHALL support the CLASS operand to specify the output class.
- 14.4 THE PRINTDS command SHALL support the COPIES operand to specify the number of copies.


## Section 12: TSO/E EDIT Command

Source: ikjb700 (TSO/E Command Reference).

### Requirement TSO-EDIT-1: EDIT Command

WHEN the user enters the EDIT command,
THE workbench SHALL open a dataset for editing in the editor panel.

Criteria:
- 1.1 THE EDIT command SHALL accept a dataset name as a positional operand.
- 1.2 THE EDIT command SHALL support the NEW operand to create a new dataset.
- 1.3 THE EDIT command SHALL support the OLD operand to edit an existing dataset.
- 1.4 THE EDIT command SHALL support the RECFM, LRECL, and BLKSIZE operands for new datasets.
- 1.5 THE EDIT command SHALL support the NONUM and NUM operands to control line numbering.
- 1.6 THE EDIT command SHALL support the CAPS and ASIS operands to control case translation.

### Requirement TSO-EDIT-2: EDIT Subcommands

WHEN the user is in an EDIT session,
THE workbench SHALL support the standard EDIT subcommands.

Criteria:
- 2.1 THE FIND subcommand SHALL search for a string in the edit buffer.
- 2.2 THE CHANGE subcommand SHALL replace a string with another string.
- 2.3 THE DELETE subcommand SHALL delete lines from the edit buffer.
- 2.4 THE INSERT subcommand SHALL insert blank lines into the edit buffer.
- 2.5 THE COPY subcommand SHALL copy lines within the edit buffer.
- 2.6 THE MOVE subcommand SHALL move lines within the edit buffer.
- 2.7 THE SAVE subcommand SHALL save the current edit buffer to the dataset.
- 2.8 THE END subcommand SHALL save and exit the edit session.
- 2.9 THE CANCEL subcommand SHALL exit without saving changes.
- 2.10 THE TOP subcommand SHALL scroll to the top of the edit buffer.
- 2.11 THE BOTTOM subcommand SHALL scroll to the bottom of the edit buffer.
- 2.12 THE UP and DOWN subcommands SHALL scroll the edit buffer.
- 2.13 THE SUBMIT subcommand SHALL submit the current edit buffer as a job.
- 2.14 THE RENUM subcommand SHALL renumber the lines in the edit buffer.
- 2.15 THE UNNUM subcommand SHALL remove line numbers from the edit buffer.
- 2.16 THE PROFILE subcommand SHALL display or change edit profile settings.
- 2.17 THE VERIFY subcommand SHALL display the current line in the edit buffer.
- 2.18 THE TABSET subcommand SHALL set tab stop positions.

### Requirement TSO-EDIT-3: Line Commands in EDIT

WHEN the user types line commands in the line number area of an edit panel,
THE workbench SHALL execute the corresponding line operations.

Criteria:
- 3.1 THE d line command SHALL delete the line.
- 3.2 THE i line command SHALL insert a blank line after the current line.
- 3.3 THE r line command SHALL repeat the line.
- 3.4 THE c line command SHALL mark a line for copy.
- 3.5 THE m line command SHALL mark a line for move.
- 3.6 THE a line command SHALL mark a line as the after-target for copy or move.
- 3.7 THE b line command SHALL mark a line as the before-target for copy or move.
- 3.8 Block line commands (dd, cc, mm) SHALL operate on a range of lines.
- 3.9 A numeric suffix on a line command SHALL repeat the operation (e.g., d3 deletes 3 lines).


## Section 13: REXX Scripting Emulation

Source: ikjc200 (TSO/E REXX User Guide).

### Requirement REXX-1: REXX Exec Execution

WHEN the user invokes a REXX exec,
THE workbench SHALL execute the exec using the Lua macro engine as the scripting bridge.

Criteria:
- 1.1 THE workbench SHALL support executing REXX-style execs stored as members of a PDS allocated to SYSEXEC or SYSPROC.
- 1.2 THE workbench SHALL support the EXEC command to run an exec explicitly: EXEC dataset(member) EXEC.
- 1.3 THE workbench SHALL support running an exec implicitly by typing the member name at the command prompt.
- 1.4 THE workbench SHALL support the % prefix to reduce search time when running an exec implicitly.
- 1.5 THE workbench SHALL support passing arguments to an exec via the EXEC command or implicit invocation.

### Requirement REXX-2: REXX Host Command Environment

WHEN a REXX exec issues a host command,
THE workbench SHALL route the command to the appropriate host command environment.

Criteria:
- 2.1 THE default host command environment SHALL be TSO, routing commands to the workbench command processor.
- 2.2 THE exec SHALL be able to change the host command environment using ADDRESS environment-name.
- 2.3 THE workbench SHALL support the ISPEXEC host command environment for ISPF service calls.
- 2.4 THE workbench SHALL support the ISREDIT host command environment for ISPF Edit macro calls.
- 2.5 THE return code from each host command SHALL be available in the RC special variable.

### Requirement REXX-3: REXX External Functions

WHEN a REXX exec calls a TSO/E external function,
THE workbench SHALL provide equivalent implementations.

Criteria:
- 3.1 THE workbench SHALL support the LISTDSI function to retrieve dataset information.
- 3.2 THE workbench SHALL support the MSG function to control message display.
- 3.3 THE workbench SHALL support the MVSVAR function to retrieve system variable values.
- 3.4 THE workbench SHALL support the OUTTRAP function to capture command output into a stem variable.
- 3.5 THE workbench SHALL support the PROMPT function to control interactive prompting.
- 3.6 THE workbench SHALL support the SYSDSN function to test whether a dataset exists.
- 3.7 THE workbench SHALL support the SYSVAR function to retrieve TSO/E session variables.
- 3.8 THE workbench SHALL support the USERID function to return the current user ID.

### Requirement REXX-4: EXECIO Command

WHEN a REXX exec uses the EXECIO command,
THE workbench SHALL support reading from and writing to datasets.

Criteria:
- 4.1 THE EXECIO command SHALL support DISKR (read from dataset) and DISKW (write to dataset) operations.
- 4.2 THE EXECIO command SHALL support the STEM option to read/write data into/from a stem variable.
- 4.3 THE EXECIO command SHALL support the FINIS option to close the dataset after the operation.
- 4.4 THE EXECIO command SHALL support the SKIP option to skip records without reading them.
- 4.5 THE EXECIO command SHALL return appropriate return codes: 0 (success), 2 (end of file), non-zero (error).

### Requirement REXX-5: Data Stack

WHEN a REXX exec uses the data stack,
THE workbench SHALL maintain a LIFO/FIFO data stack for inter-exec communication.

Criteria:
- 5.1 THE workbench SHALL support PUSH to add an element to the top of the stack.
- 5.2 THE workbench SHALL support QUEUE to add an element to the bottom of the stack.
- 5.3 THE workbench SHALL support PULL to remove an element from the top of the stack.
- 5.4 THE workbench SHALL support QUEUED to return the number of elements on the stack.
- 5.5 THE workbench SHALL support MAKEBUF to create a new buffer on the stack.
- 5.6 THE workbench SHALL support DROPBUF to remove a buffer from the stack.
- 5.7 THE workbench SHALL support NEWSTACK and DELSTACK for private stack management.


## Section 14: SDSF JES Resource and WLM Panels

Source: ikja100 (SDSF User Guide) Chapter 2.

### Requirement SDSF-JES-1: MAS Panel (MAS)

WHEN the user enters the MAS command,
THE workbench SHALL display the Multi-Access Spool panel showing JES members.

Criteria:
- 1.1 THE MAS panel SHALL display columns: NP, MemberName, Status, JESType, ActiveJobs, MaxJobs, Spool%.
- 1.2 THE panel SHALL support the D (Display) action character to show member details.

### Requirement SDSF-JES-2: Job Group Panel (JG)

WHEN the user enters the JG command,
THE workbench SHALL display the Job Group panel showing job groups.

Criteria:
- 2.1 THE JG panel SHALL display columns: NP, GroupName, Owner, Status, JobCount, Max-CC.
- 2.2 THE panel SHALL support action characters: C (Cancel), H (Hold), A (Release), D (Display), S (Browse), ? (Job Data Sets), JP (Job Dependencies).
- 2.3 THE panel SHALL display the maximum condition code for the group.

### Requirement SDSF-JES-3: WLM Service Classes Panel (SRVC)

WHEN the user enters the SRVC command,
THE workbench SHALL display the WLM Service Classes panel.

Criteria:
- 3.1 THE SRVC panel SHALL display columns: NP, ServiceClass, WorkloadName, Period, Velocity, ImportanceLevel.
- 3.2 THE panel SHALL support the L action character to list address spaces assigned to the service class.
- 3.3 THE panel SHALL support the LE action character to list enclaves.

### Requirement SDSF-JES-4: Scheduling Environment Panel (SE)

WHEN the user enters the SE command,
THE workbench SHALL display the Scheduling Environment panel.

Criteria:
- 4.1 THE SE panel SHALL display columns: NP, SchedEnvName, Status, Description.
- 4.2 THE panel SHALL support the D (Display) action character.

---

## Section 15: SDSF REXX Interface

Source: ikja100 (SDSF User Guide) Chapter 6.

### Requirement SDSF-REXX-1: ISFCALLS Host Command Environment

WHEN a REXX exec adds the SDSF host command environment,
THE workbench SHALL support SDSF operations from REXX.

Criteria:
- 1.1 THE workbench SHALL support ISFCALLS ON to add the SDSF host command environment.
- 1.2 THE workbench SHALL support ISFCALLS OFF to remove the SDSF host command environment.
- 1.3 THE ISFCALLS command SHALL return a result code indicating success or failure.

### Requirement SDSF-REXX-2: ISFEXEC Command

WHEN a REXX exec issues ISFEXEC panelname,
THE workbench SHALL execute the SDSF panel command and populate REXX variables with the results.

Criteria:
- 2.1 THE ISFEXEC command SHALL accept any valid SDSF panel command (e.g., ISFEXEC ST).
- 2.2 THE ISFEXEC command SHALL populate the ISFROWS variable with the number of rows returned.
- 2.3 THE ISFEXEC command SHALL populate column variables (e.g., JOBNAME.n, JOBID.n) for each row.
- 2.4 THE ISFEXEC command SHALL support filter commands (PREFIX, OWNER, DEST) before panel access.
- 2.5 THE ISFEXEC command SHALL support the COLS option to specify which columns to retrieve.
- 2.6 THE ISFEXEC command SHALL return a return code: 0 (success), 4 (warning), 8 (error), 12 (severe error).

### Requirement SDSF-REXX-3: ISFACT Command

WHEN a REXX exec issues ISFACT to perform action characters,
THE workbench SHALL execute the action against the specified row.

Criteria:
- 3.1 THE ISFACT command SHALL accept a row token and action character.
- 3.2 THE ISFACT command SHALL support modifying overtypeable column values.
- 3.3 THE ISFACT command SHALL return a return code indicating success or failure.
- 3.4 THE ISFACT command SHALL support the TOKEN option to specify the row by token.

### Requirement SDSF-REXX-4: ISFBROWSE Command

WHEN a REXX exec issues ISFBROWSE,
THE workbench SHALL open job output for browsing from within the exec.

Criteria:
- 4.1 THE ISFBROWSE command SHALL accept a row token identifying the output to browse.
- 4.2 THE ISFBROWSE command SHALL support the STEM option to read output into a stem variable.
- 4.3 THE ISFBROWSE command SHALL support the PARM option to pass parameters to the browse session.

### Requirement SDSF-REXX-5: ISFSLASH Command

WHEN a REXX exec issues ISFSLASH,
THE workbench SHALL issue a system command and capture the response.

Criteria:
- 5.1 THE ISFSLASH command SHALL accept a system command string.
- 5.2 THE ISFSLASH command SHALL populate REXX variables with the command response lines.
- 5.3 THE ISFSLASH command SHALL support a delay parameter to wait for all responses.
- 5.4 THE ISFSLASH command SHALL return a return code indicating success or failure.

### Requirement SDSF-REXX-6: ISFGET Command

WHEN a REXX exec issues ISFGET,
THE workbench SHALL retrieve all column values for a single row.

Criteria:
- 6.1 THE ISFGET command SHALL accept a row token.
- 6.2 THE ISFGET command SHALL populate REXX variables with all column values for the row.
- 6.3 THE ISFGET command SHALL return a return code indicating success or failure.

### Requirement SDSF-REXX-7: ISFLOG Command

WHEN a REXX exec issues ISFLOG,
THE workbench SHALL provide access to the system log from REXX.

Criteria:
- 7.1 THE ISFLOG command SHALL support reading log records into REXX variables.
- 7.2 THE ISFLOG command SHALL support filtering by time range, record type, and system name.
- 7.3 THE ISFLOG command SHALL populate special variables: ISFLOGREC (record count), ISFLOGMSG.n (message text), ISFLOGTIME.n (timestamp).


## Section 16: SDSF Session Persistence

Source: ikja100 (SDSF User Guide) Chapter 1.

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

---

## Section 17: Mapping to FileForge Workbench Sub-Projects

This section maps each requirement area to the relevant FileForge Workbench sub-project specification.

| Requirement Area | Primary Sub-Project | Secondary Sub-Projects |
|---|---|---|
| TSO Session and Logon (Sec 1) | startup-and-session | menu-and-statusbar |
| ISPF Panel Navigation (Sec 2) | menu-and-statusbar | navigation-commands, function-keys-and-history |
| SDSF Panel Framework (Sec 3) | FFW-JES | menu-and-statusbar, layout-and-docking |
| SDSF Job Queue Panels (Sec 4) | FFW-JES | dataset-catalog, virtual-file-system |
| SDSF Filter/Sort/Arrange (Sec 5) | FFW-JES | record-selection-criteria, exclude-show-filter |
| SDSF Search and Scroll (Sec 6) | find-and-replace | navigation-commands, FFW-JES |
| SDSF Log Panels (Sec 7) | FFW-JES | logging-subsystem |
| SDSF System Info Panels (Sec 8) | FFW-JES | menu-and-statusbar |
| SDSF SET Commands (Sec 9) | FFW-JES | configuration-system, function-keys-and-history |
| SDSF Browse and Print (Sec 10) | FFW-JES | custom-file-viewers, file-operations |
| TSO/E Commands (Sec 11) | command-semantics | dataset-catalog, dataset-allocator, FFW-JES |
| TSO/E EDIT Command (Sec 12) | edit-operations | line-commands, find-and-replace |
| REXX Scripting (Sec 13) | lua-macro-engine | command-framework |
| SDSF JES/WLM Panels (Sec 14) | FFW-JES | workflow-engine |
| SDSF REXX Interface (Sec 15) | FFW-JES | lua-macro-engine |
| Session Persistence (Sec 16) | startup-and-session | configuration-system |

---

## Appendix: Priority Classification

### P1 - Core Emulation (Must Have for Mainframe Workstation Experience)
- TSO-1 (Session Startup), TSO-3 (PF Keys), TSO-4 (Scrolling)
- ISPF-1 (Panel Types), ISPF-2 (Panel Hierarchy), ISPF-4 (LOCATE), ISPF-5 (RETRIEVE)
- SDSF-1 (Panel Layout), SDSF-2 (Action Characters), SDSF-4 (Main Panel)
- SDSF-JQ-1 through SDSF-JQ-5 (All Job Queue Panels)
- SDSF-FILTER-1 through SDSF-FILTER-5 (PREFIX, OWNER, DEST, FILTER, SORT)
- SDSF-SCROLL-1 (FIND), SDSF-SCROLL-2 (LOCATE)
- SDSF-LOG-1 (System Log), SDSF-LOG-2 (User Log)
- SDSF-SET-1 (SET ACTION), SDSF-SET-8 (SET MAIN), SDSF-SET-9 (SET ROWNUM), SDSF-SET-12 (WHO)
- SDSF-BROWSE-1 (Browse Job Output)
- TSO-CMD-1 through TSO-CMD-9 (Core TSO Commands)
- TSO-EDIT-1 through TSO-EDIT-3 (EDIT Command)
- SDSF-PERSIST-1 (Session Persistence)

### P2 - Enhanced Emulation (Should Have)
- ISPF-3 (Split Screen)
- SDSF-3 (Overtype Fields), SDSF-5 (Help System)
- SDSF-FILTER-6 (ARRANGE), SDSF-FILTER-7 (SET DISPLAY)
- SDSF-SCROLL-3 through SDSF-SCROLL-5 (LOG, NEXT/PREV, SNAPSHOT)
- SDSF-LOG-3 (SR Panel), SDSF-LOG-4 (ELOG Panel)
- SDSF-SYS-1 through SDSF-SYS-5 (System Info Panels)
- SDSF-SET-2 through SDSF-SET-11 (Remaining SET Commands)
- SDSF-BROWSE-2 through SDSF-BROWSE-4 (Browse Settings, Print, Show Columns)
- TSO-CMD-10 through TSO-CMD-14 (OUTPUT, CANCEL, SEND, PROFILE, PRINTDS)
- REXX-1 through REXX-5 (REXX Scripting)

### P3 - Advanced Emulation (Nice to Have)
- SDSF-JES-1 through SDSF-JES-4 (JES Resource and WLM Panels)
- SDSF-REXX-1 through SDSF-REXX-7 (SDSF REXX Interface)
- SDSF-PERSIST-2 (Special DDNames)

---

*End of TSO and SDSF EARS Requirements for FileForge Workbench.*
