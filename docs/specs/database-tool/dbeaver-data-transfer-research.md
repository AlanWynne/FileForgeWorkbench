# DBeaver Data Transfer Requirements Research

> **Source:** DBeaver public documentation, GitHub wiki, blog posts, and issue tracker.
> **Tag:** [DBV-TRANSFER]
> **Purpose:** Extract requirements for import/export wizards, bulk loading, cross-database transfer, column mapping, error handling, and progress/cancellation to inform FileForgeWorkbench database-tool sub-project.

---

## 1. Import Wizards

### 1.1 File Import to Table [DBV-TRANSFER]

1. **[DBV-TRANSFER-001]** WHEN a user initiates a data import, THE system SHALL present a wizard that guides the user through source file selection, format settings, column mapping, and confirmation steps in sequential order.

2. **[DBV-TRANSFER-002]** THE system SHALL support importing data from CSV files into existing database tables.

3. **[DBV-TRANSFER-003]** THE system SHALL support importing data from JSON files into existing database tables.

4. **[DBV-TRANSFER-004]** THE system SHALL support importing data from XML files into existing database tables.

5. **[DBV-TRANSFER-005]** WHEN the user selects a CSV file for import, THE system SHALL detect the file's delimiter character (comma, tab, semicolon, pipe, or custom) and present it as a configurable setting.

6. **[DBV-TRANSFER-006]** WHEN a CSV file is selected for import, THE system SHALL allow the user to configure: header row presence, quote character, escape character, null value representation, and character encoding.

7. **[DBV-TRANSFER-007]** WHEN the user selects a file for import and no target table exists, THE system SHALL offer to create a new table with column names and types inferred from the source file structure.

### 1.2 Column Mapping and Preview [DBV-TRANSFER]

8. **[DBV-TRANSFER-008]** WHEN importing data, THE system SHALL display a column mapping interface showing source columns on the left and target table columns on the right, allowing the user to assign each source column to a target column.

9. **[DBV-TRANSFER-009]** THE system SHALL allow the user to skip (exclude) individual source columns from the import by marking them as unmapped.

10. **[DBV-TRANSFER-010]** THE system SHALL allow the user to assign a constant value to a target column when no corresponding source column exists.

11. **[DBV-TRANSFER-011]** WHEN the user has configured column mapping, THE system SHALL provide a "Preview data" button that displays a sample of mapped rows before executing the import.

12. **[DBV-TRANSFER-012]** THE system SHALL provide "Fill Mapping" and "Clear Mapping" toolbar actions to auto-populate or reset all column assignments in the mapping interface.

### 1.3 Type Inference [DBV-TRANSFER]

13. **[DBV-TRANSFER-013]** WHEN creating a new table from imported data, THE system SHALL infer column data types by sampling the source file content (e.g., detecting integers, decimals, dates, booleans, and strings).

14. **[DBV-TRANSFER-014]** WHEN type inference has completed, THE system SHALL allow the user to override any inferred column type before table creation.

15. **[DBV-TRANSFER-015]** WHEN importing into an existing table, THE system SHALL perform implicit type conversion from source format (string) to target column type (integer, date, boolean, etc.) according to configurable format patterns.

---

## 2. Export Wizards

### 2.1 Table/Query Export [DBV-TRANSFER]

16. **[DBV-TRANSFER-016]** WHEN a user initiates a data export, THE system SHALL present a wizard that allows selection of export format, output configuration, and destination.

17. **[DBV-TRANSFER-017]** THE system SHALL support exporting data to CSV format with configurable delimiter, quote character, header inclusion, and encoding.

18. **[DBV-TRANSFER-018]** THE system SHALL support exporting data to JSON format with configurable formatting (compact or pretty-printed) and root element wrapping.

19. **[DBV-TRANSFER-019]** THE system SHALL support exporting data to SQL format as INSERT statements, with configurable options for: rows per statement, include column names, omit auto-increment columns, and transaction wrapping (BEGIN/COMMIT).

