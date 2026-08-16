//! Theme change event types and notification.
//!
//! Events are emitted by the theme system when the palette changes,
//! allowing consumers to invalidate caches or trigger re-renders.

use crate::element::Element;
use crate::mode::VisualMode;

/// Events emitted by the theme system when the palette changes.
#[derive(Debug, Clone)]
pub enum ThemeEvent {
    /// The entire palette was replaced (theme switch or hot-reload).
    PaletteChanged {
        /// Name of the previous theme.
        previous_theme: String,
        /// Name of the new theme.
        new_theme: String,
    },
    /// The visual mode was switched.
    ModeChanged {
        /// Previous visual mode.
        previous_mode: VisualMode,
        /// New visual mode.
        new_mode: VisualMode,
    },
    /// An element colour was overridden at runtime.
    ElementOverridden {
        /// The element that was overridden.
        element: Element,
    },
    /// An element colour was reset to its base value.
    ElementReset {
        /// The element that was reset.
        element: Element,
    },
}
