//! # ff-theme — Theme & Appearance System for FileForgeWorkbench
//!
//! This crate is the central visual identity layer for the entire workbench platform.
//! It manages colour palettes, font stacks, style slots, design tokens, and visual
//! mode switching (dark/light/high-contrast) through a TOML-based theme configuration
//! format.
//!
//! All rendering code obtains colour values, font selections, and spacing metrics
//! exclusively through the theme system rather than using hardcoded values.
//!
//! ## Architecture
//!
//! - **Wave 6 (UI and Rendering)** — depends on `ff-config` (Wave 2) for TOML-based
//!   configuration loading, layered overrides, and hot-reload notifications.
//! - Consumed by all rendering subsystems: syntax-highlighting, caret-and-selection,
//!   text-decorations, whitespace-and-guides, menu-and-statusbar, layout-and-docking,
//!   file-tree-panel, and the GUI shell.
//!
//! ## Quick Start
//!
//! ```rust,no_run
//! use ff_theme::{ColourRGBA, ColourToken, VisualMode};
//!
//! let colour = ColourRGBA::from_hex("#1E1E2E").unwrap();
//! assert_eq!(colour.to_hex(), "#1E1E2E");
//! ```

// ─── Public Modules ─────────────────────────────────────────────────────────

/// RGBA colour type, hex parsing, and display.
pub mod colour;

/// Semantic colour token identifiers (compile-time verified).
pub mod token;

/// Theme palette structures organised by colour group.
pub mod palette;

/// Style slot system: 256 indexed entries with font/colour/attribute combinations.
pub mod style_slot;

/// Font configuration: font stacks, sizes, zoom, and resolution.
pub mod font;

/// Visual mode (Dark / Light / High-Contrast) support.
pub mod mode;

/// Design system tokens: spacing, border radii, shadows, animations.
pub mod design_tokens;

/// Element-based colour system with optional alpha/transparency.
pub mod element;

/// Plugin theme extensions registration and resolution.
pub mod extension;

/// Theme discovery: scanning the themes directory for user-created theme files.
pub mod discovery;

/// Theme TOML loading, inheritance resolution, and validation.
pub mod loader;

/// Theme palette serialisation to TOML format.
pub mod serialiser;

/// Public API facade for theme access.
pub mod api;

/// Error types for the theme system.
pub mod error;

/// Built-in default palettes for all three visual modes.
pub mod defaults;

/// Theme change event types and notification.
pub mod event;

/// Configuration key constants (theme.* namespace).
pub mod keys;

// ─── Public API Re-exports ──────────────────────────────────────────────────

pub use colour::ColourRGBA;
pub use design_tokens::{
    AnimationDef, AnimationLevel, AnimationScale, BorderRadiusScale, DesignTokens, RadiusLevel,
    ShadowDef, ShadowLevel, ShadowScale, SpacingLevel, SpacingScale,
};
pub use discovery::{builtin_themes, export_theme, list_all_themes, scan_themes_dir, ThemeInfo};
pub use element::Element;
pub use error::ThemeError;
pub use event::ThemeEvent;
pub use extension::{ExtensionToken, ThemeExtension};
pub use font::{FontConfig, FontStack};
pub use mode::VisualMode;
pub use palette::ThemePalette;
pub use style_slot::{CaseTransform, StyleSlot, StyleSlotTable};
pub use token::ColourToken;
