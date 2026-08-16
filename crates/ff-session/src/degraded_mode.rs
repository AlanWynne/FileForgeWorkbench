//! Graceful degradation tracking — records which subsystems failed during
//! startup and provides the degraded-mode indicator state.
//!
//! Addresses: Requirement 11 (Graceful Degradation)

use crate::startup::StartupPhase;

/// Identifies a subsystem that can enter degraded mode.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum Subsystem {
    /// Session file persistence (load/save).
    SessionPersistence,
    /// Plugin initialisation.
    PluginInit,
    /// Layout state restoration.
    LayoutRestore,
    /// Recent files list loading.
    RecentFiles,
    /// Crash recovery file scanning.
    RecoveryFileScan,
    /// User Data Directory availability.
    UserDataDir,
    /// Configuration loading.
    Configuration,
}

impl Subsystem {
    /// Human-readable name for the subsystem.
    pub fn display_name(self) -> &'static str {
        match self {
            Self::SessionPersistence => "Session persistence",
            Self::PluginInit => "Plugin initialisation",
            Self::LayoutRestore => "Layout restoration",
            Self::RecentFiles => "Recent files",
            Self::RecoveryFileScan => "Recovery file scan",
            Self::UserDataDir => "User data directory",
            Self::Configuration => "Configuration",
        }
    }
}

/// A single degraded subsystem record.
///
/// Addresses: Requirement 11 AC 11.2
#[derive(Debug, Clone, PartialEq)]
pub struct DegradedSubsystem {
    /// Which subsystem failed.
    pub subsystem: Subsystem,
    /// The startup phase where failure occurred.
    pub phase: StartupPhase,
    /// Description of the failure.
    pub reason: String,
    /// Whether the issue has been resolved at runtime.
    pub resolved: bool,
}

/// Tracks which subsystems are in degraded state.
///
/// Provides the indicator text for the status bar and supports
/// per-subsystem resolution when issues are fixed at runtime.
///
/// Addresses: Requirement 11
#[derive(Debug, Clone, Default)]
pub struct DegradedModeTracker {
    /// Active degraded subsystems.
    entries: Vec<DegradedSubsystem>,
}

impl DegradedModeTracker {
    /// Create a new tracker with no degraded subsystems.
    pub fn new() -> Self {
        Self::default()
    }

    /// Record a subsystem failure.
    ///
    /// Addresses: Requirement 11 AC 11.1
    pub fn record_failure(&mut self, subsystem: Subsystem, phase: StartupPhase, reason: String) {
        // Don't add duplicate entries for the same subsystem
        if self
            .entries
            .iter()
            .any(|e| e.subsystem == subsystem && !e.resolved)
        {
            return;
        }

        self.entries.push(DegradedSubsystem {
            subsystem,
            phase,
            reason,
            resolved: false,
        });
    }

    /// Mark a subsystem's degraded state as resolved.
    ///
    /// Addresses: Requirement 11 AC 11.6
    pub fn resolve(&mut self, subsystem: Subsystem) {
        for entry in &mut self.entries {
            if entry.subsystem == subsystem {
                entry.resolved = true;
            }
        }
    }

    /// Whether any subsystem is currently in degraded mode (unresolved).
    pub fn is_degraded(&self) -> bool {
        self.entries.iter().any(|e| !e.resolved)
    }

    /// Return the number of currently degraded (unresolved) subsystems.
    pub fn degraded_count(&self) -> usize {
        self.entries.iter().filter(|e| !e.resolved).count()
    }

    /// Return all active (unresolved) degraded subsystems.
    pub fn active_entries(&self) -> Vec<&DegradedSubsystem> {
        self.entries.iter().filter(|e| !e.resolved).collect()
    }

    /// Return the summary text for the status bar indicator.
    ///
    /// Returns `None` if no subsystems are degraded.
    ///
    /// Addresses: Requirement 11 AC 11.2
    pub fn indicator_text(&self) -> Option<String> {
        let count = self.degraded_count();
        if count == 0 {
            return None;
        }

        Some(format!(
            "\u{26A0} {} component{} not loaded \u{2014} click for details",
            count,
            if count == 1 { "" } else { "s" }
        ))
    }

    /// Return a detailed summary of all degraded subsystems.
    ///
    /// Addresses: Requirement 11 AC 11.3
    pub fn detail_summary(&self) -> Vec<String> {
        self.entries
            .iter()
            .filter(|e| !e.resolved)
            .map(|e| {
                format!(
                    "{} (Phase {}): {}",
                    e.subsystem.display_name(),
                    e.phase.number(),
                    e.reason
                )
            })
            .collect()
    }

