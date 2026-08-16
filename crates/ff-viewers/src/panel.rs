//! ViewerPanel — DockablePanel implementation for viewer output.
//!
//! The ViewerPanel hosts the active viewer's rendered output and manages
//! the panel lifecycle (visibility, dock position, content buffer).

use crate::key::ViewerKey;

/// Default dock zone for the viewer panel.
pub const DEFAULT_DOCK_ZONE: &str = "Center";

/// The panel ID registered in the Panel_Registry.
pub const VIEWER_PANEL_ID: &str = "viewer";

/// Panel opening position relative to the active editor.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
pub enum ViewerPosition {
    /// Open in a vertical split to the right of the editor.
    #[default]
    SplitRight,
    /// Open in a horizontal split below the editor.
    SplitBottom,
    /// Open as a tab alongside editor tabs.
    Tab,
    /// Open as a floating window.
    Float,
}

/// The DockablePanel implementation that hosts the active viewer's rendered output.
///
/// Manages the currently active viewer, content buffer, panel lifecycle,
/// and stale-content indicator.
pub struct ViewerPanel {
    /// The currently active viewer key (None if no viewer is active).
    active_viewer_key: Option<ViewerKey>,
    /// Cached content bytes for the current resource.
    content_buffer: Vec<u8>,
    /// The URI of the currently viewed resource.
    current_resource: Option<String>,
    /// Whether the panel is currently visible.
    visible: bool,
    /// Last known dock position for reactivation.
    last_position: ViewerPosition,
    /// Stale-content indicator (set when on_content_changed fails).
    stale: bool,
    /// Dynamic title including active viewer key.
    title: String,
}

impl ViewerPanel {
    /// Create a new ViewerPanel (initially hidden, no active viewer).
    pub fn new() -> Self {
        Self {
            active_viewer_key: None,
            content_buffer: Vec::new(),
            current_resource: None,
            visible: false,
            last_position: ViewerPosition::default(),
            stale: false,
            title: "Preview".to_string(),
        }
    }

    /// Returns the panel ID.
    pub fn panel_id(&self) -> &str {
        VIEWER_PANEL_ID
    }

    /// Returns the default dock zone.
    pub fn default_dock_zone(&self) -> &str {
        DEFAULT_DOCK_ZONE
    }

    /// Returns the dynamic title including active viewer key.
    pub fn title(&self) -> &str {
        &self.title
    }

    /// Returns whether a viewer is currently active.
    pub fn is_active(&self) -> bool {
        self.active_viewer_key.is_some()
    }

    /// Returns whether the panel is currently visible.
    pub fn is_visible(&self) -> bool {
        self.visible
    }

    /// Returns the currently active viewer key (if any).
    pub fn active_viewer_key(&self) -> Option<&ViewerKey> {
        self.active_viewer_key.as_ref()
    }

    /// Returns the current resource URI (if any).
    pub fn current_resource(&self) -> Option<&str> {
        self.current_resource.as_deref()
    }

    /// Returns the content buffer.
    pub fn content_buffer(&self) -> &[u8] {
        &self.content_buffer
    }

    /// Returns whether the content is marked as stale.
    pub fn is_stale(&self) -> bool {
        self.stale
    }

    /// Activate a viewer for the given resource.
    ///
    /// Sets the panel to visible and updates the title to include the viewer key.
    pub fn activate(&mut self, viewer_key: ViewerKey, resource_uri: String, content: Vec<u8>) {
        self.title = format!("Preview: {}", viewer_key.as_str());
        self.active_viewer_key = Some(viewer_key);
        self.current_resource = Some(resource_uri);
        self.content_buffer = content;
        self.visible = true;
        self.stale = false;
    }

    /// Deactivate the current viewer and hide the panel.
    ///
    /// Preserves the last known dock position for future reactivation.
    pub fn deactivate(&mut self) {
        self.active_viewer_key = None;
        self.current_resource = None;
        self.content_buffer.clear();
        self.visible = false;
        self.stale = false;
        self.title = "Preview".to_string();
    }

