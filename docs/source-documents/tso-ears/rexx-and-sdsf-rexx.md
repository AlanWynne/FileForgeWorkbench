# REXX Scripting, JES/WLM Panels, and SDSF REXX Interface -- EARS Requirements

Source documents: ikjc200 (TSO/E REXX User Guide), ikja100 (SDSF User Guide) Chapters 2 and 6.

Priority: P2 (REXX-1 through REXX-5, SDSF-JES-1 through SDSF-JES-4) /
          P3 (SDSF-REXX-1 through SDSF-REXX-7).
Sub-project mapping: lua-macro-engine (primary), command-framework, FFW-JES, workflow-engine (secondary).

---

## Section A: REXX Scripting Emulation

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

---

## Section B: SDSF JES Resource and WLM Panels

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

## Section C: SDSF REXX Interface

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
