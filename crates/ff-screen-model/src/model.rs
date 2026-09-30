//! The logical screen model: the authoritative representation of a Context's
//! current screen, independent of egui.
//!
//! Validates: screen-snapshot-scrm Requirement 3.1-3.8, 13.5.

use serde::{Deserialize, Serialize};

/// A colour attribute for screen text. A small named palette keeps the model
/// renderer-agnostic; each renderer maps these to its own colour system (ANSI
/// SGR, HTML hex, or nothing for plain text).
///
/// Validates: Requirement 3.3.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize, Default)]
#[serde(rename_all = "snake_case")]
pub enum Colour {
    /// The Context did not specify a colour; the renderer uses its default.
    #[default]
    Default,
    Black,
    Red,
    Green,
    Yellow,
    Blue,
    Magenta,
    Cyan,
    White,
}

/// Field-level display and semantic attributes preserved with each field.
///
/// Validates: Requirement 3.2, 3.3, 3.4, 3.5, 13.5.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize, Default)]
pub struct FieldAttributes {
    /// Foreground colour.
    pub colour: Colour,
    /// Whether the field is highlighted (bold/intensified).
    pub highlight: bool,
    /// Whether the field is protected (not user-editable on the source screen).
    pub protected: bool,
    /// Whether the field carries sensitive content (password/hidden/masked).
    /// A Context sets this at capture time; export masking also honours it.
    /// Validates: Requirement 13.5.
    pub sensitive: bool,
}

impl FieldAttributes {
    /// A plain, unremarkable attribute set (default colour, no flags).
    pub fn plain() -> Self {
        Self::default()
    }

    /// Mark this attribute set sensitive (builder-style).
    pub fn sensitive(mut self) -> Self {
        self.sensitive = true;
        self
    }
}

/// A single labelled field on the screen (e.g. "Dataset Name . . : USER.TEST").
///
/// Validates: Requirement 3.1, 3.8.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct Field {
    /// The field label (may be empty for an unlabelled value).
    pub label: String,
    /// The field value.
    pub value: String,
    /// Display / semantic attributes.
    pub attrs: FieldAttributes,
}

impl Field {
    /// A plain label/value field with default attributes.
    pub fn new(label: impl Into<String>, value: impl Into<String>) -> Self {
        Self {
            label: label.into(),
            value: value.into(),
            attrs: FieldAttributes::plain(),
        }
    }

    /// Set attributes (builder-style).
    pub fn with_attrs(mut self, attrs: FieldAttributes) -> Self {
        self.attrs = attrs;
        self
    }
}

/// A tabular element: a header row plus data rows. Rendered as an aligned text
/// grid (plain/ANSI) or a Markdown/HTML table by the respective renderers.
///
/// Validates: Requirement 3.8, 4.4.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize, Default)]
pub struct Table {
    /// Optional table caption.
    pub caption: Option<String>,
    /// Column headers.
    pub headers: Vec<String>,
    /// Data rows; each row SHOULD have `headers.len()` cells.
    pub rows: Vec<Vec<String>>,
}

/// The status bar line(s) at the bottom of a screen.
///
/// Validates: Requirement 3.8.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize, Default)]
pub struct StatusBar {
    /// The status text.
    pub text: String,
}

/// The cursor position on the screen (1-based row/column), plus the focused
/// field label where one is known.
///
/// Validates: Requirement 3.6.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize, Default)]
pub struct Cursor {
    /// 1-based row; 0 means "unknown".
    pub row: u32,
    /// 1-based column; 0 means "unknown".
    pub column: u32,
    /// Label of the field the cursor is in, if known.
    pub field_label: Option<String>,
}

/// Screen dimensions in character cells.
///
/// Validates: Requirement 3.7.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub struct Dimensions {
    /// Number of character rows.
    pub rows: u32,
    /// Number of character columns.
    pub columns: u32,
}

impl Default for Dimensions {
    fn default() -> Self {
        // A conventional 24x80 screen, the classic 3270 model 2 size.
        Self {
            rows: 24,
            columns: 80,
        }
    }
}

/// The authoritative logical representation of a Context's current screen.
///
/// This is what a `ScreenProvider` yields and what every renderer consumes. It
/// is deliberately host-agnostic: no egui, no pixels.
///
/// Validates: Requirement 3.1-3.8.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize, Default)]
pub struct ScreenModel {
    /// The screen title.
    pub title: String,
    /// Labelled fields, in visual order.
    pub fields: Vec<Field>,
    /// Tabular elements, in visual order.
    pub tables: Vec<Table>,
    /// Free-standing messages (errors, info lines).
    pub messages: Vec<String>,
    /// Action buttons/commands shown on the screen (e.g. "Save", "Cancel").
    pub buttons: Vec<String>,
    /// The status bar.
    pub status_bar: StatusBar,
    /// The command line content (the `Command ===>` field), if present.
    pub command_line: String,
    /// The cursor position.
    pub cursor: Cursor,
    /// Screen dimensions.
    pub dimensions: Dimensions,
}

impl ScreenModel {
    /// Construct an empty screen with the given title.
    pub fn new(title: impl Into<String>) -> Self {
        Self {
            title: title.into(),
            ..Self::default()
        }
    }

    /// Add a field (builder-style).
    pub fn with_field(mut self, field: Field) -> Self {
        self.fields.push(field);
        self
    }

    /// Add a button (builder-style).
    pub fn with_button(mut self, label: impl Into<String>) -> Self {
        self.buttons.push(label.into());
        self
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use pretty_assertions::assert_eq;

    // Validates: Requirement 3.1, 3.8 -- a screen carries title + fields.
    #[test]
    fn screen_model_builder_collects_title_and_fields() {
        let m = ScreenModel::new("Dataset Properties")
            .with_field(Field::new("Dataset Name", "USER.TEST.PDS"))
            .with_field(Field::new("RECFM", "FB"))
            .with_button("Save");
        assert_eq!(m.title, "Dataset Properties");
        assert_eq!(m.fields.len(), 2);
        assert_eq!(m.fields[0].value, "USER.TEST.PDS");
        assert_eq!(m.buttons, vec!["Save".to_string()]);
    }

    // Validates: Requirement 3.7 -- default dimensions are 24x80.
    #[test]
    fn default_dimensions_are_24_by_80() {
        let d = Dimensions::default();
        assert_eq!(d.rows, 24);
        assert_eq!(d.columns, 80);
    }

    // Validates: Requirement 13.5 -- a field can be marked sensitive.
    #[test]
    fn field_attributes_can_be_marked_sensitive() {
        let f = Field::new("Password", "hunter2").with_attrs(FieldAttributes::plain().sensitive());
        assert!(f.attrs.sensitive);
        assert!(!Field::new("User", "alan").attrs.sensitive);
    }

    // Validates: Requirement 3.3 -- default colour round-trips.
    #[test]
    fn colour_defaults_to_default_variant() {
        assert_eq!(Colour::default(), Colour::Default);
    }
}