20. **[DBV-TRANSFER-020]** THE system SHALL support exporting data to XML format with configurable element naming and attribute vs. element representation.

21. **[DBV-TRANSFER-021]** THE system SHALL support exporting data to HTML format as a rendered table.

22. **[DBV-TRANSFER-022]** THE system SHALL support exporting data to Markdown format as a pipe-delimited table.

23. **[DBV-TRANSFER-023]** THE system SHALL support exporting data to plain text (TXT) format with configurable column alignment and spacing.

### 2.2 Export Sources [DBV-TRANSFER]

24. **[DBV-TRANSFER-024]** THE system SHALL allow exporting data from: a single table, multiple selected tables, a query result set, or the currently visible (filtered) rows in the data editor.

25. **[DBV-TRANSFER-025]** WHEN exporting from a table, THE system SHALL allow the user to select which columns to include in the export via a column selection interface.

26. **[DBV-TRANSFER-026]** WHEN exporting multiple tables simultaneously, THE system SHALL produce one output file per table (or a combined file, based on format capabilities and user preference).

### 2.3 Export Output Configuration [DBV-TRANSFER]

27. **[DBV-TRANSFER-027]** THE system SHALL allow the user to configure the output file path, file name pattern (supporting variables such as table name, timestamp, and sequence number), and whether to overwrite or append to existing files.

28. **[DBV-TRANSFER-028]** THE system SHALL allow configuring NULL value representation in the exported output (empty string, literal "NULL", or custom token).

29. **[DBV-TRANSFER-029]** WHEN the export format is SQL, THE system SHALL allow the user to choose between INSERT, INSERT OR UPDATE (UPSERT), and INSERT IGNORE statement styles where the target database supports them.

---

## 3. Bulk Loading

### 3.1 Batch INSERT Operations [DBV-TRANSFER]

30. **[DBV-TRANSFER-030]** WHEN transferring data to a database target, THE system SHALL batch multiple rows into a single INSERT statement (multi-row INSERT) to reduce round trips, with a configurable batch size (default: 200 rows per batch).

31. **[DBV-TRANSFER-031]** THE system SHALL allow the user to disable batch insert mode and fall back to individual single-row INSERT statements for debugging or when batch mode encounters errors.

32. **[DBV-TRANSFER-032]** THE system SHALL allow configuring the number of rows per INSERT statement (multi-value INSERT syntax) separately from the commit interval.

33. **[DBV-TRANSFER-033]** THE system SHALL allow configuring the commit interval (number of rows between COMMIT statements) to control transaction size during bulk loads.

### 3.2 Database-Native Bulk Load [DBV-TRANSFER]

34. **[DBV-TRANSFER-034]** WHEN the target database is PostgreSQL, THE system SHALL support using the native COPY command for bulk data loading as an alternative to INSERT statements, offering significantly higher throughput.

35. **[DBV-TRANSFER-035]** WHEN the target database is MySQL or MariaDB, THE system SHALL support using the native LOAD DATA INFILE command for bulk data loading as an alternative to INSERT statements.

36. **[DBV-TRANSFER-036]** THE system SHALL allow the user to select between standard INSERT mode and native bulk load mode (where available) in the data transfer settings.

37. **[DBV-TRANSFER-037]** WHEN using native bulk load mode, THE system SHALL handle temporary file staging, encoding, and format preparation transparently without requiring manual user intervention beyond mode selection.

### 3.3 Performance Tuning [DBV-TRANSFER]

38. **[DBV-TRANSFER-038]** THE system SHALL allow configuring the fetch size (number of rows read from source per network round trip) for data extraction, with a default of 10,000 rows.

39. **[DBV-TRANSFER-039]** THE system SHALL support multi-threaded data transfer with a configurable number of producer and consumer threads (recommended: match CPU core count).

40. **[DBV-TRANSFER-040]** THE system SHALL allow the user to choose whether to open a new database connection for the transfer or reuse the existing connection, to avoid locking conflicts with concurrent work.

