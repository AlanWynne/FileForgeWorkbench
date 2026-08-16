# DBeaver SQL Editor — Requirements Research

> **Source:** DBeaver Community Edition (open-source), DBeaver Lite/Enterprise/Ultimate documentation, GitHub wiki, and public issue tracker.
> **Tag:** [DBV-SQL]
> **Format:** EARS (Easy Approach to Requirements Syntax)

---

## 1. SQL Editing (Multi-Statement Editor)

### 1.1 Multi-Statement Script Panel [DBV-SQL]

**THE** SQL editor **SHALL** provide a script panel that supports editing multiple SQL statements within a single editor buffer, separated by a configurable statement delimiter.

### 1.2 Statement Delimiter Configuration [DBV-SQL]

**THE** SQL editor **SHALL** use a semicolon (`;`) as the default statement delimiter and **SHALL** allow the user to configure an alternative delimiter character or string per connection or per script.

### 1.3 Blank Line as Statement Delimiter [DBV-SQL]

**WHEN** the "blank line is statement delimiter" option is enabled, **THE** SQL editor **SHALL** treat one or more consecutive blank lines as an implicit statement boundary for purposes of "execute statement at cursor."

### 1.4 Statement Splitting [DBV-SQL]

**THE** SQL editor **SHALL** parse the script buffer to identify individual statement boundaries, respecting string literals, quoted identifiers, comments, and nested blocks (BEGIN...END) so that delimiters inside these constructs are not treated as statement terminators.

### 1.5 Execute Current Statement [DBV-SQL]

**WHEN** the user invokes "Execute SQL Statement" (default: Ctrl+Enter), **THE** SQL editor **SHALL** identify the single statement at or nearest to the cursor position and execute only that statement against the active connection.

### 1.6 Execute Selected Text [DBV-SQL]

**WHEN** the user selects one or more characters and invokes "Execute SQL Statement," **THE** SQL editor **SHALL** execute only the selected text as the SQL statement, regardless of delimiter boundaries.

### 1.7 Execute Entire Script [DBV-SQL]

**WHEN** the user invokes "Execute SQL Script" (default: Alt+X), **THE** SQL editor **SHALL** split the entire script (or selected portion) into individual statements using the configured delimiter and execute them sequentially against the active connection.

### 1.8 Execute in New Result Tab [DBV-SQL]

**WHEN** the user invokes "Execute SQL in new tab" (default: Ctrl+\\), **THE** SQL editor **SHALL** execute the current statement and display its results in a new, separate result tab rather than replacing the existing result.

### 1.9 Script Save and Reuse [DBV-SQL]

**THE** SQL editor **SHALL** allow saving the current script to a file and **SHALL** maintain a list of recently opened SQL scripts accessible from a "Recent SQL Script" menu or dialog.

### 1.10 Multiple Editor Instances [DBV-SQL]

**THE** SQL editor **SHALL** support opening multiple SQL editor instances concurrently, each associated with the same or different database connections.

---

## 2. Syntax Highlighting Per Database Dialect

### 2.1 Dialect-Aware Highlighting [DBV-SQL]

**THE** SQL editor **SHALL** apply syntax highlighting rules specific to the SQL dialect of the active database connection (e.g., PostgreSQL, MySQL, T-SQL, PL/SQL, SQLite, DB2, Hive).

### 2.2 Keyword Highlighting [DBV-SQL]

**THE** SQL editor **SHALL** highlight reserved keywords, built-in functions, data types, and standard SQL clauses (SELECT, FROM, WHERE, JOIN, etc.) using distinct colours or styles defined by the active colour scheme.

### 2.3 String and Numeric Literal Highlighting [DBV-SQL]

**THE** SQL editor **SHALL** highlight string literals (single-quoted, double-quoted where applicable, dollar-quoted for PostgreSQL), numeric literals, and date/time literals in a distinct style.

### 2.4 Comment Highlighting [DBV-SQL]

**THE** SQL editor **SHALL** highlight single-line comments (`--`) and multi-line comments (`/* ... */`) in a distinct style, and **SHALL** support dialect-specific comment syntax (e.g., `#` for MySQL).

