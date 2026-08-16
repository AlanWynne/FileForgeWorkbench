//! # ff-text-decorations — Visual Overlay Subsystem
//!
//! This crate manages transient, overlapping decorations applied on top of
//! (or beneath) rendered text to communicate semantic information:
//!
//! - **Search match highlighting** (current match, all matches)
//! - **Diagnostic underlines** (error squiggles, warning indicators)
//! - **Change history markers** (modified, saved, reverted lines)
//! - **Bookmarks** (user-placed navigation markers)
//! - **Plugin indicators** (custom decorations via allocated indicator numbers)
//!
//! ## Architecture
//!
//! This is a Wave 6 (UI and Rendering) crate. It stores decoration data and
//! exposes query APIs; actual rendering is performed by the shell layer (egui).
//!
//! Key data structures:
//! - [`RunStyles<T>`] — generic run-length encoded storage
//! - [`DecorationList`] — per-document aggregate of all indicator decorations
//! - [`MarkerStore`] — per-line marker bitmask storage
//! - [`IndicatorCatalogue`] — style configuration for all 44 indicator slots
//! - [`HoverState`] — mouse tracking for dynamic indicators
//!
//! ## Dependencies
//!
//! - `ff-logging` — structured diagnostic output
//! - `ff-command` — bookmark command registration
//! - `ff-config` — hot-reload configuration integration

// ─── Public Modules ─────────────────────────────────────────────────────────

pub mod allocator;
pub mod catalogue;
pub mod commands;
pub mod constants;
pub mod decoration;
pub mod decoration_list;
pub mod dpi;
pub mod edit_sync;
pub mod error;
pub mod events;
pub mod hover;
pub mod indicator;
pub mod indicator_style;
pub mod line_marker;
pub mod marker_store;
pub mod marker_symbol;
pub mod rendering;
pub mod run_styles;
pub mod theme_integration;

// ─── Public API Re-exports ──────────────────────────────────────────────────

pub use allocator::IndicatorAllocator;
pub use catalogue::IndicatorCatalogue;
pub use constants::{indicators, markers};
pub use decoration_list::DecorationList;
pub use dpi::PixelAligner;
pub use edit_sync::EditSync;
pub use error::DecorationError;
pub use events::{DecorationEvent, DecorationEventListener};
pub use hover::HoverState;
pub use indicator::{IndicatorConfig, IndicatorFlags, StyleAppearance};
pub use indicator_style::IndicatorStyle;
pub use line_marker::{LineMarkerConfig, MarkerLayer};
pub use marker_store::MarkerStore;
pub use marker_symbol::{MarkerSymbol, PixmapId};
pub use rendering::{DecorationRenderer, RenderingProvider};
pub use run_styles::{Run, RunStyles};
pub use theme_integration::ThemeDecorationProvider;

// ─── Core Newtypes ──────────────────────────────────────────────────────────

/// Indicator number (0–43).
///
/// Addresses: Requirement 13
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, PartialOrd, Ord)]
pub struct IndicatorNumber(pub u8);

impl IndicatorNumber {
    /// Maximum valid indicator number.
    pub const MAX: u8 = 43;

    /// Create a new indicator number, returning `None` if out of range.
    pub fn new(n: u8) -> Option<Self> {
        if n <= Self::MAX {
            Some(Self(n))
        } else {
            None
        }
    }
}

/// Marker number (0–31).
///
/// Addresses: Requirement 9 AC 1
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, PartialOrd, Ord)]
pub struct MarkerNumber(pub u8);

impl MarkerNumber {
    /// Maximum valid marker number.
    pub const MAX: u8 = 31;

    /// Create a new marker number, returning `None` if out of range.
    pub fn new(n: u8) -> Option<Self> {
        if n <= Self::MAX {
            Some(Self(n))
        } else {
            None
        }
    }
}

/// Bitmask of active markers on a line (bits 0–31).
///
/// Addresses: Requirement 9 AC 7
#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
pub struct MarkerMask(pub u32);

impl MarkerMask {
    /// Check if a specific marker is set in this mask.
    pub fn has(&self, marker: MarkerNumber) -> bool {
        (self.0 >> marker.0) & 1 == 1
    }

    /// Set a specific marker bit in this mask.
    pub fn set(&mut self, marker: MarkerNumber) {
        self.0 |= 1 << marker.0;
    }

    /// Clear a specific marker bit in this mask.
    pub fn clear(&mut self, marker: MarkerNumber) {
        self.0 &= !(1 << marker.0);
    }

    /// Returns true if no markers are set.
    pub fn is_empty(&self) -> bool {
        self.0 == 0
    }
}

/// RGBA colour representation (0–255 per component).
///
/// Addresses: Requirement 15 (theme integration)
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct ColourRGBA {
    pub r: u8,
    pub g: u8,
    pub b: u8,
    pub a: u8,
}

impl ColourRGBA {
    /// Create a new fully opaque colour.
    pub fn new(r: u8, g: u8, b: u8) -> Self {
        Self { r, g, b, a: 255 }
    }

    /// Create a new colour with explicit alpha.
    pub fn with_alpha(r: u8, g: u8, b: u8, a: u8) -> Self {
        Self { r, g, b, a }
    }

    /// Create a colour from a 24-bit RGB value (used for ValueFore mode).
    pub fn from_rgb24(value: u32) -> Self {
        Self {
            r: ((value >> 16) & 0xFF) as u8,
            g: ((value >> 8) & 0xFF) as u8,
            b: (value & 0xFF) as u8,
            a: 255,
        }
    }
}

impl std::fmt::Display for ColourRGBA {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "rgba({}, {}, {}, {})", self.r, self.g, self.b, self.a)
    }
}
