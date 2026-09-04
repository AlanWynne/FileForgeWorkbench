# ISPF EARS Requirements -- Primary Commands

Source: z/OS ISPF Edit and Edit Macros (SC19-3621-30), Chapter 10.

## Introduction

These requirements describe the ISPF editor primary commands: commands entered
on the Command ===> line that affect the entire data set or control the editing
environment.

## Glossary

| Term | Meaning |
|------|---------|
| Primary command | A command entered on the Command ===> line |
| Command line | The input field labelled "Command ===>" at the bottom of the edit panel |
| Ampersand prefix | Typing & before a command keeps it on the command line after execution |

## Requirements

### Requirement 1 -- Command Entry

1.1 WHEN the user types a command on the Command ===> line and presses Enter THE
    editor SHALL process that command before processing any line commands.

1.2 WHEN the user prefixes a primary command with & THE editor SHALL leave the
    command text on the command line after execution, allowing easy repetition.

1.3 WHEN the user issues the RETRIEVE command THE editor SHALL recall the
    previously entered command to the command line.

1.4 WHEN a primary command is too long for the command field THE editor SHALL
    support the ZEXPAND command to open a 255-character popup input field.

### Requirement 2 -- Find, Seek, Change, Exclude

2.1 WHEN the user issues FIND string THE editor SHALL locate the next occurrence
    of the string, scroll to it, move the cursor to it, and redisplay any
    excluded lines that contain it.

2.2 WHEN the user issues FIND string ALL THE editor SHALL locate all occurrences,
    display a count, and redisplay all excluded lines containing the string.

2.3 WHEN the user issues FIND with FIRST, LAST, NEXT, or PREV THE editor SHALL
    locate the first, last, next, or previous occurrence respectively.

2.4 WHEN the user issues FIND with PREFIX, SUFFIX, or WORD THE editor SHALL
    restrict matches to strings at the start, end, or as a whole word.

2.5 WHEN the user issues FIND with column range operands THE editor SHALL
    restrict the search to the specified column range.

2.6 WHEN the user issues FIND with X THE editor SHALL search only excluded lines.

2.7 WHEN the user issues FIND with NX THE editor SHALL search only non-excluded
    lines.

2.8 WHEN the user issues RFIND THE editor SHALL repeat the most recent FIND
    operation in the same direction.

2.9 WHEN the user issues SEEK string THE editor SHALL position the cursor to the
    next occurrence without changing the exclude status of any lines.

2.10 WHEN the user issues CHANGE string1 string2 THE editor SHALL replace the
     next occurrence of string1 with string2 and move the cursor to the change.

2.11 WHEN the user issues CHANGE string1 string2 ALL THE editor SHALL replace all
     occurrences of string1 with string2 and display a count of changes made.

2.12 WHEN the user issues RCHANGE THE editor SHALL repeat the most recent CHANGE
     operation.

2.13 WHEN the user issues EXCLUDE string THE editor SHALL hide all lines
     containing the string from the display.

2.14 WHEN the user issues EXCLUDE ALL THE editor SHALL hide all currently
     displayed lines.

2.15 WHEN the user issues FLIP THE editor SHALL reverse the exclude status of
     every line: excluded lines become visible and visible lines become excluded.

2.16 WHEN the user issues RESET THE editor SHALL redisplay all excluded lines and
     remove all ==CHG>, ==ERR>, and other temporary flags.

### Requirement 3 -- Locate and Navigate

3.1 WHEN the user issues LOCATE n THE editor SHALL scroll to line number n.

3.2 WHEN the user issues LOCATE .label THE editor SHALL scroll to the line
    carrying that label.

3.3 WHEN the user issues LOCATE FIRST THE editor SHALL scroll to the first line
    of data.

3.4 WHEN the user issues LOCATE LAST THE editor SHALL scroll to the last line
    of data.

3.5 WHEN the user issues LOCATE CHANGE THE editor SHALL scroll to the next
    ==CHG> flagged line.

3.6 WHEN the user issues LOCATE ERROR THE editor SHALL scroll to the next
    ==ERR> flagged line.

### Requirement 4 -- Copy, Move, Create, Replace

4.1 WHEN the user issues COPY dsname AFTER label THE editor SHALL insert the
    contents of the specified data set after the labelled line.

4.2 WHEN the user issues COPY dsname BEFORE label THE editor SHALL insert the
    contents of the specified data set before the labelled line.

4.3 WHEN the user issues MOVE dsname AFTER label THE editor SHALL insert the
    contents of the specified data set after the labelled line and delete the
    source data set.

4.4 WHEN the user issues CREATE dsname THE editor SHALL write the lines marked
    with C or M line commands to a new data set or member with the specified name.

4.5 WHEN the user issues REPLACE dsname THE editor SHALL overwrite the specified
    data set or member with the lines marked with C or M line commands.

