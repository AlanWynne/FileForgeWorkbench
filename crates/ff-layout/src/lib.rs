//! # ff-layout — GUI-Independent Layout Engine for FileForgeWorkbench
//!
//! This crate is the layout engine for the FileForgeWorkbench platform. It owns
//! the spatial arrangement of all panels, tab groups, floating windows, and dock
//! zones — expressing the entire workspace layout as a data model that the GUI
//! shell (`ff-desktop`) renders but does not own.
//!
//! ## Capabilities
//!
//! - **Panel System**: Dockable panels with registration, show/hide/toggle, minimize/maximize
//! - **Tab Groups**: Center area subdivision with horizontal and vertical splits
//! - **Floating Windows**: OS-level viewports for multi-monitor workflows
//! - **Personas**: Named layout presets for instant workspace switching
//! - **Drag-and-Drop**: Panel/tab rearrangement with visual indicators
//! - **Resizing**: Proportional splitters with minimum size constraints
//! - **Serialization**: TOML-based persistence with graceful degradation
//!
//! ## Architecture
//!
//! The `LayoutEngine` is the central coordinator. It owns the layout tree and
//! orchestrates all transitions. The shell layer forwards user input to the
//! engine and renders the resulting state.
//!
//! ```text
//! ff-logging ← ff-layout ← ff-desktop
//!               ff-layout ← ff-core (lifecycle)
//!               ff-layout → ff-command (command registration)
//!               ff-layout ← ff-plugin (panel registration)
//! ```
//!
//! ## GUI Independence
//!
//! This crate never imports GUI framework types for its own logic. The only
//! GUI-framework reference is the `DockablePanel::render` trait method signature,
//! which panels implement. The layout engine itself is purely data-driven.

pub mod commands;
pub mod dock;
pub mod drag;
pub mod engine;
pub mod error;
pub mod floating;
pub mod panel;
pub mod persona;
pub mod resize;
pub mod state;
pub mod tabs;

// ─── Public API Re-exports ──────────────────────────────────────────────────

pub use dock::zone::DockZone;
pub use drag::indicator::{DropIndicator, DropPlacement, SplitSide};
pub use engine::{CloseAction, LayoutEngine};
pub use error::LayoutError;
pub use floating::window::{FloatingWindow, FloatingWindowId};
pub use panel::display_state::PanelDisplayState;
pub use panel::traits::{DockState, DockablePanel};
pub use persona::definition::{Persona, PersonaKind};
pub use resize::splitter::{Splitter, SplitterId, SplitterOrientation};
pub use state::layout_state::{DockedPanelState, LayoutState};
pub use tabs::group::{SplitDirection, TabGroup, TabGroupId, TabGroupTree};

// ─── Shared Geometry Types ──────────────────────────────────────────────────

/// Logical pixel position (x, y) in screen coordinates.
#[derive(Debug, Clone, Copy, PartialEq, serde::Serialize, serde::Deserialize)]
pub struct Position {
    /// Horizontal position in logical pixels.
    pub x: f32,
    /// Vertical position in logical pixels.
    pub y: f32,
}

impl Position {
    /// Creates a new position from x and y coordinates.
    pub fn new(x: f32, y: f32) -> Self {
        Self { x, y }
    }
}

/// Logical pixel size (width, height).
#[derive(Debug, Clone, Copy, PartialEq, serde::Serialize, serde::Deserialize)]
pub struct Size {
    /// Width in logical pixels.
    pub width: f32,
    /// Height in logical pixels.
    pub height: f32,
}

impl Size {
    /// Creates a new size from width and height.
    pub fn new(width: f32, height: f32) -> Self {
        Self { width, height }
    }
}

/// A rectangle in logical pixel coordinates.
#[derive(Debug, Clone, Copy, PartialEq, serde::Serialize, serde::Deserialize)]
pub struct Rect {
    /// Left edge x-coordinate.
    pub x: f32,
    /// Top edge y-coordinate.
    pub y: f32,
    /// Width of the rectangle.
    pub width: f32,
    /// Height of the rectangle.
    pub height: f32,
}

impl Rect {
    /// Creates a new rectangle from position and dimensions.
    pub fn new(x: f32, y: f32, width: f32, height: f32) -> Self {
        Self {
            x,
            y,
            width,
            height,
        }
    }

    /// Returns true if the given point is inside this rectangle.
    pub fn contains(&self, point: Position) -> bool {
        point.x >= self.x
            && point.x <= self.x + self.width
            && point.y >= self.y
            && point.y <= self.y + self.height
    }

    /// Returns the center position of this rectangle.
    pub fn center(&self) -> Position {
        Position {
            x: self.x + self.width / 2.0,
            y: self.y + self.height / 2.0,
        }
    }

    /// Returns the area of this rectangle that overlaps with another.
    pub fn overlap_area(&self, other: &Rect) -> f32 {
        let x_overlap = (self.x + self.width).min(other.x + other.width) - self.x.max(other.x);
        let y_overlap = (self.y + self.height).min(other.y + other.height) - self.y.max(other.y);
        if x_overlap > 0.0 && y_overlap > 0.0 {
            x_overlap * y_overlap
        } else {
            0.0
        }
    }

    /// Returns the total area of this rectangle.
    pub fn area(&self) -> f32 {
        self.width * self.height
    }
}

/// The maximum number of simultaneous floating windows allowed.
pub const MAX_FLOATING_WINDOWS: usize = 16;

/// The default minimum panel size in logical pixels (both dimensions).
pub const DEFAULT_MIN_PANEL_SIZE: f32 = 48.0;

/// The minimum tab group size in logical pixels (split direction).
pub const MIN_TAB_GROUP_SIZE: f32 = 100.0;

/// The current schema version for layout state serialization.
pub const SCHEMA_VERSION: u32 = 1;
