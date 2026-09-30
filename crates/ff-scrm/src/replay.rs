//! The replay state machine: a pure, egui-free model of stepping through a
//! collection's captures (first/prev/next/last, autoplay, speed, timing).
//!
//! Validates: screen-snapshot-scrm Requirement 10.1-10.7, 15.3.

use crate::model::ScreenCollection;

/// A pure replay cursor over a collection. The UI reads `position` and asks for
/// the current capture; this type never touches egui.
///
/// Validates: Requirement 10.2, 10.3.
#[derive(Debug, Clone, PartialEq)]
pub struct ReplaySession {
    /// 0-based index into the (optionally filtered) capture order.
    position: usize,
    /// The capture indices (into `collection.screens`) in replay order, after
    /// any dialog-state filter (Requirement 15.3).
    order: Vec<usize>,
    /// Whether autoplay is running.
    pub playing: bool,
    /// Playback speed multiplier (1.0 = real elapsed time). Clamped > 0.
    pub speed: f32,
}

impl ReplaySession {
    /// Start a replay over all captures in `collection`, at the first capture.
    pub fn new(collection: &ScreenCollection) -> Self {
        Self {
            position: 0,
            order: (0..collection.screens.len()).collect(),
            playing: false,
            speed: 1.0,
        }
    }

    /// Start a replay filtered to captures whose `dialog_state` equals `state`.
    ///
    /// Validates: Requirement 15.3.
    pub fn filtered_by_dialog_state(collection: &ScreenCollection, state: &str) -> Self {
        let order: Vec<usize> = collection
            .screens
            .iter()
            .enumerate()
            .filter(|(_, c)| c.dialog_state.as_deref() == Some(state))
            .map(|(i, _)| i)
            .collect();
        Self {
            position: 0,
            order,
            playing: false,
            speed: 1.0,
        }
    }

    /// Number of captures in the replay order.
    pub fn len(&self) -> usize {
        self.order.len()
    }

    /// Whether there is nothing to replay.
    pub fn is_empty(&self) -> bool {
        self.order.is_empty()
    }

    /// The index into `collection.screens` of the current capture, or `None`
    /// when the replay is empty.
    pub fn current_index(&self) -> Option<usize> {
        self.order.get(self.position).copied()
    }

    /// The current position (0-based) within the replay order.
    pub fn position(&self) -> usize {
        self.position
    }

    /// Go to the first capture. (Requirement 10.3)
    pub fn first(&mut self) {
        self.position = 0;
    }

    /// Go to the last capture. (Requirement 10.3)
    pub fn last(&mut self) {
        if !self.order.is_empty() {
            self.position = self.order.len() - 1;
        }
    }

    /// Advance to the next capture, clamped at the last. (Requirement 10.3)
    pub fn next(&mut self) {
        if self.position + 1 < self.order.len() {
            self.position += 1;
        }
    }

    /// Step back to the previous capture, clamped at the first.
    /// (Requirement 10.3)
    pub fn previous(&mut self) {
        self.position = self.position.saturating_sub(1);
    }

    /// Set the playback speed (clamped to a small positive minimum).
    /// (Requirement 10.5)
    pub fn set_speed(&mut self, speed: f32) {
        self.speed = speed.max(0.05);
    }
}

/// The elapsed time (in whole seconds) between two consecutive captures.
///
/// Validates: Requirement 10.6, 10.7.
pub fn elapsed_between_secs(collection: &ScreenCollection, from: usize, to: usize) -> Option<i64> {
    let a = collection.screens.get(from)?;
    let b = collection.screens.get(to)?;
    Some((b.timestamp - a.timestamp).num_seconds())
}

#[cfg(test)]
mod tests {
    use super::*;
    use chrono::{DateTime, TimeZone, Utc};
    use ff_screen_model::ScreenModel;

    fn at(secs: i64) -> DateTime<Utc> {
        Utc.timestamp_opt(1_700_000_000 + secs, 0).unwrap()
    }

    fn collection_of(n: usize) -> ScreenCollection {
        let mut c = ScreenCollection::new("c", "Test", "alan", at(0));
        for i in 0..n {
            c.append_capture(
                format!("cap{i}"),
                at(i as i64 * 5),
                ScreenModel::new(format!("S{i}")),
            );
        }
        c
    }

    // Validates: Requirement 10.3 -- first/next/last/previous with clamping.
    #[test]
    fn navigation_clamps_at_ends() {
        let c = collection_of(3);
        let mut r = ReplaySession::new(&c);
        assert_eq!(r.position(), 0);
        r.previous(); // clamp at first
        assert_eq!(r.position(), 0);
        r.next();
        r.next();
        assert_eq!(r.position(), 2);
        r.next(); // clamp at last
        assert_eq!(r.position(), 2);
        r.first();
        assert_eq!(r.position(), 0);
        r.last();
        assert_eq!(r.position(), 2);
        assert_eq!(r.current_index(), Some(2));
    }

    // Validates: Requirement 10.5 -- speed is clamped positive.
    #[test]
    fn speed_is_clamped_positive() {
        let c = collection_of(1);
        let mut r = ReplaySession::new(&c);
        r.set_speed(-3.0);
        assert!(r.speed >= 0.05);
    }

    // Validates: Requirement 10.7 -- elapsed between captures.
    #[test]
    fn elapsed_between_consecutive_captures() {
        let c = collection_of(2);
        assert_eq!(elapsed_between_secs(&c, 0, 1), Some(5));
    }

    // Validates: Requirement 15.3 -- replay filtered by dialog state.
    #[test]
    fn filter_by_dialog_state_selects_matching_only() {
        let mut c = collection_of(3);
        c.screens[1].dialog_state = Some("EDIT".to_string());
        let r = ReplaySession::filtered_by_dialog_state(&c, "EDIT");
        assert_eq!(r.len(), 1);
        assert_eq!(r.current_index(), Some(1));
    }
}
