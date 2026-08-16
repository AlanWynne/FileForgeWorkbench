//! `CommandHistory` — bounded, persistent log of recently executed commands.

use std::collections::VecDeque;
use std::path::Path;
use std::sync::Mutex;

use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};

use crate::error::CommandError;
use crate::id::CommandId;
use crate::params::{CommandParams, ParamValue};

/// Minimum allowed history depth.
const MIN_DEPTH: usize = 10;
/// Maximum allowed history depth.
const MAX_DEPTH: usize = 10_000;
/// Default history depth.
const DEFAULT_DEPTH: usize = 500;

/// A single entry in the command history log.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct HistoryEntry {
    /// The command that was executed.
    pub command_id: String,
    /// UTC timestamp with millisecond precision.
    pub timestamp: DateTime<Utc>,
    /// Serialized parameters.
    pub params: serde_json::Value,
}

/// A bounded, persistent log of recently executed commands.
///
/// Records every successfully executed command with its ID, timestamp,
/// and parameters. Supports configurable depth with FIFO eviction.
pub struct CommandHistory {
    entries: Mutex<VecDeque<HistoryEntry>>,
    max_depth: usize,
}

impl CommandHistory {
    /// Creates a new history with the specified maximum depth.
    ///
    /// The depth is clamped to [10, 10000].
    pub fn new(max_depth: usize) -> Self {
        let clamped = Self::clamp_depth(max_depth);
        Self {
            entries: Mutex::new(VecDeque::with_capacity(clamped)),
            max_depth: clamped,
        }
    }

    /// Creates a history from a configuration depth value.
    ///
    /// Clamps values outside [10, 10000] and logs a WARN.
    pub fn from_config(depth_value: Option<i64>) -> Self {
        let raw = depth_value.unwrap_or(DEFAULT_DEPTH as i64);
        let clamped = if raw < MIN_DEPTH as i64 {
            ff_logging::log_warn!(
                "[command] history: configured depth {} is below minimum, clamping to {}",
                raw,
                MIN_DEPTH
            );
            MIN_DEPTH
        } else if raw > MAX_DEPTH as i64 {
            ff_logging::log_warn!(
                "[command] history: configured depth {} exceeds maximum, clamping to {}",
                raw,
                MAX_DEPTH
            );
            MAX_DEPTH
        } else {
            raw as usize
        };

        Self {
            entries: Mutex::new(VecDeque::with_capacity(clamped)),
            max_depth: clamped,
        }
    }

    /// Records a successfully executed command.
    pub fn record(&self, command_id: &CommandId, params: &CommandParams) {
        let entry = HistoryEntry {
            command_id: command_id.to_string(),
            timestamp: Utc::now(),
            params: params_to_json(params),
        };

        let mut entries = self.entries.lock().expect("history lock poisoned");
        if entries.len() >= self.max_depth {
            entries.pop_front();
        }
        entries.push_back(entry);
    }

    /// Loads persisted history from disk.
    ///
    /// Returns an empty history on failure and logs a WARN.
    pub fn load(path: &Path, max_depth: usize) -> Self {
        let clamped = Self::clamp_depth(max_depth);

        match std::fs::read_to_string(path) {
            Ok(content) => {
                match serde_json::from_str::<Vec<HistoryEntry>>(&content) {
                    Ok(loaded) => {
                        let mut entries: VecDeque<HistoryEntry> = loaded.into();
                        // Trim to max depth
                        while entries.len() > clamped {
                            entries.pop_front();
                        }
                        Self {
                            entries: Mutex::new(entries),
                            max_depth: clamped,
                        }
                    }
                    Err(e) => {
                        ff_logging::log_warn!(
                            "[command] history: failed to parse history file — {}",
                            e
                        );
                        Self::new(clamped)
                    }
                }
            }
            Err(e) => {
                ff_logging::log_warn!("[command] history: failed to read history file — {}", e);
                Self::new(clamped)
            }
        }
    }

    /// Persists current history to disk.
    pub fn save(&self, path: &Path) -> Result<(), CommandError> {
        let entries = self.entries.lock().expect("history lock poisoned");
        let vec: Vec<&HistoryEntry> = entries.iter().collect();
        let json = serde_json::to_string_pretty(&vec).map_err(|e| CommandError::HistoryIo {
            operation: "serialize".to_string(),
            source: std::io::Error::other(e.to_string()),
        })?;

        // Ensure parent directory exists
        if let Some(parent) = path.parent() {
            std::fs::create_dir_all(parent).map_err(|e| CommandError::HistoryIo {
                operation: "create directory".to_string(),
                source: e,
            })?;
        }

        std::fs::write(path, json).map_err(|e| CommandError::HistoryIo {
            operation: "save".to_string(),
            source: e,
        })
    }

