# SDSF Filter, Sort, Arrange, and Search Commands -- EARS Requirements

Source documents: ikja100 (SDSF User Guide) Chapters 1 and 9.

Priority: P1 (SDSF-FILTER-1 through SDSF-FILTER-5, SDSF-SCROLL-1, SDSF-SCROLL-2) /
          P2 (SDSF-FILTER-6, SDSF-FILTER-7, SDSF-SCROLL-3 through SDSF-SCROLL-5).
Sub-project mapping: FFW-JES (primary), record-selection-criteria, exclude-show-filter,
                     find-and-replace, navigation-commands (secondary).

---

## Section A: Filter and Sort Commands

### Requirement SDSF-FILTER-1: PREFIX Filter

WHEN the user enters PREFIX(pattern) on any job panel,
THE workbench SHALL filter the display to show only jobs whose names match the pattern.

Criteria:
- 1.1 THE PREFIX filter SHALL support wildcard characters (* for any string, % for any single character).
- 1.2 THE PREFIX filter SHALL be displayed in the filter information line as "PREFIX=pattern".
- 1.3 WHEN PREFIX=* is set, ALL jobs SHALL be displayed regardless of name.
- 1.4 THE PREFIX filter SHALL persist until changed or reset.

### Requirement SDSF-FILTER-2: OWNER Filter

WHEN the user enters OWNER(userid) on any job panel,
THE workbench SHALL filter the display to show only jobs owned by the specified user.

Criteria:
- 2.1 THE OWNER filter SHALL support wildcard characters.
- 2.2 THE OWNER filter SHALL be displayed in the filter information line as "OWNER=userid".
- 2.3 WHEN OWNER=* is set, jobs from ALL owners SHALL be displayed.

### Requirement SDSF-FILTER-3: DEST Filter

WHEN the user enters DEST(destination) on any job panel,
THE workbench SHALL filter the display to show only jobs destined for the specified output destination.

Criteria:
- 3.1 THE DEST filter SHALL support the value ALL to show all destinations.
- 3.2 THE DEST filter SHALL be displayed in the filter information line as "DEST=(destination)".

### Requirement SDSF-FILTER-4: FILTER Command

WHEN the user enters FILTER column operator value,
THE workbench SHALL apply a column-level filter to the current panel.

Criteria:
- 4.1 THE FILTER command SHALL support operators: EQ, NE, GT, LT, GE, LE, CONTAINS, OMIT.
- 4.2 THE FILTER command SHALL support multiple simultaneous filter conditions combined with AND/OR.
- 4.3 THE FILTER command SHALL support wildcard pattern matching using * and %.
- 4.4 THE user SHALL be able to clear all filters using FILTER RESET or RESET.
- 4.5 THE active filter conditions SHALL be displayed in the filter information lines below the COMMAND field.
- 4.6 THE FILTER command SHALL support the SET DISPLAY command to control which columns are shown.

### Requirement SDSF-FILTER-5: SORT Command

WHEN the user enters SORT column [A|D],
THE workbench SHALL sort the panel data by the specified column.

Criteria:
- 5.1 THE SORT command SHALL support ascending (A) and descending (D) sort order.
- 5.2 THE SORT command SHALL support sorting by multiple columns (e.g., SORT JOBNAME A JOBID D).
- 5.3 THE default sort order SHALL be ascending.
- 5.4 THE SORT command SHALL support SET CSORT to set a persistent column sort.

### Requirement SDSF-FILTER-6: ARRANGE Command

WHEN the user enters ARRANGE,
THE workbench SHALL allow the user to reorder, hide, or show columns on the current panel.

Criteria:
- 6.1 THE ARRANGE command SHALL allow columns to be moved left or right.
- 6.2 THE ARRANGE command SHALL allow columns to be hidden (ARRANGE column OFF).
- 6.3 THE ARRANGE command SHALL allow hidden columns to be restored (ARRANGE column ON).
- 6.4 Column arrangements SHALL persist for the session.

### Requirement SDSF-FILTER-7: SET DISPLAY Command

WHEN the user enters SET DISPLAY,
THE workbench SHALL control which columns are visible on the current panel.

Criteria:
- 7.1 THE SET DISPLAY command SHALL support showing the primary column set (SET DISPLAY ON).
- 7.2 THE SET DISPLAY command SHALL support showing the alternate column set (? command or SET DISPLAY ALT).
- 7.3 THE alternate column set SHALL include all primary columns plus additional delayed-access columns.

---

## Section B: Search and Scroll Commands

### Requirement SDSF-SCROLL-1: FIND Command

WHEN the user enters FIND string on a browse or log panel,
THE workbench SHALL search for the string and position the display at the first occurrence.

Criteria:
- 1.1 THE FIND command SHALL search forward from the current position by default.
- 1.2 THE FIND command SHALL support FIND string PREV to search backward.
- 1.3 THE FIND command SHALL support FIND string FIRST and FIND string LAST.
- 1.4 THE FIND command SHALL support FIND string NEXT to find the next occurrence.
- 1.5 WHEN the string is not found, THE workbench SHALL display a "string not found" message.
- 1.6 THE RFIND command (PF5) SHALL repeat the last FIND in the same direction.
- 1.7 THE FINDLIM command SHALL set the maximum number of lines to search.

### Requirement SDSF-SCROLL-2: LOCATE Command

WHEN the user enters LOCATE value on a tabular panel,
THE workbench SHALL scroll the panel to position the row matching value at the top.

Criteria:
- 2.1 THE LOCATE command SHALL match against the fixed (first) column of the panel.
- 2.2 THE LOCATE command SHALL support date/time format patterns for time-based columns.
- 2.3 WHEN no exact match exists, THE panel SHALL scroll to the nearest match.

### Requirement SDSF-SCROLL-3: LOG Command

WHEN the user enters LOG on a log panel,
THE workbench SHALL position the display at a specific date/time in the log.

Criteria:
- 3.1 THE LOG command SHALL accept a date/time parameter to position within the system log.
- 3.2 THE LOG command SHALL support relative positioning (e.g., LOG -1H for one hour ago).

### Requirement SDSF-SCROLL-4: NEXT and PREV Commands

WHEN the user enters NEXT or PREV on a log panel,
THE workbench SHALL scroll to the next or previous occurrence of a search string or log record type.

Criteria:
- 4.1 THE NEXT command SHALL scroll forward to the next matching record.
- 4.2 THE PREV command SHALL scroll backward to the previous matching record.
- 4.3 THE NEXT and PREV commands SHALL support filtering by record type.

### Requirement SDSF-SCROLL-5: SNAPSHOT Command

WHEN the user enters SNAPSHOT,
THE workbench SHALL capture the current panel state for comparison or export.

Criteria:
- 5.1 THE SNAPSHOT command SHALL capture the current panel data to a data set or file.
- 5.2 THE captured snapshot SHALL include all visible columns and rows.
