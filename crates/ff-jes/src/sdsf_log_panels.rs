//! SDSF log panels: System Log (LOG), User Log (ULOG), NEXT/PREV navigation,
//! and SNAPSHOT capture.
//!
//! Implements Requirement 18 AC 18.10-18.13:
//!   - LOG command: System Log panel (AC 18.10)
//!   - ULOG command: User Log panel (AC 18.11)
//!   - NEXT/PREV: scroll through log segments (AC 18.12)
//!   - SNAPSHOT: capture log content to file/dataset (AC 18.13)

// === LogEntry ================================================================

/// A single entry in a system or user log.
#[derive(Debug, Clone, PartialEq)]
pub struct SdsfLogEntry {
    /// Timestamp string (e.g. "2024-06-01 12:00:00").
    pub timestamp: String,
    /// Log message text.
    pub message: String,
    /// Source identifier (e.g. job name, system component).
    pub source: String,
}

impl SdsfLogEntry {
    pub fn new(timestamp: &str, source: &str, message: &str) -> Self {
        Self {
            timestamp: timestamp.to_string(),
            source: source.to_string(),
            message: message.to_string(),
        }
    }
}

// === LogSegment ==============================================================

/// A page/segment of log entries for NEXT/PREV navigation.
///
/// Addresses: Requirement 18 AC 18.12
#[derive(Debug, Clone)]
pub struct LogSegment {
    /// Entries in this segment (reverse-chronological for system log).
    pub entries: Vec<SdsfLogEntry>,
    /// Segment index (0-based).
    pub index: usize,
    /// Total number of segments available.
    pub total_segments: usize,
}

impl LogSegment {
    pub fn new(entries: Vec<SdsfLogEntry>, index: usize, total_segments: usize) -> Self {
        Self {
            entries,
            index,
            total_segments,
        }
    }

    /// Returns true if there is a next segment.
    pub fn has_next(&self) -> bool {
        self.index + 1 < self.total_segments
    }

    /// Returns true if there is a previous segment.
    pub fn has_prev(&self) -> bool {
        self.index > 0
    }
}

// === LogPanelKind ============================================================

/// Which log panel is open.
///
/// Addresses: Requirement 18 AC 18.10, 18.11
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum LogPanelKind {
    /// System log (LOG command).
    System,
    /// User log (ULOG command).
    User(String),
}

// === SnapshotDestination =====================================================

/// Where a SNAPSHOT should be written.
///
/// Addresses: Requirement 18 AC 18.13
#[derive(Debug, Clone, PartialEq)]
pub enum SnapshotDestination {
    /// Write to a local file path.
    File(String),
    /// Write to a named dataset.
    Dataset(String),
}

/// Result of a SNAPSHOT operation.
#[derive(Debug, Clone, PartialEq)]
pub enum SnapshotResult {
    /// Snapshot written successfully; number of lines captured.
    Written { destination: String, lines: usize },
    /// No content to snapshot.
    Empty,
}

// === LogPanelState ===========================================================

/// State for a LOG or ULOG panel.
///
/// Addresses: Requirement 18 AC 18.10-18.13
#[derive(Debug, Clone)]
pub struct LogPanelState {
    /// Which log this panel shows.
    pub kind: LogPanelKind,
    /// All log entries (reverse-chronological order for system log).
    entries: Vec<SdsfLogEntry>,
    /// Current segment index.
    current_segment: usize,
    /// Number of entries per segment (page size).
    segment_size: usize,
}

impl LogPanelState {
    /// Create a new log panel with the given entries.
    ///
    /// Entries are stored in reverse-chronological order (most recent first)
    /// for the system log, as required by AC 18.10.
    pub fn new_system(mut entries: Vec<SdsfLogEntry>, segment_size: usize) -> Self {
        // Reverse-chronological: most recent first
        entries.sort_by(|a, b| b.timestamp.cmp(&a.timestamp));
        Self {
            kind: LogPanelKind::System,
            entries,
            current_segment: 0,
            segment_size: segment_size.max(1),
        }
    }

    /// Create a new user log panel.
    ///
    /// Addresses: Requirement 18 AC 18.11
    pub fn new_user(user: &str, entries: Vec<SdsfLogEntry>, segment_size: usize) -> Self {
        Self {
            kind: LogPanelKind::User(user.to_string()),
            entries,
            current_segment: 0,
            segment_size: segment_size.max(1),
        }
    }

    /// Total number of segments.
    fn total_segments(&self) -> usize {
        if self.entries.is_empty() {
            1
        } else {
            self.entries.len().div_ceil(self.segment_size)
        }
    }

    /// Get the current segment.
    pub fn current_segment(&self) -> LogSegment {
        let total = self.total_segments();
        let start = self.current_segment * self.segment_size;
        let end = (start + self.segment_size).min(self.entries.len());
        let entries = if start < self.entries.len() {
            self.entries[start..end].to_vec()
        } else {
            Vec::new()
        };
        LogSegment::new(entries, self.current_segment, total)
    }

    /// Scroll to the next segment (NEXT command).
    ///
    /// Returns true if navigation occurred.
    ///
    /// Addresses: Requirement 18 AC 18.12
    pub fn advance(&mut self) -> bool {
        if self.current_segment + 1 < self.total_segments() {
            self.current_segment += 1;
            true
        } else {
            false
        }
    }

    /// Scroll to the previous segment (PREV command).
    ///
    /// Returns true if navigation occurred.
    ///
    /// Addresses: Requirement 18 AC 18.12
    pub fn prev(&mut self) -> bool {
        if self.current_segment > 0 {
            self.current_segment -= 1;
            true
        } else {
            false
        }
    }

