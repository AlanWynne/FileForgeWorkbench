# SDSF Log Panels and System Information Panels -- EARS Requirements

Source documents: ikja100 (SDSF User Guide) Chapters 1 and 2.

Priority: P1 (SDSF-LOG-1, SDSF-LOG-2) / P2 (SDSF-LOG-3, SDSF-LOG-4, SDSF-SYS-1 through SDSF-SYS-5).
Sub-project mapping: FFW-JES (primary), logging-subsystem, menu-and-statusbar (secondary).

---

## Section A: Log Panels

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

---

## Section B: System Information Panels

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
- 2.1 THE DASH panel SHALL display system attributes: sysplex name, system name, z/OS level, system clone, SMFID, JES name, JES node, JES member, LPAR name, IPL volume, IPL date/time, IEASYMS, CVTVERID, hardware name, CPC node, user ID.
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
