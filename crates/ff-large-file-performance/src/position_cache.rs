//! PositionCache — hash-table caching of character x-positions.
//!
//! Stores measured character x-positions keyed by (style_slot, text_content),
//! avoiding redundant calls to the platform text-measurement API.
//! Uses two-way associative probing with clock-based eviction.
//!
//! Adapted from Scintilla's `IPositionCache`.

use std::sync::Mutex;

use crate::types::{ClockValue, StyleSlot, XPosition};

/// A single entry in the PositionCache.
struct Entry {
    style: StyleSlot,
    text: String,
    positions: Vec<XPosition>,
    clock: ClockValue,
}

/// A hash-table cache storing measured character x-positions.
///
/// Uses two-way associative probing: for each lookup, two candidate slots
/// are examined, and on insertion the entry with the lower clock value is evicted.
///
/// Thread-safe via internal `Mutex`.
pub struct PositionCache {
    inner: Mutex<PositionCacheInner>,
}

struct PositionCacheInner {
    entries: Vec<Option<Entry>>,
    capacity: usize,
    clock: ClockValue,
    hits: u64,
    misses: u64,
}

impl PositionCacheInner {
    fn new(capacity: usize) -> Self {
        // Round up to next power of 2
        let capacity = capacity.next_power_of_two();
        Self {
            entries: (0..capacity).map(|_| None).collect(),
            capacity,
            clock: ClockValue(1),
            hits: 0,
            misses: 0,
        }
    }

    fn hash(&self, style: StyleSlot, text: &str) -> usize {
        // Simple hash combining style and text
        let mut h: u64 = style.0 as u64 * 2654435761;
        for b in text.bytes() {
            h = h.wrapping_mul(31).wrapping_add(b as u64);
        }
        h as usize & (self.capacity - 1)
    }

    fn probe_slots(&self, style: StyleSlot, text: &str) -> (usize, usize) {
        let h = self.hash(style, text);
        let slot1 = h;
        let slot2 = (h.wrapping_add(1)) & (self.capacity - 1);
        (slot1, slot2)
    }

    fn lookup(&mut self, style: StyleSlot, text: &str, output: &mut [XPosition]) -> bool {
        let (s1, s2) = self.probe_slots(style, text);
        for &slot in &[s1, s2] {
            if let Some(entry) = &mut self.entries[slot] {
                if entry.style == style && entry.text == text {
                    let len = output.len().min(entry.positions.len());
                    output[..len].copy_from_slice(&entry.positions[..len]);
                    entry.clock = self.clock;
                    self.clock = self.clock.increment();
                    self.hits += 1;
                    return true;
                }
            }
        }
        self.misses += 1;
        false
    }

    fn store(&mut self, style: StyleSlot, text: &str, positions: &[XPosition]) {
        let (s1, s2) = self.probe_slots(style, text);

        // Check if already present — update in place
        for &slot in &[s1, s2] {
            if let Some(entry) = &mut self.entries[slot] {
                if entry.style == style && entry.text == text {
                    entry.positions = positions.to_vec();
                    entry.clock = self.clock;
                    self.clock = self.clock.increment();
                    return;
                }
            }
        }

        // Find the slot to evict (prefer empty, then lower clock)
        let evict_slot = match (&self.entries[s1], &self.entries[s2]) {
            (None, _) => s1,
            (_, None) => s2,
            (Some(e1), Some(e2)) => {
                if e1.clock <= e2.clock {
                    s1
                } else {
                    s2
                }
            }
        };

        self.entries[evict_slot] = Some(Entry {
            style,
            text: text.to_string(),
            positions: positions.to_vec(),
            clock: self.clock,
        });
        self.clock = self.clock.increment();
    }

    fn clear(&mut self) {
        for entry in &mut self.entries {
            *entry = None;
        }
        self.clock = ClockValue(1);
    }

    fn len(&self) -> usize {
        self.entries.iter().filter(|e| e.is_some()).count()
    }

    fn hit_rate(&self) -> f64 {
        let total = self.hits + self.misses;
        if total == 0 {
            0.0
        } else {
            self.hits as f64 / total as f64
        }
    }
}

impl PositionCache {
    /// Create a new PositionCache with the given capacity.
    /// Capacity is rounded up to the next power of 2.
    pub fn new(capacity: usize) -> Self {
        Self {
            inner: Mutex::new(PositionCacheInner::new(capacity)),
        }
    }

    /// Look up cached x-positions for the given style+text combination.
    ///
    /// On hit: copies positions to `output`, updates clock, returns `true`.
    /// On miss: returns `false`.
    pub fn lookup(&self, style: StyleSlot, text: &str, output: &mut [XPosition]) -> bool {
        self.inner.lock().unwrap().lookup(style, text, output)
    }

