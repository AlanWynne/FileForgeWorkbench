# ISPF EARS Requirements -- Edit Recovery and Undo

Source: z/OS ISPF Edit and Edit Macros (SC19-3621-30), Chapter 2 and Chapter 10.

## Introduction

These requirements describe the ISPF edit recovery system (crash recovery) and
the UNDO command (interactive undo of edit interactions).

## Glossary

| Term | Meaning |
|------|---------|
| Edit recovery | Automatic backup to a temporary file enabling restart after a crash |
| UNDO | Interactive reversal of the most recent edit interaction |
| SETUNDO STORAGE | Undo history kept in virtual storage |
| SETUNDO RECOVERY | Undo history kept in the recovery data set |
| Recovery macro | A macro run automatically when a recovered session is resumed |

## Requirements

### Requirement 1 -- Edit Recovery (Crash Recovery)

1.1 WHEN the user issues RECOVERY ON THE editor SHALL begin writing all data
    changes and temporary changes (labels, excludes) to a temporary backup file.

1.2 WHEN RECOVERY mode is on and a system failure occurs THE editor SHALL
    preserve the backup file so the session can be recovered.

1.3 WHEN the user begins a new edit session and recovery data exists for a
    previous session THE editor SHALL display the Edit Recovery panel before
    opening the new session.

1.4 WHEN the user chooses to continue recovery from the Edit Recovery panel THE
    editor SHALL restore the data set to the state it was in at the time of
    failure.

1.5 WHEN the user chooses to defer recovery THE editor SHALL skip the current
    recovery entry and display the next one if more exist.

1.6 WHEN the user chooses to cancel recovery THE editor SHALL discard the
    recovery data for that session.

1.7 WHEN recovery is continued and a recovery macro was previously specified THE
    editor SHALL run that recovery macro after restoring the data.

1.8 WHEN the user issues RECOVERY OFF THE editor SHALL stop writing to the backup
    file and disable crash recovery for the current session.

1.9 WHEN RECOVERY mode is on THE editor SHALL record both data changes and
    temporary changes (exclude status, labels) to support full UNDO capability.

1.10 WHEN edit recovery is not supported for a member generation other than the
     current generation of a PDSE version 2 data set THE editor SHALL not offer
     recovery for that member.

### Requirement 2 -- UNDO Command

2.1 WHEN the user issues UNDO THE editor SHALL reverse the most recent edit
    interaction, restoring the data and cursor to their state before that
    interaction.

2.2 WHEN SETUNDO mode is STORAGE THE editor SHALL maintain undo history in
    virtual storage.

2.3 WHEN SETUNDO mode is RECOVERY THE editor SHALL maintain undo history in the
    recovery data set, enabling undo even after a session restart.

2.4 WHEN RECOVERY mode is off and SETUNDO is STORAGE THE editor SHALL display a
    warning message that UNDO is not available until RECOVERY ON is issued.

2.5 WHEN the user issues UNDO after a column shift that destroyed data THE editor
    SHALL restore the data to its pre-shift state if UNDO mode was on before the
    shift.

2.6 WHEN no further undo history is available THE editor SHALL display a message
    indicating that no more undo steps are available.

### Requirement 3 -- Recovery Macro

3.1 WHEN the user issues RMACRO macroname THE editor SHALL store that macro name
    in the edit profile as the recovery macro.

3.2 WHEN a recovered session is resumed and a recovery macro is defined THE
    editor SHALL run the recovery macro after the data is restored.

3.3 WHEN the recovery macro issues END or CANCEL THE editor SHALL not display
    the recovered data.
