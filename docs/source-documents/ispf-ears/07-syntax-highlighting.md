# ISPF EARS Requirements -- Syntax Highlighting (HILITE)

Source: z/OS ISPF Edit and Edit Macros (SC19-3621-30), Chapter 2.

## Introduction

These requirements describe the ISPF enhanced and language-sensitive edit
colouring system, including automatic language detection, supported languages,
logic highlighting, and the HILITE command and dialog.

## Glossary

| Term | Meaning |
|------|---------|
| HILITE | The ISPF command and feature for language-sensitive syntax colouring |
| Logic highlighting | Colouring of matched/unmatched block constructs (IF/ELSE, DO/END) |
| Overtype colour | The colour applied to characters as they are typed |
| FIND colour | The colour applied to strings matching the current FIND operation |
| Cursor phrase | The phrase containing the cursor, highlighted in a distinct colour |
| AUTO | Automatic language detection mode |

## Requirements

### Requirement 1 -- Enabling and Disabling Highlighting

1.1 WHEN the user issues HILITE language THE editor SHALL apply language-
    sensitive colouring for the specified language immediately.

1.2 WHEN the user issues HILITE AUTO THE editor SHALL detect the language from
    the first non-blank content in the file and apply appropriate colouring.

1.3 WHEN the user issues HILITE OFF THE editor SHALL disable all syntax
    colouring.

1.4 WHEN the user issues HILITE with no operands THE editor SHALL display the
    HILITE dialog panel.

1.5 WHEN highlighting is enabled THE editor SHALL store the language, coloring
    type (ON/OFF), and logic type in the edit profile.

1.6 WHEN highlighting is not available for the current session THE editor SHALL
    not display the HILITE profile line.

### Requirement 2 -- Automatic Language Detection

2.1 WHEN AUTO mode is active THE editor SHALL scan up to the first 72 bytes of
    each line to determine the language.

2.2 WHEN the first non-blank string is an asterisk in column 1 or a recognised
    assembler opcode THE editor SHALL identify the language as Assembler.

2.3 WHEN the first non-blank character is a period or colon in column 1 THE
    editor SHALL identify the language as BookMaster.