41. **[DBV-TRANSFER-041]** WHEN performing bulk insert, THE system SHALL optionally disable table indexes and constraints before loading and rebuild them after completion, to improve throughput on large loads.

---

## 4. Cross-Database Transfer

### 4.1 Table-to-Table Copy [DBV-TRANSFER]

42. **[DBV-TRANSFER-042]** THE system SHALL support transferring data from a source table in one database connection to a target table in a different database connection (cross-database transfer).

43. **[DBV-TRANSFER-043]** WHEN initiating a cross-database transfer, THE system SHALL allow the user to select the target connection and target schema/table from a browsable tree of available connections.

44. **[DBV-TRANSFER-044]** THE system SHALL support transferring multiple tables in a single transfer operation, mapping each source table to a corresponding target table.

45. **[DBV-TRANSFER-045]** WHEN the target table does not exist, THE system SHALL offer to create it automatically, deriving column names and types from the source table structure with appropriate type mapping for the target database dialect.

46. **[DBV-TRANSFER-046]** WHEN transferring between databases of different vendors (e.g., Oracle to PostgreSQL), THE system SHALL perform automatic data type mapping from source types to equivalent target types, displaying the mapping for user review.

47. **[DBV-TRANSFER-047]** IF a source column type has no direct equivalent in the target database, THEN THE system SHALL select the closest compatible type and notify the user of the type conversion in the mapping display.

### 4.2 Transfer Options [DBV-TRANSFER]

48. **[DBV-TRANSFER-048]** THE system SHALL allow the user to choose a transfer mode: INSERT only, INSERT or UPDATE (merge/upsert), or TRUNCATE then INSERT (replace all data).

49. **[DBV-TRANSFER-049]** WHEN transferring data between databases, THE system SHALL allow the user to apply a WHERE clause filter on the source to transfer only a subset of rows.

50. **[DBV-TRANSFER-050]** THE system SHALL support transferring data from a custom SQL query result (not just a physical table) to a target table in another connection.

---

## 5. Column Mapping

### 5.1 Source-to-Target Column Mapping [DBV-TRANSFER]

51. **[DBV-TRANSFER-051]** THE system SHALL present a column mapping editor as a grid/table showing: source column name, source data type, target column name, target data type, and mapping status (mapped, skipped, or constant).

52. **[DBV-TRANSFER-052]** THE system SHALL auto-populate column mappings by matching source column names to target column names (case-insensitive name matching).

53. **[DBV-TRANSFER-053]** THE system SHALL allow the user to manually override any column mapping by selecting a different target column from a dropdown list.

54. **[DBV-TRANSFER-054]** THE system SHALL allow reordering source-to-target mappings without requiring the source and target columns to be in the same ordinal position.

55. **[DBV-TRANSFER-055]** WHEN a target column has no corresponding source column and is NOT NULL without a default, THE system SHALL warn the user that the transfer may fail unless a constant value or expression is provided.

### 5.2 Type Conversion [DBV-TRANSFER]

56. **[DBV-TRANSFER-056]** THE system SHALL display both source and target data types side-by-side in the mapping editor and highlight type mismatches that may cause data loss (e.g., VARCHAR(255) → VARCHAR(50), FLOAT → INTEGER).

57. **[DBV-TRANSFER-057]** THE system SHALL perform automatic type conversion for compatible type pairs (e.g., INT → BIGINT, VARCHAR → TEXT, DATE → TIMESTAMP) without requiring user intervention.

58. **[DBV-TRANSFER-058]** WHEN a column mapping involves a narrowing type conversion (potential data loss), THE system SHALL display a warning indicator on that mapping row.

59. **[DBV-TRANSFER-059]** THE system SHALL allow the user to configure date/time format patterns for parsing string source data into date/timestamp target columns.

60. **[DBV-TRANSFER-060]** THE system SHALL allow the user to configure numeric format patterns (decimal separator, thousands separator) for parsing string source data into numeric target columns.

---

## 6. Error Handling

