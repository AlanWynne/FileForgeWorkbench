# DBeaver ER Diagram Requirements Research

> **Source:** DBeaver public documentation, GitHub wiki, and blog posts.
> **Tag:** [DBV-ER]
> **Purpose:** Extract requirements for visual ER diagram panel (entity boxes, relationship lines, cardinality notation), relationship display, auto-layout algorithms, diagram scope, diagram customization, export to image/PDF, diagram persistence, print support, and entity filtering to inform FileForgeWorkbench database-tool sub-project.

---

## 1. Visual ER Diagram Panel

### 1.1 Diagram Editor Canvas [DBV-ER]

1. **[DBV-ER-001]** THE system SHALL provide an ER Diagram editor panel that renders database entities as rectangular boxes and relationships as connecting lines on a scrollable, zoomable canvas.

2. **[DBV-ER-002]** WHEN a user double-clicks a table or view in the Database Navigator and switches to the Diagram tab, THE system SHALL display the entity and its directly related tables (neighbours connected by foreign keys) in the diagram editor.

3. **[DBV-ER-003]** WHEN a user double-clicks a schema name (or the Tables container node) in the Database Navigator, THE system SHALL display a schema-level diagram showing all tables and views within that schema and their inter-relationships.

4. **[DBV-ER-004]** THE system SHALL render each entity (table or view) as a box containing the entity name in a header region and a column list in the body region.

5. **[DBV-ER-005]** THE system SHALL distinguish primary key columns visually within entity boxes by displaying a key icon or distinct indicator beside each primary key column name.

6. **[DBV-ER-006]** THE system SHALL distinguish foreign key columns visually within entity boxes by displaying a foreign key icon or distinct indicator beside each foreign key column name.

7. **[DBV-ER-007]** THE system SHALL support selecting diagram elements by clicking individual objects, Shift-clicking to extend selection, or dragging a selection rectangle across the canvas to select multiple elements.

8. **[DBV-ER-008]** WHEN a user selects a table, THE system SHALL visually highlight the selected table and all relationship connections to and from that table.

9. **[DBV-ER-009]** WHEN a user selects a column within an entity box, THE system SHALL highlight the related key columns and foreign key connections associated with that column.

10. **[DBV-ER-010]** WHEN multiple tables are selected via drag selection, THE system SHALL highlight the connections between the selected tables to show their inter-relationships.

### 1.2 Palette Panel [DBV-ER]

11. **[DBV-ER-011]** THE system SHALL provide a Palette panel alongside the diagram editor containing tools for: Select (pointer), Pan Diagram (hand tool), Connection (create relationship), and Note (add annotation).

12. **[DBV-ER-012]** WHEN the user selects the "Connection" tool from the Palette and clicks a source table followed by a target table, THE system SHALL initiate creation of a relationship between the two tables, prompting for column selection to define the foreign key.

13. **[DBV-ER-013]** WHEN the user selects the "Note" tool and clicks on the canvas, THE system SHALL create a text annotation element at the clicked position that the user can edit by double-clicking.

### 1.3 Diagram Toolbar [DBV-ER]

14. **[DBV-ER-014]** THE system SHALL provide a diagram toolbar with the following actions: Refresh, Save, Revert, Edit Mode toggle, Keep Layout toggle, Zoom level dropdown, Zoom In, Zoom Out, Auto-arrange layout, Toggle Grid, Toggle Hand Tool, Properties, Outline (mini-map), Print, Save diagram as (export), Configuration (preferences), and Search.

15. **[DBV-ER-015]** THE system SHALL provide a zoom level dropdown in the toolbar allowing the user to set a specific percentage zoom (e.g., 25%, 50%, 75%, 100%, 150%, 200%) and Zoom In/Zoom Out buttons for incremental adjustment.

16. **[DBV-ER-016]** THE system SHALL provide an Outline mini-map panel that displays a reduced-scale overview of the entire diagram, enabling the user to navigate large diagrams by clicking or dragging within the mini-map.

---

## 2. Relationship Display

### 2.1 Foreign Key Lines [DBV-ER]

17. **[DBV-ER-017]** THE system SHALL render each foreign key relationship as a line connecting the referencing entity (child) to the referenced entity (parent).

