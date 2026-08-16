//! The high-level Document struct wrapping TextBuffer with encoding navigation,
//! watcher notifications, lifecycle management, viewport, and save-point.

use crate::encoding_nav;
use crate::error::DocumentError;
use crate::line_end::LineEndMode;
use crate::save_point::SavePointTracker;
use crate::text_buffer::TextBuffer;
use crate::types::{
    BytePosition, CharacterExtracted, DeleteResult, Direction, InsertResult, LineNumber,
    LoadingProgress, SplitView,
};
use crate::viewport::Viewport;
use crate::watcher::{DocumentWatcher, WatcherHandle, WatcherRegistry};

/// The high-level text model. Wraps TextBuffer and adds encoding navigation,
/// watcher notifications, lifecycle management, viewport, and save-point.
pub struct Document {
    /// The text storage and line index.
    buffer: TextBuffer,
    /// Registered document watchers.
    watchers: WatcherRegistry,
    /// Current loading state.
    loading_progress: LoadingProgress,
    /// Viewport position manager.
    viewport: Viewport,
    /// Save-point tracker.
    save_point: SavePointTracker,
    /// The VFS URI this document was loaded from (None for untitled).
    source_uri: Option<String>,
}

impl Document {
    /// Create a new empty document.
    pub fn new() -> Self {
        Self {
            buffer: TextBuffer::new(),
            watchers: WatcherRegistry::new(),
            loading_progress: LoadingProgress::NotStarted,
            viewport: Viewport::new(),
            save_point: SavePointTracker::new(),
            source_uri: None,
        }
    }

    /// Create a document with pre-allocated buffer capacity.
    pub fn with_capacity(capacity: u64) -> Self {
        Self {
            buffer: TextBuffer::with_capacity(capacity),
            watchers: WatcherRegistry::new(),
            loading_progress: LoadingProgress::NotStarted,
            viewport: Viewport::new(),
            save_point: SavePointTracker::new(),
            source_uri: None,
        }
    }

    // --- Lifecycle ---

    /// Get the VFS URI this document was loaded from.
    pub fn source_uri(&self) -> Option<&str> {
        self.source_uri.as_deref()
    }

    /// Set the source URI.
    pub fn set_source_uri(&mut self, uri: Option<String>) {
        self.source_uri = uri;
    }

    /// Get the current loading progress.
    pub fn loading_progress(&self) -> &LoadingProgress {
        &self.loading_progress
    }

    /// Set loading progress (used by streaming reader).
    pub fn set_loading_progress(&mut self, progress: LoadingProgress) {
        self.loading_progress = progress;
    }

    /// Register a document watcher. Returns a handle for removal.
    pub fn add_watcher(
        &mut self,
        watcher: Box<dyn DocumentWatcher>,
    ) -> Result<WatcherHandle, DocumentError> {
        self.watchers
            .add(watcher)
            .map_err(|_| DocumentError::DuplicateWatcher)
    }

    /// Remove a previously registered watcher.
    pub fn remove_watcher(&mut self, handle: WatcherHandle) -> Result<(), DocumentError> {
        if self.watchers.remove(handle) {
            Ok(())
        } else {
            Err(DocumentError::WatcherNotFound {
                handle_id: handle.id(),
            })
        }
    }

    // --- Text Access ---

    /// Total byte length of document content.
    pub fn length(&self) -> u64 {
        self.buffer.length()
    }

    /// Total number of lines (minimum 1).
    pub fn line_count(&self) -> u64 {
        self.buffer.line_count()
    }

    /// Get byte at position.
    pub fn char_at(&self, position: BytePosition) -> Option<u8> {
        self.buffer.char_at(position)
    }

    /// Get a range of bytes.
    pub fn get_range(&self, position: BytePosition, length: u64) -> Option<Vec<u8>> {
        self.buffer.get_range(position, length)
    }

    /// Get contiguous view (compacts gap).
    pub fn contiguous_view(&mut self) -> &[u8] {
        self.buffer.contiguous_view()
    }

    /// Get split view (no compaction).
    pub fn split_view(&self) -> SplitView {
        self.buffer.split_view()
    }

    /// Get line start position.
    pub fn line_start(&self, line: LineNumber) -> BytePosition {
        self.buffer.line_start(line)
    }

    /// Get line end position.
    pub fn line_end(&self, line: LineNumber) -> BytePosition {
        self.buffer.line_end(line)
    }

