//! Clipboard engine — orchestrates clipboard read/write with metadata tracking.
//!
//! The [`ClipboardEngine`] wraps a [`ClipboardProvider`] and maintains internal
//! metadata (mode, segments) to detect whether clipboard content was written by
//! this editor instance or by an external application.

use crate::config::ClipboardConfig;
use crate::entry::ClipboardEntry;
use crate::error::ClipboardError;
use crate::history::ClipboardHistoryRing;
use crate::provider::ClipboardProvider;

/// Orchestrates clipboard read/write with structured [`ClipboardEntry`] metadata.
///
/// Stores the last-written entry locally to detect internal vs external clipboard
/// content. Integrates with the clipboard history ring for recent entries.
///
/// # External Detection
///
/// When reading from the clipboard, if the system clipboard text matches the
/// last internally written text, the original [`ClipboardEntry`] (with its mode
/// and segments) is returned. If the text differs (external modification), a
/// new entry with [`ClipboardMode::Stream`] is returned.
pub struct ClipboardEngine {
    provider: Box<dyn ClipboardProvider>,
    last_written: Option<ClipboardEntry>,
    history: ClipboardHistoryRing,
    config: ClipboardConfig,
}

impl ClipboardEngine {
    /// Create a new clipboard engine with the given provider and configuration.
    pub fn new(provider: Box<dyn ClipboardProvider>, config: ClipboardConfig) -> Self {
        let history_capacity = 20; // reasonable default
        Self {
            provider,
            last_written: None,
            history: ClipboardHistoryRing::new(history_capacity),
            config,
        }
    }

    /// Write a [`ClipboardEntry`] to the system clipboard and record in history.
    ///
    /// # Errors
    ///
    /// Returns [`ClipboardError::WriteFailed`] or [`ClipboardError::Unavailable`]
    /// if the system clipboard cannot be written to.
    pub fn write(&mut self, entry: ClipboardEntry) -> Result<(), ClipboardError> {
        self.provider.write_text(entry.text())?;
        self.history.push(entry.clone());
        self.last_written = Some(entry);
        Ok(())
    }

    /// Read from the system clipboard, returning a structured [`ClipboardEntry`].
    ///
    /// If the system clipboard text matches our last write, returns the original
    /// entry with its mode and segments. Otherwise returns a new entry with
    /// [`ClipboardMode::Stream`] (external content).
    ///
    /// # Errors
    ///
    /// Returns [`ClipboardError::Empty`] if the clipboard is empty.
    /// Returns [`ClipboardError::Unavailable`] if the clipboard cannot be accessed.
    pub fn read(&self) -> Result<ClipboardEntry, ClipboardError> {
        let text = self.provider.read_text()?;

        // Check if this matches our last write (internal content)
        if let Some(ref last) = self.last_written {
            if last.text() == text {
                return Ok(last.clone());
            }
        }

        // External content — default to Stream mode
        Ok(ClipboardEntry::stream(text))
    }

    /// Check if the system clipboard has text content available for paste.
    ///
    /// # Errors
    ///
    /// Returns [`ClipboardError::Unavailable`] if the clipboard state cannot be queried.
    pub fn has_content(&self) -> Result<bool, ClipboardError> {
        self.provider.has_text()
    }

    /// Check whether the clipboard provider is available.
    pub fn is_available(&self) -> bool {
        self.provider.is_available()
    }

    /// Access the clipboard history ring.
    pub fn history(&self) -> &ClipboardHistoryRing {
        &self.history
    }

    /// Access the clipboard history ring mutably.
    pub fn history_mut(&mut self) -> &mut ClipboardHistoryRing {
        &mut self.history
    }

    /// Update configuration (e.g., after hot-reload).
    pub fn update_config(&mut self, config: ClipboardConfig) {
        self.config = config;
    }

    /// Get the current configuration.
    pub fn config(&self) -> &ClipboardConfig {
        &self.config
    }