### 6.1 Error Policies [DBV-TRANSFER]

61. **[DBV-TRANSFER-061]** THE system SHALL provide configurable error handling policies for data transfer operations, selectable before the transfer begins.

62. **[DBV-TRANSFER-062]** THE system SHALL support an "Abort on first error" policy that stops the entire transfer immediately when any row fails to insert or convert.

63. **[DBV-TRANSFER-063]** THE system SHALL support a "Skip errors" policy that logs failed rows and continues processing remaining rows.

64. **[DBV-TRANSFER-064]** THE system SHALL support a "Maximum error count" policy that continues processing until a user-specified number of errors is reached, then aborts the transfer.

65. **[DBV-TRANSFER-065]** WHEN batch insert mode encounters an error, THE system SHALL offer the option to disable batch mode and retry the failed batch row-by-row to identify the specific failing row(s).

### 6.2 Error Logging and Reporting [DBV-TRANSFER]

66. **[DBV-TRANSFER-066]** THE system SHALL maintain an error log during data transfer that records: row number, source data values, target column, error type, and error message for each failed row.

67. **[DBV-TRANSFER-067]** WHEN a data transfer completes (whether successfully or with errors), THE system SHALL display a summary showing: total rows processed, rows successfully transferred, rows skipped due to errors, and elapsed time.

68. **[DBV-TRANSFER-068]** THE system SHALL allow the user to export the error log to a file (CSV or text format) for offline analysis after transfer completion.

69. **[DBV-TRANSFER-069]** WHEN a data transfer encounters a connection failure mid-operation, THE system SHALL attempt to reconnect (up to a configurable retry count) before aborting, and SHALL report the last successfully committed row for potential resume.

### 6.3 Data Validation [DBV-TRANSFER]

70. **[DBV-TRANSFER-070]** WHEN importing data, THE system SHALL validate that NOT NULL constraints are satisfied before attempting to insert each row, and SHALL apply the configured error policy for violations.

71. **[DBV-TRANSFER-071]** WHEN importing data, THE system SHALL validate that data values conform to the target column's type and length constraints, reporting type conversion failures per the configured error policy.

72. **[DBV-TRANSFER-072]** IF a duplicate key violation occurs during insert, THEN THE system SHALL handle it according to the configured duplicate handling mode: fail, skip, or replace (upsert).

---

## 7. Progress and Cancellation

### 7.1 Progress Reporting [DBV-TRANSFER]

73. **[DBV-TRANSFER-073]** WHEN a data transfer operation is running, THE system SHALL display a progress indicator showing the number of rows processed so far.

74. **[DBV-TRANSFER-074]** WHEN the total row count is known (e.g., from a COUNT query or file size), THE system SHALL display a percentage-based progress bar and estimated time remaining.

75. **[DBV-TRANSFER-075]** THE system SHALL display the current transfer speed (rows per second) in the progress view.

76. **[DBV-TRANSFER-076]** THE system SHALL update the progress display at a minimum interval of once per second to keep the user informed without excessive UI overhead.

77. **[DBV-TRANSFER-077]** WHEN multiple tables are being transferred in a single operation, THE system SHALL display per-table progress as well as overall operation progress.

### 7.2 Background Execution [DBV-TRANSFER]

78. **[DBV-TRANSFER-078]** THE system SHALL execute data transfer operations in a background thread, allowing the user to continue interacting with the application (browsing objects, editing queries) during the transfer.

79. **[DBV-TRANSFER-079]** THE system SHALL display active data transfer operations in a background tasks view accessible from the main application window.

80. **[DBV-TRANSFER-080]** THE system SHALL allow multiple data transfer operations to run concurrently (subject to connection and resource limits).

### 7.3 Cancellation [DBV-TRANSFER]

81. **[DBV-TRANSFER-081]** WHEN a data transfer operation is running, THE system SHALL provide a Cancel button that requests graceful cancellation of the operation.

