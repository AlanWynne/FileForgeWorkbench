# ISPF EARS Requirements -- Edit Session Lifecycle

Source: z/OS ISPF Edit and Edit Macros (SC19-3621-30), Chapter 1-2.

## Introduction

These requirements describe how an ISPF-style editor session begins, operates,
and ends. They cover the entry panel, session startup, data loading, and the
END/CANCEL/RETURN commands that terminate a session.

## Glossary

| Term | Meaning |
|------|---------|
| Edit session | A single interactive editing context for one data set or member |
| Edit profile | A named set of persistent mode settings associated with a data set type |
| Initial macro | A macro run automatically after data is loaded but before first display |
| Autosave | Profile mode that saves data automatically on END |
| CANCEL | End session discarding all changes since last SAVE |
| END | End session, saving if autosave is on |

## Requirements

### Requirement 1 -- Session Entry

1.1 WHEN a user selects the Edit option THE editor SHALL display an entry panel
    that accepts a data set name, member name, profile name, initial macro name,
    and encoding options before opening the edit session.

1.2 WHEN the user specifies a member name on the entry panel THE editor SHALL
    load that member into working storage and display it for editing.

1.3 WHEN the user leaves the member field blank or enters a pattern THE editor
    SHALL display a member selection list for the specified partitioned data set.

1.4 WHEN the user specifies a non-existent member of a partitioned data set THE
    editor SHALL create a new empty member with that name and display an empty
    edit panel.

1.5 WHEN the user specifies an empty sequential data set THE editor SHALL display
    an empty edit panel with quote characters in the line command fields.

1.6 WHEN the user specifies an Initial Macro on the entry panel THE editor SHALL
    run that macro after loading the data but before displaying the first panel,
    overriding any initial macro stored in the edit profile.

1.7 WHEN the user types NONE in the Initial Macro field THE editor SHALL suppress
    any initial macro defined in the edit profile for that session.

### Requirement 2 -- Data Loading

2.1 WHEN an edit session begins THE editor SHALL read the selected member or
    sequential data set into virtual working storage.

2.2 WHEN the data contains characters that cannot be displayed THE editor SHALL
    replace those characters with blanks on the panel without altering the
    underlying data.

2.3 WHEN the data contains ASA printer control characters THE editor SHALL
    display and treat them as editable data.

2.4 WHEN the data contains machine control characters THE editor SHALL replace
    them with blanks on the panel without altering the underlying data.

2.5 WHEN the editor loads a member THE editor SHALL examine the standard sequence
    field and, for COBOL data set types, the COBOL sequence field, to determine
    whether number mode should be on or off.

2.6 WHEN the editor detects valid ascending sequence numbers in the data THE
    editor SHALL turn number mode on automatically.

2.7 WHEN the editor detects no valid sequence numbers THE editor SHALL turn
    number mode off automatically.

2.8 WHEN the editor changes number mode to match the data and the change differs
    from the profile setting THE editor SHALL display a message line informing
    the user of the change.

### Requirement 3 -- Edit Profile Selection

3.1 WHEN the Profile Name field on the entry panel is blank THE editor SHALL use
    the data set type (last qualifier of the data set name) as the profile name.

3.2 WHEN the user specifies a profile name on the entry panel THE editor SHALL
    use that profile, overriding the data set type default.

3.3 WHEN the specified profile does not exist THE editor SHALL create a new
    profile using default mode settings, blank mask and tabs, and default bounds.

3.4 WHEN editing a z/OS UNIX file that has a suffix THE editor SHALL use the
    first 8 characters of the suffix (converted to uppercase) as the profile name.

3.5 WHEN editing a z/OS UNIX file with no suffix THE editor SHALL use the profile
    name HFSPROF.

### Requirement 4 -- Ending a Session with END

4.1 WHEN the user issues the END command and autosave mode is on and data has
    changed THE editor SHALL save the data to the first library in the
    concatenation sequence.

4.2 WHEN the user issues the END command and both number mode and autonum mode
    are on THE editor SHALL renumber the data before saving.

4.3 WHEN the user issues the END command and STATS mode is on and the data is a
    PDS member THE editor SHALL update or create ISPF statistics for that member.

4.4 WHEN the user issues the END command and autolist mode is on THE editor SHALL
    record a source listing in the ISPF list data set.

4.5 WHEN the user issues the END command and autosave mode is off with PROMPT THE
    editor SHALL display a prompting message allowing the user to SAVE or CANCEL.

4.6 WHEN the user issues the END command and autosave mode is off with NOPROMPT
    THE editor SHALL discard all changes without prompting, equivalent to CANCEL.

4.7 WHEN the END command is issued from within a macro THE editor SHALL not
    complete the session until the macro terminates all processing.

4.8 WHEN saving variable-length data THE editor SHALL truncate trailing blanks
    from each line unless the PRESERVE option is active.

4.9 WHEN saving a completely blank variable-length line with no line number THE
    editor SHALL add a single blank so the record length is not zero.

### Requirement 5 -- Ending a Session with CANCEL

5.1 WHEN the user issues the CANCEL command THE editor SHALL discard all changes
    made since the beginning of the session or the last SAVE command.

5.2 WHEN the Confirm Cancel option is selected and data changes have been made
    THE editor SHALL display a confirmation panel before discarding changes.

5.3 WHEN the user issues CANCEL with no data changes THE editor SHALL end the
    session immediately without displaying a confirmation panel.

### Requirement 6 -- RETURN Command

6.1 WHEN the user issues the RETURN command THE editor SHALL perform the same
    end-of-session actions as END and return to the ISPF Primary Option Menu.