4.6 WHEN the user issues CREATE or REPLACE and the target has inconsistent
    attributes THE editor SHALL display a confirmation panel warning of possible
    truncation before proceeding.

### Requirement 5 -- Sort

5.1 WHEN the user issues SORT THE editor SHALL sort all non-excluded lines in
    ascending order by the first field within the current bounds.

5.2 WHEN the user issues SORT with column range and A or D operands THE editor
    SHALL sort by the specified columns in ascending or descending order.

5.3 WHEN the user issues SORT with label range operands THE editor SHALL sort
    only the lines within the specified label range.

### Requirement 6 -- Sequence Numbers

6.1 WHEN the user issues NUMBER ON THE editor SHALL generate standard sequence
    numbers in the last 8 characters of each fixed-length line.

6.2 WHEN the user issues NUMBER ON COBOL THE editor SHALL generate COBOL sequence
    numbers in the first 6 characters of each fixed-length line.

6.3 WHEN the user issues NUMBER ON STD COBOL THE editor SHALL generate both
    standard and COBOL sequence numbers.

6.4 WHEN the user issues NUMBER OFF THE editor SHALL turn off number mode without
    removing existing sequence numbers from the data.

6.5 WHEN the user issues RENUM THE editor SHALL renumber all lines preserving the
    modification level in the last two digits of each sequence number.

6.6 WHEN the user issues UNNUMBER THE editor SHALL turn off number mode and blank
    the sequence number fields on all lines.

6.7 WHEN the user issues AUTONUM ON THE editor SHALL renumber lines automatically
    whenever the data is saved.

### Requirement 7 -- Bounds

7.1 WHEN the user issues BOUNDS left right THE editor SHALL set the left and
    right column boundaries used by FIND, CHANGE, EXCLUDE, SORT, and shift
    commands.

7.2 WHEN the user issues BOUNDS with no operands THE editor SHALL reset the
    boundaries to the default values for the current data set type and number mode.

7.3 WHEN number mode is turned on or off and the bounds are at their default
    values THE editor SHALL automatically adjust the bounds to the new defaults.

7.4 WHEN the user specifies a right boundary greater than the logical record
    length THE editor SHALL reset that boundary to the default value.

### Requirement 8 -- Tabs

8.1 WHEN the user issues TABS ON THE editor SHALL enable tab processing using
    the positions defined in the =TABS> line.

8.2 WHEN the user issues TABS OFF THE editor SHALL disable tab processing.

8.3 WHEN the user issues TABS with column positions THE editor SHALL define
    software tab stops at those columns.

8.4 WHEN the user presses the hardware tab key and software tabs are defined THE
    editor SHALL advance the cursor to the next defined tab stop.

### Requirement 9 -- Save and Recovery

9.1 WHEN the user issues SAVE THE editor SHALL write the current data to the
    data set without ending the edit session.

9.2 WHEN the user issues RECOVERY ON THE editor SHALL begin writing data to a
    temporary backup file to enable recovery after a system failure.

9.3 WHEN the user issues RECOVERY OFF THE editor SHALL stop writing to the
    backup file.

9.4 WHEN the user issues SETUNDO STORAGE THE editor SHALL use virtual storage
    for undo history.

9.5 WHEN the user issues SETUNDO RECOVERY THE editor SHALL use the recovery data
    set for undo history.

9.6 WHEN the user issues UNDO THE editor SHALL reverse the most recent edit
    interaction, restoring the data and cursor to their previous state.

### Requirement 10 -- Miscellaneous Primary Commands

10.1 WHEN the user issues COLS THE editor SHALL display a temporary column
     identification line at the top of the data area.

10.2 WHEN the user issues HEX ON THE editor SHALL display all data in
     hexadecimal format with two rows of hex digits below each character row.

10.3 WHEN the user issues HEX OFF THE editor SHALL return to normal character
     display.

10.4 WHEN the user issues HILITE language THE editor SHALL apply language-
     sensitive syntax colouring for the specified language.

10.5 WHEN the user issues HILITE AUTO THE editor SHALL detect the language from
     the first non-blank line and apply appropriate colouring.

10.6 WHEN the user issues HILITE OFF THE editor SHALL disable all syntax
     colouring.

10.7 WHEN the user issues SUBMIT THE editor SHALL submit the current data as a
     batch job.

10.8 WHEN the user issues PROFILE name THE editor SHALL switch to the named
     profile immediately.

10.9 WHEN the user issues HIDE THE editor SHALL suppress the display of the
     excluded-lines count message between groups of excluded lines.

10.10 WHEN the user issues NULLS ON THE editor SHALL write trailing spaces as
      null characters to allow insertion without overtyping.

10.11 WHEN the user issues PRESERVE THE editor SHALL store the original length
      of each variable-length record and use it as the minimum length on save.