18. **[DBV-ER-018]** WHEN a foreign key column is nullable (allows NULL), THE system SHALL render the relationship line as a dashed line to distinguish optional relationships from mandatory ones.

19. **[DBV-ER-019]** WHEN a foreign key column is NOT NULL (mandatory), THE system SHALL render the relationship line as a solid line.

20. **[DBV-ER-020]** WHEN the user clicks on a relationship connection line, THE system SHALL display the detailed relationship information including the foreign key name, referencing columns, and referenced columns.

### 2.2 Cardinality Annotations [DBV-ER]

21. **[DBV-ER-021]** THE system SHALL display cardinality indicators at each end of a relationship line to communicate the nature of the relationship (one-to-one, one-to-many, many-to-many).

22. **[DBV-ER-022]** WHEN using IDEF1X notation, THE system SHALL render cardinality using solid circles (for the "many" side) and no marker (for the "one" side), with identifying relationships shown as solid lines connected to a rounded child entity corner.

23. **[DBV-ER-023]** WHEN using Crow's Foot notation, THE system SHALL render cardinality using crow's foot (fork) symbols for the "many" side, a single line for the "one" side, a circle for "zero" (optional), and a bar for "one" (mandatory).

24. **[DBV-ER-024]** WHEN using Bachman notation, THE system SHALL render cardinality using arrows to indicate the direction of the relationship, with appropriate symbols for one-to-one and one-to-many relationships.

25. **[DBV-ER-025]** THE system SHALL visually distinguish identifying relationships (child entity's primary key includes the parent's foreign key) from non-identifying relationships through line style or entity corner rendering appropriate to the selected notation.

### 2.3 Connection Routing [DBV-ER]

26. **[DBV-ER-026]** THE system SHALL support a "Shortest paths" routing type (default) that calculates and displays the shortest possible lines connecting entities for a compact diagram representation.

27. **[DBV-ER-027]** THE system SHALL support an "Orthogonal paths" routing type that uses right-angled (rectilinear) lines for clear, structured layouts showing direct relationships between tables and columns.

28. **[DBV-ER-028]** WHEN the user selects a routing type from the context menu or toolbar configuration, THE system SHALL re-route all connection lines in the diagram according to the selected algorithm.

### 2.4 Virtual Relationships [DBV-ER]

29. **[DBV-ER-029]** THE system SHALL support creation of virtual (logical) relationships in custom diagrams that are stored as virtual foreign keys and do not modify the physical database schema.

30. **[DBV-ER-030]** WHEN the user creates a relationship in a custom diagram, THE system SHALL store the relationship definition locally as a virtual foreign key, persisted with the diagram file.

---

## 3. Auto-Layout Algorithms

### 3.1 Automatic Arrangement [DBV-ER]

31. **[DBV-ER-031]** THE system SHALL provide an "Auto-arrange layout" action (accessible from toolbar and context menu) that automatically repositions all entities into a compact, readable layout minimising connection crossings.

32. **[DBV-ER-032]** WHEN the user invokes auto-arrange, THE system SHALL reposition entities to reduce overlapping lines, minimise total connection length, and group related entities near each other.

33. **[DBV-ER-033]** THE system SHALL preserve the auto-arranged layout only if the user explicitly enables "Keep layout"; otherwise, the layout SHALL revert to the computed arrangement on editor reopen.

### 3.2 Grid Alignment [DBV-ER]

34. **[DBV-ER-034]** THE system SHALL provide a configurable grid overlay on the diagram canvas with adjustable grid width and grid height (cell size in pixels).

35. **[DBV-ER-035]** THE system SHALL provide a "Toggle Grid" action that shows or hides the grid lines on the diagram canvas.

36. **[DBV-ER-036]** THE system SHALL provide a "Snap to Grid" option that, when enabled, constrains entity movement to grid cell boundaries for aligned placement.

### 3.3 Manual Placement [DBV-ER]

37. **[DBV-ER-037]** THE system SHALL allow the user to manually reposition any entity on the diagram canvas by dragging it to a new location.

38. **[DBV-ER-038]** THE system SHALL allow the user to change the z-order (stacking order) of entities via "Bring to front" and "Send to back" context menu actions.

39. **[DBV-ER-039]** THE system SHALL provide a Pan Diagram tool (hand tool) that allows the user to scroll the diagram viewport by dragging without selecting or moving entities.

