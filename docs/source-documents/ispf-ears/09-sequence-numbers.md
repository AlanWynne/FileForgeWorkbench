# ISPF EARS Requirements -- Sequence Numbers

Source: z/OS ISPF Edit and Edit Macros (SC19-3621-30), Chapter 2.

## Introduction

These requirements describe how ISPF generates, displays, and manages sequence
numbers in data sets, including standard sequence fields, COBOL sequence fields,
and the relationship between sequence numbers and modification levels.

## Glossary

| Term | Meaning |
|------|---------|
| Standard sequence field | Last 8 characters of a fixed-length record, or first 8 of variable |
| COBOL sequence field | First 6 characters of a fixed-length record |
| Modification level | Last 2 digits of a standard sequence number when STATS is on |
| NUMBER ON STD | Generate numbers in the standard sequence field |
| NUMBER ON COBOL | Generate numbers in the COBOL sequence field |
| AUTONUM | Automatically renumber on save |
| RENUM | Renumber all lines preserving modification levels |
| UNNUMBER | Remove all sequence numbers |

## Requirements

### Requirement 1 -- Number Mode Initialisation

1.1 WHEN the editor loads data and all lines contain numeric characters in
    ascending order in the standard sequence field THE editor SHALL turn number
    mode on automatically.

1.2 WHEN the data set type is COBOL and all lines contain numeric characters in
    ascending order in the COBOL sequence field THE editor SHALL also examine
    the COBOL field when determining number mode.

1.3 WHEN the editor detects no valid sequence numbers THE editor SHALL turn
    number mode off.

1.4 WHEN the first setting of number mode differs from the profile setting THE
    editor SHALL display a message indicating the mode change.

1.5 WHEN editing a new member or empty sequential data set THE editor SHALL
    determine the initial number mode from the current edit profile.

1.6 WHEN no edit profile exists for the data set type THE editor SHALL default
    to NUMBER ON for standard sequence fields and NUMBER ON COBOL for COBOL
    data set types.

### Requirement 2 -- Standard Sequence Numbers

2.1 WHEN the user issues NUMBER ON or NUMBER ON STD THE editor SHALL generate
    sequence numbers in the last 8 characters of each fixed-length record.

2.2 WHEN the user issues NUMBER ON STD for variable-length records THE editor
    SHALL generate sequence numbers in the first 8 characters of each record.

2.3 WHEN STATS mode is on and standard numbers are generated THE editor SHALL
    format sequence numbers as 6 digits followed by a 2-digit modification level.

2.4 WHEN STATS mode is off or the data is a sequential data set THE editor SHALL
    format standard sequence numbers as 8 digits right-justified.

2.5 WHEN lines are inserted between existing numbered lines THE editor SHALL use
    the tens or units positions of the sequence number to maintain order.

2.6 WHEN the available sub-positions are exhausted THE editor SHALL automatically
    renumber one or more succeeding lines to maintain ascending order.

2.7 WHEN sequence numbers start THE editor SHALL begin at 100 and increment by
    100 for each subsequent line.

### Requirement 3 -- COBOL Sequence Numbers

3.1 WHEN the user issues NUMBER ON COBOL THE editor SHALL generate 6-digit
    sequence numbers in the first 6 characters of each fixed-length record.

3.2 WHEN the user issues NUMBER ON STD COBOL THE editor SHALL generate both
    standard and COBOL sequence numbers simultaneously.

3.3 WHEN COBOL sequence numbers are generated THE editor SHALL always use 6
    digits regardless of the STATS mode setting.

3.4 WHEN number mode is off and the user issues NUMBER ON COBOL THE editor SHALL
    warn the user that data in the first 6 columns will be replaced if those
    columns are not blank.

### Requirement 4 -- Sequence Number Display

4.1 WHEN number mode is on THE editor SHALL display the sequence number in the
    line command field to the left of each line.

4.2 WHEN number mode is on THE editor SHALL automatically scroll left or right
    to avoid showing the data columns that contain the sequence numbers.

4.3 WHEN the user issues NUMBER ON DISPLAY or RENUM DISPLAY THE editor SHALL
    keep the sequence number columns visible in the data window.

4.4 WHEN sequence numbers are displayed in the data window THE editor SHALL
    make them visible but not editable.

### Requirement 5 -- Renumbering

5.1 WHEN the user issues RENUM THE editor SHALL renumber all lines starting at
    100 and incrementing by 100, preserving the modification level in the last
    two digits of each sequence number.

5.2 WHEN the user issues AUTONUM ON THE editor SHALL renumber all lines
    automatically whenever the data is saved, preserving modification levels.

5.3 WHEN the user issues UNNUMBER THE editor SHALL turn off number mode and
    blank the sequence number fields on all lines, deleting all modification
    level records.

### Requirement 6 -- Interaction with Bounds and Scrolling

6.1 WHEN number mode is on and the bounds are at their default values THE editor
    SHALL set the left bound to exclude the sequence number columns from the
    editable area.

6.2 WHEN the user scrolls left to the boundary THE editor SHALL stop at the
    bound; a subsequent left scroll SHALL reveal the sequence number columns.

6.3 WHEN the user changes number mode to off THE editor SHALL scroll the display
    so that column 1 is the first column displayed.

6.4 WHEN the user changes number mode back to on THE editor SHALL scroll the
    display back to the first non-sequence column.
