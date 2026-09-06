//! Notification system -- channel-based, non-blocking.
//!
//! Provides `NotificationSender` (Clone + Send) for emitting notifications
//! from any subsystem, and `NotificationQueue` for storing and querying them.
//!
//! Validates: notification-system Requirement 3, 2

#![allow(dead_code)]

use std::collections::VecDeque;
use std::sync::mpsc;

// == Types ====================================================================

/// Severity level of a notification.
///
/// Validates: notification-system Requirement 3.4
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum NotificationLevel {
    Info,
    Success,
    Warning,
    Error,
}

impl NotificationLevel {
    /// Returns true for levels that increment the unread badge (Warning, Error).
    pub fn is_attention(&self) -> bool {
        matches!(self, Self::Warning | Self::Error)
    }

    /// Returns true for levels that auto-dismiss (Info, Success).
    pub fn auto_dismisses(&self) -> bool {
        matches!(self, Self::Info | Self::Success)
    }

    pub fn label(&self) -> &'static str {
        match self {
            Self::Info => "INFO",
            Self::Success => "OK",
            Self::Warning => "WARN",
            Self::Error => "ERR",
        }
    }
}

/// A single notification event.
///
/// Validates: notification-system Requirement 3.4
#[derive(Debug, Clone)]
pub struct Notification {
    pub level: NotificationLevel,
    pub title: String,
    pub detail: Option<String>,
    pub timestamp: String,
}

impl Notification {
    pub fn new(level: NotificationLevel, title: String, detail: Option<String>) -> Self {
        let timestamp = chrono::Local::now().format("%H:%M:%S").to_string();
        Self {
            level,
            title,
            detail,
            timestamp,
        }
    }
}

// == Queue ====================================================================

/// In-memory store of all notifications since startup.
///
/// Validates: notification-system Requirement 2.2, 2.7
pub struct NotificationQueue {
    entries: VecDeque<Notification>,
    unread: usize,
}

impl NotificationQueue {
    pub fn new() -> Self {
        Self {
            entries: VecDeque::new(),
            unread: 0,
        }
    }

    /// Prepend to front, cap at 1000, increment unread for Warning/Error.
    ///
    /// Validates: notification-system Requirement 2.7, 4.2
    pub fn push(&mut self, n: Notification) {
        if n.level.is_attention() {
            self.unread += 1;
        }
        self.entries.push_front(n);
        while self.entries.len() > 1000 {
            self.entries.pop_back();
        }
    }

    pub fn len(&self) -> usize {
        self.entries.len()
    }

    pub fn is_empty(&self) -> bool {
        self.entries.is_empty()
    }

    pub fn unread(&self) -> usize {
        self.unread
    }

    /// Mark all notifications as read.
    ///
    /// Validates: notification-system Requirement 2.4, 4.3
    pub fn mark_all_read(&mut self) {
        self.unread = 0;
    }

    /// Remove all entries.
    ///
    /// Validates: notification-system Requirement 2.6
    pub fn clear(&mut self) {
        self.entries.clear();
        self.unread = 0;
    }

    /// Return entries matching the given level (newest first).
    ///
    /// Validates: notification-system Requirement 2.4
    pub fn filter_by_level(&self, level: NotificationLevel) -> Vec<&Notification> {
        self.entries.iter().filter(|n| n.level == level).collect()
    }

    /// Immutable slice for rendering (newest first).
    pub fn entries(&self) -> &VecDeque<Notification> {
        &self.entries
    }
}

impl Default for NotificationQueue {
    fn default() -> Self {
        Self::new()
    }
}

// == Sender ===================================================================

/// Non-blocking sender handle -- Clone + Send.
///
/// Validates: notification-system Requirement 3.1, 3.2, 3.3
#[derive(Clone)]
pub struct NotificationSender {
    tx: mpsc::SyncSender<Notification>,
}

impl NotificationSender {
    pub fn new(tx: mpsc::SyncSender<Notification>) -> Self {
        Self { tx }
    }

    /// Emit an Info notification. Non-blocking -- drops silently if channel full.
    pub fn info(&self, title: String, detail: Option<String>) {
        self.send(NotificationLevel::Info, title, detail);
    }

    /// Emit a Success notification.
    pub fn success(&self, title: String, detail: Option<String>) {
        self.send(NotificationLevel::Success, title, detail);
    }

    /// Emit a Warning notification.
    pub fn warning(&self, title: String, detail: Option<String>) {
        self.send(NotificationLevel::Warning, title, detail);
    }

    /// Emit an Error notification.
    pub fn error(&self, title: String, detail: Option<String>) {
        self.send(NotificationLevel::Error, title, detail);
    }

    fn send(&self, level: NotificationLevel, title: String, detail: Option<String>) {
        // Non-blocking: drop if channel full (Req 3.3)
        let _ = self.tx.try_send(Notification::new(level, title, detail));
    }
}

// == Tests ====================================================================

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn queue_caps_at_1000_entries() {
        // Validates: notification-system Requirement 2.7
        let mut q = NotificationQueue::new();
        for i in 0..1100u32 {
            q.push(Notification::new(
                NotificationLevel::Info,
                format!("m{i}"),
                None,
            ));
        }
        assert!(q.len() <= 1000);
    }

    #[test]
    fn push_warning_increments_unread() {
        // Validates: notification-system Requirement 2.7
        let mut q = NotificationQueue::new();
        q.push(Notification::new(
            NotificationLevel::Warning,
            "w".to_string(),
            None,
        ));
        assert_eq!(q.unread(), 1);
        q.push(Notification::new(
            NotificationLevel::Info,
            "i".to_string(),
            None,
        ));
        assert_eq!(q.unread(), 1);
    }

    #[test]
    fn mark_all_read_clears_unread() {
        // Validates: notification-system Requirement 2.4
        let mut q = NotificationQueue::new();
        q.push(Notification::new(
            NotificationLevel::Error,
            "e".to_string(),
            None,
        ));
        q.mark_all_read();
        assert_eq!(q.unread(), 0);
    }

    #[test]
    fn clear_empties_queue_and_unread() {
        // Validates: notification-system Requirement 2.6
        let mut q = NotificationQueue::new();
        q.push(Notification::new(
            NotificationLevel::Error,
            "e".to_string(),
            None,
        ));
        q.clear();
        assert_eq!(q.len(), 0);
        assert_eq!(q.unread(), 0);
    }

    #[test]
    fn filter_by_level_returns_matching() {
        // Validates: notification-system Requirement 2.4
        let mut q = NotificationQueue::new();
        q.push(Notification::new(
            NotificationLevel::Info,
            "i".to_string(),
            None,
        ));
        q.push(Notification::new(
            NotificationLevel::Error,
            "e".to_string(),
            None,
        ));
        let errors = q.filter_by_level(NotificationLevel::Error);
        assert_eq!(errors.len(), 1);
        assert_eq!(errors[0].title, "e");
    }

    #[test]
    fn sender_is_clone() {
        // Validates: notification-system Requirement 3.2
        let (tx, _rx) = mpsc::sync_channel(4);
        let s = NotificationSender::new(tx);
        let _s2 = s.clone();
    }

    #[test]
    fn full_channel_drops_without_panic() {
        // Validates: notification-system Requirement 3.3
        let (tx, _rx) = mpsc::sync_channel(1);
        let s = NotificationSender::new(tx);
        s.info("a".to_string(), None);
        s.info("b".to_string(), None); // channel full -- must not panic
    }
}