    /// Get the last entry written internally (for testing/inspection).
    pub fn last_written(&self) -> Option<&ClipboardEntry> {
        self.last_written.as_ref()
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::entry::ClipboardMode;
    use crate::provider::InMemoryClipboardProvider;

    fn make_engine() -> ClipboardEngine {
        let provider = InMemoryClipboardProvider::new();
        ClipboardEngine::new(Box::new(provider), ClipboardConfig::default())
    }

    fn make_engine_with_provider() -> (ClipboardEngine, InMemoryClipboardProvider) {
        let provider = InMemoryClipboardProvider::new();
        let provider_clone = provider.clone();
        let engine = ClipboardEngine::new(Box::new(provider), ClipboardConfig::default());
        (engine, provider_clone)
    }

    #[test]
    fn write_and_read_returns_identical_entry() {
        // Validates: Requirement 1.2, 1.3
        let mut engine = make_engine();
        let entry = ClipboardEntry::stream("hello world".to_string());
        engine.write(entry.clone()).unwrap();

        let read_back = engine.read().unwrap();
        assert_eq!(read_back.text(), "hello world");
        assert_eq!(read_back.mode(), ClipboardMode::Stream);
    }

    #[test]
    fn write_preserves_mode_through_read_cycle() {
        // Validates: Requirement 1.4, 1.5
        let mut engine = make_engine();

        let entry = ClipboardEntry::line("full line\n".to_string());
        engine.write(entry).unwrap();
        let read_back = engine.read().unwrap();
        assert_eq!(read_back.mode(), ClipboardMode::Line);

        let segments = vec!["col1".to_string(), "col2".to_string()];
        let rect_entry = ClipboardEntry::rectangular(segments.clone());
        engine.write(rect_entry).unwrap();
        let read_back = engine.read().unwrap();
        assert_eq!(read_back.mode(), ClipboardMode::Rectangular);
        assert_eq!(read_back.segments(), &segments);
    }

    #[test]
    fn external_modification_defaults_to_stream_mode() {
        // Validates: Requirement 1.5
        let (mut engine, provider) = make_engine_with_provider();

        let entry = ClipboardEntry::line("internal\n".to_string());
        engine.write(entry).unwrap();

        // Simulate external app modifying clipboard
        provider.set_content_externally("external content");

        let read_back = engine.read().unwrap();
        assert_eq!(read_back.text(), "external content");
        assert_eq!(read_back.mode(), ClipboardMode::Stream);
        assert!(read_back.segments().is_empty());
    }

    #[test]
    fn read_empty_clipboard_returns_error() {
        // Validates: Requirement 6.1
        let engine = make_engine();
        let result = engine.read();
        assert!(matches!(result, Err(ClipboardError::Empty)));
    }

    #[test]
    fn unavailable_provider_returns_error_on_write() {
        // Validates: Requirement 1.6
        let (mut engine, provider) = make_engine_with_provider();
        provider.set_available(false);

        let entry = ClipboardEntry::stream("text".to_string());
        let result = engine.write(entry);
        assert!(matches!(result, Err(ClipboardError::Unavailable { .. })));
    }

    #[test]
    fn unavailable_provider_returns_error_on_read() {
        // Validates: Requirement 1.6
        let (engine, provider) = make_engine_with_provider();
        provider.set_available(false);

        let result = engine.read();
        assert!(matches!(result, Err(ClipboardError::Unavailable { .. })));
    }

    #[test]
    fn has_content_delegates_to_provider() {
        // Validates: Requirement 1.1
        let (mut engine, _) = make_engine_with_provider();
        assert!(!engine.has_content().unwrap());

        engine
            .write(ClipboardEntry::stream("data".to_string()))
            .unwrap();
        assert!(engine.has_content().unwrap());
    }

    #[test]
    fn is_available_delegates_to_provider() {
        // Validates: Requirement 1.7
        let (engine, provider) = make_engine_with_provider();
        assert!(engine.is_available());
        provider.set_available(false);
        assert!(!engine.is_available());
    }

    #[test]
    fn write_pushes_to_history() {
        let mut engine = make_engine();
        assert!(engine.history().is_empty());

        engine
            .write(ClipboardEntry::stream("first".to_string()))
            .unwrap();
        assert_eq!(engine.history().len(), 1);

        engine
            .write(ClipboardEntry::stream("second".to_string()))
            .unwrap();
        assert_eq!(engine.history().len(), 2);
    }

    #[test]
    fn engine_never_panics_on_any_operation_sequence() {
        // Validates: Requirement 1.6
        let (mut engine, provider) = make_engine_with_provider();

        // Read when empty
        let _ = engine.read();

        // Write, then read
        let _ = engine.write(ClipboardEntry::stream("a".to_string()));
        let _ = engine.read();

        // Make unavailable, try operations
        provider.set_available(false);
        let _ = engine.read();
        let _ = engine.write(ClipboardEntry::stream("b".to_string()));
        let _ = engine.has_content();
        let _ = engine.is_available();

        // Make available again
        provider.set_available(true);
        let _ = engine.write(ClipboardEntry::stream("c".to_string()));
        let _ = engine.read();
    }
}