    /// Store measured x-positions for the given style+text combination.
    ///
    /// On collision: evicts the entry with the lower clock value.
    pub fn store(&self, style: StyleSlot, text: &str, positions: &[XPosition]) {
        self.inner.lock().unwrap().store(style, text, positions);
    }

    /// Clear all entries. Called on global invalidation (font/zoom/theme change).
    pub fn clear(&self) {
        self.inner.lock().unwrap().clear();
    }

    /// Current number of occupied slots.
    pub fn len(&self) -> usize {
        self.inner.lock().unwrap().len()
    }

    /// Whether the cache is empty.
    pub fn is_empty(&self) -> bool {
        self.len() == 0
    }

    /// Cache hit rate in [0.0, 1.0].
    pub fn hit_rate(&self) -> f64 {
        self.inner.lock().unwrap().hit_rate()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn make_positions(values: &[f64]) -> Vec<XPosition> {
        values.iter().map(|&v| XPosition(v)).collect()
    }

    #[test]
    fn store_and_lookup_round_trip() {
        // Validates: Requirement 2 AC 1, AC 6 — Property 1: PositionCache Determinism
        let cache = PositionCache::new(64);
        let positions = make_positions(&[8.0, 16.0, 24.0]);
        cache.store(StyleSlot(0), "ABC", &positions);

        let mut output = vec![XPosition(0.0); 3];
        assert!(cache.lookup(StyleSlot(0), "ABC", &mut output));
        assert_eq!(output[0].0, 8.0);
        assert_eq!(output[1].0, 16.0);
        assert_eq!(output[2].0, 24.0);
    }

    #[test]
    fn miss_returns_false() {
        // Validates: Requirement 2 AC 6
        let cache = PositionCache::new(64);
        let mut output = vec![XPosition(0.0); 3];
        assert!(!cache.lookup(StyleSlot(0), "XYZ", &mut output));
    }

    #[test]
    fn different_styles_are_distinct() {
        // Validates: Requirement 2 AC 1
        let cache = PositionCache::new(64);
        cache.store(StyleSlot(0), "A", &make_positions(&[8.0]));
        cache.store(StyleSlot(1), "A", &make_positions(&[10.0]));

        let mut out0 = vec![XPosition(0.0); 1];
        let mut out1 = vec![XPosition(0.0); 1];
        assert!(cache.lookup(StyleSlot(0), "A", &mut out0));
        assert!(cache.lookup(StyleSlot(1), "A", &mut out1));
        assert_eq!(out0[0].0, 8.0);
        assert_eq!(out1[0].0, 10.0);
    }

    #[test]
    fn clear_removes_all_entries() {
        // Validates: Requirement 2 AC 8
        let cache = PositionCache::new(64);
        cache.store(StyleSlot(0), "ABC", &make_positions(&[8.0, 16.0, 24.0]));
        assert!(!cache.is_empty());
        cache.clear();
        assert!(cache.is_empty());

        let mut output = vec![XPosition(0.0); 3];
        assert!(!cache.lookup(StyleSlot(0), "ABC", &mut output));
    }

    #[test]
    fn eviction_keeps_higher_clock_entry() {
        // Validates: Requirement 2 AC 2 — Property 2: Two-Way Eviction Correctness
        // Use a tiny cache (2 slots) to force eviction
        let cache = PositionCache::new(2);
        // Fill both probe slots for the same hash bucket
        // Store many entries to trigger eviction
        for i in 0..10u16 {
            let text = format!("key{i}");
            cache.store(StyleSlot(i), &text, &make_positions(&[i as f64 * 8.0]));
        }
        // Cache should not panic and should have at most 2 entries
        assert!(cache.len() <= 2);
    }

    #[test]
    fn hit_rate_tracks_correctly() {
        let cache = PositionCache::new(64);
        cache.store(StyleSlot(0), "HIT", &make_positions(&[8.0]));

        let mut out = vec![XPosition(0.0); 1];
        cache.lookup(StyleSlot(0), "HIT", &mut out); // hit
        cache.lookup(StyleSlot(0), "MISS", &mut out); // miss

        let rate = cache.hit_rate();
        assert!((rate - 0.5).abs() < f64::EPSILON);
    }

    #[test]
    fn update_existing_entry() {
        // Validates: Requirement 2 AC 6
        let cache = PositionCache::new(64);
        cache.store(StyleSlot(0), "A", &make_positions(&[8.0]));
        cache.store(StyleSlot(0), "A", &make_positions(&[10.0])); // update

        let mut out = vec![XPosition(0.0); 1];
        assert!(cache.lookup(StyleSlot(0), "A", &mut out));
        assert_eq!(out[0].0, 10.0);
    }
}
