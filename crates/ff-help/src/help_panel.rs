//! Help Panel model — manages panel state, navigation, search.
//!
//! The GUI shell reads this model to render the panel. This module provides
//! the non-GUI logic for topic display, breadcrumb computation, TOC
//! extraction, and toggle behaviour.

use std::sync::Arc;

use crate::config::HelpConfig;
use crate::error::HelpError;
use crate::navigation::NavigationStack;
use crate::registry::HelpTopicRegistry;
use crate::search::{HelpSearch, SearchResult};
use crate::topic::HelpTopic;
use crate::topic_key::{TopicCategory, TopicKey};

/// A breadcrumb entry showing the path to the current topic.
#[derive(Debug, Clone, PartialEq)]
pub struct BreadcrumbEntry {
    /// Display label.
    pub label: String,
    /// Navigation target.
    pub topic_key: TopicKey,
}

/// A Table of Contents entry extracted from Markdown headings.
#[derive(Debug, Clone, PartialEq)]
pub struct TocEntry {
    /// Heading text.
    pub heading: String,
    /// Heading level (1–6).
    pub level: u8,
    /// Anchor identifier for scroll-to.
    pub anchor: String,
}

/// The core model for the Help Panel — manages display state, navigation, search.
///
/// The GUI shell reads this model to render the panel content.
pub struct HelpPanelModel {
    /// Currently displayed topic (None if panel is closed).
    current_topic: Option<HelpTopic>,
    /// Whether the panel is open.
    is_open: bool,
    /// Breadcrumb path to current topic.
    breadcrumb: Vec<BreadcrumbEntry>,
    /// Table of contents for current topic.
    toc_entries: Vec<TocEntry>,
    /// Whether the TOC sidebar is expanded.
    toc_visible: bool,
    /// Vertical scroll position.
    scroll_offset: usize,
    /// Navigation history.
    navigation: NavigationStack,
    /// Search engine.
    search: HelpSearch,
    /// Active search results.
    search_results: Vec<SearchResult>,
    /// Active search query.
    search_query: String,
    /// Topic registry reference.
    registry: Arc<HelpTopicRegistry>,
    /// Configuration.
    config: HelpConfig,
    /// Whether the panel is too narrow to display content.
    is_narrow: bool,
}

impl HelpPanelModel {
    /// Create a new Help Panel model with the given registry and config.
    pub fn new(registry: Arc<HelpTopicRegistry>, config: HelpConfig) -> Self {
        Self {
            current_topic: None,
            is_open: false,
            breadcrumb: Vec::new(),
            toc_entries: Vec::new(),
            toc_visible: false,
            scroll_offset: 0,
            navigation: NavigationStack::new(),
            search: HelpSearch::new(),
            search_results: Vec::new(),
            search_query: String::new(),
            registry,
            config,
            is_narrow: false,
        }
    }

    /// Whether the panel is currently open.
    pub fn is_open(&self) -> bool {
        self.is_open
    }

    /// Returns the currently displayed topic, if any.
    pub fn current_topic(&self) -> Option<&HelpTopic> {
        self.current_topic.as_ref()
    }

    /// Returns the current topic key, if any.
    pub fn current_topic_key(&self) -> Option<&TopicKey> {
        self.current_topic.as_ref().map(|t| t.key())
    }

    /// Returns the breadcrumb path.
    pub fn breadcrumb(&self) -> &[BreadcrumbEntry] {
        &self.breadcrumb
    }

    /// Returns the TOC entries.
    pub fn toc_entries(&self) -> &[TocEntry] {
        &self.toc_entries
    }

    /// Whether the TOC sidebar is visible.
    pub fn toc_visible(&self) -> bool {
        self.toc_visible
    }

    /// Returns the current scroll offset.
    pub fn scroll_offset(&self) -> usize {
        self.scroll_offset
    }

    /// Returns the navigation stack.
    pub fn navigation(&self) -> &NavigationStack {
        &self.navigation
    }

    /// Returns the active search query.
    pub fn search_query(&self) -> &str {
        &self.search_query
    }

    /// Returns the active search results.
    pub fn search_results(&self) -> &[SearchResult] {
        &self.search_results
    }

    /// Whether the panel is too narrow for content.
    pub fn is_narrow(&self) -> bool {
        self.is_narrow
    }

