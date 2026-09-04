# ISPF EARS Requirements -- Edit Boundaries, Tabs, and Masks

Source: z/OS ISPF Edit and Edit Macros (SC19-3621-30), Chapters 2 and 3.

## Introduction

These requirements describe the ISPF boundary system (=BNDS>), the tab system
(=TABS>), and the mask system (=MASK>), which together control column-sensitive
editing behaviour.

## Glossary

| Term | Meaning |
|------|---------|
| Bounds | Left and right column limits for commands that operate within columns |
| =BNDS> line | A special temporary line showing the current boundary positions |
| =TABS> line | A special temporary line defining software tab stop positions |
| =MASK> line | A special temporary line whose content is inserted into new lines |
| Software tab | A tab stop defined by the user in the =TABS> line |
| Hardware tab | A tab stop defined by attribute bytes in a formatted data set |

## Requirements

### Requirement 1 -- Boundary Defaults

1.1 WHEN editing a fixed-length ASM data set with standard numbers THE editor
    SHALL default the bounds to columns 1 and LRECL-8 (1 and 71 for LRECL=80).

1.2 WHEN editing a fixed-length ASM data set without numbers THE editor SHALL
    default the bounds to columns 1 and LRECL (1 and 71 for LRECL=80).

1.3 WHEN editing a fixed-length COBOL data set without numbers THE editor SHALL
    default the bounds to columns 1 and LRECL (1 and 80 for LRECL=80).

1.4 WHEN editing a fixed-length COBOL data set with standard numbers THE editor
    SHALL default the bounds to columns 1 and LRECL-8 (1 and 72 for LRECL=80).

1.5 WHEN editing a fixed-length COBOL data set with COBOL standard numbers THE
    editor SHALL default the bounds to columns 7 and LRECL-8 (7 and 72 for
    LRECL=80).

1.6 WHEN editing a fixed-length COBOL data set with COBOL numbers (no standard)
    THE editor SHALL default the bounds to columns 7 and LRECL (7 and 80 for
    LRECL=80).

1.7 WHEN editing a fixed-length OTHER data set with standard numbers THE editor
    SHALL default the bounds to columns 1 and LRECL-8 (1 and 72 for LRECL=80).

1.8 WHEN editing a fixed-length OTHER data set without numbers THE editor SHALL
    default the bounds to columns 1 and LRECL (1 and 80 for LRECL=80).

1.9 WHEN editing a variable-length data set with standard numbers THE editor
    SHALL default the bounds to columns 9 and the record length.

1.10 WHEN editing a variable-length data set without numbers THE editor SHALL
     default the bounds to columns 1 and the record length.

### Requirement 2 -- Boundary Behaviour

2.1 WHEN the bounds are at their default values and number mode is turned on or
    off THE editor SHALL automatically adjust the bounds to the new defaults.

2.2 WHEN the user has explicitly changed the bounds from the defaults THE editor
    SHALL NOT automatically adjust them when number mode changes.

2.3 WHEN a left or right scroll would move the display past a boundary THE editor
    SHALL stop scrolling at the boundary; a subsequent scroll request SHALL then
    scroll past it.

2.4 WHEN the user specifies an invalid right boundary value THE editor SHALL
    reset that boundary to the default value.

2.5 WHEN the user specifies an invalid left boundary value THE editor SHALL reset
    that boundary to the default value.

### Requirement 3 -- Commands Affected by Bounds

3.1 WHEN the CHANGE command is issued THE editor SHALL restrict replacements to
    within the current bounds unless overriding column operands are specified.

3.2 WHEN the FIND command is issued THE editor SHALL restrict the search to
    within the current bounds unless overriding column operands are specified.

3.3 WHEN the EXCLUDE command is issued THE editor SHALL restrict the search to
    within the current bounds.

3.4 WHEN the SORT command is issued THE editor SHALL sort within the current
    bounds unless overriding column operands are specified.

3.5 WHEN the LEFT or RIGHT scroll commands are issued THE editor SHALL stop at
    the boundary as described in Requirement 2.3.

3.6 WHEN the column shift line commands ( and ) are issued THE editor SHALL
    shift within the current bounds.

3.7 WHEN the data shift line commands < and > are issued THE editor SHALL shift
    within the current bounds.

3.8 WHEN the TE, TF, and TS text commands are issued THE editor SHALL operate
    within the current bounds.

### Requirement 4 -- Tab Definitions

4.1 WHEN the user issues TABS ON THE editor SHALL enable tab processing using
    the positions defined in the =TABS> line.

4.2 WHEN the user issues TABS OFF THE editor SHALL disable tab processing.

4.3 WHEN the user defines software tab positions in the =TABS> line THE editor
    SHALL advance the cursor to the next defined tab stop when the tab key is
    pressed.

4.4 WHEN the user defines hardware tab positions using attribute bytes in a
    formatted data set THE editor SHALL use those positions for tab navigation.

4.5 WHEN the =TABS> line contains all blanks THE editor SHALL omit it from the
    default profile display.

### Requirement 5 -- Mask Line

5.1 WHEN the user defines content in the =MASK> line and issues the I (insert)
    line command THE editor SHALL pre-fill the new line with the mask content.

5.2 WHEN the =MASK> line contains all blanks THE editor SHALL insert blank lines
    when the I command is used.

5.3 WHEN the =MASK> line contains all blanks THE editor SHALL omit it from the
    default profile display.

5.4 WHEN a format name is in use THE editor SHALL not display the =MASK> line
    in the profile display, because masks are ignored in formatted edit sessions.

### Requirement 6 -- Special Temporary Lines

6.1 WHEN the user ends an edit session THE editor SHALL NOT save =PROF>, =MASK>,
    =TABS>, =BNDS>, =COLS>, ==MSG>, =NOTE=, or ====== lines as part of the data.

6.2 WHEN the user applies the MD (Make Dataline) line command to a =COLS>,
    ==MSG>, =NOTE=, or ====== line THE editor SHALL convert it to a permanent
    data line that is saved with the data set.

6.3 WHEN the user issues the RESET command THE editor SHALL remove all ==CHG>,
    ==ERR>, and other temporary flag lines from the display.