### 2.5 Identifier and Object Name Highlighting [DBV-SQL]

**THE** SQL editor **SHALL** highlight quoted identifiers (double-quoted, backtick-quoted for MySQL, square-bracket-quoted for T-SQL) distinctly from unquoted identifiers.

### 2.6 Semantic Object Highlighting [DBV-SQL]

**WHEN** semantic analysis is enabled, **THE** SQL editor **SHALL** highlight recognised database objects (tables, views, columns, functions) referenced in the script and **SHALL** mark unrecognised references with a problem indicator.

### 2.7 Operator and Punctuation Highlighting [DBV-SQL]

**THE** SQL editor **SHALL** highlight operators (=, <>, >=, <=, +, -, *, /, ||) and punctuation (parentheses, commas, semicolons) in a configurable style.

### 2.8 Procedure and Block Keyword Highlighting [DBV-SQL]

**THE** SQL editor **SHALL** highlight procedural block keywords (BEGIN, END, DECLARE, IF, ELSE, LOOP, FOR, WHILE, RETURN, EXCEPTION, RAISE) specific to the active dialect's procedural language (PL/pgSQL, PL/SQL, T-SQL, MySQL stored procedures).

### 2.9 Colour Scheme Configuration [DBV-SQL]

**THE** SQL editor **SHALL** allow the user to configure the colour and font style (bold, italic, underline) for each syntax token category through a preferences dialog.

---

## 3. Auto-Complete (Context-Aware Completion)

### 3.1 Completion Invocation [DBV-SQL]

**WHEN** the user presses the completion shortcut (default: Ctrl+Space) or types a trigger character (`.` after a schema/table alias), **THE** SQL editor **SHALL** display a completion popup listing relevant suggestions.

### 3.2 Table and View Name Completion [DBV-SQL]

**THE** auto-complete **SHALL** suggest table and view names from the database metadata, filtered by the current schema context and any typed prefix.

### 3.3 Column Name Completion [DBV-SQL]

**WHEN** the cursor follows a table alias or table name with a dot separator, **THE** auto-complete **SHALL** suggest column names belonging to that table or view.

### 3.4 Schema-Qualified Completion [DBV-SQL]

**WHEN** the user types a schema name followed by a dot, **THE** auto-complete **SHALL** suggest objects (tables, views, functions, types) within that schema.

### 3.5 SQL Keyword Completion [DBV-SQL]

**THE** auto-complete **SHALL** suggest SQL keywords and clauses appropriate to the current cursor context (e.g., suggest JOIN keywords after FROM clause, suggest WHERE/GROUP BY/ORDER BY after table reference).

### 3.6 Function Name Completion [DBV-SQL]

**THE** auto-complete **SHALL** suggest built-in and user-defined function names relevant to the active database dialect, including parameter signature hints.

### 3.7 Alias Resolution [DBV-SQL]

**THE** auto-complete **SHALL** resolve table aliases defined in the current query and provide column completions when the alias is used with a dot separator.

### 3.8 Multiple Completion Engines [DBV-SQL]

**THE** SQL editor **SHALL** support multiple completion engine modes (e.g., basic keyword completion, metadata-aware completion, and semantic/AI-assisted completion) and **SHALL** allow the user to select the active engine through preferences.

### 3.9 Completion Filtering and Ranking [DBV-SQL]

**THE** auto-complete popup **SHALL** filter suggestions as the user types additional characters and **SHALL** rank suggestions by relevance (exact prefix match first, then fuzzy match, with recently used items prioritised).

### 3.10 Auto-Close Brackets and Quotes [DBV-SQL]

**WHEN** the user types an opening bracket, parenthesis, or quote character, **THE** SQL editor **SHALL** automatically insert the corresponding closing character and place the cursor between them.

---

## 4. Query Execution

### 4.1 Execute Single Statement [DBV-SQL]

**WHEN** the user invokes "Execute SQL Statement," **THE** SQL editor **SHALL** send the statement to the database via the active connection, display a progress indicator during execution, and present results upon completion.

### 4.2 Execute Script Sequentially [DBV-SQL]