    /// Update content after a debounced document change.
    ///
    /// Clears the stale indicator on success.
    pub fn refresh_content(&mut self, new_content: Vec<u8>) {
        self.content_buffer = new_content;
        self.stale = false;
    }

    /// Mark content as stale (viewer failed to process update).
    pub fn mark_stale(&mut self) {
        self.stale = true;
    }

    /// Clear the stale indicator after successful refresh.
    pub fn clear_stale(&mut self) {
        self.stale = false;
    }

    /// Set the last known dock position.
    pub fn set_position(&mut self, position: ViewerPosition) {
        self.last_position = position;
    }

    /// Returns the last known dock position.
    pub fn last_position(&self) -> ViewerPosition {
        self.last_position
    }
}

impl Default for ViewerPanel {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn new_panel_is_inactive_and_hidden() {
        // Validates: Requirement 7 AC 1
        let panel = ViewerPanel::new();
        assert!(!panel.is_active());
        assert!(!panel.is_visible());
        assert_eq!(panel.panel_id(), "viewer");
        assert_eq!(panel.default_dock_zone(), "Center");
    }

    #[test]
    fn activate_makes_panel_active_and_visible() {
        // Validates: Requirement 7 AC 3
        let mut panel = ViewerPanel::new();
        let key = ViewerKey::new("hex").unwrap();
        panel.activate(key.clone(), "file:///test.bin".to_string(), vec![0x00]);

        assert!(panel.is_active());
        assert!(panel.is_visible());
        assert_eq!(panel.active_viewer_key(), Some(&key));
    }

    #[test]
    fn title_includes_active_viewer_key() {
        // Validates: Requirement 7 AC 1
        let mut panel = ViewerPanel::new();
        let key = ViewerKey::new("asa-report").unwrap();
        panel.activate(key, "file:///report.lst".to_string(), vec![]);

        assert_eq!(panel.title(), "Preview: asa-report");
    }

    #[test]
    fn deactivate_hides_panel_preserving_position() {
        // Validates: Requirement 7 AC 4
        let mut panel = ViewerPanel::new();
        panel.set_position(ViewerPosition::SplitBottom);
        let key = ViewerKey::new("hex").unwrap();
        panel.activate(key, "file:///test".to_string(), vec![]);

        panel.deactivate();

        assert!(!panel.is_active());
        assert!(!panel.is_visible());
        assert_eq!(panel.last_position(), ViewerPosition::SplitBottom);
    }

    #[test]
    fn refresh_content_updates_buffer_and_clears_stale() {
        // Validates: Requirement 9 AC 1
        let mut panel = ViewerPanel::new();
        let key = ViewerKey::new("csv-table").unwrap();
        panel.activate(key, "file:///data.csv".to_string(), b"old".to_vec());
        panel.mark_stale();
        assert!(panel.is_stale());

        panel.refresh_content(b"new content".to_vec());
        assert_eq!(panel.content_buffer(), b"new content");
        assert!(!panel.is_stale());
    }

    #[test]
    fn mark_stale_sets_indicator() {
        // Validates: Requirement 9 AC 5
        let mut panel = ViewerPanel::new();
        let key = ViewerKey::new("hex").unwrap();
        panel.activate(key, "file:///test".to_string(), vec![]);

        panel.mark_stale();
        assert!(panel.is_stale());

        panel.clear_stale();
        assert!(!panel.is_stale());
    }

    #[test]
    fn panel_does_not_expose_editing_affordances() {
        // Validates: Requirement 8 AC 2, AC 3
        // The ViewerPanel API has no methods that modify document content.
        // content_buffer() returns &[u8] — immutable reference.
        let mut panel = ViewerPanel::new();
        let key = ViewerKey::new("hex").unwrap();
        panel.activate(key, "file:///test".to_string(), b"content".to_vec());

        let buffer = panel.content_buffer();
        // buffer is &[u8] — no mutation possible
        assert_eq!(buffer, b"content");
    }
}
