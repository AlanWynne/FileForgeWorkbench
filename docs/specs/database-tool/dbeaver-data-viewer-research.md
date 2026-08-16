# DBeaver Data Viewer — Requirements Research [DBV-DATA]

> **Source:** DBeaver Community Edition documentation and wiki (public domain).
> Content was rephrased for compliance with licensing restrictions.
> References: [Data Viewing and Editing](https://github.com/dbeaver/dbeaver/wiki/Data-Viewing-and-Editing), [Data Filters](https://github.com/dbeaver/dbeaver/wiki/Data-Filters), [Data View and Format](https://github.com/dbeaver/dbeaver/wiki/Data-View-and-Format), [Data Export](https://github.com/dbeaver/dbeaver/wiki/Data-export), [Data Editor Preferences](https://dbeaver.com/docs/dbeaver/Data-Editor-preferences/)

---

## 1. Grid / Table View

### 1.1 Result Grid Display [DBV-DATA]

THE data viewer SHALL display query results in a scrollable grid with rows as horizontal entries and columns as vertical fields, analogous to a spreadsheet layout.

### 1.2 Column Resizing [DBV-DATA]

WHEN the user drags a column border in the grid header, THE data viewer SHALL resize that column to the user-specified width and persist the width for the duration of the session.

### 1.3 Column Reordering [DBV-DATA]

WHEN the user drags a column header to a different position, THE data viewer SHALL reorder the columns in the display without altering the underlying query or data.

### 1.4 Row Numbering [DBV-DATA]

THE data viewer SHALL display a sequential row number in the leftmost gutter column, starting at 1 for the first fetched row and incrementing by 1 for each subsequent row.

### 1.5 Configurable Fetch Size [DBV-DATA]

THE data viewer SHALL fetch rows from the database in configurable batch sizes (default: 200 rows per fetch) and allow the user to adjust the fetch size in preferences.

### 1.6 Incremental Scrolling / Pagination [DBV-DATA]

WHEN the user scrolls past the last fetched row, THE data viewer SHALL automatically fetch the next batch of rows from the server and append them to the grid (auto-fetch next segment).

### 1.7 Manual Next-Page Fetch [DBV-DATA]

WHEN the user activates the "Fetch next page" action, THE data viewer SHALL retrieve the next batch of rows equal to the configured fetch size and append them to the result set.

### 1.8 Total Row Count [DBV-DATA]

WHEN the user activates the "Calculate total row count" action, THE data viewer SHALL execute a COUNT query against the data source and display the total row count in the status bar.

### 1.9 Record View Toggle [DBV-DATA]

WHEN the user activates the Record view toggle, THE data viewer SHALL transpose the display so that column names appear as row labels and the cell values for the currently selected row appear in a single "Value" column.

### 1.10 Column Visibility [DBV-DATA]

THE data viewer SHALL allow the user to show or hide individual columns via a column management dialog, without affecting the underlying query.

### 1.11 Column Pinning [DBV-DATA]

WHEN the user pins a column, THE data viewer SHALL lock that column at its current horizontal position so it remains visible during horizontal scrolling.

---

## 2. Cell Editing

### 2.1 Inline Cell Editing [DBV-DATA]

WHEN the user double-clicks a cell, presses Enter on a focused cell, or selects "Inline edit" from the context menu, THE data viewer SHALL make the cell editable in place, accepting typed input directly in the grid.

### 2.2 Cell Editor Panel [DBV-DATA]

WHEN the user activates the cell editor (Shift+Enter or toolbar button), THE data viewer SHALL open a dedicated editor panel displaying column metadata and a value edit area supporting multi-line text entry.

### 2.3 Set Cell to NULL [DBV-DATA]

WHEN the user selects "Set to NULL" from the cell context menu, THE data viewer SHALL replace the cell value with the database NULL marker and visually indicate the NULL state.

### 2.4 Set Cell to Default Value [DBV-DATA]

WHEN the user selects "Set to default" from the cell context menu, THE data viewer SHALL replace the cell value with the column's defined default value as reported by the database metadata.

### 2.5 Row Addition [DBV-DATA]

WHEN the user activates the "Add row" action, THE data viewer SHALL insert a new empty row below the currently focused row and allow the user to populate it via inline editing or the cell editor.

### 2.6 Row Duplication [DBV-DATA]

WHEN the user selects one or more rows and activates the "Duplicate row" action, THE data viewer SHALL create copies of the selected rows and insert them immediately below the originals.

### 2.7 Row Deletion [DBV-DATA]

WHEN the user selects one or more rows and activates the "Delete row" action, THE data viewer SHALL mark those rows for deletion with a visual indicator (red highlighting) and remove them from the database upon save.

### 2.8 Save Changes (Commit) [DBV-DATA]

WHEN the user activates the "Save" action, THE data viewer SHALL generate and execute the appropriate SQL statements (INSERT, UPDATE, DELETE) to persist all pending changes to the database.

### 2.9 Cancel Changes (Rollback) [DBV-DATA]

WHEN the user activates the "Cancel" action, THE data viewer SHALL discard all pending unsaved changes and restore the grid to the last committed state.

### 2.10 Preview Generated SQL [DBV-DATA]

WHEN the user selects "Generate Script" from the Save dropdown, THE data viewer SHALL display a read-only preview of the SQL statements that would be executed to persist the pending changes.

### 2.11 Auto-Commit and Manual-Commit Modes [DBV-DATA]

THE data viewer SHALL support both auto-commit mode (changes committed immediately after each statement) and manual-commit mode (changes held in a transaction until explicit Commit or Rollback).

### 2.12 Virtual Key Support for Editing [DBV-DATA]

IF a table lacks a unique key, THEN THE data viewer SHALL allow the user to define a virtual unique key from one or more columns to enable row identification for editing operations.

---

## 3. Inline Filtering

### 3.1 SQL Expression Filter Bar [DBV-DATA]

THE data viewer SHALL provide a filter text field above the grid where the user can type arbitrary SQL WHERE-clause expressions, which are applied to the result set upon pressing Enter or clicking Apply.

### 3.2 Column Header Filter [DBV-DATA]

WHEN the user clicks the filter icon in a column header, THE data viewer SHALL display a dropdown allowing the user to filter by specific cell values, comparison operators (=, <>, >, <, IS NULL, IS NOT NULL), or custom expressions scoped to that column.

### 3.3 Preset Filter Templates [DBV-DATA]

THE data viewer SHALL provide preset SQL filter expressions (equals, not-equals, greater-than, less-than, IS NULL, IS NOT NULL) accessible from the column header dropdown or context menu.

### 3.4 Clipboard-Based Filtering [DBV-DATA]

WHEN the user selects a clipboard-based filter option from the column menu, THE data viewer SHALL apply a filter comparing the column value against the current clipboard content using the selected operator.

### 3.5 Filter History [DBV-DATA]

THE data viewer SHALL maintain a history of previously applied filter expressions and allow the user to navigate forward/backward through that history or select from a saved filters list.

### 3.6 Clear All Filters [DBV-DATA]

WHEN the user activates the "Remove All Filters/Orderings" action, THE data viewer SHALL remove all active filter expressions and sort orderings, returning the result set to its unfiltered state.

### 3.7 Custom WHERE and ORDER BY [DBV-DATA]

THE data viewer SHALL provide a custom filter dialog where the user can enter arbitrary WHERE conditions and ORDER BY clauses that are appended to the data-fetching query.

### 3.8 Column-Level Criteria in Settings Dialog [DBV-DATA]

THE data viewer SHALL provide a Result Set Order/Filter Settings dialog showing all columns with per-column checkboxes for visibility, pinning state, sort order, and a criteria expression field.

---

## 4. Sorting

### 4.1 Single-Column Sort Toggle [DBV-DATA]

WHEN the user clicks a column header or selects an ordering option from the column dropdown, THE data viewer SHALL sort the result set by that column in ascending order; a subsequent activation SHALL toggle to descending order; a third activation SHALL remove the sort.

### 4.2 Multi-Column Sort [DBV-DATA]

THE data viewer SHALL support sorting by multiple columns simultaneously, where each column is assigned a sort priority (primary, secondary, etc.) and an independent ascending/descending direction.

### 4.3 Sort Direction Indicator [DBV-DATA]

WHEN a column has an active sort order, THE data viewer SHALL display an ascending (↑) or descending (↓) indicator in that column's header.

### 4.4 Server-Side vs Client-Side Ordering [DBV-DATA]

THE data viewer SHALL support configurable ordering modes: "Always on server" (appends ORDER BY to the SQL query), "Always on client" (sorts fetched rows locally), and "Adaptive" (automatically selects the most appropriate strategy based on context).

### 4.5 Sort via Context Menu [DBV-DATA]

WHEN the user right-clicks a column and selects "Order by [column] ASC" or "Order by [column] DESC", THE data viewer SHALL apply the selected sort order to that column.

### 4.6 Sort via Settings Dialog [DBV-DATA]

THE data viewer SHALL allow sort configuration through the Result Set Order/Filter Settings dialog, where clicking the Order cell next to a column name cycles through ascending, descending, and no-sort states.

---

## 5. Data Export

### 5.1 Export to CSV [DBV-DATA]

THE data viewer SHALL support exporting result set data to CSV format with configurable delimiter, quote character, header inclusion, encoding, and line separator options.

### 5.2 Export to JSON [DBV-DATA]

THE data viewer SHALL support exporting result set data to JSON format, producing an array of objects where each object represents a row with column names as keys.

### 5.3 Export to SQL INSERT Statements [DBV-DATA]

THE data viewer SHALL support exporting result set data as SQL INSERT statements, generating syntactically correct INSERT INTO statements for the target table with proper value quoting and type handling.

### 5.4 Export to XML [DBV-DATA]

THE data viewer SHALL support exporting result set data to well-formed XML documents with configurable element naming for rows and columns.

### 5.5 Export to Clipboard [DBV-DATA]

THE data viewer SHALL support copying selected rows/cells to the clipboard in multiple formats (TAB-delimited, CSV, JSON, SQL, HTML, XML, Markdown, plain text) via Advanced Copy options.

### 5.6 Column Selection for Export [DBV-DATA]

WHEN the user initiates a data export, THE data viewer SHALL allow selection of specific columns to include in the export output via a column list with checkboxes.

### 5.7 Export Row Scope [DBV-DATA]

THE data viewer SHALL allow the user to export all rows, only selected rows, or a custom row range when performing data export operations.

### 5.8 Export Fetch Size [DBV-DATA]

THE data viewer SHALL allow configuration of the fetch batch size used during export operations, separate from the display fetch size, to optimise throughput for large exports.

### 5.9 Copy Configuration [DBV-DATA]

THE data viewer SHALL provide a "Configure Copy-As commands" dialog allowing the user to customise delimiters, quoting behaviour, header inclusion, and value display format for each clipboard export format.

### 5.10 Export via Data Transfer Wizard [DBV-DATA]

THE data viewer SHALL integrate with a Data Transfer wizard that guides the user through source selection, format selection, format-specific settings, and output destination (file path or clipboard).

---

## 6. LOB Handling

### 6.1 CLOB Display in Grid [DBV-DATA]

WHEN a column contains CLOB (Character Large Object) data, THE data viewer SHALL display a truncated text preview in the grid cell with a length indicator, and open the full text in a dedicated editor tab upon cell editor activation.

### 6.2 BLOB Display in Grid [DBV-DATA]

WHEN a column contains BLOB (Binary Large Object) data, THE data viewer SHALL display a size/type indicator in the grid cell (e.g., "[BLOB, 4.2 KB]") rather than attempting to render raw binary content inline.

### 6.3 Hex Viewer for BLOB [DBV-DATA]

WHEN the user opens a BLOB value in the cell editor, THE data viewer SHALL provide a hexadecimal viewer displaying the binary content in hex-offset-ASCII format for inspection and editing.

### 6.4 Text Viewer for CLOB [DBV-DATA]

WHEN the user opens a CLOB value in the cell editor, THE data viewer SHALL display the content in a text editor panel supporting multi-line viewing, syntax highlighting (if applicable), and editing.

### 6.5 Image Rendering for BLOB [DBV-DATA]

IF a BLOB column contains image data in a recognised format (PNG, JPEG, GIF, BMP), THEN THE data viewer SHALL automatically render the image in the value viewer panel instead of displaying raw binary.

### 6.6 Save LOB to File [DBV-DATA]

THE data viewer SHALL allow the user to save BLOB or CLOB content to an external file on the local filesystem via a "Save to file" action.

### 6.7 Load LOB from File [DBV-DATA]

THE data viewer SHALL allow the user to load BLOB or CLOB content from an external file into the selected cell via a "Load from file" action.

### 6.8 LOB Content Caching Control [DBV-DATA]

THE data viewer SHALL provide a preference to enable or disable LOB content caching, where disabling prevents automatic fetching of LOB column contents until explicitly requested by the user.

### 6.9 Value Viewer Panel [DBV-DATA]

WHEN the user activates the Value Viewer panel (F7), THE data viewer SHALL display a side panel showing the full content of the currently selected cell, with format-appropriate rendering (text, hex, image).

---

## 7. NULL Display

### 7.1 Configurable NULL Representation [DBV-DATA]

THE data viewer SHALL display NULL values using a configurable text representation (default: "[NULL]") that is visually distinct from empty strings and regular data values.

### 7.2 NULL Visual Styling [DBV-DATA]

THE data viewer SHALL render NULL cell values with a distinct visual style (e.g., greyed-out italic text, different background colour) so they are immediately distinguishable from non-NULL values.

### 7.3 NULL vs Empty String Distinction [DBV-DATA]

THE data viewer SHALL visually distinguish between NULL values and empty strings, displaying the configured NULL marker for NULL and showing an empty (but stylistically normal) cell for empty strings.

### 7.4 Show NULLs Preference [DBV-DATA]

THE data viewer SHALL provide a "Show NULLs" preference that controls whether NULL values are rendered with the NULL marker text or left visually blank.

### 7.5 NULL-Aware Column Width [DBV-DATA]

IF the "Show NULLs" preference is enabled, THEN THE data viewer SHALL ensure column widths accommodate the NULL marker text (e.g., "[NULL]") when auto-sizing columns.

### 7.6 NULL Paste Support [DBV-DATA]

WHEN pasting data with the "Insert NULLs" option enabled, THE data viewer SHALL interpret cells matching the configured NULL value mark as database NULL rather than as literal text.

---

## Summary

| Category | Requirement Count |
|----------|------------------|
| Grid / Table View | 11 |
| Cell Editing | 12 |
| Inline Filtering | 8 |
| Sorting | 6 |
| Data Export | 10 |
| LOB Handling | 9 |
| NULL Display | 6 |
| **Total** | **52** |

---

*Research extracted for FileForgeWorkbench database-tool sub-project, task 16.3.*