---

## 4. Diagram Scope

### 4.1 Single Table and Neighbours [DBV-ER]

40. **[DBV-ER-040]** WHEN the user opens a diagram for a single table, THE system SHALL display that table plus all tables directly connected to it via foreign key relationships (immediate neighbours).

41. **[DBV-ER-041]** THE system SHALL provide a "View Diagram" context menu action on any selected table that opens a new diagram focused on that table and its related objects.

### 4.2 Selected Tables (Custom Diagram) [DBV-ER]

42. **[DBV-ER-042]** THE system SHALL allow creation of custom diagrams where the user explicitly selects which tables to include by dragging them from the Database Navigator onto the diagram canvas.

43. **[DBV-ER-043]** THE system SHALL allow combining tables from different database connections and different database types (e.g., PostgreSQL and MySQL tables) within a single custom diagram.

44. **[DBV-ER-044]** WHEN a user creates a custom diagram via the "Create New Diagram" wizard, THE system SHALL allow optional pre-selection of initial tables to include in the diagram.

### 4.3 Entire Schema [DBV-ER]

45. **[DBV-ER-045]** WHEN the user opens a schema-level diagram, THE system SHALL render all tables and views belonging to that schema along with all foreign key relationships between them.

46. **[DBV-ER-046]** THE system SHALL provide a preference option "Show views" that controls whether database views are displayed on schema-level diagrams (default: shown).

47. **[DBV-ER-047]** THE system SHALL provide a preference option "Show partitions" that controls whether table partitions are displayed on the diagram.

### 4.4 Diagram Refresh [DBV-ER]

48. **[DBV-ER-048]** THE system SHALL provide a "Refresh" toolbar action that reloads the diagram to reflect any external changes made to the database schema since the diagram was opened.

---

## 5. Diagram Customization

### 5.1 Notation Styles [DBV-ER]

49. **[DBV-ER-049]** THE system SHALL support IDEF1X notation (default) for rendering entity relationships, emphasising detailed entity constraints and identifying/non-identifying relationship distinction.

50. **[DBV-ER-050]** THE system SHALL support Crow's Foot notation for rendering entity relationships, using fork symbols for "many" and bar/circle for "one"/"zero" cardinality at line endpoints.

51. **[DBV-ER-051]** THE system SHALL support Bachman notation for rendering entity relationships, using arrow-based representation of data structure relationships.

52. **[DBV-ER-052]** WHEN the user changes the notation type via the context menu (Notation submenu) or diagram preferences, THE system SHALL re-render all relationship lines and cardinality indicators according to the newly selected notation style.

### 5.2 Colour Themes and Customization [DBV-ER]

53. **[DBV-ER-053]** THE system SHALL allow the user to assign a custom background colour to any entity box via a "Set color" context menu action.

54. **[DBV-ER-054]** THE system SHALL allow the user to remove a previously assigned custom colour from an entity via a "Remove color" context menu action, reverting it to the default appearance.

55. **[DBV-ER-055]** THE system SHALL provide a "Colorize other connections" preference that, when enabled, applies colour highlighting to connections from non-selected tables to distinguish them visually.

56. **[DBV-ER-056]** THE system SHALL provide a "Colorize other schemas" preference that, when enabled, applies distinct colours to tables belonging to schemas other than the current schema to visually distinguish cross-schema references.

### 5.3 Attribute Styles [DBV-ER]

57. **[DBV-ER-057]** THE system SHALL provide a "Show icons" attribute style option that displays icons indicating column types and roles (PK, FK, index) within entity boxes.

58. **[DBV-ER-058]** THE system SHALL provide a "Show data types" attribute style option that displays column data types next to column names within entity boxes.

59. **[DBV-ER-059]** THE system SHALL provide a "Show nullability" attribute style option that displays NULL or NOT NULL markers for each column.

60. **[DBV-ER-060]** THE system SHALL provide a "Show comments" attribute style option that displays column comments (if available in the database metadata) within entity boxes.

61. **[DBV-ER-061]** THE system SHALL provide a "Show fully qualified names" attribute style option that displays schema-qualified table names in entity box headers.

62. **[DBV-ER-062]** THE system SHALL provide a "Sort columns alphabetically" attribute style option that orders columns alphabetically by name within entity boxes instead of ordinal position.

