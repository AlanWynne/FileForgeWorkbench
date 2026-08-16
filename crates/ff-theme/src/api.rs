//! Public API facade for theme access.
//!
//! `ThemeApi` is the primary entry point for rendering code to obtain
//! colours, fonts, design tokens, and element colours from the active theme.

use std::sync::{Arc, RwLock};

use crate::colour::ColourRGBA;
use crate::defaults;
use crate::design_tokens::{
    AnimationDef, AnimationLevel, DesignTokens, RadiusLevel, ShadowDef, ShadowLevel, SpacingLevel,
};
use crate::element::Element;
use crate::error::ThemeError;
use crate::event::ThemeEvent;
use crate::extension::{ExtensionRegistry, ThemeExtension};
use crate::font::{FontConfig, ZoomLevel};
use crate::mode::VisualMode;
use crate::palette::ThemePalette;
use crate::style_slot::StyleSlot;
use crate::token::ColourToken;

/// Type alias for theme event subscriber callbacks.
type ThemeSubscriber = Box<dyn Fn(&ThemeEvent) + Send + Sync>;

/// Thread-safe handle for accessing the active theme.
///
/// This is the primary API surface consumed by all rendering subsystems.
/// It provides access to colours, fonts, design tokens, element colours,
/// style slots, and supports mode switching and plugin extensions.
///
/// The handle is cheaply clonable and shareable across threads.
#[derive(Clone)]
pub struct ThemeApi {
    /// The current active palette.
    palette: Arc<RwLock<Arc<ThemePalette>>>,
    /// Zoom level for monospace font.
    zoom: Arc<RwLock<ZoomLevel>>,
    /// Extension registry for plugin tokens.
    extensions: Arc<RwLock<ExtensionRegistry>>,
    /// Event subscribers (callbacks).
    subscribers: Arc<RwLock<Vec<ThemeSubscriber>>>,
}

impl ThemeApi {
    /// Create a new `ThemeApi` with the default dark palette.
    pub fn new() -> Self {
        Self::with_palette(defaults::dark_palette())
    }

    /// Create a `ThemeApi` with a specific palette.
    pub fn with_palette(palette: ThemePalette) -> Self {
        Self {
            palette: Arc::new(RwLock::new(Arc::new(palette))),
            zoom: Arc::new(RwLock::new(ZoomLevel::default())),
            extensions: Arc::new(RwLock::new(ExtensionRegistry::new())),
            subscribers: Arc::new(RwLock::new(Vec::new())),
        }
    }

    /// Get a snapshot of the current palette.
    ///
    /// The returned `Arc` is guaranteed to be consistent — no partial
    /// updates are visible within the same snapshot.
    pub fn palette(&self) -> Arc<ThemePalette> {
        self.palette.read().unwrap().clone()
    }

    /// Get a colour by compile-time token.
    ///
    /// This is the primary colour access method for rendering code.
    pub fn colour(&self, token: ColourToken) -> ColourRGBA {
        self.palette.read().unwrap().colour(token)
    }

    /// Get the active visual mode.
    pub fn mode(&self) -> VisualMode {
        self.palette.read().unwrap().mode
    }

    /// Get the active theme name.
    pub fn theme_name(&self) -> String {
        self.palette.read().unwrap().name.clone()
    }

    /// Get the style slot at the given index (0–255).
    pub fn style_slot(&self, index: u8) -> StyleSlot {
        self.palette.read().unwrap().style_slots.get(index).clone()
    }

    /// Get the resolved font configuration.
    pub fn font_config(&self) -> FontConfig {
        self.palette.read().unwrap().fonts.clone()
    }

    /// Get a reference to the design tokens.
    pub fn design_tokens(&self) -> DesignTokens {
        self.palette.read().unwrap().design.clone()
    }

    /// Get a spacing value by level.
    pub fn spacing(&self, level: SpacingLevel) -> f32 {
        self.palette.read().unwrap().design.spacing(level)
    }

    /// Get a border radius value by level.
    pub fn border_radius(&self, level: RadiusLevel) -> f32 {
        self.palette.read().unwrap().design.border_radius(level)
    }

    /// Get a shadow definition by level.
    pub fn shadow(&self, level: ShadowLevel) -> ShadowDef {
        self.palette.read().unwrap().design.shadow(level).clone()
    }

    /// Get an animation timing definition by level.
    pub fn animation(&self, level: AnimationLevel) -> AnimationDef {
        self.palette.read().unwrap().design.animation(level).clone()
    }

    /// Get the colour for a named UI element.
    pub fn element_colour(&self, element: Element) -> Option<ColourRGBA> {
        self.palette.read().unwrap().elements.get(element)
    }

    /// Check if an element supports translucent alpha.
    pub fn element_allows_translucent(&self, element: Element) -> bool {
        element.allows_translucent()
    }

    /// Override an element's colour at runtime.
    pub fn set_element_colour(&self, element: Element, colour: ColourRGBA) {
        let mut palette_lock = self.palette.write().unwrap();
        let mut palette = (**palette_lock).clone();
        palette.elements.set(element, colour);
        *palette_lock = Arc::new(palette);
        drop(palette_lock);
        self.emit(ThemeEvent::ElementOverridden { element });
    }

    /// Reset an element colour to its base (theme-defined) value.
    pub fn reset_element(&self, element: Element) {
        let mut palette_lock = self.palette.write().unwrap();
        let mut palette = (**palette_lock).clone();
        palette.elements.reset(element);
        *palette_lock = Arc::new(palette);
        drop(palette_lock);
        self.emit(ThemeEvent::ElementReset { element });
    }