    /// Open the panel and display the given topic.
    ///
    /// # Errors
    ///
    /// Returns `HelpError::TopicNotFound` if the key is not in the registry.
    pub fn show_topic(&mut self, key: &TopicKey) -> Result<(), HelpError> {
        let topic = self
            .registry
            .get(key)
            .ok_or_else(|| HelpError::TopicNotFound {
                key: key.to_string(),
            })?;

        self.navigation.push(key.clone());
        self.breadcrumb = Self::compute_breadcrumb(&topic);
        self.toc_entries = Self::extract_toc(&topic);
        self.current_topic = Some(topic);
        self.is_open = true;
        self.scroll_offset = 0;
        self.search_query.clear();
        self.search_results.clear();
        Ok(())
    }

    /// Close the Help Panel and clear navigation history.
    pub fn close(&mut self) {
        self.is_open = false;
        self.current_topic = None;
        self.breadcrumb.clear();
        self.toc_entries.clear();
        self.scroll_offset = 0;
        self.navigation.clear();
        self.search_query.clear();
        self.search_results.clear();
    }

    /// Toggle: if open with same topic, close; otherwise show new topic.
    ///
    /// # Errors
    ///
    /// Returns `HelpError::TopicNotFound` if the key is not in the registry
    /// (only when opening/navigating).
    pub fn toggle(&mut self, key: &TopicKey) -> Result<(), HelpError> {
        if self.is_open && self.current_topic_key() == Some(key) {
            self.close();
            return Ok(());
        }
        self.show_topic(key)
    }

    /// Navigate back in history.
    ///
    /// # Errors
    ///
    /// Returns `HelpError::NavigationStackEmpty` if there is no previous topic.
    pub fn navigate_back(&mut self) -> Result<(), HelpError> {
        let key =
            self.navigation
                .back()
                .cloned()
                .ok_or_else(|| HelpError::NavigationStackEmpty {
                    direction: "back".to_string(),
                })?;

        let topic = self
            .registry
            .get(&key)
            .ok_or_else(|| HelpError::TopicNotFound {
                key: key.to_string(),
            })?;

        self.breadcrumb = Self::compute_breadcrumb(&topic);
        self.toc_entries = Self::extract_toc(&topic);
        self.current_topic = Some(topic);
        self.scroll_offset = 0;
        Ok(())
    }

    /// Navigate forward in history.
    ///
    /// # Errors
    ///
    /// Returns `HelpError::NavigationStackEmpty` if there is no next topic.
    pub fn navigate_forward(&mut self) -> Result<(), HelpError> {
        let key =
            self.navigation
                .forward()
                .cloned()
                .ok_or_else(|| HelpError::NavigationStackEmpty {
                    direction: "forward".to_string(),
                })?;

        let topic = self
            .registry
            .get(&key)
            .ok_or_else(|| HelpError::TopicNotFound {
                key: key.to_string(),
            })?;

        self.breadcrumb = Self::compute_breadcrumb(&topic);
        self.toc_entries = Self::extract_toc(&topic);
        self.current_topic = Some(topic);
        self.scroll_offset = 0;
        Ok(())
    }

    /// Follow a cross-reference link to another topic.
    ///
    /// # Errors
    ///
    /// Returns `HelpError::TopicNotFound` if the linked topic doesn't exist.
    pub fn follow_link(&mut self, key: &TopicKey) -> Result<(), HelpError> {
        self.show_topic(key)
    }

    /// Execute a search query. Updates search results.
    pub fn search(&mut self, query: &str) {
        self.search_query = query.to_string();
        let all_topics = self.registry.all_topics();
        self.search_results = self.search.query(&all_topics, query);
    }

    /// Clear active search and return to current topic display.
    pub fn clear_search(&mut self) {
        self.search_query.clear();
        self.search_results.clear();
    }

    /// Scroll up by one unit.
    pub fn scroll_up(&mut self) {
        self.scroll_offset = self.scroll_offset.saturating_sub(1);
    }

    /// Scroll down by one unit.
    pub fn scroll_down(&mut self) {
        self.scroll_offset = self.scroll_offset.saturating_add(1);
    }

    /// Set narrow-width state based on panel width.
    ///
    /// When width is below 200 pixels, suggests resize/undock.
    pub fn set_width(&mut self, width_px: f32) {
        self.is_narrow = width_px < 200.0;
    }

    /// Update configuration.
    pub fn update_config(&mut self, config: HelpConfig) {
        self.config = config;
    }

    /// Returns the current configuration.
    pub fn config(&self) -> &HelpConfig {
        &self.config
    }

