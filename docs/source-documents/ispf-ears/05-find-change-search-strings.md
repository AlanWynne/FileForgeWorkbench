# ISPF EARS Requirements -- Find, Change, and Search Strings

Source: z/OS ISPF Edit and Edit Macros (SC19-3621-30), Chapter 3.

## Introduction

These requirements describe how ISPF specifies and processes search strings for
the FIND, SEEK, CHANGE, and EXCLUDE commands, including string types, qualifiers,
column ranges, and label ranges.

## Glossary

| Term | Meaning |
|------|---------|
| Simple string | A string with no embedded blanks, commas, or asterisks, not quoted |
| Delimited string | A string enclosed in single or double quotes |
| Hexadecimal string | A delimited string preceded or followed by X |
| Character string | A delimited string preceded or followed by C |
| Picture string | A search pattern using special placeholder characters |
| Regular expression | A pattern using regex syntax, preceded or followed by R |
| Text string | A string preceded or followed by T, ignoring multiple blanks |
| Word | A string bounded by blanks or line boundaries |

## Requirements

### Requirement 1 -- String Types

1.1 WHEN the user specifies a simple string (no quotes, no embedded blanks) THE
    editor SHALL search for that exact sequence of characters.

1.2 WHEN the user specifies a delimited string enclosed in single or double
    quotes THE editor SHALL search for the characters between the delimiters,
    including any embedded blanks.

1.3 WHEN the user specifies a hexadecimal string (X'...') THE editor SHALL
    search for the byte sequence represented by the hex digits.

1.4 WHEN the user specifies a character string (C'...') THE editor SHALL search
    for the exact character sequence, treating the string as case-sensitive.

1.5 WHEN the user specifies a picture string (P'...') THE editor SHALL search
    using the picture pattern where = matches any character, @ matches any
    alphabetic, # matches any numeric, and $ matches any special character.

1.6 WHEN the user specifies a regular expression string (R'...') THE editor SHALL
    search using regular expression matching rules.

1.7 WHEN the user specifies a text string (T'...') THE editor SHALL search for
    the words in the string treating multiple consecutive blanks as equivalent
    to a single blank.

1.8 WHEN the user specifies a single asterisk as string2 in a CHANGE command THE
    editor SHALL reuse the previous string2 value.

### Requirement 2 -- Case Sensitivity

2.1 WHEN the user specifies a simple string and CAPS mode is off THE editor SHALL
    perform a case-insensitive search.

2.2 WHEN the user specifies a character string (C'...') THE editor SHALL perform
    a case-sensitive search regardless of CAPS mode.

2.3 WHEN CAPS mode is on THE editor SHALL convert the search string to uppercase
    before searching.

### Requirement 3 -- Search Qualifiers

3.1 WHEN the user specifies the PREFIX qualifier THE editor SHALL match only
    occurrences where the string begins at the start of a word.

3.2 WHEN the user specifies the SUFFIX qualifier THE editor SHALL match only
    occurrences where the string ends at the end of a word.

3.3 WHEN the user specifies the WORD qualifier THE editor SHALL match only
    occurrences where the string constitutes a complete word bounded by blanks
    or line boundaries.

3.4 WHEN the user specifies FIRST THE editor SHALL find only the first occurrence
    in the data set.

3.5 WHEN the user specifies LAST THE editor SHALL find only the last occurrence
    in the data set.

3.6 WHEN the user specifies NEXT THE editor SHALL find the next occurrence after
    the current cursor position (default behaviour).

3.7 WHEN the user specifies PREV THE editor SHALL find the previous occurrence
    before the current cursor position.

3.8 WHEN the user specifies ALL THE editor SHALL find all occurrences and display
    a count.

### Requirement 4 -- Column Range

4.1 WHEN the user specifies a start column THE editor SHALL restrict the search
    to characters at or after that column.

4.2 WHEN the user specifies both a left and right column THE editor SHALL
    restrict the search to characters within that column range.

4.3 WHEN no column range is specified THE editor SHALL search within the current
    boundary settings.

### Requirement 5 -- Label Range

5.1 WHEN the user specifies .label1 .label2 operands THE editor SHALL restrict
    the search to lines between and including the two labelled lines.

5.2 WHEN the user specifies .ZFIRST THE editor SHALL treat it as the first line
    of data.

5.3 WHEN the user specifies .ZLAST THE editor SHALL treat it as the last line
    of data.

### Requirement 6 -- Excluded Line Filtering

6.1 WHEN the user specifies the X qualifier THE editor SHALL search only lines
    that are currently excluded from the display.

6.2 WHEN the user specifies the NX qualifier THE editor SHALL search only lines
    that are currently visible (not excluded).

6.3 WHEN FIND locates a string in an excluded line THE editor SHALL redisplay
    that line and remove it from the excluded group.

6.4 WHEN CHANGE modifies a string in an excluded line THE editor SHALL redisplay
    that line and mark it with a ==CHG> flag.

### Requirement 7 -- CHANGE Behaviour

7.1 WHEN CHANGE replaces string1 with a longer string2 THE editor SHALL shift
    characters to the right to accommodate the longer replacement.

7.2 WHEN CHANGE replaces string1 with a shorter string2 THE editor SHALL shift
    characters to the left and fill with blanks at the right.

7.3 WHEN a CHANGE would cause data to extend beyond the right boundary THE editor
    SHALL mark the line with an ==ERR> flag and not make the change.

7.4 WHEN the user issues CHANGE ALL THE editor SHALL display a count of the
    number of changes made.

7.5 WHEN the user issues CHANGE and no occurrence is found THE editor SHALL
    display a NOT FOUND message.

### Requirement 8 -- FIND Cursor Positioning

8.1 WHEN FIND locates a string THE editor SHALL move the cursor to the first
    character of the found string.

8.2 WHEN FIND locates a string that is not currently visible THE editor SHALL
    scroll the display to bring the string into view.

8.3 WHEN FIND reaches the end of the data without finding the string THE editor
    SHALL display a NOT FOUND message.

8.4 WHEN FIND ALL is issued THE editor SHALL display the count of occurrences
    found in the short message area.
