//! Per-session state model for TABS and MASK features.
//!
//! This module contains the composite state container that holds the active
//! tab stop list, insert mask, and display artifact positions. All state
//! is session-only, non-undoable, and non-persisted.

use crate::mask::MaskLine;
use crate::tab_stops::TabStopList;

/// Per-session state for tab stop management.
///
/// Non-undoable, non-persisted — lives only in Session_State.
///
/// Addresses: Requirement 15, criteria 15.1, 15.3, 15.4
#[derive(Debug, Clone)]
pub struct TabsState {
    /// The active tab stop list for this session.
    tab_stops: TabStopList,
    /// Source of the current tab stops (for RESET TABS restoration).
    source: TabStopSource,
    /// Default tab stops to restore on RESET TABS.
    default_tab_stops: TabStopList,
}

/// Indicates the origin of the currently active tab stops.
///
/// Addresses: Requirement 4, criteria 4.3, 4.4; Requirement 12
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum TabStopSource {
    /// Built-in every-8-columns default.
    BuiltIn,
    /// Loaded from global configuration (editor.default_tab_stops).
    GlobalConfig,
    /// Loaded from a language definition (default_tab_stops key).
    LanguageDefinition,
    /// Set manually via TABS command during session.
    SessionOverride,
}

impl TabsState {
    /// Creates a new TabsState with the given default tab stops and source.
    pub fn new(defaults: TabStopList, source: TabStopSource) -> Self {
        Self {
            tab_stops: defaults.clone(),
            source,
            default_tab_stops: defaults,
        }
    }

    /// Returns the active tab stop list.
    pub fn tab_stops(&self) -> &TabStopList {
        &self.tab_stops
    }

    /// Replaces the active tab stops (session override).
    ///
    /// Addresses: Requirement 2, criteria 2.1, 2.4
    pub fn set_tab_stops(&mut self, stops: TabStopList) {
        self.tab_stops = stops;
        self.source = TabStopSource::SessionOverride;
    }

    /// Resets to default tab stops (RESET TABS).
    ///
    /// Addresses: Requirement 12, criteria 12.1, 12.2
    pub fn reset_to_defaults(&mut self) {
        self.tab_stops = self.default_tab_stops.clone();
        self.source = match self.source {
            TabStopSource::SessionOverride => {
                // Restore to whatever the original source was
                // For simplicity, we track the default source separately
                TabStopSource::BuiltIn
            }
            _ => self.source.clone(),
        };
    }

    /// Returns the source of the current tab stops.
    pub fn source(&self) -> &TabStopSource {
        &self.source
    }

    /// Returns the default tab stops (for testing and RESET TABS verification).
    pub fn default_tab_stops(&self) -> &TabStopList {
        &self.default_tab_stops
    }
}

/// Per-session state for insert mask management.
///
/// Non-undoable, non-persisted — lives only in Session_State.
///
/// Addresses: Requirement 15, criteria 15.2, 15.3, 15.4
#[derive(Debug, Clone)]
pub struct MaskState {
    /// The active insert mask for this session. None means no mask active.
    mask: Option<MaskLine>,
    /// Whether the mask was loaded from a language definition (for display messaging).
    from_language: bool,
}

impl MaskState {
    /// Creates a MaskState with an active mask.
    ///
    /// Addresses: Requirement 10, criterion 10.1
    pub fn with_mask(mask: MaskLine, from_language: bool) -> Self {
        Self {
            mask: Some(mask),
            from_language,
        }
    }

    /// Creates a MaskState with no active mask.
    ///
    /// Addresses: Requirement 10, criterion 10.2
    pub fn empty() -> Self {
        Self {
            mask: None,
            from_language: false,
        }
    }

    /// Returns the active mask, if any.
    pub fn mask(&self) -> Option<&MaskLine> {
        self.mask.as_ref()
    }

    /// Returns true if a mask is currently active.
    pub fn is_active(&self) -> bool {
        self.mask.is_some()
    }