    /// Find line from byte position.
    pub fn line_from_position(&self, position: BytePosition) -> LineNumber {
        self.buffer.line_from_position(position)
    }

    /// Check if text contains a line ending for current mode.
    pub fn contains_line_end(&self, text: &[u8]) -> bool {
        self.buffer.contains_line_end(text)
    }

    // --- Mutation ---

    /// Insert text at position. Notifies watchers and returns result.
    pub fn insert(
        &mut self,
        position: BytePosition,
        text: &[u8],
    ) -> Result<InsertResult, DocumentError> {
        if self.buffer.is_read_only() {
            self.watchers.notify_modify_attempt();
            return Err(DocumentError::ReadOnly {
                operation: "insert".to_string(),
            });
        }

        let result = self.buffer.insert(position, text)?;

        // Notify watchers
        self.watchers
            .notify_insert(position, text, result.lines_added);

        // Update save point
        let was_at_save = self.save_point.record_mutation();
        if was_at_save {
            self.watchers.notify_save_point(false);
        }

        Ok(result)
    }

    /// Delete bytes at position. Notifies watchers and returns result.
    pub fn delete(
        &mut self,
        position: BytePosition,
        length: u64,
    ) -> Result<DeleteResult, DocumentError> {
        if self.buffer.is_read_only() {
            self.watchers.notify_modify_attempt();
            return Err(DocumentError::ReadOnly {
                operation: "delete".to_string(),
            });
        }

        let result = self.buffer.delete(position, length)?;

        // Notify watchers
        self.watchers
            .notify_delete(position, length, result.lines_removed);

        // Update save point
        let was_at_save = self.save_point.record_mutation();
        if was_at_save {
            self.watchers.notify_save_point(false);
        }

        Ok(result)
    }

    /// Set read-only mode.
    pub fn set_read_only(&mut self, read_only: bool) {
        self.buffer.set_read_only(read_only);
    }

    /// Query read-only state.
    pub fn is_read_only(&self) -> bool {
        self.buffer.is_read_only()
    }

    /// Set line-end recognition mode.
    pub fn set_line_end_mode(&mut self, mode: LineEndMode) {
        self.buffer.set_line_end_mode(mode);
    }

    /// Get current line-end mode.
    pub fn line_end_mode(&self) -> LineEndMode {
        self.buffer.line_end_mode()
    }

    // --- Character Navigation ---

    /// Get the byte length of the character at position.
    pub fn char_length_at(&self, position: BytePosition) -> u8 {
        encoding_nav::char_length_at(self.buffer.gap_buffer(), position.0)
    }

    /// Move position outside a multi-byte sequence to nearest boundary.
    pub fn move_position_outside_char(
        &self,
        position: BytePosition,
        direction: Direction,
    ) -> BytePosition {
        BytePosition(encoding_nav::move_position_outside_char(
            self.buffer.gap_buffer(),
            position.0,
            direction,
        ))
    }

    /// Advance to next valid character position.
    pub fn next_position(
        &self,
        position: BytePosition,
        direction: Direction,
    ) -> Option<BytePosition> {
        encoding_nav::next_position(self.buffer.gap_buffer(), position.0, direction)
            .map(BytePosition)
    }

    /// Extract the character at position.
    pub fn character_at(&self, position: BytePosition) -> Option<CharacterExtracted> {
        encoding_nav::character_at(self.buffer.gap_buffer(), position.0)
    }

    /// Extract the character before position.
    pub fn character_before(&self, position: BytePosition) -> Option<CharacterExtracted> {
        encoding_nav::character_before(self.buffer.gap_buffer(), position.0)
    }

    /// Move by character offset from start position.
    pub fn relative_position(
        &self,
        start: BytePosition,
        character_offset: i64,
    ) -> Option<BytePosition> {
        encoding_nav::relative_position(self.buffer.gap_buffer(), start.0, character_offset)
            .map(BytePosition)
    }

    // --- Viewport Management ---

    /// Get the current top-line (1-based).
    pub fn top_line(&self) -> u64 {
        self.viewport.top_line()
    }

    /// Scroll down by `visible_count` lines (page down).
    pub fn scroll_page_down(&mut self, visible_count: u64) {
        self.viewport
            .scroll_page_down(visible_count, self.buffer.line_count());
    }

    /// Scroll up by `visible_count` lines (page up).
    pub fn scroll_page_up(&mut self, visible_count: u64) {
        self.viewport.scroll_page_up(visible_count);
    }

