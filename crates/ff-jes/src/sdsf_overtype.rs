//! SDSF overtype field support.
//!
//! Implements Requirement 18 AC 18.1-18.4:
//!   - Visual distinction for overtypeable fields (AC 18.1)
//!   - Direct overtype: apply change on Enter (AC 18.2)
//!   - Command-line overtype syntax: `<field> <value>` (AC 18.3)
//!   - Overtype Extension pop-up for values exceeding column width (AC 18.4)

// === OvertypeField ===========================================================

/// Whether a field is overtypeable or read-only.
///
/// Addresses: Requirement 18 AC 18.1
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum FieldKind {
    /// Field can be edited by the user.
    Overtypeable,
    /// Field is display-only.
    ReadOnly,
}

/// A single field definition in an SDSF panel row.
///
/// Addresses: Requirement 18 AC 18.1
#[derive(Debug, Clone)]
pub struct OvertypeField {
    /// Column name (uppercase).
    pub name: String,
    /// Current display value.
    pub value: String,
    /// Whether the field can be overtyped.
    pub kind: FieldKind,
    /// Maximum display width (characters).
    pub display_width: usize,
}

impl OvertypeField {
    pub fn overtypeable(name: &str, value: &str, display_width: usize) -> Self {
        Self {
            name: name.to_uppercase(),
            value: value.to_string(),
            kind: FieldKind::Overtypeable,
            display_width,
        }
    }

    pub fn read_only(name: &str, value: &str, display_width: usize) -> Self {
        Self {
            name: name.to_uppercase(),
            value: value.to_string(),
            kind: FieldKind::ReadOnly,
            display_width,
        }
    }

    /// Returns true if the current value exceeds the display width.
    ///
    /// Addresses: Requirement 18 AC 18.4
    pub fn needs_extension_popup(&self) -> bool {
        self.kind == FieldKind::Overtypeable && self.value.len() > self.display_width
    }
}

// === OvertypeResult ==========================================================

/// Result of applying an overtype operation.
#[derive(Debug, Clone, PartialEq)]
pub enum OvertypeResult {
    /// Value applied successfully.
    Applied { field: String, new_value: String },
    /// Field is read-only; change rejected.
    ReadOnly(String),
    /// Field name not found in the row.
    FieldNotFound(String),
}

// === OvertypeRow =============================================================

/// A row of fields that supports overtype editing.
///
/// Addresses: Requirement 18 AC 18.1-18.4
#[derive(Debug, Clone, Default)]
pub struct OvertypeRow {
    fields: Vec<OvertypeField>,
}

impl OvertypeRow {
    pub fn new(fields: Vec<OvertypeField>) -> Self {
        Self { fields }
    }

    /// Apply a direct overtype: set the named field to new_value.
    ///
    /// Addresses: Requirement 18 AC 18.2
    pub fn apply_direct(&mut self, field_name: &str, new_value: &str) -> OvertypeResult {
        let key = field_name.to_uppercase();
        match self.fields.iter_mut().find(|f| f.name == key) {
            None => OvertypeResult::FieldNotFound(key),
            Some(f) if f.kind == FieldKind::ReadOnly => OvertypeResult::ReadOnly(key),
            Some(f) => {
                f.value = new_value.to_string();
                OvertypeResult::Applied {
                    field: key,
                    new_value: new_value.to_string(),
                }
            }
        }
    }

    /// Get a field by name.
    pub fn get(&self, field_name: &str) -> Option<&OvertypeField> {
        let key = field_name.to_uppercase();
        self.fields.iter().find(|f| f.name == key)
    }

    /// Returns all fields that need an extension pop-up.
    ///
    /// Addresses: Requirement 18 AC 18.4
    pub fn fields_needing_extension(&self) -> Vec<&OvertypeField> {
        self.fields
            .iter()
            .filter(|f| f.needs_extension_popup())
            .collect()
    }
}

// === CommandLineOvertype =====================================================

/// Parsed command-line overtype: `<field-name> <value>`.
///
/// Addresses: Requirement 18 AC 18.3
#[derive(Debug, Clone, PartialEq)]
pub struct CommandLineOvertype {
    pub field_name: String,
    pub value: String,
}

impl CommandLineOvertype {
    /// Parse a command-line overtype string.
    /// Returns None if the input does not match the `<field> <value>` pattern.
    ///
    /// Addresses: Requirement 18 AC 18.3
    pub fn parse(input: &str) -> Option<Self> {
        let trimmed = input.trim();
        let mut parts = trimmed.splitn(2, char::is_whitespace);
        let field = parts.next()?.trim();
        let value = parts.next()?.trim();
        if field.is_empty() || value.is_empty() {
            return None;
        }
        Some(Self {
            field_name: field.to_uppercase(),
            value: value.to_string(),
        })
    }
}