82. **[DBV-TRANSFER-082]** WHEN cancellation is requested, THE system SHALL complete the current batch (to avoid partial row corruption), then stop processing further batches, and report the number of rows successfully transferred before cancellation.

83. **[DBV-TRANSFER-083]** WHEN a transfer is cancelled, THE system SHALL leave already-committed data in place (no automatic rollback of committed batches) unless the user explicitly configured the entire operation as a single transaction.

84. **[DBV-TRANSFER-084]** IF the entire transfer was configured as a single transaction (no intermediate commits), THEN WHEN cancelled, THE system SHALL roll back the transaction, leaving the target table unchanged.

### 7.4 Task Persistence and Resume [DBV-TRANSFER]

85. **[DBV-TRANSFER-085]** THE system SHALL allow saving a data transfer configuration (source, target, mappings, settings) as a reusable named task for repeated execution.

86. **[DBV-TRANSFER-086]** THE system SHALL allow scheduling saved data transfer tasks for automated execution at specified times or intervals (via integration with OS task scheduler or internal scheduler).

87. **[DBV-TRANSFER-087]** WHEN a data transfer task completes, THE system SHALL record execution history including: start time, end time, rows transferred, error count, and final status (success, partial, failed, cancelled).

88. **[DBV-TRANSFER-088]** WHEN a previously cancelled or failed transfer is re-executed, THE system SHALL allow the user to choose between restarting from the beginning or attempting to resume from the last committed position (where the source supports deterministic ordering).

---

## Summary

| Category | Requirement Count | IDs |
|----------|------------------|-----|
| Import Wizards | 15 | DBV-TRANSFER-001 to 015 |
| Export Wizards | 14 | DBV-TRANSFER-016 to 029 |
| Bulk Loading | 12 | DBV-TRANSFER-030 to 041 |
| Cross-Database Transfer | 9 | DBV-TRANSFER-042 to 050 |
| Column Mapping | 10 | DBV-TRANSFER-051 to 060 |
| Error Handling | 12 | DBV-TRANSFER-061 to 072 |
| Progress and Cancellation | 16 | DBV-TRANSFER-073 to 088 |
| **Total** | **88** | |

---

## References

- [DBeaver Data Transfer documentation](https://dbeaver.com/docs/dbeaver/Data-transfer/) (content rephrased for compliance with licensing restrictions)
- [DBeaver Data Import documentation](https://dbeaver.com/docs/dbeaver/Data-import/) (content rephrased for compliance with licensing restrictions)
- [DBeaver Data Export documentation](https://dbeaver.com/docs/dbeaver/Data-export/) (content rephrased for compliance with licensing restrictions)
- [DBeaver Data Migration documentation](https://dbeaver.com/docs/dbeaver/Data-migration/) (content rephrased for compliance with licensing restrictions)
- [DBeaver Data Import and Replace](https://dbeaver.com/docs/dbeaver/Data-Import-and-Replace/) (content rephrased for compliance with licensing restrictions)
- [DBeaver Task Management](https://dbeaver.com/docs/dbeaver/Task-Management/) (content rephrased for compliance with licensing restrictions)
- [DBeaver Background Tasks](https://dbeaver.com/docs/dbeaver/Background-Tasks/) (content rephrased for compliance with licensing restrictions)
- [DBeaver Blog: Import Data with DBeaver](https://dbeaver.com/2022/06/23/import-data-with-dbeaver/) (content rephrased for compliance with licensing restrictions)
- [DBeaver Blog: How to Export Data](https://dbeaver.com/2024/09/19/how-to-export-data-in-dbeaver/) (content rephrased for compliance with licensing restrictions)
- [DBeaver Blog: Migrate Data from Oracle to MariaDB](https://dbeaver.com/2024/04/25/how-to-migrate-data-from-oracle-to-mariadb-with-dbeaver/) (content rephrased for compliance with licensing restrictions)
- [DBeaver GitHub Wiki: Data Transfer](https://github.com/dbeaver/dbeaver/wiki/Data-transfer) (content rephrased for compliance with licensing restrictions)
