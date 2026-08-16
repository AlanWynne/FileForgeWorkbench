//! Theme integration — palette reload for decoration colours.
//!
//! Defines the `ThemeDecorationProvider` trait that abstracts
//! theme palette access, decoupling this crate from concrete theme implementations.

use crate::indicator_style::IndicatorStyle;
use crate::marker_symbol::MarkerSymbol;
use crate::{ColourRGBA, IndicatorNumber, MarkerNumber};

/// Trait abstracting theme palette access for decoration colours.
///
/// Implemented by the theme system's palette to avoid hard-coupling
/// to the concrete theme crate.
///
/// Addresses: Requirement 15 AC 1–8
pub trait ThemeDecorationProvider: Send + Sync {
    /// Get the configured colour for an indicator number.
    fn indicator_fore(&self, indicator: IndicatorNumber) -> Option<ColourRGBA>;

    /// Get the configured fill alpha for an indicator.
    fn indicator_fill_alpha(&self, indicator: IndicatorNumber) -> Option<u8>;

    /// Get the configured outline alpha for an indicator.
    fn indicator_outline_alpha(&self, indicator: IndicatorNumber) -> Option<u8>;

    /// Get the configured stroke width for an indicator.
    fn indicator_stroke_width(&self, indicator: IndicatorNumber) -> Option<f32>;

    /// Get the configured style override for an indicator.
    fn indicator_style(&self, indicator: IndicatorNumber) -> Option<IndicatorStyle>;

    /// Get the configured foreground colour for a marker number.
    fn marker_fore(&self, marker: MarkerNumber) -> Option<ColourRGBA>;

    /// Get the configured background colour for a marker number.
    fn marker_back(&self, marker: MarkerNumber) -> Option<ColourRGBA>;

    /// Get the configured background-selected colour for a marker number.
    fn marker_back_selected(&self, marker: MarkerNumber) -> Option<ColourRGBA>;

    /// Get the configured alpha for a marker number.
    fn marker_alpha(&self, marker: MarkerNumber) -> Option<u8>;

    /// Get the configured symbol for a marker number.
    fn marker_symbol(&self, marker: MarkerNumber) -> Option<MarkerSymbol>;
}
