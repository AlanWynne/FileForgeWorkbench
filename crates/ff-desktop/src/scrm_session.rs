//! Shell-side SCRM session state and CAPTURE command logic (CR-NR-098, Wave 2).
//!
//! Holds the ACTIVE screen Collection and the automatic-capture mode. The actual
//! collection/capture data model, persistence and replay live in the
//! GUI-independent `ff-scrm` crate; this module is the thin shell-side owner that
//! the CAPTURE commands drive and the auto-capture hook feeds.
//!
//! The methods here are deliberately pure with respect to egui (they take a
//! ready-built [`ScreenModel`]), so they are unit-testable without a shell.
//!
//! Validates: screen-snapshot-scrm Requirement 7, 8, 9.1, 18.4 (shell side).

use chrono::Utc;
use ff_screen_model::ScreenModel;
use ff_scrm::ScreenCollection;

/// Owns the active Collection and the automatic-capture flag for the shell.
#[derive(Debug, Default)]
pub struct ScrmSession {
    /// The active Collection, or `None` when no collection session is open.
    active: Option<ScreenCollection>,
    /// Whether automatic capture (on Context transition) is enabled.
    auto_capture: bool,
    /// Monotonic counter used to mint unique capture ids within a process.
    next_capture_seq: u64,
}

impl ScrmSession {
    /// Whether a Collection is currently active.
    pub fn is_active(&self) -> bool {
        self.active.is_some()
    }

    /// Whether automatic capture is enabled.
    pub fn auto_capture_enabled(&self) -> bool {
        self.auto_capture
    }

    /// Number of captures in the active Collection (0 when none active).
    #[cfg(test)]
    pub fn capture_count(&self) -> usize {
        self.active.as_ref().map(|c| c.len()).unwrap_or(0)
    }

    /// The active Collection, if any (read-only, for the viewer/export).
    pub fn active_collection(&self) -> Option<&ScreenCollection> {
        self.active.as_ref()
    }

    /// Mint a unique capture id (process-local).
    fn mint_id(&mut self) -> String {
        self.next_capture_seq += 1;
        format!("cap-{}", self.next_capture_seq)
    }

    /// Start a new Collection with the given name (CAPTURE START). Replaces any
    /// existing active Collection (the previous one should be saved first by the
    /// caller if desired). Returns a status string.
    ///
    /// Validates: Requirement 7.1, 7.2, 7.3.
    pub fn start(&mut self, name: &str, created_by: &str) -> String {
        let name = if name.trim().is_empty() {
            default_collection_name()
        } else {
            name.trim().to_string()
        };
        let id = format!("col-{}", Utc::now().timestamp_millis());
        self.active = Some(ScreenCollection::new(
            id,
            name.clone(),
            created_by,
            Utc::now(),
        ));
        format!("Screen collection '{name}' started.")
    }

    /// Stop the active collection SESSION: disable automatic capture and end the
    /// session without discarding the Collection (it remains available to the
    /// viewer/export until replaced or purged). Returns a status string.
    ///
    /// Validates: Requirement 7.6.
    pub fn stop(&mut self) -> String {
        self.auto_capture = false;
        if self.active.is_some() {
            "Screen collection stopped.".to_string()
        } else {
            "No active screen collection.".to_string()
        }
    }

    /// Enable/disable automatic capture (used by CAPTURE START auto-mode and by
    /// tests). Starting collection does not by itself enable auto capture.
    ///
    /// Validates: Requirement 9.1.
    pub fn set_auto_capture(&mut self, on: bool) {
        self.auto_capture = on;
    }

    /// Capture `model` into the active Collection, auto-starting a default
    /// Collection when none is active (Requirement 8.4). `screen_name` labels the
    /// capture. Returns the assigned sequence number.
    ///
    /// Validates: Requirement 8.1, 8.2, 8.3, 8.4.
    pub fn capture(&mut self, model: ScreenModel, screen_name: Option<String>) -> u32 {
        if self.active.is_none() {
            // Req 8.4: auto-start a timestamp-named default Collection.
            self.start("", "user");
        }
        let id = self.mint_id();
        let collection = self.active.as_mut().expect("active after auto-start");
        let seq = collection.append_capture(id, Utc::now(), model);
        if let Some(name) = screen_name {
            if let Some(cap) = collection.screens.last_mut() {
                cap.screen_name = Some(name);
            }
        }
        seq
    }