    /// Get the current zoom level offset.
    pub fn zoom_level(&self) -> i32 {
        self.zoom.read().unwrap().level()
    }

    /// Set the zoom level offset.
    pub fn set_zoom_level(&self, level: i32) {
        self.zoom.write().unwrap().set_level(level);
    }

    /// Get the effective monospace font size (base + zoom, clamped).
    pub fn effective_monospace_size(&self) -> f32 {
        let zoom = self.zoom.read().unwrap();
        let palette = self.palette.read().unwrap();
        zoom.effective_size(palette.fonts.monospace.base_size_pt)
    }

    /// Switch the active visual mode.
    ///
    /// Rebuilds the palette from the default for the new mode and notifies consumers.
    pub fn set_mode(&self, mode: VisualMode) {
        let previous_mode = self.mode();
        if previous_mode == mode {
            return;
        }
        let new_palette = defaults::default_palette_for_mode(mode);
        let mut palette_lock = self.palette.write().unwrap();
        *palette_lock = Arc::new(new_palette);
        drop(palette_lock);
        self.emit(ThemeEvent::ModeChanged {
            previous_mode,
            new_mode: mode,
        });
    }

    /// Replace the active palette (for hot-reload or theme switch).
    pub fn set_palette(&self, palette: ThemePalette) {
        let previous_name = self.theme_name();
        let new_name = palette.name.clone();
        let mut palette_lock = self.palette.write().unwrap();
        *palette_lock = Arc::new(palette);
        drop(palette_lock);
        self.emit(ThemeEvent::PaletteChanged {
            previous_theme: previous_name,
            new_theme: new_name,
        });
    }

    /// Register a plugin's theme extension tokens.
    pub fn register_extension(&self, extension: ThemeExtension) -> Result<(), ThemeError> {
        self.extensions.write().unwrap().register(extension)
    }

    /// Deregister a plugin's extension tokens.
    pub fn deregister_extension(&self, plugin_id: &str) {
        self.extensions.write().unwrap().deregister(plugin_id);
    }

    /// Get a plugin extension token colour for the active mode.
    pub fn extension_colour(&self, plugin_id: &str, token_name: &str) -> Option<ColourRGBA> {
        let mode = self.mode();
        self.extensions
            .read()
            .unwrap()
            .resolve(plugin_id, token_name, mode)
    }

    /// Subscribe to theme change events.
    pub fn on_change(&self, callback: impl Fn(&ThemeEvent) + Send + Sync + 'static) {
        self.subscribers.write().unwrap().push(Box::new(callback));
    }

    /// Emit a theme event to all subscribers.
    fn emit(&self, event: ThemeEvent) {
        if let Ok(subscribers) = self.subscribers.read() {
            for subscriber in subscribers.iter() {
                subscriber(&event);
            }
        }
    }
}

impl Default for ThemeApi {
    fn default() -> Self {
        Self::new()
    }
}

impl std::fmt::Debug for ThemeApi {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("ThemeApi")
            .field("theme", &self.theme_name())
            .field("mode", &self.mode())
            .finish()
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::sync::atomic::{AtomicBool, Ordering};

    #[test]
    fn api_colour_returns_correct_value() {
        // Validates: Requirement 8.7
        let api = ThemeApi::new();
        let bg = api.colour(ColourToken::EditorBackground);
        let palette = defaults::dark_palette();
        assert_eq!(bg, palette.editor.background);
    }

    #[test]
    fn api_mode_switch_updates_palette() {
        // Validates: Requirement 5.7
        let api = ThemeApi::new();
        assert_eq!(api.mode(), VisualMode::Dark);
        api.set_mode(VisualMode::Light);
        assert_eq!(api.mode(), VisualMode::Light);
        // Colours should reflect light mode
        let light = defaults::light_palette();
        assert_eq!(
            api.colour(ColourToken::EditorBackground),
            light.editor.background
        );
    }

    #[test]
    fn api_zoom_level_controls_effective_size() {
        // Validates: Requirement 4.7, 4.8
        let api = ThemeApi::new();
        assert_eq!(api.zoom_level(), 0);
        api.set_zoom_level(5);
        assert_eq!(api.zoom_level(), 5);
        assert_eq!(api.effective_monospace_size(), 14.0 + 5.0);
    }

    #[test]
    fn api_element_colour_override_and_reset() {
        // Validates: Requirement 10.6
        let api = ThemeApi::new();
        api.set_element_colour(Element::CaretFg, ColourRGBA::rgb(255, 0, 0));
        assert_eq!(
            api.element_colour(Element::CaretFg),
            Some(ColourRGBA::rgb(255, 0, 0))
        );
        api.reset_element(Element::CaretFg);
        assert_eq!(api.element_colour(Element::CaretFg), None);
    }

    #[test]
    fn api_notifies_subscribers_on_mode_change() {
        // Validates: Requirement 5.4, 7.7
        let api = ThemeApi::new();
        let notified = Arc::new(AtomicBool::new(false));
        let notified_clone = notified.clone();
        api.on_change(move |event| {
            if matches!(event, ThemeEvent::ModeChanged { .. }) {
                notified_clone.store(true, Ordering::SeqCst);
            }
        });
        api.set_mode(VisualMode::Light);
        assert!(notified.load(Ordering::SeqCst));
    }

    #[test]
    fn api_is_send_and_sync() {
        // Validates: Requirement 7.2, 12.3
        fn assert_send_sync<T: Send + Sync>() {}
        assert_send_sync::<ThemeApi>();
    }
}
