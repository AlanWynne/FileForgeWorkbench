//! Logical record ID system for stable line identification.
//!
//! Each record (line) in a document is assigned a unique, stable ID at file-open
//! time. IDs are never reused — retired IDs are tracked to prevent reuse.
//! This allows bulk undo operations to reference records even after intervening
//! insertions or deletions.

use std::collections::{BTreeMap, HashMap, HashSet};

use serde::{Deserialize, Serialize};

use crate::error::UndoError;

/// A stable identifier for a document record (line).
///
/// Assigned at file-open, invariant under insertions/deletions of other records.
/// IDs start at 1 and increment monotonically. Retired IDs are never reused.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, PartialOrd, Ord, Serialize, Deserialize)]
pub struct LogicalRecordId(pub u64);

/// Maps logical record IDs to current byte offsets.
///
/// Updated on every document modification to keep offsets accurate.
/// Supports O(1) ID-to-offset lookup via HashMap.
#[derive(Debug, Clone)]
pub struct RecordIdMap {
    /// Next ID to assign.
    next_id: u64,
    /// Active mapping: record ID → current byte offset.
    id_to_offset: HashMap<LogicalRecordId, u64>,
    /// Reverse mapping: byte offset → record ID (for line starts).
    offset_to_id: BTreeMap<u64, LogicalRecordId>,
    /// Retired IDs (never reused).
    retired: HashSet<LogicalRecordId>,
}

impl RecordIdMap {
    /// Creates a new map, assigning sequential IDs to `initial_line_count` records.
    ///
    /// Records are assumed to start at offset 0 with no gaps (placeholder offsets).
    /// Real offsets should be updated via `update_offsets` or `set_offset`.
    pub fn new(initial_line_count: u64) -> Self {
        let mut id_to_offset = HashMap::with_capacity(initial_line_count as usize);
        let mut offset_to_id = BTreeMap::new();

        for i in 0..initial_line_count {
            let id = LogicalRecordId(i + 1);
            // Placeholder offsets — caller should set real offsets
            id_to_offset.insert(id, i);
            offset_to_id.insert(i, id);
        }

        Self {
            next_id: initial_line_count + 1,
            id_to_offset,
            offset_to_id,
            retired: HashSet::new(),
        }
    }

    /// Assigns a new unique ID for an inserted record.
    ///
    /// IDs are never reused, even after retirement.
    pub fn assign_id(&mut self) -> LogicalRecordId {
        let id = LogicalRecordId(self.next_id);
        self.next_id += 1;
        id
    }

    /// Assigns and registers a new ID at the given offset.
    pub fn assign_id_at(&mut self, offset: u64) -> LogicalRecordId {
        let id = self.assign_id();
        self.id_to_offset.insert(id, offset);
        self.offset_to_id.insert(offset, id);
        id
    }

    /// Retires an ID (record deleted). The ID is never reused.
    pub fn retire_id(&mut self, id: LogicalRecordId) {
        if let Some(offset) = self.id_to_offset.remove(&id) {
            self.offset_to_id.remove(&offset);
        }
        self.retired.insert(id);
    }

    /// Returns the current byte offset for a record ID, or None if retired/unknown.
    pub fn offset_for(&self, id: LogicalRecordId) -> Option<u64> {
        self.id_to_offset.get(&id).copied()
    }

    /// Updates offsets after a document modification.
    ///
    /// All records at or after `position` are shifted by `delta` bytes.
    pub fn update_offsets(&mut self, position: u64, delta: i64) {
        // Collect entries that need updating
        let affected: Vec<(u64, LogicalRecordId)> = self
            .offset_to_id
            .range(position..)
            .map(|(&off, &id)| (off, id))
            .collect();

        for (old_offset, id) in affected {
            self.offset_to_id.remove(&old_offset);
            let new_offset = if delta >= 0 {
                old_offset + delta as u64
            } else {
                old_offset.saturating_sub((-delta) as u64)
            };
            self.offset_to_id.insert(new_offset, id);
            self.id_to_offset.insert(id, new_offset);
        }
    }

    /// Sets the offset for an existing ID.
    pub fn set_offset(&mut self, id: LogicalRecordId, offset: u64) {
        // Remove old offset mapping
        if let Some(&old_offset) = self.id_to_offset.get(&id) {
            self.offset_to_id.remove(&old_offset);
        }
        self.id_to_offset.insert(id, offset);
        self.offset_to_id.insert(offset, id);
    }

    /// Returns the number of active (non-retired) records.
    pub fn active_count(&self) -> usize {
        self.id_to_offset.len()
    }

    /// Returns the total number of IDs ever assigned (active + retired).
    pub fn total_assigned(&self) -> u64 {
        self.next_id - 1
    }

    /// Returns true if the given ID has been retired.
    pub fn is_retired(&self, id: LogicalRecordId) -> bool {
        self.retired.contains(&id)
    }