// === Tests ===================================================================

#[cfg(test)]
mod tests {
    use super::*;

    fn sample_row() -> OvertypeRow {
        OvertypeRow::new(vec![
            OvertypeField::overtypeable("PRTY", "5", 3),
            OvertypeField::overtypeable("CLASS", "A", 1),
            OvertypeField::read_only("JOBID", "JOB00001", 8),
        ])
    }

    // Validates: Requirement 18.1
    #[test]
    fn field_kind_distinguishes_overtypeable_from_read_only() {
        let row = sample_row();
        assert_eq!(row.get("PRTY").unwrap().kind, FieldKind::Overtypeable);
        assert_eq!(row.get("JOBID").unwrap().kind, FieldKind::ReadOnly);
    }

    // Validates: Requirement 18.2
    #[test]
    fn apply_direct_updates_overtypeable_field() {
        let mut row = sample_row();
        let result = row.apply_direct("PRTY", "8");
        assert_eq!(
            result,
            OvertypeResult::Applied {
                field: "PRTY".to_string(),
                new_value: "8".to_string()
            }
        );
        assert_eq!(row.get("PRTY").unwrap().value, "8");
    }

    // Validates: Requirement 18.2
    #[test]
    fn apply_direct_rejects_read_only_field() {
        let mut row = sample_row();
        let result = row.apply_direct("JOBID", "JOB99999");
        assert_eq!(result, OvertypeResult::ReadOnly("JOBID".to_string()));
        assert_eq!(row.get("JOBID").unwrap().value, "JOB00001");
    }

    // Validates: Requirement 18.2
    #[test]
    fn apply_direct_returns_not_found_for_unknown_field() {
        let mut row = sample_row();
        let result = row.apply_direct("BOGUS", "X");
        assert_eq!(result, OvertypeResult::FieldNotFound("BOGUS".to_string()));
    }

    // Validates: Requirement 18.3
    #[test]
    fn command_line_overtype_parses_field_and_value() {
        let parsed = CommandLineOvertype::parse("PRTY 9").unwrap();
        assert_eq!(parsed.field_name, "PRTY");
        assert_eq!(parsed.value, "9");
    }

    // Validates: Requirement 18.3
    #[test]
    fn command_line_overtype_uppercases_field_name() {
        let parsed = CommandLineOvertype::parse("prty 9").unwrap();
        assert_eq!(parsed.field_name, "PRTY");
    }

    // Validates: Requirement 18.3
    #[test]
    fn command_line_overtype_returns_none_for_missing_value() {
        assert!(CommandLineOvertype::parse("PRTY").is_none());
        assert!(CommandLineOvertype::parse("").is_none());
    }

    // Validates: Requirement 18.3
    #[test]
    fn command_line_overtype_value_may_contain_spaces() {
        let parsed = CommandLineOvertype::parse("CLASS A B").unwrap();
        assert_eq!(parsed.field_name, "CLASS");
        assert_eq!(parsed.value, "A B");
    }

    // Validates: Requirement 18.4
    #[test]
    fn field_needs_extension_popup_when_value_exceeds_width() {
        let field = OvertypeField::overtypeable("DSN", "VERY.LONG.DATASET.NAME.EXCEEDS", 8);
        assert!(field.needs_extension_popup());
    }

    // Validates: Requirement 18.4
    #[test]
    fn field_does_not_need_extension_when_value_fits() {
        let field = OvertypeField::overtypeable("CLASS", "A", 4);
        assert!(!field.needs_extension_popup());
    }

    // Validates: Requirement 18.4
    #[test]
    fn read_only_field_never_needs_extension_popup() {
        let field = OvertypeField::read_only("JOBID", "VERY.LONG.VALUE.EXCEEDS.WIDTH", 4);
        assert!(!field.needs_extension_popup());
    }

    // Validates: Requirement 18.4
    #[test]
    fn row_reports_fields_needing_extension() {
        let row = OvertypeRow::new(vec![
            OvertypeField::overtypeable("DSN", "A.VERY.LONG.NAME.THAT.EXCEEDS", 8),
            OvertypeField::overtypeable("CLASS", "A", 4),
        ]);
        let ext = row.fields_needing_extension();
        assert_eq!(ext.len(), 1);
        assert_eq!(ext[0].name, "DSN");
    }
}