**WHEN** the user invokes "Execute SQL Script," **THE** SQL editor **SHALL** execute all statements in the script sequentially, reporting per-statement success/failure in an execution log panel.

### 4.3 Cancel Running Query [DBV-SQL]

**WHEN** a query is executing and the user invokes "Cancel," **THE** SQL editor **SHALL** attempt to cancel the in-progress query on the database server and return control to the editor.

### 4.4 Execution Timeout [DBV-SQL]

**IF** a configurable query timeout is set, **THEN THE** SQL editor **SHALL** automatically cancel queries that exceed the timeout duration and report the timeout to the user.

### 4.5 Transaction Control [DBV-SQL]

**THE** SQL editor **SHALL** indicate the current transaction mode (auto-commit ON/OFF) in the toolbar or status bar and **SHALL** provide explicit Commit and Rollback actions when auto-commit is disabled.

### 4.6 Execution Statistics [DBV-SQL]

**AFTER** each statement execution, **THE** SQL editor **SHALL** display execution statistics including: elapsed time, number of rows affected or returned, and any server-reported warnings.

### 4.7 Execution Log Panel [DBV-SQL]

**THE** SQL editor **SHALL** maintain an execution log panel that records all executed statements with timestamps, duration, row count, and success/error status for the current session.

### 4.8 Execute in Background [DBV-SQL]

**THE** SQL editor **SHALL** execute queries asynchronously so that the editor UI remains responsive during long-running query execution.

### 4.9 Native Script Execution [DBV-SQL]

**WHEN** the user invokes "Execute SQL Script natively," **THE** SQL editor **SHALL** launch the database's native command-line client (e.g., psql, mysql, sqlplus) with the current script and display the console output in a text panel.

---

## 5. Result Set Display

### 5.1 Tabular Grid Display [DBV-SQL]

**WHEN** a SELECT query completes successfully, **THE** SQL editor **SHALL** display the result set in a tabular grid with column headers matching the query's output columns.

### 5.2 Multiple Result Tabs [DBV-SQL]

**WHEN** multiple queries produce result sets, **THE** SQL editor **SHALL** display each result set in a separate result tab, allowing the user to navigate between them.

### 5.3 Single Tab Mode for Multiple Results [DBV-SQL]

**WHEN** the "show results in a single tab" option is enabled, **THE** SQL editor **SHALL** display multiple result sets vertically stacked within a single result panel.

### 5.4 Row Count Display [DBV-SQL]

**THE** result set panel **SHALL** display the total number of rows fetched and, where supported by the database, the total row count available.

### 5.5 Column Resizing and Reordering [DBV-SQL]

**THE** result grid **SHALL** allow the user to resize column widths by dragging column borders and **SHALL** support reordering columns by drag-and-drop of column headers.

### 5.6 Cell Value Display [DBV-SQL]

**THE** result grid **SHALL** display cell values with appropriate formatting: NULL values shown with a distinct visual indicator, truncated long text with tooltip or expansion, and binary/LOB data shown as a type indicator with size.

### 5.7 Result Set Pagination [DBV-SQL]

**IF** the result set exceeds a configurable fetch-size limit, **THEN THE** result panel **SHALL** fetch rows incrementally and provide navigation controls (next page, previous page, or scroll-based lazy loading).

### 5.8 Result Filtering and Sorting [DBV-SQL]

**THE** result grid **SHALL** support client-side column sorting (ascending/descending) and inline column filtering without re-executing the query.

### 5.9 Copy and Export from Results [DBV-SQL]

**THE** result grid **SHALL** allow the user to copy selected cells/rows to the clipboard and **SHALL** provide export options to save the result set as CSV, JSON, SQL INSERT statements, or other formats.

### 5.10 Result Set Details Panel [DBV-SQL]

**WHEN** the user opens the result details panel, **THE** SQL editor **SHALL** display execution statistics for the query including execution time, data volume, and processing cost metrics.

---

## 6. Explain Plan (Query Execution Plan)

### 6.1 Generate Execution Plan [DBV-SQL]

**WHEN** the user invokes "Explain Execution Plan" (default: Ctrl+Shift+E), **THE** SQL editor **SHALL** generate the execution plan for the current statement without executing it and display the plan in a result tab.

