# SDSF Browse and Print -- EARS Requirements

Source documents: ikja100 (SDSF User Guide) Chapter 1.

Priority: P1 (SDSF-BROWSE-1) / P2 (SDSF-BROWSE-2, SDSF-BROWSE-3, SDSF-BROWSE-4).
Sub-project mapping: FFW-JES (primary), custom-file-viewers, file-operations (secondary).

---

## Requirement SDSF-BROWSE-1: Browse Job Output

WHEN the user types S in the NP column of a job,
THE workbench SHALL open the job output in a browse viewer.

Criteria:
- 1.1 THE browse viewer SHALL display the job output in line-mode format.
- 1.2 THE browse viewer SHALL support FIND, RFIND, UP, DOWN, LEFT, RIGHT, TOP, BOTTOM scroll commands.
- 1.3 THE browse viewer SHALL support SET HEX to toggle hexadecimal display.
- 1.4 THE user SHALL be able to open job output in ISPF Browse (SB action), ISPF Edit (SE action), or ISPF View (SV action).
- 1.5 THE user SHALL be able to browse a specific output data set using the Sn action character (where n is the data set sequence number).

---

## Requirement SDSF-BROWSE-2: Browse Session Settings

WHEN the user configures browse behavior,
THE workbench SHALL support SET BROWSE settings.

Criteria:
- 2.1 SET BROWSE ISPF SHALL cause the S action to invoke ISPF Browse instead of SDSF browse.
- 2.2 SET BROWSE SDSF SHALL cause the S action to use the SDSF line-mode browser.
- 2.3 The browse setting SHALL persist across sessions.

---

## Requirement SDSF-BROWSE-3: Print from SDSF Panels

WHEN the user invokes print from an SDSF panel,
THE workbench SHALL support printing panel content to a file or output destination.

Criteria:
- 3.1 THE PRINT command SHALL support printing the current panel to a data set, SYSOUT, a file, or a DDNAME.
- 3.2 THE PRINT command SHALL support printing a tabular panel with all visible columns.
- 3.3 THE PRINT command SHALL support PRINT CLOSE to close the print data set.
- 3.4 THE PRINT command SHALL support PRINT OPEN to open a print data set before printing.

---

## Requirement SDSF-BROWSE-4: Show All Column Values

WHEN the user types / (slash) in the NP column of a row,
THE workbench SHALL display a pop-up showing all column values for that row.

Criteria:
- 4.1 THE Show Columns pop-up SHALL display all columns and their values in a scrollable list.
- 4.2 THE pop-up SHALL include an option to show all columns (including blank values) or only columns with values.
- 4.3 THE pop-up SHALL include an option to format values using the panel column width or maximum width.
