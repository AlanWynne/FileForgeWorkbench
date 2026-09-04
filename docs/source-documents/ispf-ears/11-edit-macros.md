# ISPF EARS Requirements -- Edit Macros

Source: z/OS ISPF Edit and Edit Macros (SC19-3621-30), Chapters 5-7.

## Introduction

These requirements describe the ISPF edit macro system: how macros are invoked,
what commands they can use, how they interact with the editor, and how initial
macros and line command macros work.

## Glossary

| Term | Meaning |
|------|---------|
| Edit macro | A CLIST, REXX exec, or program that runs as an editor primary or line command |
| Initial macro | A macro run automatically after data is loaded but before first display |
| Application-wide macro | A macro run for all edit sessions via the ZUSERMAC variable |
| Site-wide macro | A macro run for all users, specified in the ISPF configuration table |
| Line command macro | A macro associated with a user-defined line command via a table |
| ISREDIT | The prefix used to invoke edit macro commands from CLIST or REXX |
| Macro level | The nesting depth of macro invocations |

## Requirements

### Requirement 1 -- Invoking Macros

1.1 WHEN the user types a macro name on the command line and presses Enter THE
    editor SHALL invoke that macro as a primary command macro.

1.2 WHEN the user types a user-defined line command in the line command field THE
    editor SHALL invoke the associated macro from the line command table.

1.3 WHEN an initial macro is defined in the edit profile THE editor SHALL run
    that macro after loading the data but before displaying the first panel.

1.4 WHEN the ZUSERMAC variable is set in the shared or profile pool THE editor
    SHALL run the named macro after the site-wide macro and before the initial
    macro specified on the entry panel or in the profile.

1.5 WHEN a site-wide macro is configured in the ISPF configuration table THE
    editor SHALL run it before any user-specified initial macros.

1.6 WHEN an initial macro issues END or CANCEL THE editor SHALL not display the
    data.

### Requirement 2 -- Macro Commands

2.1 WHEN a macro issues ISREDIT FIND string THE editor SHALL execute the FIND
    command as if entered on the command line.

2.2 WHEN a macro issues ISREDIT CHANGE string1 string2 THE editor SHALL execute
    the CHANGE command.

2.3 WHEN a macro issues ISREDIT LINE n THE editor SHALL return the content of
    line n in the specified variable.

2.4 WHEN a macro issues ISREDIT LINE n = value THE editor SHALL replace the
    content of line n with the specified value.

2.5 WHEN a macro issues ISREDIT LINE_AFTER n = value THE editor SHALL insert a
    new line after line n with the specified content.

2.6 WHEN a macro issues ISREDIT LINE_BEFORE n = value THE editor SHALL insert a
    new line before line n with the specified content.

2.7 WHEN a macro issues ISREDIT DELETE n THE editor SHALL delete line n.

2.8 WHEN a macro issues ISREDIT CURSOR THE editor SHALL return the current cursor
    position (line number and column) in the specified variables.

2.9 WHEN a macro issues ISREDIT CURSOR = line col THE editor SHALL move the
    cursor to the specified line and column.

2.10 WHEN a macro issues ISREDIT LABEL n = .label THE editor SHALL assign the
     specified label to line n.

2.11 WHEN a macro issues ISREDIT LINENUM .label THE editor SHALL return the line
     number of the labelled line.

2.12 WHEN a macro issues ISREDIT DISPLAY_LINES THE editor SHALL return the number
     of lines currently visible in the scrollable area.

2.13 WHEN a macro issues ISREDIT DISPLAY_COLS THE editor SHALL return the first
     and last column numbers currently visible.

2.14 WHEN a macro issues ISREDIT UP n THE editor SHALL scroll the display up by
     n lines.

2.15 WHEN a macro issues ISREDIT DOWN n THE editor SHALL scroll the display down
     by n lines.

2.16 WHEN a macro issues ISREDIT LEFT n THE editor SHALL scroll the display left
     by n columns.

2.17 WHEN a macro issues ISREDIT RIGHT n THE editor SHALL scroll the display right
     by n columns.

### Requirement 3 -- Macro Query Commands

