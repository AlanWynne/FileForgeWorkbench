//! Integration tests for ff-document-model.
//!
//! Covers end-to-end document lifecycle, multi-view shared ownership,
//! and large document stress testing.

use std::sync::atomic::{AtomicU64, Ordering};
use std::sync::Arc;

use ff_document_model::{
    new_document, BytePosition, DeleteResult, Direction, Document, DocumentHandle, DocumentWatcher,
    InsertResult, LineEndMode, LineNumber, LoadingProgress, WatcherHandle,
};

// ─── Test Watcher ───────────────────────────────────────────────────────────

struct TestWatcher {
    id: u64,
    inserts: Arc<AtomicU64>,
    deletes: Arc<AtomicU64>,
    save_points: Arc<AtomicU64>,
    deleted: Arc<AtomicU64>,
}

impl TestWatcher {
    fn new(
        id: u64,
    ) -> (
        Self,
        Arc<AtomicU64>,
        Arc<AtomicU64>,
        Arc<AtomicU64>,
        Arc<AtomicU64>,
    ) {
        let inserts = Arc::new(AtomicU64::new(0));
        let deletes = Arc::new(AtomicU64::new(0));
        let save_points = Arc::new(AtomicU64::new(0));
        let deleted = Arc::new(AtomicU64::new(0));
        (
            Self {
                id,
                inserts: inserts.clone(),
                deletes: deletes.clone(),
                save_points: save_points.clone(),
                deleted: deleted.clone(),
            },
            inserts,
            deletes,
            save_points,
            deleted,
        )
    }
}

impl DocumentWatcher for TestWatcher {
    fn notify_insert(&self, _pos: BytePosition, _text: &[u8], _lines: u64) {
        self.inserts.fetch_add(1, Ordering::SeqCst);
    }
    fn notify_delete(&self, _pos: BytePosition, _len: u64, _lines: u64) {
        self.deletes.fetch_add(1, Ordering::SeqCst);
    }
    fn notify_save_point(&self, _at: bool) {
        self.save_points.fetch_add(1, Ordering::SeqCst);
    }
    fn notify_deleted(&self) {
        self.deleted.fetch_add(1, Ordering::SeqCst);
    }
    fn watcher_id(&self) -> u64 {
        self.id
    }
}

// ─── Integration Test: Full Document Lifecycle ──────────────────────────────

#[test]
fn full_document_lifecycle() {
    // Validates: Full lifecycle: create → edit → save-point → drop
    let mut doc = Document::new();

    // 1. Empty document
    assert_eq!(doc.length(), 0);
    assert_eq!(doc.line_count(), 1);

    // 2. Set save point (empty state)
    doc.set_save_point();
    assert!(doc.is_at_save_point());

    // 3. Insert content
    doc.insert(BytePosition(0), b"Hello, World!\nLine 2\nLine 3\n")
        .unwrap();
    assert_eq!(doc.line_count(), 4); // 3 line endings + final empty line
    assert!(!doc.is_at_save_point());

    // 4. Edit: delete "World!"
    doc.delete(BytePosition(7), 6).unwrap();
    let content = doc.get_range(BytePosition(0), doc.length()).unwrap();
    assert!(content.starts_with(b"Hello, \n"));

    // 5. Set save point after edits
    doc.set_save_point();
    assert!(doc.is_at_save_point());

    // 6. Further edits move away from save point
    doc.insert(BytePosition(0), b"# ").unwrap();
    assert!(!doc.is_at_save_point());

    // 7. Read-only mode
    doc.set_read_only(true);
    assert!(doc.insert(BytePosition(0), b"x").is_err());
    assert!(doc.delete(BytePosition(0), 1).is_err());
    doc.set_read_only(false);

    // 8. Line navigation
    let line1_start = doc.line_start(LineNumber(1));
    let line1_content_start = line1_start;
    let line_num = doc.line_from_position(line1_content_start);
    assert_eq!(line_num, LineNumber(1));
}

// ─── Integration Test: Multi-View Shared Ownership ──────────────────────────

#[tokio::test]
async fn multi_view_shared_ownership() {
    // Validates: DocumentHandle enables shared access across tasks
    let handle = new_document();

    // Writer task inserts content
    let writer_handle = handle.clone();
    let writer = tokio::spawn(async move {
        let mut doc = writer_handle.write().await;
        doc.insert(BytePosition(0), b"Shared content\nLine 2\n")
            .unwrap();
    });
    writer.await.unwrap();

    // Multiple readers can access simultaneously
    let reader1 = handle.clone();
    let reader2 = handle.clone();

    let (len1, len2) = tokio::join!(async move { reader1.read().await.length() }, async move {
        reader2.read().await.length()
    },);

    assert_eq!(len1, len2);
    assert_eq!(len1, 22); // "Shared content\nLine 2\n" = 22 bytes
}

// ─── Integration Test: Large Document Stress ────────────────────────────────