---

## 6. Entity Filtering (Attribute Visibility)

### 6.1 Column Visibility Levels [DBV-ER]

63. **[DBV-ER-063]** THE system SHALL provide an "All" attribute visibility mode that shows all columns for tables in the diagram.

64. **[DBV-ER-064]** THE system SHALL provide an "Any keys" attribute visibility mode that shows only primary key and foreign key columns, hiding all non-key columns.

65. **[DBV-ER-065]** THE system SHALL provide a "Primary key" attribute visibility mode that shows only primary key columns, hiding all other columns including foreign keys.

66. **[DBV-ER-066]** THE system SHALL provide a "None" attribute visibility mode that hides all columns, displaying only the entity header (table name).

67. **[DBV-ER-067]** THE system SHALL allow applying attribute visibility settings either globally (to all tables in the diagram) or individually (to the currently selected table only).

### 6.2 Content Filtering [DBV-ER]

68. **[DBV-ER-068]** THE system SHALL provide a preference option to control whether database views are included in schema diagrams (Show views toggle).

69. **[DBV-ER-069]** THE system SHALL provide a preference option to control whether table partitions are included in diagrams (Show partitions toggle).

---

## 7. Export to Image and GraphML

### 7.1 Image Export Formats [DBV-ER]

70. **[DBV-ER-070]** THE system SHALL provide a "Save diagram as…" action (accessible from the toolbar and context menu) that exports the current diagram to an image file.

71. **[DBV-ER-071]** THE system SHALL support exporting diagrams to PNG raster image format.

72. **[DBV-ER-072]** THE system SHALL support exporting diagrams to GIF raster image format.

73. **[DBV-ER-073]** THE system SHALL support exporting diagrams to BMP raster image format.

74. **[DBV-ER-074]** THE system SHALL support exporting diagrams to SVG vector image format for scalable, resolution-independent output.

75. **[DBV-ER-075]** THE system SHALL support exporting diagrams to GraphML format for interoperability with graph analysis tools and other diagramming applications.

### 7.2 Export Behaviour [DBV-ER]

76. **[DBV-ER-076]** WHEN the user invokes "Save diagram as…", THE system SHALL present a file chooser dialog allowing the user to specify the output file path, file name, and format.

77. **[DBV-ER-077]** THE system SHALL export the complete diagram canvas content (all visible entities and connections) at the current visual fidelity, preserving colours, notation, and attribute visibility settings in the exported image.

---

## 8. Diagram Persistence

### 8.1 Custom Diagram Save/Load [DBV-ER]

78. **[DBV-ER-078]** THE system SHALL persist custom diagrams as project resources that can be saved, closed, and reopened from the Project Explorer under a "Diagrams" node.

79. **[DBV-ER-079]** WHEN a user creates a custom diagram, THE system SHALL save the diagram definition including: entity list, entity positions, virtual relationships, notes, colour assignments, and display settings.

80. **[DBV-ER-080]** WHEN a user reopens a previously saved custom diagram, THE system SHALL restore all entities, positions, relationships, notes, and visual settings to their saved state.

### 8.2 Layout Persistence [DBV-ER]

81. **[DBV-ER-081]** THE system SHALL provide a "Keep layout" toggle that, when enabled, saves the current manual or auto-arranged entity positions locally so they are preserved when the diagram editor is closed and reopened.

82. **[DBV-ER-082]** WHEN "Keep layout" is disabled and the user closes and reopens a schema or table diagram, THE system SHALL recompute entity positions using the auto-layout algorithm rather than restoring previous positions.

### 8.3 Diagram State [DBV-ER]

83. **[DBV-ER-083]** THE system SHALL persist diagram preferences (notation type, routing type, attribute visibility, attribute styles, grid settings) so that user customizations are restored across application sessions.

84. **[DBV-ER-084]** THE system SHALL provide a "Revert" action that cancels all unsaved changes made in edit mode, restoring the diagram to its last saved state.

---

## 9. Print Support

### 9.1 Print Action [DBV-ER]

85. **[DBV-ER-085]** THE system SHALL provide a "Print" toolbar action that sends the current diagram to the system printer, rendering entities, connections, and annotations at print resolution.