2.4 WHEN the first string is # or // (and the data set type is not .CNTL, .JCL,
    or .ISPCTLx) or /* with data set type .C THE editor SHALL identify the
    language as C.

2.5 WHEN the first non-blank is an asterisk or slash in column 7 THE editor
    SHALL identify the language as COBOL.

2.6 WHEN the first non-blank character is < and the first non-comment tag is
    <!DOCTYPE HTML> or <?HTML> THE editor SHALL identify the language as HTML.

2.7 WHEN the first non-blank character is < and the file is not HTML or XML THE
    editor SHALL identify the language as ISPF DTL.

2.8 WHEN the first string is ) in column 1 followed by a panel section name, or
    % in column 1 THE editor SHALL identify the language as ISPF Panel.

2.9 WHEN the first string is ) in column 1 and the file does not appear to be a
    panel THE editor SHALL identify the language as ISPF Skeleton.

2.10 WHEN the first string is //anything followed by DD, JOB, EXEC, PROC, or
     similar JCL keywords, or //* in column 1 THE editor SHALL identify the
     language as JCL.

2.11 WHEN the first string is (* or /* with data set type .PASCAL THE editor
     SHALL identify the language as Pascal.

2.12 WHEN the first string is % or /* or *PROCESS in column 1 THE editor SHALL
     identify the language as PL/I.

2.13 WHEN the first string is a /* comment containing REXX, or /* with data set
     type .EXEC or .REXX THE editor SHALL identify the language as REXX.

2.14 WHEN the first non-blank character is < and the first non-comment tag is
     <!DOCTYPE XML> or <?XML> THE editor SHALL identify the language as XML.

2.15 WHEN the first word is PROC, CONTROL, ISPEXEC, or ISREDIT THE editor SHALL
     identify the language as Other (CLIST-like).

### Requirement 3 -- Supported Languages

3.1 WHEN the language is Assembler THE editor SHALL highlight only columns 1
    through 72 and treat any word in opcode position as a keyword.

3.2 WHEN the language is COBOL THE editor SHALL highlight only columns 7 through
    72 and support both single and double quotes as string delimiters.

3.3 WHEN the language is C THE editor SHALL recognise C++ comments (//) and
    highlight curly braces for logic matching.

3.4 WHEN the language is C THE editor SHALL treat keywords as case-sensitive and
    highlight only lowercase keyword forms.

3.5 WHEN the language is REXX THE editor SHALL highlight IF/THEN/ELSE logic but
    SHALL NOT support a terminating semicolon in the IF expression for logic
    matching.

3.6 WHEN the language is JCL THE editor SHALL highlight conditional JCL logic
    (IF/ELSE) but SHALL NOT support it in the LOGIC option.

3.7 WHEN the language is PL/I THE editor SHALL not scan column 1 after the first
    non-blank line except to search for *PROCESS statements.

3.8 WHEN the language is XML, HTML, or DTL THE editor SHALL highlight only items
    within tags and treat any < as the start of a tag.

### Requirement 4 -- Highlighting Categories

4.1 WHEN highlighting is active THE editor SHALL colour language keywords in the
    keyword colour.

4.2 WHEN highlighting is active THE editor SHALL colour comments in the comment
    colour.

4.3 WHEN highlighting is active THE editor SHALL colour quoted strings in the
    string colour.

4.4 WHEN highlighting is active THE editor SHALL colour compiler directives in
    the directive colour (for C, COBOL, PL/I, and Pascal).

4.5 WHEN highlighting is active THE editor SHALL colour special characters
    defined for the language in the special character colour.

4.6 WHEN LOGIC highlighting is enabled THE editor SHALL colour unmatched block
    constructs (e.g. unmatched END, ELSE, }) in reverse video pink.

4.7 WHEN FIND highlighting is enabled THE editor SHALL colour strings matching
    the current FIND operation in the FIND colour.

4.8 WHEN cursor phrase highlighting is enabled THE editor SHALL colour the phrase
    containing the cursor in the cursor phrase colour.

4.9 WHEN the user types characters THE editor SHALL display them in the overtype
    colour until Enter or a function key is pressed.

### Requirement 5 -- Highlighting Limitations

5.1 WHEN the data set has records longer than 255 characters THE editor SHALL
    apply only CURSOR and FIND highlighting.

5.2 WHEN the session is in mixed mode (DBCS) THE editor SHALL not apply language
    highlighting.

5.3 WHEN the session uses a format definition THE editor SHALL not apply language
    highlighting.

5.4 WHEN sequence numbers are in use THE editor SHALL highlight only the editable
    data columns and display sequence numbers in the overtype colour.

### Requirement 6 -- HILITE Dialog

6.1 WHEN the HILITE dialog is displayed THE editor SHALL allow the user to select
    a language or enable AUTO detection.

6.2 WHEN the HILITE dialog is displayed THE editor SHALL allow the user to assign
    colours to each language element category.

6.3 WHEN the HILITE dialog is displayed THE editor SHALL allow the user to enable
    or disable logic and parenthesis matching.

6.4 WHEN the HILITE dialog is displayed THE editor SHALL allow the user to turn
    FIND colouring on or off and set its colour.

6.5 WHEN the HILITE dialog is displayed THE editor SHALL allow the user to turn
    cursor phrase colouring on or off and set its colour.

6.6 WHEN the HILITE dialog is displayed THE editor SHALL allow the user to view
    the keyword list for each supported language.

6.7 WHEN the user saves changes in the HILITE dialog THE editor SHALL apply the
    new settings to the current session and save the language, type, and logic
    settings to the edit profile.