3.1 WHEN a macro issues ISREDIT DATASET THE editor SHALL return the current and
    original data set names.

3.2 WHEN a macro issues ISREDIT MEMBER THE editor SHALL return the current member
    name.

3.3 WHEN a macro issues ISREDIT LRECL THE editor SHALL return the logical record
    length.

3.4 WHEN a macro issues ISREDIT RECFM THE editor SHALL return the record format.

3.5 WHEN a macro issues ISREDIT DATA_CHANGED THE editor SHALL return YES if the
    data has been changed since the last save, NO otherwise.

3.6 WHEN a macro issues ISREDIT DATA_WIDTH THE editor SHALL return the width of
    the data area.

3.7 WHEN a macro issues ISREDIT FIND_COUNTS THE editor SHALL return the number
    of occurrences found by the most recent FIND command.

3.8 WHEN a macro issues ISREDIT CHANGE_COUNTS THE editor SHALL return the number
    of changes made by the most recent CHANGE command.

3.9 WHEN a macro issues ISREDIT EXCLUDE_COUNTS THE editor SHALL return the number
    of currently excluded lines.

3.10 WHEN a macro issues ISREDIT LINE_STATUS n THE editor SHALL return the source
     and change information for line n.

3.11 WHEN a macro issues ISREDIT XSTATUS n THE editor SHALL return whether line n
     is excluded.

3.12 WHEN a macro issues ISREDIT SESSION THE editor SHALL return whether the
     session is an edit session or a view session.

### Requirement 4 -- Macro Mode Commands

4.1 WHEN a macro issues ISREDIT CAPS ON or OFF THE editor SHALL set caps mode
    accordingly.

4.2 WHEN a macro issues ISREDIT NUMBER ON or OFF THE editor SHALL set number mode
    accordingly.

4.3 WHEN a macro issues ISREDIT RECOVERY ON or OFF THE editor SHALL set recovery
    mode accordingly.

4.4 WHEN a macro issues ISREDIT BOUNDS left right THE editor SHALL set the
    boundary columns accordingly.

4.5 WHEN a macro issues ISREDIT TABS ON or OFF THE editor SHALL set tabs mode
    accordingly.

4.6 WHEN a macro issues ISREDIT PROFILE name THE editor SHALL switch to the
    named profile.

4.7 WHEN a macro issues ISREDIT IMACRO name THE editor SHALL store the named
    macro as the initial macro in the current profile.

### Requirement 5 -- Macro Levels and PROCESS

5.1 WHEN a macro invokes another macro THE editor SHALL increment the macro
    nesting level.

5.2 WHEN a macro issues ISREDIT MACRO_LEVEL THE editor SHALL return the current
    nesting level.

5.3 WHEN a line command macro issues ISREDIT PROCESS THE editor SHALL process
    the pending line commands and return the affected line range to the macro.

5.4 WHEN a macro issues ISREDIT RANGE_CMD THE editor SHALL return the line
    command that was entered to invoke the macro.

### Requirement 6 -- Initial Macro Restrictions

6.1 WHEN an initial macro issues a command that references display values
    (DISPLAY_COLS, DISPLAY_LINES, DOWN, LEFT, RIGHT, UP, LOCATE) THE editor
    SHALL return an error because no data has been displayed yet.

6.2 WHEN an initial macro issues ISREDIT CAPS ON THE editor SHALL set caps mode
    on, overriding the profile setting that was set from the data content.

### Requirement 7 -- Line Command Tables

7.1 WHEN the user specifies a line command table name on the edit entry panel THE
    editor SHALL use that table to resolve user-defined line commands.

7.2 WHEN the user types a command from the line command table in the line command
    field THE editor SHALL invoke the associated macro.

7.3 WHEN a line command table entry specifies multiline format THE editor SHALL
    allow a numeric suffix on the command to indicate the number of lines.

7.4 WHEN a line command table entry specifies block format THE editor SHALL allow
    the command to be entered on two lines to define a block.

7.5 WHEN a line command table entry specifies a destination THE editor SHALL
    require an A or B destination line command to complete the operation.