    /// Discard the active Collection (CAPTURE PURGE) and disable auto capture.
    ///
    /// Validates: Requirement 7 (PURGE command).
    pub fn purge(&mut self) -> String {
        let had = self.active.is_some();
        self.active = None;
        self.auto_capture = false;
        if had {
            "Screen collection purged.".to_string()
        } else {
            "No active screen collection to purge.".to_string()
        }
    }

    /// A one-line status summary (CAPTURE STATUS).
    ///
    /// Validates: Requirement 7.7.
    pub fn status(&self) -> String {
        match &self.active {
            Some(c) => format!(
                "Collection '{}': {} capture(s), auto-capture {}.",
                c.name,
                c.len(),
                if self.auto_capture { "ON" } else { "OFF" }
            ),
            None => "No active screen collection.".to_string(),
        }
    }

    /// Replace the active Collection with `collection` (CAPTURE LOAD / OPEN).
    ///
    /// Validates: Requirement 11.1 (LOAD/OPEN).
    pub fn load(&mut self, collection: ScreenCollection) {
        self.active = Some(collection);
    }

    /// A multi-line listing of the active Collection's captures (CAPTURE LIST).
    ///
    /// Validates: Requirement 7 (LIST command).
    pub fn list(&self) -> String {
        match &self.active {
            Some(c) if !c.is_empty() => {
                let mut out = format!("Collection '{}' ({} captures):", c.name, c.len());
                for cap in &c.screens {
                    let name = cap.screen_name.as_deref().unwrap_or(&cap.screen.title);
                    out.push_str(&format!("\n  {}. {}", cap.sequence_number, name));
                }
                out
            }
            Some(c) => format!("Collection '{}' is empty.", c.name),
            None => "No active screen collection.".to_string(),
        }
    }
}

/// A timestamp-based default Collection name (Requirement 8.4).
fn default_collection_name() -> String {
    format!("Collection-{}", Utc::now().format("%Y-%m-%d-%H-%M-%S"))
}

#[cfg(test)]
mod tests {
    use super::*;
    use ff_screen_model::ScreenModel;

    // Validates: Requirement 7.1, 7.7 -- start creates an active collection.
    #[test]
    fn start_creates_active_collection() {
        let mut s = ScrmSession::default();
        assert!(!s.is_active());
        let msg = s.start("Repro", "alan");
        assert!(msg.contains("Repro"));
        assert!(s.is_active());
        assert!(s.status().contains("Repro"));
    }

    // Validates: Requirement 8.2, 8.3 -- capture appends with sequential numbers.
    #[test]
    fn capture_appends_sequentially() {
        let mut s = ScrmSession::default();
        s.start("C", "alan");
        let a = s.capture(ScreenModel::new("One"), Some("One".to_string()));
        let b = s.capture(ScreenModel::new("Two"), None);
        assert_eq!(a, 1);
        assert_eq!(b, 2);
        assert_eq!(s.capture_count(), 2);
    }

    // Validates: Requirement 8.4 -- capture with no active collection auto-starts.
    #[test]
    fn capture_auto_starts_default_collection() {
        let mut s = ScrmSession::default();
        assert!(!s.is_active());
        let seq = s.capture(ScreenModel::new("Screen"), None);
        assert_eq!(seq, 1);
        assert!(
            s.is_active(),
            "capture must auto-start a default collection"
        );
    }

    // Validates: Requirement 9.1 -- auto-capture flag toggles.
    #[test]
    fn auto_capture_flag_toggles() {
        let mut s = ScrmSession::default();
        assert!(!s.auto_capture_enabled());
        s.set_auto_capture(true);
        assert!(s.auto_capture_enabled());
        // stop() disables auto capture.
        s.start("C", "alan");
        s.stop();
        assert!(!s.auto_capture_enabled());
    }

    // Validates: Requirement 7 -- purge discards the active collection.
    #[test]
    fn purge_discards_collection() {
        let mut s = ScrmSession::default();
        s.start("C", "alan");
        s.capture(ScreenModel::new("S"), None);
        let msg = s.purge();
        assert!(msg.contains("purged"));
        assert!(!s.is_active());
    }

    // Validates: Requirement 7 -- list enumerates captures.
    #[test]
    fn list_enumerates_captures() {
        let mut s = ScrmSession::default();
        s.start("C", "alan");
        s.capture(ScreenModel::new("Alpha"), Some("Alpha".to_string()));
        let listing = s.list();
        assert!(listing.contains("Alpha"));
        assert!(listing.contains("1."));
    }
}
