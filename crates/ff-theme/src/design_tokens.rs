//! Design system tokens: spacing, border radii, shadows, animations.
//!
//! These non-colour tokens ensure visual consistency across all workbench
//! UI components through a shared vocabulary of spacing values, corner
//! radii, elevation shadows, and animation timings.

use serde::{Deserialize, Serialize};

use crate::colour::ColourRGBA;

/// Spacing level identifiers for the design system.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum SpacingLevel {
    /// Extra small spacing.
    Xs,
    /// Small spacing.
    Sm,
    /// Medium spacing (default).
    Md,
    /// Large spacing.
    Lg,
    /// Extra large spacing.
    Xl,
}

/// Border radius level identifiers.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum RadiusLevel {
    /// No rounding (sharp corners).
    None,
    /// Small border radius.
    Sm,
    /// Medium border radius.
    Md,
    /// Large border radius.
    Lg,
    /// Full rounding (pill/circle shape).
    Full,
}

/// Shadow level identifiers.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum ShadowLevel {
    /// Small/subtle shadow.
    Sm,
    /// Medium shadow.
    Md,
    /// Large/prominent shadow.
    Lg,
}

/// Animation speed level identifiers.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum AnimationLevel {
    /// Fast animation (micro-interactions).
    Fast,
    /// Normal animation speed.
    Normal,
    /// Slow animation (emphasis transitions).
    Slow,
}

/// Spacing scale values in logical pixels.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct SpacingScale {
    /// Extra small spacing (default: 2.0).
    pub xs: f32,
    /// Small spacing (default: 4.0).
    pub sm: f32,
    /// Medium spacing (default: 8.0).
    pub md: f32,
    /// Large spacing (default: 16.0).
    pub lg: f32,
    /// Extra large spacing (default: 32.0).
    pub xl: f32,
}

impl Default for SpacingScale {
    fn default() -> Self {
        Self {
            xs: 2.0,
            sm: 4.0,
            md: 8.0,
            lg: 16.0,
            xl: 32.0,
        }
    }
}

impl SpacingScale {
    /// Get the spacing value for a given level.
    pub fn get(&self, level: SpacingLevel) -> f32 {
        match level {
            SpacingLevel::Xs => self.xs,
            SpacingLevel::Sm => self.sm,
            SpacingLevel::Md => self.md,
            SpacingLevel::Lg => self.lg,
            SpacingLevel::Xl => self.xl,
        }
    }
}

/// Border radius scale values in logical pixels.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct BorderRadiusScale {
    /// No rounding (default: 0.0).
    pub none: f32,
    /// Small radius (default: 2.0).
    pub sm: f32,
    /// Medium radius (default: 4.0).
    pub md: f32,
    /// Large radius (default: 8.0).
    pub lg: f32,
    /// Full rounding (default: 9999.0).
    pub full: f32,
}

impl Default for BorderRadiusScale {
    fn default() -> Self {
        Self {
            none: 0.0,
            sm: 2.0,
            md: 4.0,
            lg: 8.0,
            full: 9999.0,
        }
    }
}

impl BorderRadiusScale {
    /// Get the border radius value for a given level.
    pub fn get(&self, level: RadiusLevel) -> f32 {
        match level {
            RadiusLevel::None => self.none,
            RadiusLevel::Sm => self.sm,
            RadiusLevel::Md => self.md,
            RadiusLevel::Lg => self.lg,
            RadiusLevel::Full => self.full,
        }
    }
}

/// A shadow definition with offset, blur, spread, and colour.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct ShadowDef {
    /// Horizontal offset in logical pixels.
    pub offset_x: f32,
    /// Vertical offset in logical pixels.
    pub offset_y: f32,
    /// Blur radius in logical pixels.
    pub blur_radius: f32,
    /// Spread radius in logical pixels.
    pub spread: f32,
    /// Shadow colour (typically with low alpha).
    pub colour: ColourRGBA,
}

/// Shadow scale with small, medium, and large presets.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct ShadowScale {
    /// Small/subtle shadow.
    pub sm: ShadowDef,
    /// Medium shadow.
    pub md: ShadowDef,
    /// Large/prominent shadow.
    pub lg: ShadowDef,
}

impl Default for ShadowScale {
    fn default() -> Self {
        Self {
            sm: ShadowDef {
                offset_x: 0.0,
                offset_y: 1.0,
                blur_radius: 2.0,
                spread: 0.0,
                colour: ColourRGBA::rgba(0, 0, 0, 25),
            },
            md: ShadowDef {
                offset_x: 0.0,
                offset_y: 2.0,
                blur_radius: 4.0,
                spread: 0.0,
                colour: ColourRGBA::rgba(0, 0, 0, 50),
            },
            lg: ShadowDef {
                offset_x: 0.0,
                offset_y: 4.0,
                blur_radius: 8.0,
                spread: 2.0,
                colour: ColourRGBA::rgba(0, 0, 0, 75),
            },
        }
    }
}

