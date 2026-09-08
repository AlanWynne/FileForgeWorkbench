//! Configuration audit logging.
//!
//! Provides `AuditEntry`, `AuditFilter`, and `AuditLog` -- a ring-buffer
//! that records every effective-value change in the configuration system.
//!
//! Addresses: Requirement 16

use std::time::SystemTime;

use crate::layer::ConfigLayer;
use crate::value::ConfigValue;

/// A single audit record for one configuration key change.
///
/// Addresses: Requirement 16.5
#[derive(Debug, Clone)]
pub struct AuditEntry {
    /// When the change occurred.
    pub timestamp: SystemTime,
    /// The dot-separated configuration key that changed.
    pub key: String,
    /// The effective value before the change (None if the key was absent).
    pub old_value: Option<ConfigValue>,
    /// The effective value after the change (None if the key was removed).
    pub new_value: Option<ConfigValue>,
    /// The layer that caused the change.
    pub layer: ConfigLayer,
    /// The actor that triggered the change (defaults to "user").
    pub actor: String,
}

/// Filter criteria for `AuditLog::query`.
///
/// All fields are optional; only non-None fields are applied.
/// Addresses: Requirement 16.3
#[derive(Debug, Default, Clone)]
pub struct AuditFilter {
    /// Only return entries whose key starts with this prefix.
    pub key_prefix: Option<String>,
    /// Only return entries from this layer.
    pub layer: Option<ConfigLayer>,
    /// Only return entries at or after this time.
    pub since: Option<SystemTime>,
    /// Only return entries at or before this time.
    pub until: Option<SystemTime>,
    /// Only return entries with this actor string.
    pub actor: Option<String>,
}

/// Ring-buffer audit log with a maximum of 10,000 entries.
///
/// When the buffer is full the oldest entry is discarded.
/// Addresses: Requirement 16.2
pub struct AuditLog {
    entries: std::collections::VecDeque<AuditEntry>,
    max_entries: usize,
}

impl AuditLog {
    /// Create a new `AuditLog` with the default 10,000-entry limit.
    pub fn new() -> Self {
        Self {
            entries: std::collections::VecDeque::new(),
            max_entries: 10_000,
        }
    }

    /// Append an entry, discarding the oldest if the buffer is full.
    ///
    /// Addresses: Requirement 16.1
    pub fn record(&mut self, entry: AuditEntry) {
        if self.entries.len() >= self.max_entries {
            self.entries.pop_front();
        }
        self.entries.push_back(entry);
    }

    /// Query entries matching the given filter.
    ///
    /// Returns a `Vec` of matching entries in chronological order.
    /// Addresses: Requirement 16.3
    pub fn query(&self, filter: &AuditFilter) -> Vec<AuditEntry> {
        self.entries
            .iter()
            .filter(|e| {
                if let Some(ref prefix) = filter.key_prefix {
                    if !e.key.starts_with(prefix.as_str()) {
                        return false;
                    }
                }
                if let Some(layer) = filter.layer {
                    if e.layer != layer {
                        return false;
                    }
                }
                if let Some(since) = filter.since {
                    if e.timestamp < since {
                        return false;
                    }
                }
                if let Some(until) = filter.until {
                    if e.timestamp > until {
                        return false;
                    }
                }
                if let Some(ref actor) = filter.actor {
                    if &e.actor != actor {
                        return false;
                    }
                }
                true
            })
            .cloned()
            .collect()
    }

    /// Remove all entries from the in-memory buffer.
    ///
    /// Addresses: Requirement 16.6
    pub fn clear(&mut self) {
        self.entries.clear();
    }

    /// Number of entries currently in the buffer.
    pub fn len(&self) -> usize {
        self.entries.len()
    }

    /// True when the buffer contains no entries.
    pub fn is_empty(&self) -> bool {
        self.entries.is_empty()
    }
}

impl Default for AuditLog {
    fn default() -> Self {
        Self::new()
    }
}