### 6.2 Tree/Table Plan Display [DBV-SQL]

**THE** execution plan view **SHALL** display the plan as a hierarchical tree or table, with each node representing a database operation (scan, join, sort, aggregate, etc.) and showing estimated cost, row count, and width.

### 6.3 Plan Node Details [DBV-SQL]

**WHEN** the user selects a plan node, **THE** execution plan view **SHALL** display detailed statistics for that node in a side or bottom panel, including operation type, estimated rows, estimated cost, actual time (if ANALYZE was used), and filter conditions.

### 6.4 Visual Graph Plan Display [DBV-SQL]

**THE** execution plan view **SHALL** provide an advanced graphical/visual representation of the plan as a directed graph, with nodes colour-coded or sized by relative cost, and edges showing data flow direction.

### 6.5 Cost-Based Node Highlighting [DBV-SQL]

**THE** visual plan display **SHALL** highlight the most expensive (highest-cost) nodes in the plan to draw attention to performance bottlenecks.

### 6.6 Plan Layout Options [DBV-SQL]

**THE** visual plan display **SHALL** support multiple layout orientations (horizontal left-to-right, vertical top-to-bottom) and **SHALL** allow the user to toggle between them.

### 6.7 EXPLAIN Options (Database-Specific) [DBV-SQL]

**WHEN** the database supports additional EXPLAIN parameters (e.g., ANALYSE, VERBOSE, COSTS, BUFFERS, TIMING for PostgreSQL), **THE** execution plan dialog **SHALL** allow the user to enable or disable these options before generating the plan.

### 6.8 Reevaluate Plan [DBV-SQL]

**THE** execution plan view **SHALL** provide a "Reevaluate" action that regenerates the plan for the same query, allowing the user to observe plan changes after index creation or statistics refresh.

### 6.9 View Plan Source [DBV-SQL]

**THE** execution plan view **SHALL** provide a "View Source" action that shows the original SQL statement on which the plan was generated.

### 6.10 Export Plan [DBV-SQL]

**THE** execution plan view **SHALL** allow exporting the plan as an image (PNG/SVG) or as structured data (JSON) for sharing or external analysis.

---

## 7. Parameter Binding

### 7.1 Dynamic Parameter Detection [DBV-SQL]

**WHEN** the SQL editor detects parameter placeholders in the script (e.g., `?`, `:name`, `$1`, `${variable}`, `@variable`), **THE** SQL editor **SHALL** identify them as bind parameters requiring values before execution.

### 7.2 Parameter Value Prompt Dialog [DBV-SQL]

**WHEN** execution is invoked on a statement containing unresolved parameters, **THE** SQL editor **SHALL** display a dialog prompting the user to provide values for each detected parameter, showing parameter name/position, expected type (if inferrable), and an input field.

### 7.3 Named Parameter Binding [DBV-SQL]

**THE** SQL editor **SHALL** support named parameters (e.g., `:employee_id`, `@salary`) where the same parameter name used multiple times in a query is bound to a single value provided once by the user.

### 7.4 Positional Parameter Binding [DBV-SQL]

**THE** SQL editor **SHALL** support positional parameters (e.g., `?`, `$1`, `$2`) where each placeholder is bound to a value by its ordinal position.

### 7.5 Parameter Type Specification [DBV-SQL]

**THE** parameter binding dialog **SHALL** allow the user to specify the SQL data type for each parameter value (e.g., VARCHAR, INTEGER, DATE, TIMESTAMP, BOOLEAN) to ensure correct type marshalling to the database driver.

### 7.6 Ignore Parameters Option [DBV-SQL]

**THE** parameter binding dialog **SHALL** provide an "Ignore" or "Skip" option that executes the statement as-is without substituting parameter values, for cases where the placeholders are part of the intended SQL (e.g., PREPARE statements).

### 7.7 Client-Side Variable Assignment [DBV-SQL]

**THE** SQL editor **SHALL** support client-side variable assignment using `@set variable = value` syntax, allowing variables to be defined once and reused across multiple statements in the same script without re-prompting.

### 7.8 Variables Panel [DBV-SQL]