    /// Retrieves the last N entries (most recent first).
    pub fn last_n(&self, n: usize) -> Vec<HistoryEntry> {
        let entries = self.entries.lock().expect("history lock poisoned");
        entries.iter().rev().take(n).cloned().collect()
    }

    /// Retrieves entries matching a command ID prefix.
    pub fn by_prefix(&self, prefix: &str) -> Vec<HistoryEntry> {
        let entries = self.entries.lock().expect("history lock poisoned");
        entries
            .iter()
            .filter(|e| e.command_id.starts_with(prefix))
            .cloned()
            .collect()
    }

    /// Retrieves entries within a time range.
    pub fn by_time_range(&self, from: DateTime<Utc>, to: DateTime<Utc>) -> Vec<HistoryEntry> {
        let entries = self.entries.lock().expect("history lock poisoned");
        entries
            .iter()
            .filter(|e| e.timestamp >= from && e.timestamp <= to)
            .cloned()
            .collect()
    }

    /// Returns the current number of entries.
    pub fn len(&self) -> usize {
        let entries = self.entries.lock().expect("history lock poisoned");
        entries.len()
    }

    /// Returns true if the history is empty.
    pub fn is_empty(&self) -> bool {
        self.len() == 0
    }

    /// Returns the configured maximum depth.
    pub fn max_depth(&self) -> usize {
        self.max_depth
    }

    /// Clamps a depth value to [MIN_DEPTH, MAX_DEPTH].
    pub fn clamp_depth(depth: usize) -> usize {
        depth.clamp(MIN_DEPTH, MAX_DEPTH)
    }

    /// Clamps an i64 depth value to [MIN_DEPTH, MAX_DEPTH].
    pub fn clamp_depth_i64(depth: i64) -> usize {
        if depth < MIN_DEPTH as i64 {
            MIN_DEPTH
        } else if depth > MAX_DEPTH as i64 {
            MAX_DEPTH
        } else {
            depth as usize
        }
    }
}

/// Converts `CommandParams` to a JSON value for serialization.
fn params_to_json(params: &CommandParams) -> serde_json::Value {
    let mut map = serde_json::Map::new();
    for (key, value) in params.iter() {
        map.insert(key.clone(), param_value_to_json(value));
    }
    serde_json::Value::Object(map)
}