    /// Serializes the record ID map for recovery file inclusion.
    pub fn serialize(&self) -> Result<Vec<u8>, UndoError> {
        let data = SerializedRecordIdMap {
            next_id: self.next_id,
            entries: self
                .id_to_offset
                .iter()
                .map(|(&id, &offset)| (id.0, offset))
                .collect(),
            retired: self.retired.iter().map(|id| id.0).collect(),
        };
        serde_json::to_vec(&data).map_err(|e| UndoError::Serialization(e.to_string()))
    }

    /// Deserializes from recovery data.
    pub fn deserialize(data: &[u8]) -> Result<Self, UndoError> {
        let serialized: SerializedRecordIdMap =
            serde_json::from_slice(data).map_err(|e| UndoError::Serialization(e.to_string()))?;

        let mut id_to_offset = HashMap::with_capacity(serialized.entries.len());
        let mut offset_to_id = BTreeMap::new();

        for (id_val, offset) in serialized.entries {
            let id = LogicalRecordId(id_val);
            id_to_offset.insert(id, offset);
            offset_to_id.insert(offset, id);
        }

        let retired: HashSet<LogicalRecordId> = serialized
            .retired
            .into_iter()
            .map(LogicalRecordId)
            .collect();

        Ok(Self {
            next_id: serialized.next_id,
            id_to_offset,
            offset_to_id,
            retired,
        })
    }

    /// Resets all state (for delete_history).
    pub fn reset(&mut self) {
        self.next_id = 1;
        self.id_to_offset.clear();
        self.offset_to_id.clear();
        self.retired.clear();
    }
}

/// Serialization format for RecordIdMap.
#[derive(Serialize, Deserialize)]
struct SerializedRecordIdMap {
    next_id: u64,
    entries: Vec<(u64, u64)>,
    retired: Vec<u64>,
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn new_assigns_sequential_ids_from_one() {
        let map = RecordIdMap::new(5);
        assert_eq!(map.active_count(), 5);
        assert_eq!(map.offset_for(LogicalRecordId(1)), Some(0));
        assert_eq!(map.offset_for(LogicalRecordId(5)), Some(4));
    }

    #[test]
    fn assign_id_returns_unique_ids() {
        let mut map = RecordIdMap::new(3);
        let id1 = map.assign_id();
        let id2 = map.assign_id();
        assert_ne!(id1, id2);
        assert_eq!(id1, LogicalRecordId(4));
        assert_eq!(id2, LogicalRecordId(5));
    }

    #[test]
    fn retire_id_removes_from_active() {
        let mut map = RecordIdMap::new(3);
        map.retire_id(LogicalRecordId(2));
        assert_eq!(map.active_count(), 2);
        assert_eq!(map.offset_for(LogicalRecordId(2)), None);
        assert!(map.is_retired(LogicalRecordId(2)));
    }

    #[test]
    fn retired_ids_are_never_reused() {
        let mut map = RecordIdMap::new(3);
        map.retire_id(LogicalRecordId(2));
        let new_id = map.assign_id();
        assert_eq!(new_id, LogicalRecordId(4)); // Skips retired IDs
    }

    #[test]
    fn update_offsets_shifts_affected_records() {
        let mut map = RecordIdMap::new(3); // IDs 1,2,3 at offsets 0,1,2
                                           // Insert 10 bytes at position 1 — records at offset >=1 shift by +10
        map.update_offsets(1, 10);
        assert_eq!(map.offset_for(LogicalRecordId(1)), Some(0)); // Before position, unchanged
        assert_eq!(map.offset_for(LogicalRecordId(2)), Some(11)); // 1 + 10
        assert_eq!(map.offset_for(LogicalRecordId(3)), Some(12)); // 2 + 10
    }

    #[test]
    fn update_offsets_with_negative_delta() {
        let mut map = RecordIdMap::new(0);
        let id1 = map.assign_id_at(0);
        let id2 = map.assign_id_at(10);
        let id3 = map.assign_id_at(20);

        // Delete 5 bytes starting at position 5
        map.update_offsets(5, -5);
        assert_eq!(map.offset_for(id1), Some(0));
        assert_eq!(map.offset_for(id2), Some(5)); // 10 - 5
        assert_eq!(map.offset_for(id3), Some(15)); // 20 - 5
    }

    #[test]
    fn serialize_deserialize_roundtrip() {
        let mut map = RecordIdMap::new(3);
        map.retire_id(LogicalRecordId(2));
        let _new_id = map.assign_id_at(100);

        let data = map.serialize().unwrap();
        let restored = RecordIdMap::deserialize(&data).unwrap();

        assert_eq!(restored.active_count(), map.active_count());
        assert_eq!(
            restored.offset_for(LogicalRecordId(1)),
            map.offset_for(LogicalRecordId(1))
        );
        assert!(restored.is_retired(LogicalRecordId(2)));
        assert_eq!(
            restored.offset_for(LogicalRecordId(4)),
            map.offset_for(LogicalRecordId(4))
        );
    }

    #[test]
    fn reset_clears_all_state() {
        let mut map = RecordIdMap::new(5);
        map.assign_id();
        map.retire_id(LogicalRecordId(3));
        map.reset();
        assert_eq!(map.active_count(), 0);
        assert_eq!(map.total_assigned(), 0);
    }
}