**THE** SQL editor **SHALL** provide a Variables panel that displays all currently assigned variables and their values, and **SHALL** allow the user to view and modify variable assignments interactively.

### 7.9 Parameter Pattern Configuration [DBV-SQL]

**THE** SQL editor **SHALL** allow the user to configure which character patterns are recognised as parameter placeholders (e.g., enable/disable `?`, `:name`, `${name}`) through SQL processing preferences, to avoid false-positive detection in dialect-specific syntax.

### 7.10 Pre-Configured System Variables [DBV-SQL]

**THE** SQL editor **SHALL** provide pre-configured system variables (e.g., `${host}`, `${port}`, `${database}`, `${user}`, `${date}`, `${time}`) that resolve to connection metadata or current date/time values without user input.

---

## 8. SQL Code Editor Features (Supporting Capabilities)

### 8.1 SQL Formatting [DBV-SQL]

**WHEN** the user invokes "Format SQL" (default: Ctrl+Shift+F), **THE** SQL editor **SHALL** reformat the selected SQL text (or entire script if nothing is selected) according to configurable formatting rules including keyword case, indentation, and line wrapping.

### 8.2 SQL Templates [DBV-SQL]

**THE** SQL editor **SHALL** support SQL templates (code snippets) that can be inserted by typing a short abbreviation and pressing a trigger key, expanding into a full SQL statement with editable placeholder positions.

### 8.3 Code Folding [DBV-SQL]

**THE** SQL editor **SHALL** support code folding for procedural blocks (BEGIN...END, DECLARE...END, CREATE PROCEDURE/FUNCTION bodies) allowing the user to collapse and expand logical sections of the script.

### 8.4 Bracket Matching [DBV-SQL]

**WHEN** the cursor is positioned adjacent to a bracket or parenthesis, **THE** SQL editor **SHALL** highlight the matching bracket/parenthesis pair.

### 8.5 Problem Markers [DBV-SQL]

**WHEN** semantic analysis detects an error or warning in the script (e.g., reference to a non-existent table), **THE** SQL editor **SHALL** display a problem marker (underline, icon in gutter) at the location of the issue with a tooltip describing the problem.

### 8.6 Line Numbers and Gutter [DBV-SQL]

**THE** SQL editor **SHALL** display line numbers in a gutter and **SHALL** support gutter indicators for breakpoints, bookmarks, and problem markers.

### 8.7 Current Statement Highlight [DBV-SQL]

**THE** SQL editor **SHALL** visually indicate the boundaries of the current statement (the statement that would be executed by "Execute SQL Statement") using a background highlight or margin indicator.

### 8.8 Hyperlink Navigation [DBV-SQL]

**WHEN** the user Ctrl+clicks (or equivalent) on a database object name in the SQL script, **THE** SQL editor **SHALL** navigate to that object's definition in the schema browser or open its properties editor.

---

## References

- [DBeaver SQL Editor Wiki](https://github.com/dbeaver/dbeaver/wiki/SQL-Editor)
- [DBeaver SQL Execution](https://dbeaver.com/docs/dbeaver/SQL-Execution/)
- [DBeaver SQL Assist and Auto Complete](https://dbeaver.com/docs/dbeaver/SQL-Assist-and-Auto-Complete/)
- [DBeaver SQL Code Editor](https://dbeaver.com/docs/dbeaver/SQL-Code-Editor/)
- [DBeaver Query Execution Plan](https://github.com/dbeaver/dbeaver/wiki/Query-Execution-Plan)
- [DBeaver SQL Templates](https://dbeaver.com/docs/dbeaver/SQL-Templates/)
- [DBeaver SQL Formatting](https://dbeaver.com/docs/dbeaver/SQL-Formatting/)
- [DBeaver Variables Panel](https://dbeaver.com/docs/dbeaver/Variables-panel/)
- [DBeaver Client Side Scripting](https://github.com/dbeaver/dbeaver/wiki/Client-Side-Scripting)
- [DBeaver Result Details Panel](https://dbeaver.com/docs/dbeaver/Result-Details-Panel/)

Content was rephrased for compliance with licensing restrictions.
