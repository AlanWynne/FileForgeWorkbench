# TSO/E Command Emulation and EDIT Command -- EARS Requirements

Source documents: ikjb700 (TSO/E Command Reference).

Priority: P1 (TSO-CMD-1 through TSO-CMD-9, TSO-EDIT-1 through TSO-EDIT-3) /
          P2 (TSO-CMD-10 through TSO-CMD-14).
Sub-project mapping: command-semantics (primary), dataset-catalog, dataset-allocator,
                     FFW-JES, edit-operations, line-commands, find-and-replace (secondary).

---

## Section A: TSO/E Command Emulation

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

---

## Section B: TSO/E EDIT Command

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
