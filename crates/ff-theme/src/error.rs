//! Error types for the theme system.
//!
//! All theme errors use the `[theme] operation: description` format for
//! consistent diagnostic output.

use thiserror::Error;

/// Errors that can occur within the theme system.
///
/// Each variant carries enough context to diagnose the problem without
/// requiring additional logging at the call site.
#[derive(Debug, Error)]
#[non_exhaustive]
pub enum ThemeError {
    /// A colour hex string could not be parsed.
    #[error("[theme] parse colour: invalid hex format '{input}'")]
    InvalidColourFormat {
        /// The input string that failed to parse.
        input: String,
    },

    /// The specified theme file was not found.
    #[error("[theme] load: file not found '{path}'")]
    FileNotFound {
        /// The path that was attempted.
        path: String,
    },

    /// The theme file contained invalid TOML syntax.
    #[error("[theme] parse: TOML syntax error in '{path}': {detail}")]
    ParseError {
        /// Path to the file with the error.
        path: String,
        /// Description of the parse error.
        detail: String,
    },

    /// A font size was outside the valid range.
    #[error("[theme] validate font: size {size} is outside valid range [{min}, {max}]")]
    InvalidFontSize {
        /// The invalid font size value.
        size: f32,
        /// Minimum allowed value.
        min: f32,
        /// Maximum allowed value.
        max: f32,
    },

    /// No more style slots are available for allocation.
    #[error(
        "[theme] allocate style slots: exhausted (requested {requested}, available {available})"
    )]
    SlotAllocationExhausted {
        /// Number of slots requested.
        requested: u8,
        /// Number of slots actually available.
        available: u8,
    },

    /// A plugin extension token collides with a core palette token name.
    #[error("[theme] register extension: token '{token}' from plugin '{plugin_id}' collides with core token")]
    ExtensionCollision {
        /// The plugin identifier.
        plugin_id: String,
        /// The colliding token name.
        token: String,
    },

    /// The declared base theme could not be found.
    #[error("[theme] resolve base: theme '{base_name}' not found")]
    InvalidBase {
        /// The name of the base theme that was not found.
        base_name: String,
    },

    /// An I/O error reading a theme file or directory.
    #[error("[theme] io: error accessing '{path}': {detail}")]
    Io {
        /// The path that was being accessed.
        path: String,
        /// The underlying I/O error description.
        detail: String,
    },

    /// A foreground/background colour pair violates contrast requirements.
    #[error("[theme] contrast: pair ({fg}, {bg}) has ratio {ratio:.2}:1, minimum required is {minimum:.1}:1")]
    ContrastViolation {
        /// The foreground colour as hex.
        fg: String,
        /// The background colour as hex.
        bg: String,
        /// The computed contrast ratio.
        ratio: f64,
        /// The minimum required ratio.
        minimum: f64,
    },
}