    /// Scroll down by `count` lines.
    /// Uses line_count as max (no viewport clamping for simple scroll).
    pub fn scroll_line_down(&mut self, count: u64) {
        // For simple line-down without viewport, just advance clamped to line_count
        let max = self.buffer.line_count();
        let new_top = self.viewport.top_line() + count;
        // Set with a visible_count of 1 to allow scrolling to the last line
        self.viewport.set_top_line(new_top, max, 1);
    }

    /// Scroll down by `count` lines with viewport size for clamping.
    pub fn scroll_line_down_clamped(&mut self, count: u64, visible_count: u64) {
        self.viewport
            .scroll_line_down(count, self.buffer.line_count(), visible_count);
    }

    /// Scroll up by `count` lines.
    pub fn scroll_line_up(&mut self, count: u64) {
        self.viewport.scroll_line_up(count);
    }

    /// Set top_line to a specific value, clamped.
    pub fn set_top_line(&mut self, line: u64) {
        // Use a large visible_count for clamping (max_top_line = line_count if visible_count=1)
        self.viewport
            .set_top_line(line, self.buffer.line_count(), 1);
    }

    /// Set top_line with viewport size for proper clamping.
    pub fn set_top_line_with_viewport(&mut self, line: u64, visible_count: u64) {
        self.viewport
            .set_top_line(line, self.buffer.line_count(), visible_count);
    }

    /// Maximum valid top_line for a given viewport height.
    pub fn max_top_line(&self, visible_count: u64) -> u64 {
        Viewport::compute_max_top_line(self.buffer.line_count(), visible_count)
    }

    // --- Save Point ---

    /// Record the current undo position as the save point.
    pub fn set_save_point(&mut self) {
        self.save_point.set_save_point();
        self.watchers.notify_save_point(true);
    }

    /// Check if at save point (no unsaved modifications).
    pub fn is_at_save_point(&self) -> bool {
        self.save_point.is_at_save_point()
    }

    // --- Internal access for streaming ---

    /// Mutable access to the text buffer (for streaming loader).
    #[allow(dead_code)]
    pub(crate) fn text_buffer_mut(&mut self) -> &mut TextBuffer {
        &mut self.buffer
    }

    /// Access the text buffer.
    #[allow(dead_code)]
    pub(crate) fn text_buffer(&self) -> &TextBuffer {
        &self.buffer
    }
}

impl Default for Document {
    fn default() -> Self {
        Self::new()
    }
}

impl Drop for Document {
    fn drop(&mut self) {
        self.watchers.notify_deleted();
    }
}

// Compile-time assertion that Document is Send when wrapped.
// Document itself isn't Sync (has interior state), but DocumentHandle is.
#[allow(dead_code)]
const _: fn() = || {
    fn assert_send<T: Send>() {}
    assert_send::<Document>();
};

impl std::fmt::Debug for Document {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("Document")
            .field("length", &self.buffer.length())
            .field("line_count", &self.buffer.line_count())
            .field("read_only", &self.buffer.is_read_only())
            .field("loading_progress", &self.loading_progress)
            .field("top_line", &self.viewport.top_line())
            .field("source_uri", &self.source_uri)
            .finish()
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::sync::atomic::{AtomicU64, Ordering};
    use std::sync::Arc;

    struct CountingWatcher {
        id: u64,
        inserts: Arc<AtomicU64>,
        deletes: Arc<AtomicU64>,
        save_points: Arc<AtomicU64>,
    }

    impl DocumentWatcher for CountingWatcher {
        fn notify_insert(&self, _pos: BytePosition, _text: &[u8], _lines: u64) {
            self.inserts.fetch_add(1, Ordering::SeqCst);
        }
        fn notify_delete(&self, _pos: BytePosition, _len: u64, _lines: u64) {
            self.deletes.fetch_add(1, Ordering::SeqCst);
        }
        fn notify_save_point(&self, _at: bool) {
            self.save_points.fetch_add(1, Ordering::SeqCst);
        }
        fn watcher_id(&self) -> u64 {
            self.id
        }
    }

    #[test]
    fn new_document_is_empty() {
        let doc = Document::new();
        assert_eq!(doc.length(), 0);
        assert_eq!(doc.line_count(), 1);
        assert!(!doc.is_read_only());
    }