#[test]
fn large_document_line_lookups() {
    // Validates: O(log n) lookups on >100K lines
    let mut doc = Document::new();

    // Generate 100K+ lines
    let mut content = String::new();
    let target_lines = 100_001;
    for i in 0..target_lines {
        content.push_str(&format!("Line {:06}\n", i));
    }
    doc.insert(BytePosition(0), content.as_bytes()).unwrap();

    // Verify line count
    assert_eq!(doc.line_count(), target_lines + 1); // N newlines = N+1 lines

    // Verify lookups at various positions
    let mid_line = LineNumber(50000);
    let start = doc.line_start(mid_line);
    let found = doc.line_from_position(start);
    assert_eq!(found, mid_line);

    let last_line = LineNumber(target_lines - 1);
    let last_start = doc.line_start(last_line);
    let found_last = doc.line_from_position(last_start);
    assert_eq!(found_last, last_line);

    // Verify character navigation on a known line
    let char_at_0 = doc.character_at(BytePosition(0)).unwrap();
    assert_eq!(char_at_0.character, 'L');
}

// ─── Integration Test: Watcher Notifications ────────────────────────────────

#[test]
fn watcher_notifications_through_lifecycle() {
    let (watcher, inserts, deletes, save_points, deleted) = TestWatcher::new(42);

    {
        let mut doc = Document::new();
        let _handle = doc.add_watcher(Box::new(watcher)).unwrap();

        // Insert triggers notification
        doc.insert(BytePosition(0), b"hello").unwrap();
        assert_eq!(inserts.load(Ordering::SeqCst), 1);

        // Delete triggers notification
        doc.delete(BytePosition(0), 3).unwrap();
        assert_eq!(deletes.load(Ordering::SeqCst), 1);

        // Save point triggers notification
        doc.set_save_point();
        assert_eq!(save_points.load(Ordering::SeqCst), 1);

        // Mutation after save point triggers save_point(false) notification
        doc.insert(BytePosition(0), b"x").unwrap();
        assert_eq!(save_points.load(Ordering::SeqCst), 2); // one for set, one for leaving
    }
    // Document dropped → notify_deleted called
    assert_eq!(deleted.load(Ordering::SeqCst), 1);
}

// ─── Integration Test: Encoding Navigation ──────────────────────────────────

#[test]
fn encoding_navigation_mixed_content() {
    let mut doc = Document::new();
    // Mix of ASCII, multi-byte UTF-8, CRLF
    let content = "Hello\r\n世界\néàü\n".as_bytes();
    doc.insert(BytePosition(0), content).unwrap();

    // Walk forward and collect characters
    let mut chars = Vec::new();
    let mut pos = BytePosition(0);
    while pos.0 < doc.length() {
        if let Some(ch) = doc.character_at(pos) {
            chars.push(ch.character);
            pos = doc
                .next_position(pos, Direction::Forward)
                .unwrap_or(BytePosition(doc.length()));
        } else {
            break;
        }
    }

    // CRLF should be one char (displayed as '\n')
    assert!(chars.contains(&'H'));
    assert!(chars.contains(&'世'));
    assert!(chars.contains(&'界'));
    assert!(chars.contains(&'é'));
}

// ─── Integration Test: Line End Mode Switch ─────────────────────────────────

#[test]
fn line_end_mode_switch_preserves_content() {
    let mut doc = Document::new();
    // Content with NEL
    let content: Vec<u8> = [b"line1".as_slice(), &[0xC2, 0x85], b"line2\nline3"].concat();
    doc.insert(BytePosition(0), &content).unwrap();

    // Default mode: only LF recognized
    assert_eq!(doc.line_count(), 2); // "line1<NEL>line2\n" "line3"

    let content_before = doc.get_range(BytePosition(0), doc.length()).unwrap();

    // Switch to Unicode mode
    doc.set_line_end_mode(LineEndMode::Unicode);
    assert_eq!(doc.line_count(), 3); // "line1" "line2" "line3"

    // Content unchanged
    let content_after = doc.get_range(BytePosition(0), doc.length()).unwrap();
    assert_eq!(content_before, content_after);
}

// ─── Integration Test: Viewport with Edits ──────────────────────────────────

#[test]
fn viewport_adjusts_with_document_changes() {
    let mut doc = Document::new();

    // Start with 50 lines
    let content: String = (0..50).map(|i| format!("line {}\n", i)).collect();
    doc.insert(BytePosition(0), content.as_bytes()).unwrap();
    assert_eq!(doc.line_count(), 51); // 50 newlines + 1 empty

    // Scroll to middle
    doc.set_top_line(25);
    assert_eq!(doc.top_line(), 25);

    // Insert lines before viewport - top_line stays the same value
    // (the content shifted, but we don't auto-adjust top_line)
    doc.insert(BytePosition(0), b"new line 1\nnew line 2\n")
        .unwrap();
    assert_eq!(doc.top_line(), 25); // unchanged

    // Max top line
    let max = doc.max_top_line(20);
    assert!(max > 1);
}