    /// Compute the breadcrumb path for a topic based on its category.
    fn compute_breadcrumb(topic: &HelpTopic) -> Vec<BreadcrumbEntry> {
        let mut crumbs = vec![BreadcrumbEntry {
            label: "Help".to_string(),
            topic_key: TopicKey::index(),
        }];

        match topic.key().category() {
            TopicCategory::Command => {
                crumbs.push(BreadcrumbEntry {
                    label: "Commands".to_string(),
                    topic_key: TopicKey::index(), // Commands section of index
                });
                crumbs.push(BreadcrumbEntry {
                    label: topic.title().to_string(),
                    topic_key: topic.key().clone(),
                });
            }
            TopicCategory::LineCommand => {
                crumbs.push(BreadcrumbEntry {
                    label: "Line Commands".to_string(),
                    topic_key: TopicKey::line_index(),
                });
                crumbs.push(BreadcrumbEntry {
                    label: topic.title().to_string(),
                    topic_key: topic.key().clone(),
                });
            }
            TopicCategory::Mode => {
                crumbs.push(BreadcrumbEntry {
                    label: "Modes".to_string(),
                    topic_key: TopicKey::index(),
                });
                crumbs.push(BreadcrumbEntry {
                    label: topic.title().to_string(),
                    topic_key: topic.key().clone(),
                });
            }
            TopicCategory::Feature => {
                crumbs.push(BreadcrumbEntry {
                    label: "Features".to_string(),
                    topic_key: TopicKey::index(),
                });
                crumbs.push(BreadcrumbEntry {
                    label: topic.title().to_string(),
                    topic_key: topic.key().clone(),
                });
            }
            TopicCategory::Config => {
                crumbs.push(BreadcrumbEntry {
                    label: "Configuration".to_string(),
                    topic_key: TopicKey::feature("configuration"),
                });
                crumbs.push(BreadcrumbEntry {
                    label: topic.title().to_string(),
                    topic_key: topic.key().clone(),
                });
            }
            TopicCategory::Api => {
                crumbs.push(BreadcrumbEntry {
                    label: "Macro API".to_string(),
                    topic_key: TopicKey::feature("macros"),
                });
                crumbs.push(BreadcrumbEntry {
                    label: topic.title().to_string(),
                    topic_key: topic.key().clone(),
                });
            }
            TopicCategory::Index | TopicCategory::GettingStarted => {
                // Top-level topics — just "Help"
            }
        }

        crumbs
    }

