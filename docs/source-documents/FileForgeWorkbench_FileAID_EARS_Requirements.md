# FileForgeWorkbench Requirements Derived from File-AID Behaviour

## Purpose
This specification captures EARS-style requirements inspired by common File-AID capabilities for mainframe dataset management, browsing, editing, comparison, reformatting, selection criteria, layouts, and utilities.

## Dataset Browsing

REQ-FAID-001
WHEN a user selects a dataset for browsing, THE SYSTEM SHALL display the dataset contents in read-only mode.

REQ-FAID-002
WHEN a dataset record  layout is available, THE SYSTEM SHALL present field names and formatted field values.

REQ-FAID-003
WHERE a record layout is not available, THE SYSTEM SHALL allow the dataset to be viewed in character and unformatted modes.

REQ-FAID-004
WHEN a user navigates through records, THE SYSTEM SHALL support forward, backward, first, and last record navigation.

REQ-FAID-005
THE SYSTEM SHALL support sequential files, partitioned datasets, VSAM datasets, and database-backed data sources.

## Dataset Editing

REQ-FAID-006
WHEN a user has update authority, THE SYSTEM SHALL permit record insertion, modification, duplication, and deletion.

REQ-FAID-007
IF a user does not possess update authority THEN THE SYSTEM SHALL prevent data modifications.

REQ-FAID-008
WHEN a record is modified, THE SYSTEM SHALL validate the modified data against the active record layout.

REQ-FAID-009
THE SYSTEM SHALL provide an undo capability for changes made during the current editing session.

## Record Layout Support

REQ-FAID-010
WHEN a COBOL, PL/I, or compatible layout definition is supplied, THE SYSTEM SHALL interpret records according to the layout definition.

REQ-FAID-011
THE SYSTEM SHALL support layout cross-reference mappings that automatically associate layouts with datasets.

REQ-FAID-012
WHEN multiple record types exist within a dataset, THE SYSTEM SHALL support conditional layout selection.

## Search, Filter and Selection Criteria

REQ-FAID-013
WHEN a user specifies selection criteria, THE SYSTEM SHALL display only matching records.

REQ-FAID-014
THE SYSTEM SHALL support filtering by field value, range, wildcard pattern, and logical expressions.

REQ-FAID-015
WHEN a search is performed, THE SYSTEM SHALL locate matching records without requiring manual record traversal.

## Data Presentation

REQ-FAID-016
THE SYSTEM SHALL support formatted, vertical formatted, character, hexadecimal, and unformatted viewing modes.

REQ-FAID-017
WHEN requested by the user, THE SYSTEM SHALL hide, show, reorder, and freeze columns or fields.

REQ-FAID-018
THE SYSTEM SHALL support bookmarks for frequently referenced records.

## Compare Capability

REQ-FAID-019
WHEN two datasets are selected for comparison, THE SYSTEM SHALL identify added, removed, and modified records.

REQ-FAID-020
THE SYSTEM SHALL allow comparison results to be exported.

## Reformat and Conversion

REQ-FAID-021
WHEN a target layout is provided, THE SYSTEM SHALL convert source data into the target format.

REQ-FAID-022
THE SYSTEM SHALL support copy and reformat operations between supported dataset types.

## Data Extraction and Test Data

REQ-FAID-023
WHEN selection criteria are supplied, THE SYSTEM SHALL extract matching subsets into a new dataset.

REQ-FAID-024
THE SYSTEM SHALL support creation of representative test datasets from production-like source data.

REQ-FAID-025
IF sensitive information is present THEN THE SYSTEM SHALL support masking or anonymisation rules during extraction.

## Printing and Export

REQ-FAID-026
WHEN a user requests output, THE SYSTEM SHALL export records to supported file formats.

REQ-FAID-027
THE SYSTEM SHALL support printable dataset reports.

## Security and Audit

REQ-FAID-028
THE SYSTEM SHALL enforce dataset permissions before allowing browse, edit, copy, compare, or delete operations.

REQ-FAID-029
WHEN a dataset modification occurs, THE SYSTEM SHALL record the user, timestamp, dataset identifier, and change details in an audit log.

REQ-FAID-030
IF a protected dataset is accessed THEN THE SYSTEM SHALL authenticate the user before access is granted.

## FileForgeWorkbench Enhancements

REQ-FAID-031
THE SYSTEM SHALL expose all dataset operations through both graphical and command-driven interfaces.

REQ-FAID-032
WHEN an AI assistant is enabled, THE SYSTEM SHALL allow the assistant to explain dataset structures and assist with filter creation without modifying data unless explicitly authorised.

REQ-FAID-033
THE SYSTEM SHALL provide plugin extension points for additional dataset types and record layout parsers.

REQ-FAID-034
THE SYSTEM SHALL maintain compatibility with emulated DDNAME, PDS, VSAM, GDG, and mainframe-style file abstractions used by FileForgeWorkbench.

## References
These requirements were derived from publicly documented File-AID capabilities including browse, edit, compare, reformat, selection criteria, record layouts, cross references, data extraction, and security controls as described in File-AID tutorials and BMC File-AID documentation.