/// Append a single `AuditEntry` to the on-disk audit log file.
///
/// Each entry is written as a single JSON line (newline-delimited JSON).
/// Write failures emit a WARN log and are otherwise ignored so that audit
/// log I/O never blocks or fails a configuration change.
///
/// Addresses: Requirement 16.2, 16.4
pub fn persist_entry(path: &std::path::Path, entry: &AuditEntry) {
    use std::io::Write;

    let line = format_entry_as_json(entry);
    match std::fs::OpenOptions::new()
        .create(true)
        .append(true)
        .open(path)
    {
        Ok(mut file) => {
            if let Err(e) = writeln!(file, "{}", line) {
                ff_logging::log_warn!("[config] audit: failed to write audit log entry: {e}");
            }
        }
        Err(e) => {
            ff_logging::log_warn!("[config] audit: failed to open audit log file: {e}");
        }
    }
}

/// Truncate the on-disk audit log file.
///
/// Addresses: Requirement 16.6
pub fn truncate_log_file(path: &std::path::Path) {
    if path.exists() {
        if let Err(e) = std::fs::write(path, b"") {
            ff_logging::log_warn!("[config] audit: failed to truncate audit log file: {e}");
        }
    }
}

/// Format an `AuditEntry` as a minimal JSON line for on-disk persistence.
fn format_entry_as_json(entry: &AuditEntry) -> String {
    let ts = entry
        .timestamp
        .duration_since(std::time::UNIX_EPOCH)
        .map(|d| d.as_secs())
        .unwrap_or(0);
    let old = entry
        .old_value
        .as_ref()
        .map(config_value_to_json_str)
        .unwrap_or_else(|| "null".to_string());
    let new = entry
        .new_value
        .as_ref()
        .map(config_value_to_json_str)
        .unwrap_or_else(|| "null".to_string());
    format!(
        "{{\"ts\":{ts},\"key\":\"{key}\",\"old\":{old},\"new\":{new},\"layer\":\"{layer:?}\",\"actor\":\"{actor}\"}}",
        key = entry.key,
        layer = entry.layer,
        actor = entry.actor,
    )
}

