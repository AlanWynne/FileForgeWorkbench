//! Clipboard provider trait — platform-independent clipboard access abstraction.
//!
//! The [`ClipboardProvider`] trait abstracts over OS-specific clipboard APIs
//! (Win32, X11/Wayland, NSPasteboard). GUI shells provide the concrete
//! implementation at application startup; tests use [`InMemoryClipboardProvider`].

use crate::error::ClipboardError;
use std::sync::{Arc, Mutex};

/// Platform-independent clipboard access abstraction.
///
/// Implementors wrap OS-specific clipboard APIs. The trait is object-safe
/// and requires `Send + Sync` for cross-thread usage in the workbench.
///
/// # Errors
///
/// Methods return [`ClipboardError`] for platform failures, empty clipboard,
/// or non-text content conditions.
pub trait ClipboardProvider: Send + Sync {
    /// Write plain UTF-8 text to the system clipboard.
    ///
    /// # Errors
    ///
    /// Returns [`ClipboardError::WriteFailed`] if the platform clipboard cannot be written.
    /// Returns [`ClipboardError::Unavailable`] if the clipboard is not accessible.
    fn write_text(&self, text: &str) -> Result<(), ClipboardError>;

    /// Read plain text from the system clipboard.
    ///
    /// # Errors
    ///
    /// Returns [`ClipboardError::Empty`] if the clipboard holds no content.
    /// Returns [`ClipboardError::NoTextContent`] if the clipboard holds non-text data.
    /// Returns [`ClipboardError::Unavailable`] if the clipboard is not accessible.
    fn read_text(&self) -> Result<String, ClipboardError>;

    /// Check whether the clipboard currently contains text content.
    ///
    /// # Errors
    ///
    /// Returns [`ClipboardError::Unavailable`] if the clipboard state cannot be queried.
    fn has_text(&self) -> Result<bool, ClipboardError>;

    /// Check whether the clipboard is accessible (permissions, platform availability).
    fn is_available(&self) -> bool;
}

/// In-memory clipboard provider for testing.
///
/// Stores clipboard text in memory and is always available unless explicitly
/// configured otherwise. Thread-safe via internal `Mutex`.
#[derive(Debug, Clone)]
pub struct InMemoryClipboardProvider {
    content: Arc<Mutex<Option<String>>>,
    available: Arc<Mutex<bool>>,
}

impl InMemoryClipboardProvider {
    /// Create a new empty in-memory clipboard provider.
    pub fn new() -> Self {
        Self {
            content: Arc::new(Mutex::new(None)),
            available: Arc::new(Mutex::new(true)),
        }
    }

    /// Create a provider pre-loaded with the given text content.
    pub fn with_content(text: &str) -> Self {
        Self {
            content: Arc::new(Mutex::new(Some(text.to_string()))),
            available: Arc::new(Mutex::new(true)),
        }
    }

    /// Set whether the clipboard is available (for simulating platform failures).
    pub fn set_available(&self, available: bool) {
        *self.available.lock().unwrap() = available;
    }

    /// Directly set the content (simulates external application writing to clipboard).
    pub fn set_content_externally(&self, text: &str) {
        *self.content.lock().unwrap() = Some(text.to_string());
    }

    /// Clear the clipboard content directly.
    pub fn clear(&self) {
        *self.content.lock().unwrap() = None;
    }

    /// Get the current content (for test assertions).
    pub fn get_content(&self) -> Option<String> {
        self.content.lock().unwrap().clone()
    }
}

impl Default for InMemoryClipboardProvider {
    fn default() -> Self {
        Self::new()
    }
}

impl ClipboardProvider for InMemoryClipboardProvider {
    fn write_text(&self, text: &str) -> Result<(), ClipboardError> {
        if !self.is_available() {
            return Err(ClipboardError::Unavailable {
                reason: "clipboard is not available".to_string(),
            });
        }
        *self.content.lock().unwrap() = Some(text.to_string());
        Ok(())
    }

    fn read_text(&self) -> Result<String, ClipboardError> {
        if !self.is_available() {
            return Err(ClipboardError::Unavailable {
                reason: "clipboard is not available".to_string(),
            });
        }
        match self.content.lock().unwrap().as_ref() {
            Some(text) => Ok(text.clone()),
            None => Err(ClipboardError::Empty),
        }
    }

    fn has_text(&self) -> Result<bool, ClipboardError> {
        if !self.is_available() {
            return Err(ClipboardError::Unavailable {
                reason: "clipboard is not available".to_string(),
            });
        }
        Ok(self.content.lock().unwrap().is_some())
    }

    fn is_available(&self) -> bool {
        *self.available.lock().unwrap()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn in_memory_provider_write_and_read_roundtrip() {
        // Validates: Requirement 1.2, 1.3
        let provider = InMemoryClipboardProvider::new();
        provider.write_text("hello").unwrap();
        assert_eq!(provider.read_text().unwrap(), "hello");
    }

    #[test]
    fn in_memory_provider_empty_returns_error() {
        // Validates: Requirement 6.1
        let provider = InMemoryClipboardProvider::new();
        let result = provider.read_text();
        assert!(matches!(result, Err(ClipboardError::Empty)));
    }

    #[test]
    fn in_memory_provider_has_text_when_populated() {
        // Validates: Requirement 1.1
        let provider = InMemoryClipboardProvider::new();
        assert!(!provider.has_text().unwrap());
        provider.write_text("data").unwrap();
        assert!(provider.has_text().unwrap());
    }

    #[test]
    fn in_memory_provider_unavailable_returns_error() {
        // Validates: Requirement 1.6
        let provider = InMemoryClipboardProvider::new();
        provider.set_available(false);

        assert!(!provider.is_available());
        assert!(matches!(
            provider.write_text("test"),
            Err(ClipboardError::Unavailable { .. })
        ));
        assert!(matches!(
            provider.read_text(),
            Err(ClipboardError::Unavailable { .. })
        ));
        assert!(matches!(
            provider.has_text(),
            Err(ClipboardError::Unavailable { .. })
        ));
    }

    #[test]
    fn in_memory_provider_external_write_simulation() {
        // Validates: Requirement 1.5 (external modification)
        let provider = InMemoryClipboardProvider::new();
        provider.write_text("internal").unwrap();
        provider.set_content_externally("external");
        assert_eq!(provider.read_text().unwrap(), "external");
    }

    #[test]
    fn in_memory_provider_clear() {
        let provider = InMemoryClipboardProvider::with_content("data");
        assert!(provider.has_text().unwrap());
        provider.clear();
        assert!(!provider.has_text().unwrap());
    }

    #[test]
    fn in_memory_provider_is_send_and_sync() {
        fn assert_send_sync<T: Send + Sync>() {}
        assert_send_sync::<InMemoryClipboardProvider>();
    }
}
