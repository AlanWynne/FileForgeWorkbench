//! Terminal cell and attributes.
//!
//! Defines the basic rendering unit of the terminal grid — a single character
//! cell with visual attributes (color, bold, etc.).

/// Color model for terminal cells — supports ANSI 16, 256-color, and RGB.
#[derive(Debug, Default, Clone, Copy, PartialEq, Eq)]
pub enum TerminalColor {
    /// Default foreground/background from theme.
    #[default]
    Default,
    /// Standard ANSI color (0–7 normal, 8–15 bright).
    Ansi(u8),
    /// 256-color palette index.
    Palette(u8),
    /// True-color RGB.
    Rgb(u8, u8, u8),
}

/// Visual attributes for a terminal cell.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct CellAttributes {
    /// Foreground color.
    pub foreground: TerminalColor,
    /// Background color.
    pub background: TerminalColor,
    /// Bold text.
    pub bold: bool,
    /// Italic text.
    pub italic: bool,
    /// Underlined text.
    pub underline: bool,
    /// Strikethrough text.
    pub strikethrough: bool,
    /// Inverse/reverse video.
    pub inverse: bool,
    /// Dim/faint text.
    pub dim: bool,
}

impl Default for CellAttributes {
    fn default() -> Self {
        Self {
            foreground: TerminalColor::Default,
            background: TerminalColor::Default,
            bold: false,
            italic: false,
            underline: false,
            strikethrough: false,
            inverse: false,
            dim: false,
        }
    }
}

impl CellAttributes {
    /// Resets all attributes to default.
    pub fn reset(&mut self) {
        *self = Self::default();
    }
}

/// A single character cell in the terminal grid.
#[derive(Debug, Clone, PartialEq)]
pub struct Cell {
    /// The Unicode character displayed in this cell.
    pub character: char,
    /// Visual attributes (color, bold, underline, etc.).
    pub attrs: CellAttributes,
}

impl Default for Cell {
    fn default() -> Self {
        Self {
            character: ' ',
            attrs: CellAttributes::default(),
        }
    }
}

impl Cell {
    /// Creates a new cell with the given character and default attributes.
    pub fn new(character: char) -> Self {
        Self {
            character,
            attrs: CellAttributes::default(),
        }
    }

    /// Creates a new cell with the given character and attributes.
    pub fn with_attrs(character: char, attrs: CellAttributes) -> Self {
        Self { character, attrs }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    // Validates: Requirement 7.8
    #[test]
    fn default_cell_is_space_with_default_attrs() {
        let cell = Cell::default();
        assert_eq!(cell.character, ' ');
        assert_eq!(cell.attrs.foreground, TerminalColor::Default);
        assert_eq!(cell.attrs.background, TerminalColor::Default);
        assert!(!cell.attrs.bold);
    }

    // Validates: Requirement 7.8
    #[test]
    fn cell_attributes_reset() {
        let mut attrs = CellAttributes {
            bold: true,
            italic: true,
            foreground: TerminalColor::Ansi(1),
            ..Default::default()
        };
        attrs.reset();
        assert_eq!(attrs, CellAttributes::default());
    }

    // Validates: Requirement 7.8
    #[test]
    fn terminal_color_default_variant() {
        assert_eq!(TerminalColor::default(), TerminalColor::Default);
    }
}
