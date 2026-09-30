//! Evidence packages and content hashing (CR-NR-098, Wave 3).
//!
//! An [`EvidencePackage`] wraps a Collection with the provenance an audit record
//! needs: who generated it, when, the session, a test-case identifier, a
//! pass/fail status, and a cryptographic content HASH that makes any later
//! alteration DETECTABLE (the tamper-EVIDENCE of Requirement 20.4). The same
//! [`content_hash`] is embedded in a protected PDF's metadata.
//!
//! Validates: screen-snapshot-scrm Requirement 14.1, 14.2, 14.3, 14.4, 20.4, 20.5.

use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use sha2::{Digest, Sha256};

use crate::model::ScreenCollection;

/// Pass/fail status of an evidence package (Requirement 14.4).
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum EvidenceStatus {
    /// The evidenced activity passed.
    Pass,
    /// The evidenced activity failed.
    Fail,
    /// No pass/fail judgement recorded.
    NotAssessed,
}

/// A documentation/audit package generated from a Collection.
///
/// Validates: screen-snapshot-scrm Requirement 14.1, 14.2, 14.3, 14.4, 20.5.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct EvidencePackage {
    /// Who generated the package.
    pub user: String,
    /// When it was generated.
    pub generated: DateTime<Utc>,
    /// The originating session id, if known.
    pub session_id: Option<String>,
    /// The collection id this evidences.
    pub collection_id: String,
    /// The collection name.
    pub collection_name: String,
    /// Number of screens evidenced.
    pub screen_count: usize,
    /// Optional test-case identifier (Requirement 14.3).
    pub test_case_id: Option<String>,
    /// Pass/fail status (Requirement 14.4).
    pub status: EvidenceStatus,
    /// Cryptographic content hash of the source collection (Requirement 20.4).
    pub content_hash: String,
}

impl EvidencePackage {
    /// Serialise this package to pretty JSON. Kept in `ff-scrm` so the desktop
    /// shell does not need its own `serde_json` dependency.
    ///
    /// Validates: Requirement 14.2, 20.5.
    pub fn to_json(&self) -> crate::error::Result<String> {
        serde_json::to_string_pretty(self)
            .map_err(|e| crate::error::ScrmError::Serialize(e.to_string()))
    }

    /// Build an evidence package around `collection`, computing its content hash.
    ///
    /// Validates: Requirement 14.1, 14.2, 20.4.
    pub fn build(
        collection: &ScreenCollection,
        user: impl Into<String>,
        test_case_id: Option<String>,
        status: EvidenceStatus,
    ) -> Self {
        Self {
            user: user.into(),
            generated: Utc::now(),
            session_id: collection.session_id.clone(),
            collection_id: collection.collection_id.clone(),
            collection_name: collection.name.clone(),
            screen_count: collection.len(),
            test_case_id,
            status,
            content_hash: content_hash(collection),
        }
    }
}

/// Compute a stable SHA-256 content hash of a Collection, as a lowercase hex
/// string prefixed with the algorithm (`sha256:...`). The hash is taken over the
/// canonical (serde_json) serialisation of the collection, so it is
/// deterministic for equal collections and changes if any capture changes.
///
/// Validates: screen-snapshot-scrm Requirement 20.4.
pub fn content_hash(collection: &ScreenCollection) -> String {
    // serde_json with a stable field order (structs serialise in declaration
    // order; BTreeMap metadata is ordered) gives a deterministic byte stream.
    let bytes = serde_json::to_vec(collection).unwrap_or_default();
    let mut hasher = Sha256::new();
    hasher.update(&bytes);
    let digest = hasher.finalize();
    let hex: String = digest.iter().map(|b| format!("{b:02x}")).collect();
    format!("sha256:{hex}")
}

#[cfg(test)]
mod tests {
    use super::*;
    use chrono::TimeZone;
    use ff_screen_model::ScreenModel;

    fn at(secs: i64) -> DateTime<Utc> {
        Utc.timestamp_opt(1_700_000_000 + secs, 0).unwrap()
    }

    fn collection(n: usize) -> ScreenCollection {
        let mut c = ScreenCollection::new("cid", "Repro", "alan", at(0));
        c.session_id = Some("sess-1".to_string());
        for i in 0..n {
            c.append_capture(
                format!("cap{i}"),
                at(i as i64),
                ScreenModel::new(format!("S{i}")),
            );
        }
        c
    }

    // Validates: Req 14.1, 14.2 -- package carries user/session/collection/count.
    #[test]
    fn build_populates_provenance() {
        let c = collection(2);
        let pkg = EvidencePackage::build(&c, "auditor", None, EvidenceStatus::NotAssessed);
        assert_eq!(pkg.user, "auditor");
        assert_eq!(pkg.session_id.as_deref(), Some("sess-1"));
        assert_eq!(pkg.collection_id, "cid");
        assert_eq!(pkg.screen_count, 2);
    }

    // Validates: Req 14.3, 14.4 -- test-case id and pass/fail recorded.
    #[test]
    fn build_records_test_case_and_status() {
        let c = collection(1);
        let pkg = EvidencePackage::build(
            &c,
            "auditor",
            Some("TC-42".to_string()),
            EvidenceStatus::Pass,
        );
        assert_eq!(pkg.test_case_id.as_deref(), Some("TC-42"));
        assert_eq!(pkg.status, EvidenceStatus::Pass);
    }

    // Validates: Req 20.4 -- content hash is stable for equal collections and
    // changes when the collection changes.
    #[test]
    fn content_hash_is_stable_and_sensitive() {
        let a = collection(2);
        let b = collection(2);
        let c = collection(3);
        assert_eq!(
            content_hash(&a),
            content_hash(&b),
            "equal collections hash equally"
        );
        assert_ne!(
            content_hash(&a),
            content_hash(&c),
            "different collections differ"
        );
        assert!(content_hash(&a).starts_with("sha256:"));
    }
}