/// Converts a single `ParamValue` to JSON.
fn param_value_to_json(value: &ParamValue) -> serde_json::Value {
    match value {
        ParamValue::String(s) => serde_json::Value::String(s.clone()),
        ParamValue::Integer(i) => serde_json::Value::Number((*i).into()),
        ParamValue::Float(f) => serde_json::Number::from_f64(*f)
            .map(serde_json::Value::Number)
            .unwrap_or(serde_json::Value::Null),
        ParamValue::Boolean(b) => serde_json::Value::Bool(*b),
        ParamValue::Map(m) => {
            let mut obj = serde_json::Map::new();
            for (k, v) in m {
                obj.insert(k.clone(), param_value_to_json(v));
            }
            serde_json::Value::Object(obj)
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::thread;
    use tempfile::TempDir;

    fn make_id(s: &str) -> CommandId {
        CommandId::new(s).unwrap()
    }

    // Validates: Requirement 7.1
    #[test]
    fn record_stores_entry_with_timestamp() {
        let history = CommandHistory::new(100);
        let id = make_id("file.save");
        let params = CommandParams::new().with("path", "/tmp/test.txt");

        history.record(&id, &params);

        assert_eq!(history.len(), 1);
        let entries = history.last_n(1);
        assert_eq!(entries[0].command_id, "file.save");
    }

    // Validates: Requirement 7.4
    #[test]
    fn fifo_eviction_when_at_max_depth() {
        let history = CommandHistory::new(10);

        for i in 0..15 {
            let id = make_id("test.cmd");
            let params = CommandParams::new().with("index", i as i64);
            history.record(&id, &params);
        }

        assert_eq!(history.len(), 10);
        // Oldest entries (0-4) should be gone, newest (5-14) remain
        let entries = history.last_n(10);
        // Most recent is first in the returned list
        let last = entries.last().unwrap();
        let val: i64 = last.params.get("index").unwrap().as_i64().unwrap();
        assert_eq!(val, 5); // oldest remaining is index 5
    }

    // Validates: Requirement 7.3
    #[test]
    fn depth_clamping_below_minimum() {
        let history = CommandHistory::new(3);
        assert_eq!(history.max_depth(), MIN_DEPTH);
    }

    // Validates: Requirement 7.3
    #[test]
    fn depth_clamping_above_maximum() {
        let history = CommandHistory::new(99999);
        assert_eq!(history.max_depth(), MAX_DEPTH);
    }

    // Validates: Requirement 7.3
    #[test]
    fn depth_within_range_unchanged() {
        let history = CommandHistory::new(500);
        assert_eq!(history.max_depth(), 500);
    }

    // Validates: Requirement 7.3
    #[test]
    fn from_config_clamps_values() {
        let h1 = CommandHistory::from_config(Some(5));
        assert_eq!(h1.max_depth(), MIN_DEPTH);

        let h2 = CommandHistory::from_config(Some(20000));
        assert_eq!(h2.max_depth(), MAX_DEPTH);

        let h3 = CommandHistory::from_config(Some(100));
        assert_eq!(h3.max_depth(), 100);

        let h4 = CommandHistory::from_config(None);
        assert_eq!(h4.max_depth(), DEFAULT_DEPTH);
    }

    // Validates: Requirement 7.8
    #[test]
    fn last_n_returns_most_recent_entries() {
        let history = CommandHistory::new(100);
        for i in 0..5 {
            let id_str = format!("test.cmd{}", i);
            let id = make_id(&id_str);
            history.record(&id, &CommandParams::new());
        }

        let last2 = history.last_n(2);
        assert_eq!(last2.len(), 2);
        assert_eq!(last2[0].command_id, "test.cmd4");
        assert_eq!(last2[1].command_id, "test.cmd3");
    }

    // Validates: Requirement 7.8
    #[test]
    fn by_prefix_filters_entries() {
        let history = CommandHistory::new(100);
        history.record(&make_id("file.save"), &CommandParams::new());
        history.record(&make_id("file.open"), &CommandParams::new());
        history.record(&make_id("edit.copy"), &CommandParams::new());

        let file_entries = history.by_prefix("file.");
        assert_eq!(file_entries.len(), 2);
    }

    // Validates: Requirement 7.5, 7.6
    #[test]
    fn save_and_load_round_trip() {
        let dir = TempDir::new().unwrap();
        let path = dir.path().join("history.json");

        let history = CommandHistory::new(100);
        history.record(
            &make_id("file.save"),
            &CommandParams::new().with("path", "/tmp/a.txt"),
        );
        history.record(&make_id("edit.copy"), &CommandParams::new());

        history.save(&path).unwrap();

        let loaded = CommandHistory::load(&path, 100);
        assert_eq!(loaded.len(), 2);
        let entries = loaded.last_n(2);
        assert_eq!(entries[0].command_id, "edit.copy");
        assert_eq!(entries[1].command_id, "file.save");
    }

    // Validates: Requirement 7.6
    #[test]
    fn load_missing_file_returns_empty_history() {
        let path = Path::new("/nonexistent/path/history.json");
        let history = CommandHistory::load(path, 100);
        assert_eq!(history.len(), 0);
    }

    // Validates: Requirement 7.6
    #[test]
    fn load_corrupted_file_returns_empty_history() {
        let dir = TempDir::new().unwrap();
        let path = dir.path().join("history.json");
        std::fs::write(&path, "not valid json!!!").unwrap();

        let history = CommandHistory::load(&path, 100);
        assert_eq!(history.len(), 0);
    }

    // Validates: Requirement 7.7
    #[test]
    fn concurrent_access_is_safe() {
        let history = std::sync::Arc::new(CommandHistory::new(100));
        let mut handles = Vec::new();

        for i in 0..10 {
            let h = history.clone();
            handles.push(thread::spawn(move || {
                for j in 0..10 {
                    let id_str = format!("thread{}.cmd{}", i, j);
                    let id = make_id(&id_str);
                    h.record(&id, &CommandParams::new());
                }
            }));
        }

        for handle in handles {
            handle.join().unwrap();
        }

        assert_eq!(history.len(), 100);
    }
}