impl ShadowScale {
    /// Get the shadow definition for a given level.
    pub fn get(&self, level: ShadowLevel) -> &ShadowDef {
        match level {
            ShadowLevel::Sm => &self.sm,
            ShadowLevel::Md => &self.md,
            ShadowLevel::Lg => &self.lg,
        }
    }
}

/// An animation timing definition.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct AnimationDef {
    /// Duration in milliseconds.
    pub duration_ms: u32,
    /// Named easing curve (e.g., "ease-in-out", "linear", "ease-out").
    pub easing: String,
}

/// Animation timing scale with fast, normal, and slow presets.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct AnimationScale {
    /// Fast animation (micro-interactions).
    pub fast: AnimationDef,
    /// Normal animation speed.
    pub normal: AnimationDef,
    /// Slow animation (emphasis transitions).
    pub slow: AnimationDef,
}

impl Default for AnimationScale {
    fn default() -> Self {
        Self {
            fast: AnimationDef {
                duration_ms: 100,
                easing: "ease-out".to_string(),
            },
            normal: AnimationDef {
                duration_ms: 250,
                easing: "ease-in-out".to_string(),
            },
            slow: AnimationDef {
                duration_ms: 500,
                easing: "ease-in-out".to_string(),
            },
        }
    }
}

impl AnimationScale {
    /// Get the animation definition for a given level.
    pub fn get(&self, level: AnimationLevel) -> &AnimationDef {
        match level {
            AnimationLevel::Fast => &self.fast,
            AnimationLevel::Normal => &self.normal,
            AnimationLevel::Slow => &self.slow,
        }
    }
}

/// Non-colour design system tokens for consistent UI geometry and motion.
///
/// All token values are configurable through the theme TOML file.
/// Missing tokens fall back to the built-in defaults.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize, Default)]
pub struct DesignTokens {
    /// Spacing scale in logical pixels.
    pub spacing: SpacingScale,
    /// Border radius scale in logical pixels.
    pub border_radius: BorderRadiusScale,
    /// Shadow presets.
    pub shadows: ShadowScale,
    /// Animation timing presets.
    pub animations: AnimationScale,
}

impl DesignTokens {
    /// Get a spacing value by level.
    pub fn spacing(&self, level: SpacingLevel) -> f32 {
        self.spacing.get(level)
    }

    /// Get a border radius value by level.
    pub fn border_radius(&self, level: RadiusLevel) -> f32 {
        self.border_radius.get(level)
    }

    /// Get a shadow definition by level.
    pub fn shadow(&self, level: ShadowLevel) -> &ShadowDef {
        self.shadows.get(level)
    }

    /// Get an animation timing definition by level.
    pub fn animation(&self, level: AnimationLevel) -> &AnimationDef {
        self.animations.get(level)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn default_spacing_values_are_correct() {
        // Validates: Requirement 6.1
        let tokens = DesignTokens::default();
        assert_eq!(tokens.spacing(SpacingLevel::Xs), 2.0);
        assert_eq!(tokens.spacing(SpacingLevel::Sm), 4.0);
        assert_eq!(tokens.spacing(SpacingLevel::Md), 8.0);
        assert_eq!(tokens.spacing(SpacingLevel::Lg), 16.0);
        assert_eq!(tokens.spacing(SpacingLevel::Xl), 32.0);
    }

    #[test]
    fn default_border_radius_values_are_correct() {
        // Validates: Requirement 6.2
        let tokens = DesignTokens::default();
        assert_eq!(tokens.border_radius(RadiusLevel::None), 0.0);
        assert_eq!(tokens.border_radius(RadiusLevel::Sm), 2.0);
        assert_eq!(tokens.border_radius(RadiusLevel::Md), 4.0);
        assert_eq!(tokens.border_radius(RadiusLevel::Lg), 8.0);
        assert_eq!(tokens.border_radius(RadiusLevel::Full), 9999.0);
    }

    #[test]
    fn default_shadow_definitions_have_expected_structure() {
        // Validates: Requirement 6.3
        let tokens = DesignTokens::default();
        let sm = tokens.shadow(ShadowLevel::Sm);
        assert_eq!(sm.offset_x, 0.0);
        assert!(sm.blur_radius > 0.0);
    }

    #[test]
    fn default_animation_timings_are_ordered() {
        // Validates: Requirement 6.4
        let tokens = DesignTokens::default();
        let fast = tokens.animation(AnimationLevel::Fast);
        let normal = tokens.animation(AnimationLevel::Normal);
        let slow = tokens.animation(AnimationLevel::Slow);
        assert!(fast.duration_ms < normal.duration_ms);
        assert!(normal.duration_ms < slow.duration_ms);
    }
}
