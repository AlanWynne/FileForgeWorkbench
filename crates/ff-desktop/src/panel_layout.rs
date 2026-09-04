//! Panel layout contract types.
//!
//! Defines the structural contracts for ISPF-style data entry panels and
//! list panels, ensuring conformance with the ISPF layout model.
//!
//! Validates: Requirement 19.5, 19.6

/// Elements required in a data entry panel.
///
/// A data entry panel conforms to the ISPF layout model:
/// title line, command field, labelled ===> input fields, function key bar.
///
/// Validates: Requirement 19.5
#[derive(Debug, Clone, PartialEq, Eq)]
#[allow(dead_code)]
pub struct DataEntryPanelLayout {
    /// Panel title text.
    pub title: String,
    /// Whether the command field is present.
    pub has_command_field: bool,
    /// Named input fields with their labels.
    pub fields: Vec<DataEntryField>,
    /// Whether the function key bar is shown.
    pub has_key_bar: bool,
}

/// A single labelled input field in a data entry panel.
#[derive(Debug, Clone, PartialEq, Eq)]
#[allow(dead_code)]
pub struct DataEntryField {
    /// Field label (e.g. "Dataset Name").
    pub label: String,
    /// Whether the field is required.
    pub required: bool,
}

/// Elements required in a list panel.
///
/// A list panel conforms to the ISPF layout model:
/// title line, command field, filter information lines, NP column,
/// scrollable rows.
///
/// Validates: Requirement 19.6
#[derive(Debug, Clone, PartialEq, Eq)]
#[allow(dead_code)]
pub struct ListPanelLayout {
    /// Panel title text.
    pub title: String,
    /// Whether the command field is present.
    pub has_command_field: bool,
    /// Whether filter information lines are shown.
    pub has_filter_lines: bool,
    /// Whether the NP (action) column is present.
    pub has_np_column: bool,
    /// Column definitions for the list.
    pub columns: Vec<ListColumn>,
    /// Whether the list is scrollable.
    pub scrollable: bool,
}

/// A column definition in a list panel.
#[derive(Debug, Clone, PartialEq, Eq)]
#[allow(dead_code)]
pub struct ListColumn {
    /// Column header label.
    pub label: String,
    /// Minimum display width in characters.
    pub min_width: usize,
}

impl DataEntryPanelLayout {
    /// Create a minimal data entry panel layout.
    #[allow(dead_code)]
    pub fn new(title: impl Into<String>) -> Self {
        Self {
            title: title.into(),
            has_command_field: true,
            fields: Vec::new(),
            has_key_bar: true,
        }
    }

    /// Add a field to the layout.
    #[allow(dead_code)]
    pub fn with_field(mut self, label: impl Into<String>, required: bool) -> Self {
        self.fields.push(DataEntryField {
            label: label.into(),
            required,
        });
        self
    }
}

impl ListPanelLayout {
    /// Create a minimal list panel layout.
    #[allow(dead_code)]
    pub fn new(title: impl Into<String>) -> Self {
        Self {
            title: title.into(),
            has_command_field: true,
            has_filter_lines: false,
            has_np_column: false,
            columns: Vec::new(),
            scrollable: true,
        }
    }

    /// Add a column to the layout.
    #[allow(dead_code)]
    pub fn with_column(mut self, label: impl Into<String>, min_width: usize) -> Self {
        self.columns.push(ListColumn {
            label: label.into(),
            min_width,
        });
        self
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn data_entry_panel_has_required_elements() {
        // Validates: Requirement 19.5
        let panel = DataEntryPanelLayout::new("Dataset Allocation")
            .with_field("Dataset Name", true)
            .with_field("Volume Serial", false);
        assert!(panel.has_command_field);
        assert!(panel.has_key_bar);
        assert_eq!(panel.fields.len(), 2);
        assert_eq!(panel.fields[0].label, "Dataset Name");
        assert!(panel.fields[0].required);
        assert!(!panel.fields[1].required);
    }

    #[test]
    fn list_panel_has_required_elements() {
        // Validates: Requirement 19.6
        let panel = ListPanelLayout::new("Dataset List")
            .with_column("Name", 44)
            .with_column("Type", 8);
        assert!(panel.has_command_field);
        assert!(panel.scrollable);
        assert_eq!(panel.columns.len(), 2);
        assert_eq!(panel.columns[0].label, "Name");
        assert_eq!(panel.columns[0].min_width, 44);
    }

    #[test]
    fn list_panel_default_no_filter_no_np() {
        // Validates: Requirement 19.6 -- filter lines and NP column are optional
        let panel = ListPanelLayout::new("Simple List");
        assert!(!panel.has_filter_lines);
        assert!(!panel.has_np_column);
    }
}
