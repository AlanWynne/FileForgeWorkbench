//! Caret shape and style types.
//!
//! Defines `CaretStyle`, `CaretWidth`, and `CaretShape` — the building blocks
//! for configuring how the caret is drawn.

use ff_edit_operations::EditMode;
use serde::{Deserialize, Serialize};

/// The visual shape of the caret.
///
/// # Variants
///
/// - `Invisible` — caret is not drawn
/// - `Line` — vertical bar with configurable width
/// - `Block` — solid rectangle spanning one character cell
///
/// Addresses: Requirement 1, criteria 1.1–1.2
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Default, Serialize, Deserialize)]
pub enum CaretStyle {
    /// Caret is not drawn.
    Invisible,
    /// Vertical bar with configurable width.
    #[default]
    Line,
    /// Solid rectangle spanning one character cell.
    Block,
}

/// Pixel width for a Line-style caret, clamped to [1, 20].
///
/// Addresses: Requirement 1, criteria 1.4–1.6
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub struct CaretWidth(u8);

impl CaretWidth {
    /// The minimum caret width in pixels.
    pub const MIN: u8 = 1;
    /// The maximum caret width in pixels.
    pub const MAX: u8 = 20;

    /// Creates a caret width, clamping the input to [1, 20].
    pub fn new(pixels: u8) -> Self {
        Self(pixels.clamp(Self::MIN, Self::MAX))
    }

    /// Returns the pixel width value.
    pub fn pixels(&self) -> u8 {
        self.0
    }
}

impl Default for CaretWidth {
    fn default() -> Self {
        Self(1)
    }
}

/// Composes caret style, width, and overstrike-override flag into a single shape descriptor.
///
/// Addresses: Requirement 1, criteria 1.1–1.10
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct CaretShape {
    /// The configured caret style.
    style: CaretStyle,
    /// The pixel width for Line style.
    width: CaretWidth,
    /// Whether overstrike mode forces Block style.
    overstrike_forces_block: bool,
}

impl CaretShape {
    /// Creates a new caret shape with the given style and width.
    pub fn new(style: CaretStyle, width: CaretWidth) -> Self {
        Self {
            style,
            width,
            overstrike_forces_block: true,
        }
    }

    /// Creates a new caret shape with explicit overstrike override setting.
    pub fn with_overstrike_override(
        style: CaretStyle,
        width: CaretWidth,
        overstrike_forces_block: bool,
    ) -> Self {
        Self {
            style,
            width,
            overstrike_forces_block,
        }
    }

    /// Returns the effective caret style considering the current edit mode.
    ///
    /// When in Overstrike mode and `overstrike_forces_block` is true,
    /// returns `Block` regardless of the configured style.
    ///
    /// Addresses: Requirement 1, criterion 1.3
    pub fn effective_style(&self, edit_mode: EditMode) -> CaretStyle {
        if self.overstrike_forces_block && edit_mode == EditMode::Overstrike {
            CaretStyle::Block
        } else {
            self.style
        }
    }

    /// Returns the effective pixel width for Line style.
    ///
    /// The width is always within [1, 20] due to `CaretWidth` clamping.
    pub fn effective_width(&self) -> u8 {
        self.width.pixels()
    }

    /// Returns the configured style (without edit mode override).
    pub fn style(&self) -> CaretStyle {
        self.style
    }

    /// Sets the caret style.
    pub fn set_style(&mut self, style: CaretStyle) {
        self.style = style;
    }

    /// Returns the configured width.
    pub fn width(&self) -> CaretWidth {
        self.width
    }

    /// Sets the caret width.
    pub fn set_width(&mut self, width: CaretWidth) {
        self.width = width;
    }

    /// Returns whether overstrike mode forces block style.
    pub fn overstrike_forces_block(&self) -> bool {
        self.overstrike_forces_block
    }

    /// Sets whether overstrike mode forces block style.
    pub fn set_overstrike_forces_block(&mut self, forces_block: bool) {
        self.overstrike_forces_block = forces_block;
    }
}

impl Default for CaretShape {
    fn default() -> Self {
        Self {
            style: CaretStyle::default(),
            width: CaretWidth::default(),
            overstrike_forces_block: true,
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    // ─── CaretStyle tests ───────────────────────────────────────────────────

    #[test]
    fn default_caret_style_is_line() {
        // Validates: Requirement 1.2
        assert_eq!(CaretStyle::default(), CaretStyle::Line);
    }

    // ─── CaretWidth tests ───────────────────────────────────────────────────

    #[test]
    fn default_caret_width_is_one_pixel() {
        // Validates: Requirement 1.5
        assert_eq!(CaretWidth::default().pixels(), 1);
    }

    #[test]
    fn caret_width_clamps_zero_to_one() {
        // Validates: Requirement 1.6
        let w = CaretWidth::new(0);
        assert_eq!(w.pixels(), 1);
    }

    #[test]
    fn caret_width_clamps_above_max_to_twenty() {
        // Validates: Requirement 1.6
        let w = CaretWidth::new(25);
        assert_eq!(w.pixels(), 20);
    }

    #[test]
    fn caret_width_clamps_u8_max_to_twenty() {
        // Validates: Requirement 1.6
        let w = CaretWidth::new(255);
        assert_eq!(w.pixels(), 20);
    }

    #[test]
    fn caret_width_passes_through_valid_values() {
        // Validates: Requirement 1.6
        for v in 1..=20u8 {
            let w = CaretWidth::new(v);
            assert_eq!(w.pixels(), v);
        }
    }

    // ─── CaretShape tests ───────────────────────────────────────────────────

    #[test]
    fn default_caret_shape_is_line_width_one() {
        let shape = CaretShape::default();
        assert_eq!(shape.style(), CaretStyle::Line);
        assert_eq!(shape.effective_width(), 1);
        assert!(shape.overstrike_forces_block());
    }

    #[test]
    fn effective_style_returns_block_in_overstrike_mode() {
        // Validates: Requirement 1.3
        let shape = CaretShape::new(CaretStyle::Line, CaretWidth::new(2));
        assert_eq!(
            shape.effective_style(EditMode::Overstrike),
            CaretStyle::Block
        );
    }

    #[test]
    fn effective_style_returns_configured_in_insert_mode() {
        // Validates: Requirement 1.3
        let shape = CaretShape::new(CaretStyle::Line, CaretWidth::new(2));
        assert_eq!(shape.effective_style(EditMode::Insert), CaretStyle::Line);
    }

    #[test]
    fn effective_style_returns_configured_in_browse_mode() {
        let shape = CaretShape::new(CaretStyle::Block, CaretWidth::new(1));
        assert_eq!(shape.effective_style(EditMode::Browse), CaretStyle::Block);
    }

    #[test]
    fn effective_style_invisible_preserved_in_insert_mode() {
        let shape = CaretShape::new(CaretStyle::Invisible, CaretWidth::new(1));
        assert_eq!(
            shape.effective_style(EditMode::Insert),
            CaretStyle::Invisible
        );
    }

    #[test]
    fn overstrike_override_disabled_preserves_style() {
        let shape =
            CaretShape::with_overstrike_override(CaretStyle::Line, CaretWidth::new(2), false);
        assert_eq!(
            shape.effective_style(EditMode::Overstrike),
            CaretStyle::Line
        );
    }

    #[test]
    fn effective_width_returns_clamped_value() {
        let shape = CaretShape::new(CaretStyle::Line, CaretWidth::new(5));
        assert_eq!(shape.effective_width(), 5);
    }
}
