# ISPF EARS Requirements -- Hexadecimal Display

Source: z/OS ISPF Edit and Edit Macros (SC19-3621-30), Chapter 10.

## Introduction

These requirements describe the ISPF hexadecimal display mode, which shows data
as both characters and their hexadecimal representations, and the HX line command
for per-line hex display.

## Glossary

| Term | Meaning |
|------|---------|
| HEX ON | Primary command to enable hexadecimal display for the entire session |
| HX | Line command to display a single line in hexadecimal format |
| Vertical representation | Hex digits displayed in two rows below the character row |
| Data representation | Hex digits displayed inline with the character data |
| Undisplayable character | A character that cannot be rendered on the terminal |

## Requirements

### Requirement 1 -- HEX Mode

1.1 WHEN the user issues HEX ON THE editor SHALL display all data lines in
    hexadecimal format with two rows of hex digits below each character row.

1.2 WHEN the user issues HEX OFF THE editor SHALL return to normal character
    display.

1.3 WHEN HEX mode is on THE editor SHALL display the character representation
    on the first row and the hexadecimal digits on the two rows below it.

1.4 WHEN HEX mode is on and the user types hexadecimal digits in the hex rows
    THE editor SHALL update the corresponding character in the data.

1.5 WHEN HEX mode is on THE editor SHALL save the HEX ON setting in the edit
    profile.

### Requirement 2 -- HX Line Command

2.1 WHEN the user enters the HX line command THE editor SHALL display that single
    line in hexadecimal format with two rows of hex digits below the character row.

2.2 WHEN the user enters HX on a line that is already in hex display THE editor
    SHALL return it to normal character display.

### Requirement 3 -- Undisplayable Characters

3.1 WHEN the data contains characters that cannot be displayed on the terminal
    THE editor SHALL replace them with blanks on the panel without altering the
    underlying data.

3.2 WHEN the user needs to view or edit undisplayable characters THE editor SHALL
    support doing so through hexadecimal mode or through FIND and CHANGE with
    hexadecimal strings.

3.3 WHEN the user issues FIND X'hexvalue' THE editor SHALL search for the byte
    sequence represented by the hex string.

3.4 WHEN the user issues CHANGE X'hexvalue1' X'hexvalue2' THE editor SHALL
    replace the byte sequence hexvalue1 with hexvalue2.