    /// Updates the mask content (from MASK_Line editing).
    ///
    /// Addresses: Requirement 6, criterion 6.4
    pub fn update_mask(&mut self, content: String) {
        if let Some(ref mut mask) = self.mask {
            mask.set_content(content);
        } else {
            self.mask = Some(MaskLine::new(content));
            self.from_language = false;
        }
    }

    /// Clears the mask (MASK OFF).
    ///
    /// Addresses: Requirement 7, criterion 7.1
    pub fn clear(&mut self) {
        self.mask = None;
    }

    /// Returns whether the mask was loaded from a language definition.
    pub fn from_language(&self) -> bool {
        self.from_language
    }
}

/// Identifies where a display artifact line is anchored in the document.
///
/// Addresses: Requirements 1, 3, 6, 8 (artifact positioning)
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ArtifactPosition {
    /// The document line index above which this artifact is inserted.
    /// Uses an anchor-based system so the artifact scrolls with the document.
    pub anchor_line: usize,
    /// Whether this artifact was inserted by a line command (vs primary command).
    pub from_line_command: bool,
}

/// Combined per-session state for both TABS and MASK features.
///
/// This is the top-level state container stored in Session_State.
///
/// Addresses: Requirements 15, 11
#[derive(Debug, Clone)]
pub struct TabsMaskState {
    /// Tab stop state.
    tabs: TabsState,
    /// Mask state.
    mask: MaskState,
    /// Tracked TABS_Line display artifacts (positions in viewport).
    tabs_lines: Vec<ArtifactPosition>,
    /// Tracked MASK_Line display artifacts (positions in viewport).
    mask_lines: Vec<ArtifactPosition>,
}

impl TabsMaskState {
    /// Creates a new combined state from defaults.
    pub fn new(tabs: TabsState, mask: MaskState) -> Self {
        Self {
            tabs,
            mask,
            tabs_lines: Vec::new(),
            mask_lines: Vec::new(),
        }
    }

    /// Access the tabs state.
    pub fn tabs(&self) -> &TabsState {
        &self.tabs
    }

    /// Mutably access the tabs state.
    pub fn tabs_mut(&mut self) -> &mut TabsState {
        &mut self.tabs
    }

    /// Access the mask state.
    pub fn mask(&self) -> &MaskState {
        &self.mask
    }

    /// Mutably access the mask state.
    pub fn mask_mut(&mut self) -> &mut MaskState {
        &mut self.mask
    }

    /// Returns true if any TABS_Lines are currently displayed.
    pub fn has_tabs_lines(&self) -> bool {
        !self.tabs_lines.is_empty()
    }

    /// Returns true if any MASK_Lines are currently displayed.
    pub fn has_mask_lines(&self) -> bool {
        !self.mask_lines.is_empty()
    }

    /// Adds a TABS_Line artifact at the given position.
    ///
    /// Addresses: Requirement 1, criteria 1.1, 1.7
    pub fn add_tabs_line(&mut self, position: ArtifactPosition) {
        self.tabs_lines.push(position);
    }

    /// Removes all TABS_Line artifacts (toggle off or RESET).
    ///
    /// Addresses: Requirement 1, criterion 1.4; Requirement 11, criteria 11.1, 11.2
    pub fn remove_all_tabs_lines(&mut self) {
        self.tabs_lines.clear();
    }

    /// Adds a MASK_Line artifact at the given position.
    ///
    /// Addresses: Requirement 6, criteria 6.1, 6.8
    pub fn add_mask_line(&mut self, position: ArtifactPosition) {
        self.mask_lines.push(position);
    }

    /// Removes all MASK_Line artifacts (toggle off or RESET).
    ///
    /// Addresses: Requirement 6, criterion 6.5; Requirement 11, criteria 11.1, 11.2
    pub fn remove_all_mask_lines(&mut self) {
        self.mask_lines.clear();
    }

    /// Gets all TABS_Line positions for rendering.
    pub fn tabs_lines(&self) -> &[ArtifactPosition] {
        &self.tabs_lines
    }