    /// Extract TOC entries from a topic's Markdown body.
    fn extract_toc(topic: &HelpTopic) -> Vec<TocEntry> {
        let mut entries = Vec::new();
        for line in topic.body().lines() {
            let trimmed = line.trim();
            if trimmed.starts_with('#') {
                let level = trimmed.chars().take_while(|c| *c == '#').count() as u8;
                let text = trimmed.trim_start_matches('#').trim();
                if !text.is_empty() {
                    let anchor = text
                        .to_lowercase()
                        .replace(' ', "-")
                        .replace(|c: char| !c.is_alphanumeric() && c != '-', "");
                    entries.push(TocEntry {
                        heading: text.to_string(),
                        level,
                        anchor,
                    });
                }
            }
        }
        entries
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::topic::TopicSource;
    use std::path::PathBuf;

    fn make_registry_with_topic(key: TopicKey, title: &str, body: &str) -> Arc<HelpTopicRegistry> {
        let registry = Arc::new(HelpTopicRegistry::new());
        registry.register_file_topic(HelpTopic::new(
            key,
            title.to_string(),
            body.to_string(),
            TopicSource::FileBased {
                file_path: PathBuf::from("test.help.md"),
            },
        ));
        registry
    }

    // Validates: Requirement 2.1 — Panel open/close state
    #[test]
    fn panel_starts_closed() {
        let registry = Arc::new(HelpTopicRegistry::new());
        let panel = HelpPanelModel::new(registry, HelpConfig::default());
        assert!(!panel.is_open());
        assert!(panel.current_topic().is_none());
    }

    // Validates: Requirement 2.1 — Show topic opens panel
    #[test]
    fn show_topic_opens_panel() {
        let key = TopicKey::command("FIND");
        let registry = make_registry_with_topic(key.clone(), "FIND", "## Syntax\n\nfind text");
        let mut panel = HelpPanelModel::new(registry, HelpConfig::default());

        panel.show_topic(&key).unwrap();
        assert!(panel.is_open());
        assert_eq!(panel.current_topic().unwrap().title(), "FIND");
    }

    // Validates: Requirement 2.4 — Close dismisses panel
    #[test]
    fn close_panel_clears_state() {
        let key = TopicKey::command("FIND");
        let registry = make_registry_with_topic(key.clone(), "FIND", "body");
        let mut panel = HelpPanelModel::new(registry, HelpConfig::default());

        panel.show_topic(&key).unwrap();
        panel.close();
        assert!(!panel.is_open());
        assert!(panel.current_topic().is_none());
    }

    // Validates: Requirement 1.6 — Toggle behaviour
    #[test]
    fn toggle_same_topic_closes_panel() {
        let key = TopicKey::command("FIND");
        let registry = make_registry_with_topic(key.clone(), "FIND", "body");
        let mut panel = HelpPanelModel::new(registry, HelpConfig::default());

        panel.toggle(&key).unwrap(); // opens
        assert!(panel.is_open());

        panel.toggle(&key).unwrap(); // closes (same topic)
        assert!(!panel.is_open());
    }

    // Validates: Requirement 1.6 — Toggle to different topic navigates
    #[test]
    fn toggle_different_topic_navigates() {
        let key1 = TopicKey::command("FIND");
        let key2 = TopicKey::command("CHANGE");
        let registry = Arc::new(HelpTopicRegistry::new());
        registry.register_file_topic(HelpTopic::new(
            key1.clone(),
            "FIND".to_string(),
            "body".to_string(),
            TopicSource::FileBased {
                file_path: PathBuf::from("test.help.md"),
            },
        ));
        registry.register_file_topic(HelpTopic::new(
            key2.clone(),
            "CHANGE".to_string(),
            "body".to_string(),
            TopicSource::FileBased {
                file_path: PathBuf::from("test.help.md"),
            },
        ));
        let mut panel = HelpPanelModel::new(registry, HelpConfig::default());

        panel.toggle(&key1).unwrap();
        assert_eq!(panel.current_topic_key(), Some(&key1));

        panel.toggle(&key2).unwrap(); // different topic — navigates
        assert!(panel.is_open());
        assert_eq!(panel.current_topic_key(), Some(&key2));
    }

    // Validates: Requirement 2.8 — Breadcrumb computation
    #[test]
    fn breadcrumb_computed_for_command_topic() {
        let key = TopicKey::command("FIND");
        let registry = make_registry_with_topic(key.clone(), "FIND Command", "body");
        let mut panel = HelpPanelModel::new(registry, HelpConfig::default());

        panel.show_topic(&key).unwrap();
        let crumbs = panel.breadcrumb();
        assert_eq!(crumbs[0].label, "Help");
        assert_eq!(crumbs[1].label, "Commands");
        assert_eq!(crumbs[2].label, "FIND Command");
    }

    // Validates: Requirement 3.4 — TOC extraction from headings
    #[test]
    fn toc_extracted_from_markdown_headings() {
        let body = "## Syntax\n\ntext\n\n## Description\n\nmore text\n\n### Examples\n\ncode";
        let key = TopicKey::command("TEST");
        let registry = make_registry_with_topic(key.clone(), "TEST", body);
        let mut panel = HelpPanelModel::new(registry, HelpConfig::default());

        panel.show_topic(&key).unwrap();
        let toc = panel.toc_entries();
        assert_eq!(toc.len(), 3);
        assert_eq!(toc[0].heading, "Syntax");
        assert_eq!(toc[0].level, 2);
        assert_eq!(toc[1].heading, "Description");
        assert_eq!(toc[2].heading, "Examples");
        assert_eq!(toc[2].level, 3);
    }

    // Validates: Requirement 2.9 — Narrow width detection
    #[test]
    fn narrow_width_detection() {
        let registry = Arc::new(HelpTopicRegistry::new());
        let mut panel = HelpPanelModel::new(registry, HelpConfig::default());

        panel.set_width(300.0);
        assert!(!panel.is_narrow());

        panel.set_width(150.0);
        assert!(panel.is_narrow());
    }

    // Validates: Requirement 2.10 — Scroll bounds
    #[test]
    fn scroll_up_does_not_underflow() {
        let registry = Arc::new(HelpTopicRegistry::new());
        let mut panel = HelpPanelModel::new(registry, HelpConfig::default());
        assert_eq!(panel.scroll_offset(), 0);
        panel.scroll_up();
        assert_eq!(panel.scroll_offset(), 0);
    }

    // Validates: Requirement 5.5 — Topic not found error
    #[test]
    fn show_topic_returns_error_for_missing_key() {
        let registry = Arc::new(HelpTopicRegistry::new());
        let mut panel = HelpPanelModel::new(registry, HelpConfig::default());
        let result = panel.show_topic(&TopicKey::command("NONEXISTENT"));
        assert!(result.is_err());
    }
}