    #[test]
    fn insert_and_read_back() {
        let mut doc = Document::new();
        doc.insert(BytePosition(0), b"hello world").unwrap();
        assert_eq!(doc.length(), 11);
        assert_eq!(
            doc.get_range(BytePosition(0), 11),
            Some(b"hello world".to_vec())
        );
    }

    #[test]
    fn delete_content() {
        let mut doc = Document::new();
        doc.insert(BytePosition(0), b"hello world").unwrap();
        doc.delete(BytePosition(5), 6).unwrap();
        assert_eq!(doc.get_range(BytePosition(0), 5), Some(b"hello".to_vec()));
    }

    #[test]
    fn read_only_blocks_mutations() {
        let mut doc = Document::new();
        doc.insert(BytePosition(0), b"test").unwrap();
        doc.set_read_only(true);

        assert!(doc.insert(BytePosition(0), b"x").is_err());
        assert!(doc.delete(BytePosition(0), 1).is_err());
    }

    #[test]
    fn watchers_receive_notifications() {
        let mut doc = Document::new();
        let inserts = Arc::new(AtomicU64::new(0));
        let deletes = Arc::new(AtomicU64::new(0));
        let save_points = Arc::new(AtomicU64::new(0));

        let watcher = CountingWatcher {
            id: 1,
            inserts: inserts.clone(),
            deletes: deletes.clone(),
            save_points: save_points.clone(),
        };
        doc.add_watcher(Box::new(watcher)).unwrap();

        doc.insert(BytePosition(0), b"hello").unwrap();
        assert_eq!(inserts.load(Ordering::SeqCst), 1);

        doc.delete(BytePosition(0), 3).unwrap();
        assert_eq!(deletes.load(Ordering::SeqCst), 1);
    }

    #[test]
    fn save_point_tracking() {
        let mut doc = Document::new();
        doc.set_save_point();
        assert!(doc.is_at_save_point());

        doc.insert(BytePosition(0), b"change").unwrap();
        assert!(!doc.is_at_save_point());

        doc.set_save_point();
        assert!(doc.is_at_save_point());
    }

    #[test]
    fn duplicate_watcher_rejected() {
        let mut doc = Document::new();
        let inserts = Arc::new(AtomicU64::new(0));
        let w1 = CountingWatcher {
            id: 1,
            inserts: inserts.clone(),
            deletes: Arc::new(AtomicU64::new(0)),
            save_points: Arc::new(AtomicU64::new(0)),
        };
        let w2 = CountingWatcher {
            id: 1,
            inserts: inserts.clone(),
            deletes: Arc::new(AtomicU64::new(0)),
            save_points: Arc::new(AtomicU64::new(0)),
        };
        doc.add_watcher(Box::new(w1)).unwrap();
        assert!(doc.add_watcher(Box::new(w2)).is_err());
    }

    #[test]
    fn viewport_operations() {
        let mut doc = Document::new();
        // Insert 100 lines
        let content: String = (0..100).map(|i| format!("line {}\n", i)).collect();
        doc.insert(BytePosition(0), content.as_bytes()).unwrap();

        assert_eq!(doc.top_line(), 1);
        doc.scroll_line_down(10);
        assert_eq!(doc.top_line(), 11);
        doc.scroll_line_up(5);
        assert_eq!(doc.top_line(), 6);
    }

    #[test]
    fn character_navigation() {
        let mut doc = Document::new();
        doc.insert(BytePosition(0), "aéb".as_bytes()).unwrap();
        // 'a' = 1 byte, 'é' = 2 bytes, 'b' = 1 byte
        assert_eq!(doc.char_length_at(BytePosition(0)), 1); // 'a'
        assert_eq!(doc.char_length_at(BytePosition(1)), 2); // 'é'
        assert_eq!(doc.char_length_at(BytePosition(3)), 1); // 'b'

        let ch = doc.character_at(BytePosition(1)).unwrap();
        assert_eq!(ch.character, 'é');
        assert_eq!(ch.byte_width, 2);
    }

    #[test]
    fn line_end_mode_change() {
        let mut doc = Document::new();
        let content: Vec<u8> = [b"hello".as_slice(), &[0xC2, 0x85], b"world"].concat();
        doc.insert(BytePosition(0), &content).unwrap();

        assert_eq!(doc.line_count(), 1); // Default mode
        doc.set_line_end_mode(LineEndMode::Unicode);
        assert_eq!(doc.line_count(), 2); // Unicode mode recognizes NEL
    }
}