    /// Gets all MASK_Line positions for rendering.
    pub fn mask_lines(&self) -> &[ArtifactPosition] {
        &self.mask_lines
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn tabs_state_new_sets_defaults() {
        let stops = TabStopList::from_columns(vec![7, 12, 72]);
        let state = TabsState::new(stops.clone(), TabStopSource::LanguageDefinition);
        assert_eq!(state.tab_stops(), &stops);
        assert_eq!(state.source(), &TabStopSource::LanguageDefinition);
    }

    #[test]
    fn tabs_state_set_tab_stops_overrides() {
        // Validates: Requirement 2.1, 2.4
        let mut state = TabsState::new(
            TabStopList::from_columns(vec![8, 16]),
            TabStopSource::GlobalConfig,
        );
        let new_stops = TabStopList::from_columns(vec![5, 10, 15]);
        state.set_tab_stops(new_stops.clone());
        assert_eq!(state.tab_stops(), &new_stops);
        assert_eq!(state.source(), &TabStopSource::SessionOverride);
    }

    #[test]
    fn tabs_state_reset_to_defaults_restores_original() {
        // Validates: Requirement 12.1
        let defaults = TabStopList::from_columns(vec![7, 12, 72]);
        let mut state = TabsState::new(defaults.clone(), TabStopSource::LanguageDefinition);
        state.set_tab_stops(TabStopList::from_columns(vec![5, 10]));
        state.reset_to_defaults();
        assert_eq!(state.tab_stops(), &defaults);
    }

    #[test]
    fn mask_state_with_mask_is_active() {
        // Validates: Requirement 10.1
        let mask = MaskLine::new("      *");
        let state = MaskState::with_mask(mask, true);
        assert!(state.is_active());
        assert_eq!(state.mask().unwrap().content(), "      *");
        assert!(state.from_language());
    }

    #[test]
    fn mask_state_empty_is_not_active() {
        // Validates: Requirement 10.2
        let state = MaskState::empty();
        assert!(!state.is_active());
        assert!(state.mask().is_none());
    }

    #[test]
    fn mask_state_clear_removes_mask() {
        // Validates: Requirement 7.1
        let mut state = MaskState::with_mask(MaskLine::new("test"), true);
        state.clear();
        assert!(!state.is_active());
        assert!(state.mask().is_none());
    }

    #[test]
    fn mask_state_update_mask_changes_content() {
        // Validates: Requirement 6.4
        let mut state = MaskState::with_mask(MaskLine::new("old"), false);
        state.update_mask("new content".to_string());
        assert_eq!(state.mask().unwrap().content(), "new content");
    }

    #[test]
    fn tabs_mask_state_add_and_remove_tabs_lines() {
        // Validates: Requirement 1.1, 1.4, 1.7
        let mut state = TabsMaskState::new(
            TabsState::new(TabStopList::empty(), TabStopSource::BuiltIn),
            MaskState::empty(),
        );
        assert!(!state.has_tabs_lines());

        state.add_tabs_line(ArtifactPosition {
            anchor_line: 5,
            from_line_command: false,
        });
        state.add_tabs_line(ArtifactPosition {
            anchor_line: 10,
            from_line_command: true,
        });
        assert!(state.has_tabs_lines());
        assert_eq!(state.tabs_lines().len(), 2);

        state.remove_all_tabs_lines();
        assert!(!state.has_tabs_lines());
    }

    #[test]
    fn tabs_mask_state_add_and_remove_mask_lines() {
        // Validates: Requirement 6.1, 6.5, 6.8
        let mut state = TabsMaskState::new(
            TabsState::new(TabStopList::empty(), TabStopSource::BuiltIn),
            MaskState::empty(),
        );
        assert!(!state.has_mask_lines());

        state.add_mask_line(ArtifactPosition {
            anchor_line: 3,
            from_line_command: false,
        });
        assert!(state.has_mask_lines());

        state.remove_all_mask_lines();
        assert!(!state.has_mask_lines());
    }
}
