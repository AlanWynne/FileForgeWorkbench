//! Structure versioning — version increment and conflict detection.
//!
//! Provides version management logic for structure definitions including
//! monotonic increment, timestamp management, and external modification detection.

use chrono::Utc;

use crate::model::StructureMetadata;

/// Manages structure versioning and edit conflict detection.
pub struct VersionManager;

impl VersionManager {
    /// Increment the version and update `modified_at` timestamp.
    ///
    /// Called on every save operation for a structure definition.
    pub fn increment(metadata: &mut StructureMetadata) {
        metadata.version += 1;
        metadata.modified_at = Some(Utc::now());
    }

    /// Reset version to 1 for a duplicated entry.
    ///
    /// Sets a new `created_at` and clears `modified_at`.
    pub fn reset_for_duplicate(metadata: &mut StructureMetadata) {
        metadata.version = 1;
        metadata.created_at = Utc::now();
        metadata.modified_at = None;
    }

    /// Check if the definition has been modified externally.
    ///
    /// Compares the loaded `modified_at` with the on-disk value.
    /// Returns `true` if they differ (external modification detected).
    pub fn has_external_modification(
        loaded_modified_at: Option<chrono::DateTime<Utc>>,
        disk_modified_at: Option<chrono::DateTime<Utc>>,
    ) -> bool {
        loaded_modified_at != disk_modified_at
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::model::StructureMetadata;
    use chrono::TimeZone;

    // Validates: Requirement 9.2 — version increments by 1
    #[test]
    fn increment_adds_one_to_version() {
        let mut meta = StructureMetadata::new("TEST");
        assert_eq!(meta.version, 1);
        VersionManager::increment(&mut meta);
        assert_eq!(meta.version, 2);
        VersionManager::increment(&mut meta);
        assert_eq!(meta.version, 3);
    }

    // Validates: Requirement 9.4 — modified_at updated on save
    #[test]
    fn increment_sets_modified_at() {
        let mut meta = StructureMetadata::new("TEST");
        assert!(meta.modified_at.is_none());
        VersionManager::increment(&mut meta);
        assert!(meta.modified_at.is_some());
    }

    // Validates: Requirement 9.7 — duplicate resets to version 1
    #[test]
    fn reset_for_duplicate_sets_version_1() {
        let mut meta = StructureMetadata::new("TEST");
        meta.version = 5;
        meta.modified_at = Some(Utc::now());
        VersionManager::reset_for_duplicate(&mut meta);
        assert_eq!(meta.version, 1);
        assert!(meta.modified_at.is_none());
    }

    // Validates: Requirement 9.5 — external modification detection
    #[test]
    fn detects_external_modification() {
        let loaded = Some(Utc.with_ymd_and_hms(2024, 1, 1, 10, 0, 0).unwrap());
        let disk = Some(Utc.with_ymd_and_hms(2024, 1, 1, 12, 0, 0).unwrap());
        assert!(VersionManager::has_external_modification(loaded, disk));
    }

    // Validates: Requirement 9.5 — no modification when timestamps match
    #[test]
    fn no_modification_when_timestamps_match() {
        let ts = Some(Utc.with_ymd_and_hms(2024, 1, 1, 10, 0, 0).unwrap());
        assert!(!VersionManager::has_external_modification(ts, ts));
    }

    // Validates: Requirement 9.5 — both None means no modification
    #[test]
    fn no_modification_when_both_none() {
        assert!(!VersionManager::has_external_modification(None, None));
    }
}