86. **[DBV-ER-086]** THE system SHALL provide print preferences for configuring: page mode (e.g., Tile for multi-page diagrams), and margins (top, bottom, left, right) in pixels.

87. **[DBV-ER-087]** WHEN the diagram exceeds a single printed page, THE system SHALL tile the diagram across multiple pages with alignment markers to assist reassembly.

---

## 10. Edit Mode (Schema Editing via Diagram)

### 10.1 Visual Schema Modification [DBV-ER]

88. **[DBV-ER-088]** THE system SHALL provide an Edit Mode toggle that switches the diagram from read-only exploration mode to schema editing mode.

89. **[DBV-ER-089]** WHEN Edit Mode is enabled, THE system SHALL allow the user to create new tables, add columns to existing tables, create foreign keys, and create indexes directly from the diagram context menu.

90. **[DBV-ER-090]** WHEN the user performs schema modifications in Edit Mode, THE system SHALL generate an SQL script (DDL statements) reflecting all changes, which the user can review before executing against the database.

91. **[DBV-ER-091]** THE system SHALL provide Undo and Redo actions for reverting or re-applying schema editing actions within Edit Mode before the SQL script is executed.

---

## 11. Search and Navigation

### 11.1 Diagram Search [DBV-ER]

92. **[DBV-ER-092]** THE system SHALL provide a search function (accessible via toolbar or Ctrl+F) that allows the user to search for table names and column names within the diagram.

93. **[DBV-ER-093]** WHEN the user enters a search term, THE system SHALL highlight matching tables and columns on the canvas and scroll the viewport to bring the first match into view.

### 11.2 SQL Generation [DBV-ER]

94. **[DBV-ER-094]** THE system SHALL provide a "Generate SQL" context menu action that produces SQL statements (DDL) for the selected tables or relationships in the diagram.

### 11.3 Keyboard Accessibility [DBV-ER]

95. **[DBV-ER-095]** THE system SHALL support keyboard navigation within diagrams, allowing users to move between tables, select elements, and invoke actions without requiring a mouse.

---

## Summary

| Category | Requirement Count | IDs |
|----------|------------------|-----|
| Visual ER Diagram Panel | 16 | DBV-ER-001 to 016 |
| Relationship Display | 14 | DBV-ER-017 to 030 |
| Auto-Layout Algorithms | 9 | DBV-ER-031 to 039 |
| Diagram Scope | 9 | DBV-ER-040 to 048 |
| Diagram Customization | 14 | DBV-ER-049 to 062 |
| Entity Filtering | 7 | DBV-ER-063 to 069 |
| Export to Image and GraphML | 8 | DBV-ER-070 to 077 |
| Diagram Persistence | 7 | DBV-ER-078 to 084 |
| Print Support | 3 | DBV-ER-085 to 087 |
| Edit Mode (Schema Editing) | 4 | DBV-ER-088 to 091 |
| Search and Navigation | 4 | DBV-ER-092 to 095 |
| **Total** | **95** | |

---

## References

- [DBeaver ER Diagrams documentation](https://dbeaver.com/docs/dbeaver/ER-Diagrams/) (content rephrased for compliance with licensing restrictions)
- [DBeaver How to use Diagrams documentation](https://dbeaver.com/docs/dbeaver/How-to-use-Diagrams/) (content rephrased for compliance with licensing restrictions)
- [DBeaver Custom Diagrams documentation](https://dbeaver.com/docs/dbeaver/Custom-Diagrams/) (content rephrased for compliance with licensing restrictions)
- [DBeaver Edit Mode documentation](https://dbeaver.com/docs/dbeaver/Edit-mode/) (content rephrased for compliance with licensing restrictions)
- [DBeaver GitHub Wiki: ER Diagrams](https://github.com/dbeaver/dbeaver/wiki/ER-Diagrams) (content rephrased for compliance with licensing restrictions)
- [DBeaver GitHub Wiki: Database Structure Diagrams](https://github.com/dbeaver/dbeaver/wiki/Database-Structure-Diagrams) (content rephrased for compliance with licensing restrictions)
- [DBeaver Blog: Two ways to use ERD in DBeaver](https://dbeaver.com/2022/06/30/two-ways-to-use-erd-in-dbeaver/) (content rephrased for compliance with licensing restrictions)