    /// Capture the current log content to a destination (SNAPSHOT command).
    ///
    /// In this implementation the content is returned as a string for the
    /// caller to write; the shell layer handles actual file/dataset I/O.
    ///
    /// Addresses: Requirement 18 AC 18.13
    pub fn snapshot(&self, destination: SnapshotDestination) -> (SnapshotResult, String) {
        if self.entries.is_empty() {
            return (SnapshotResult::Empty, String::new());
        }
        let content: String = self
            .entries
            .iter()
            .map(|e| format!("{} {} {}", e.timestamp, e.source, e.message))
            .collect::<Vec<_>>()
            .join("\n");
        let dest_str = match &destination {
            SnapshotDestination::File(p) => p.clone(),
            SnapshotDestination::Dataset(d) => d.clone(),
        };
        let lines = self.entries.len();
        (
            SnapshotResult::Written {
                destination: dest_str,
                lines,
            },
            content,
        )
    }

    /// Returns all entries (for test inspection).
    pub fn all_entries(&self) -> &[SdsfLogEntry] {
        &self.entries
    }
}

// === Tests ===================================================================

#[cfg(test)]
mod tests {
    use super::*;

    fn sample_entries(n: usize) -> Vec<SdsfLogEntry> {
        (0..n)
            .map(|i| {
                SdsfLogEntry::new(
                    &format!("2024-06-01 12:00:{:02}", i),
                    "SYSTEM",
                    &format!("Message {i}"),
                )
            })
            .collect()
    }

    // Validates: Requirement 18.10
    #[test]
    fn system_log_panel_opens_in_reverse_chronological_order() {
        let entries = sample_entries(3);
        let panel = LogPanelState::new_system(entries, 10);
        let seg = panel.current_segment();
        // Most recent (index 2) should be first
        assert!(seg.entries[0].timestamp > seg.entries[1].timestamp);
    }

    // Validates: Requirement 18.10
    #[test]
    fn system_log_panel_kind_is_system() {
        let panel = LogPanelState::new_system(vec![], 10);
        assert_eq!(panel.kind, LogPanelKind::System);
    }

    // Validates: Requirement 18.11
    #[test]
    fn user_log_panel_kind_is_user() {
        let panel = LogPanelState::new_user("TESTUSER", vec![], 10);
        assert_eq!(panel.kind, LogPanelKind::User("TESTUSER".to_string()));
    }

    // Validates: Requirement 18.11
    #[test]
    fn user_log_panel_stores_user_name() {
        let panel = LogPanelState::new_user("ALICE", sample_entries(2), 10);
        assert_eq!(panel.kind, LogPanelKind::User("ALICE".to_string()));
    }

    // Validates: Requirement 18.12
    #[test]
    fn next_advances_to_next_segment() {
        let mut panel = LogPanelState::new_system(sample_entries(10), 3);
        assert_eq!(panel.current_segment, 0);
        assert!(panel.advance());
        assert_eq!(panel.current_segment, 1);
    }

    // Validates: Requirement 18.12
    #[test]
    fn next_returns_false_at_last_segment() {
        let mut panel = LogPanelState::new_system(sample_entries(3), 10);
        assert!(!panel.advance()); // only 1 segment
    }

    // Validates: Requirement 18.12
    #[test]
    fn prev_goes_back_to_previous_segment() {
        let mut panel = LogPanelState::new_system(sample_entries(10), 3);
        panel.advance();
        assert!(panel.prev());
        assert_eq!(panel.current_segment, 0);
    }

    // Validates: Requirement 18.12
    #[test]
    fn prev_returns_false_at_first_segment() {
        let mut panel = LogPanelState::new_system(sample_entries(5), 10);
        assert!(!panel.prev());
    }

    // Validates: Requirement 18.12
    #[test]
    fn segment_has_next_and_prev_flags() {
        let mut panel = LogPanelState::new_system(sample_entries(10), 3);
        let seg0 = panel.current_segment();
        assert!(!seg0.has_prev());
        assert!(seg0.has_next());
        panel.advance();
        let seg1 = panel.current_segment();
        assert!(seg1.has_prev());
    }

    // Validates: Requirement 18.12
    #[test]
    fn segment_contains_correct_entries() {
        let panel = LogPanelState::new_system(sample_entries(5), 2);
        let seg = panel.current_segment();
        assert_eq!(seg.entries.len(), 2);
    }

    // Validates: Requirement 18.13
    #[test]
    fn snapshot_to_file_returns_written_result() {
        let panel = LogPanelState::new_system(sample_entries(3), 10);
        let (result, content) =
            panel.snapshot(SnapshotDestination::File("/tmp/log.txt".to_string()));
        assert!(matches!(result, SnapshotResult::Written { lines: 3, .. }));
        assert!(!content.is_empty());
    }

    // Validates: Requirement 18.13
    #[test]
    fn snapshot_to_dataset_returns_written_result() {
        let panel = LogPanelState::new_system(sample_entries(2), 10);
        let (result, _) = panel.snapshot(SnapshotDestination::Dataset("MY.LOG.DS".to_string()));
        assert!(matches!(result, SnapshotResult::Written { lines: 2, .. }));
    }

    // Validates: Requirement 18.13
    #[test]
    fn snapshot_empty_log_returns_empty() {
        let panel = LogPanelState::new_system(vec![], 10);
        let (result, content) =
            panel.snapshot(SnapshotDestination::File("/tmp/log.txt".to_string()));
        assert_eq!(result, SnapshotResult::Empty);
        assert!(content.is_empty());
    }

    // Validates: Requirement 18.13
    #[test]
    fn snapshot_content_contains_all_entries() {
        let panel = LogPanelState::new_system(sample_entries(3), 10);
        let (_, content) = panel.snapshot(SnapshotDestination::File("/tmp/log.txt".to_string()));
        assert_eq!(content.lines().count(), 3);
    }
}