    /// Clear all entries (reset tracker).
    pub fn clear(&mut self) {
        self.entries.clear();
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn new_tracker_is_not_degraded() {
        // Validates: Requirement 11 AC 11.1
        let tracker = DegradedModeTracker::new();
        assert!(!tracker.is_degraded());
        assert_eq!(tracker.degraded_count(), 0);
        assert!(tracker.indicator_text().is_none());
    }

    #[test]
    fn record_failure_puts_tracker_in_degraded_state() {
        // Validates: Requirement 11 AC 11.1
        let mut tracker = DegradedModeTracker::new();
        tracker.record_failure(
            Subsystem::PluginInit,
            StartupPhase::LoadPlugins,
            "plugin X failed to load".to_string(),
        );

        assert!(tracker.is_degraded());
        assert_eq!(tracker.degraded_count(), 1);
    }

    #[test]
    fn multiple_failures_tracked_independently() {
        // Validates: Requirement 11 AC 11.1
        let mut tracker = DegradedModeTracker::new();
        tracker.record_failure(
            Subsystem::PluginInit,
            StartupPhase::LoadPlugins,
            "plugin failed".to_string(),
        );
        tracker.record_failure(
            Subsystem::SessionPersistence,
            StartupPhase::LoadSessionState,
            "session file corrupt".to_string(),
        );
        tracker.record_failure(
            Subsystem::LayoutRestore,
            StartupPhase::RestoreLayout,
            "layout data invalid".to_string(),
        );

        assert_eq!(tracker.degraded_count(), 3);
    }

    #[test]
    fn resolve_clears_specific_subsystem() {
        // Validates: Requirement 11 AC 11.6
        let mut tracker = DegradedModeTracker::new();
        tracker.record_failure(
            Subsystem::PluginInit,
            StartupPhase::LoadPlugins,
            "failed".to_string(),
        );
        tracker.record_failure(
            Subsystem::SessionPersistence,
            StartupPhase::LoadSessionState,
            "corrupt".to_string(),
        );

        tracker.resolve(Subsystem::PluginInit);

        assert!(tracker.is_degraded()); // SessionPersistence still degraded
        assert_eq!(tracker.degraded_count(), 1);
    }

    #[test]
    fn resolve_all_clears_degraded_state() {
        // Validates: Requirement 11 AC 11.6
        let mut tracker = DegradedModeTracker::new();
        tracker.record_failure(
            Subsystem::PluginInit,
            StartupPhase::LoadPlugins,
            "failed".to_string(),
        );

        tracker.resolve(Subsystem::PluginInit);

        assert!(!tracker.is_degraded());
        assert!(tracker.indicator_text().is_none());
    }

    #[test]
    fn indicator_text_shows_count() {
        // Validates: Requirement 11 AC 11.2
        let mut tracker = DegradedModeTracker::new();
        tracker.record_failure(
            Subsystem::PluginInit,
            StartupPhase::LoadPlugins,
            "failed".to_string(),
        );

        let text = tracker.indicator_text().unwrap();
        assert!(text.contains("1 component"));
        assert!(text.contains("click for details"));
    }

    #[test]
    fn indicator_text_pluralises_for_multiple() {
        // Validates: Requirement 11 AC 11.2
        let mut tracker = DegradedModeTracker::new();
        tracker.record_failure(
            Subsystem::PluginInit,
            StartupPhase::LoadPlugins,
            "failed".to_string(),
        );
        tracker.record_failure(
            Subsystem::LayoutRestore,
            StartupPhase::RestoreLayout,
            "failed".to_string(),
        );

        let text = tracker.indicator_text().unwrap();
        assert!(text.contains("2 components"));
    }

    #[test]
    fn detail_summary_includes_subsystem_names_and_reasons() {
        // Validates: Requirement 11 AC 11.3
        let mut tracker = DegradedModeTracker::new();
        tracker.record_failure(
            Subsystem::PluginInit,
            StartupPhase::LoadPlugins,
            "dependency missing".to_string(),
        );

        let details = tracker.detail_summary();
        assert_eq!(details.len(), 1);
        assert!(details[0].contains("Plugin initialisation"));
        assert!(details[0].contains("Phase 5"));
        assert!(details[0].contains("dependency missing"));
    }

    #[test]
    fn duplicate_failure_for_same_subsystem_not_added() {
        let mut tracker = DegradedModeTracker::new();
        tracker.record_failure(
            Subsystem::PluginInit,
            StartupPhase::LoadPlugins,
            "first failure".to_string(),
        );
        tracker.record_failure(
            Subsystem::PluginInit,
            StartupPhase::LoadPlugins,
            "second failure".to_string(),
        );

        assert_eq!(tracker.degraded_count(), 1);
    }

    #[test]
    fn subsystem_display_names_are_descriptive() {
        assert_eq!(
            Subsystem::SessionPersistence.display_name(),
            "Session persistence"
        );
        assert_eq!(
            Subsystem::PluginInit.display_name(),
            "Plugin initialisation"
        );
        assert_eq!(
            Subsystem::LayoutRestore.display_name(),
            "Layout restoration"
        );
        assert_eq!(Subsystem::RecentFiles.display_name(), "Recent files");
        assert_eq!(
            Subsystem::RecoveryFileScan.display_name(),
            "Recovery file scan"
        );
        assert_eq!(Subsystem::UserDataDir.display_name(), "User data directory");
        assert_eq!(Subsystem::Configuration.display_name(), "Configuration");
    }
}