/// Minimal JSON serialisation for a `ConfigValue` (no external dep required).
fn config_value_to_json_str(v: &ConfigValue) -> String {
    match v {
        ConfigValue::String(s) => format!("\"{}\"", s.replace('"', "\\\"")),
        ConfigValue::Integer(i) => i.to_string(),
        ConfigValue::Float(f) => f.to_string(),
        ConfigValue::Boolean(b) => b.to_string(),
        ConfigValue::Array(_) => "\"<array>\"".to_string(),
        ConfigValue::Table(_) => "\"<table>\"".to_string(),
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::time::{Duration, UNIX_EPOCH};

    fn make_entry(key: &str, layer: ConfigLayer, actor: &str) -> AuditEntry {
        AuditEntry {
            timestamp: SystemTime::now(),
            key: key.to_string(),
            old_value: None,
            new_value: Some(ConfigValue::Integer(1)),
            layer,
            actor: actor.to_string(),
        }
    }

    // Validates: Requirement 16.1
    #[test]
    fn audit_entry_recorded_on_record_call() {
        let mut log = AuditLog::new();
        assert!(log.is_empty());
        log.record(make_entry("editor.tab_size", ConfigLayer::User, "user"));
        assert_eq!(log.len(), 1);
    }

    // Validates: Requirement 16.2 -- ring buffer discards oldest when full
    #[test]
    fn ring_buffer_discards_oldest_when_full() {
        let mut log = AuditLog {
            entries: std::collections::VecDeque::new(),
            max_entries: 3,
        };
        for i in 0..4u64 {
            log.record(AuditEntry {
                timestamp: UNIX_EPOCH + Duration::from_secs(i),
                key: format!("key.{i}"),
                old_value: None,
                new_value: None,
                layer: ConfigLayer::User,
                actor: "user".to_string(),
            });
        }
        assert_eq!(log.len(), 3);
        // Oldest (key.0) should be gone
        let all = log.query(&AuditFilter::default());
        assert!(!all.iter().any(|e| e.key == "key.0"));
        assert!(all.iter().any(|e| e.key == "key.3"));
    }

    // Validates: Requirement 16.3 -- filter by key prefix
    #[test]
    fn audit_filter_by_key_prefix() {
        let mut log = AuditLog::new();
        log.record(make_entry("editor.tab_size", ConfigLayer::User, "user"));
        log.record(make_entry("logging.level", ConfigLayer::User, "user"));

        let filter = AuditFilter {
            key_prefix: Some("editor".to_string()),
            ..Default::default()
        };
        let results = log.query(&filter);
        assert_eq!(results.len(), 1);
        assert_eq!(results[0].key, "editor.tab_size");
    }

    // Validates: Requirement 16.3 -- filter by layer
    #[test]
    fn audit_filter_by_layer() {
        let mut log = AuditLog::new();
        log.record(make_entry("editor.tab_size", ConfigLayer::User, "user"));
        log.record(make_entry("editor.tab_size", ConfigLayer::System, "admin"));

        let filter = AuditFilter {
            layer: Some(ConfigLayer::System),
            ..Default::default()
        };
        let results = log.query(&filter);
        assert_eq!(results.len(), 1);
        assert_eq!(results[0].actor, "admin");
    }

    // Validates: Requirement 16.3 -- filter by time range
    #[test]
    fn audit_filter_by_time_range() {
        let mut log = AuditLog::new();
        let t0 = UNIX_EPOCH + Duration::from_secs(100);
        let t1 = UNIX_EPOCH + Duration::from_secs(200);
        let t2 = UNIX_EPOCH + Duration::from_secs(300);

        log.record(AuditEntry {
            timestamp: t0,
            key: "a".to_string(),
            old_value: None,
            new_value: None,
            layer: ConfigLayer::User,
            actor: "user".to_string(),
        });
        log.record(AuditEntry {
            timestamp: t1,
            key: "b".to_string(),
            old_value: None,
            new_value: None,
            layer: ConfigLayer::User,
            actor: "user".to_string(),
        });
        log.record(AuditEntry {
            timestamp: t2,
            key: "c".to_string(),
            old_value: None,
            new_value: None,
            layer: ConfigLayer::User,
            actor: "user".to_string(),
        });

        let filter = AuditFilter {
            since: Some(t1),
            until: Some(t1),
            ..Default::default()
        };
        let results = log.query(&filter);
        assert_eq!(results.len(), 1);
        assert_eq!(results[0].key, "b");
    }

    // Validates: Requirement 16.6 -- clear empties the buffer
    #[test]
    fn clear_audit_log_empties_buffer() {
        let mut log = AuditLog::new();
        log.record(make_entry("editor.tab_size", ConfigLayer::User, "user"));
        log.record(make_entry("logging.level", ConfigLayer::User, "user"));
        assert_eq!(log.len(), 2);
        log.clear();
        assert!(log.is_empty());
    }

    // Validates: Requirement 16.4 -- persist_entry does not panic on bad path
    #[test]
    fn audit_write_failure_does_not_block_config_change() {
        // Writing to a non-existent directory should not panic
        let bad_path = std::path::Path::new("/nonexistent/dir/audit.log");
        let entry = make_entry("editor.tab_size", ConfigLayer::User, "user");
        // Should not panic -- just logs a warning
        persist_entry(bad_path, &entry);
    }

    // Validates: Requirement 16.5 -- AuditEntry fields are accessible
    #[test]
    fn audit_entry_fields_are_public() {
        let ts = SystemTime::now();
        let entry = AuditEntry {
            timestamp: ts,
            key: "editor.tab_size".to_string(),
            old_value: Some(ConfigValue::Integer(4)),
            new_value: Some(ConfigValue::Integer(8)),
            layer: ConfigLayer::User,
            actor: "user".to_string(),
        };
        assert_eq!(entry.key, "editor.tab_size");
        assert_eq!(entry.layer, ConfigLayer::User);
        assert_eq!(entry.actor, "user");
    }
}
