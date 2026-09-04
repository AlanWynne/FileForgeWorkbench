//! Staged transaction protocol for operations spanning SQLite and the filesystem.
//!
//! Every dataset create or delete is recorded in an `OperationJournal` before
//! any physical work begins.  If the process is interrupted, the journal entry
//! survives and the startup recovery scan can complete or roll back the
//! incomplete operation deterministically.
//!
//! ## State machine
//!
//! Create path:
//!   Staging -> Reserved -> Published -> Active
//!
//! Delete path:
//!   Active -> PendingDelete -> Tombstoned
//!
//! Any state other than Active or Tombstoned is transitional and indicates an
//! incomplete operation that requires recovery.
//!
//! ## Version tokens
//!
//! Each journal entry carries a `version` counter.  Callers supply the version
//! they last read; the update is rejected with `VersionConflict` if another
//! writer has incremented it in the meantime (optimistic locking).
//!
//! Validates: Requirement 25.1, 25.2, 25.3, 25.4, 25.5, 25.6

use std::path::{Path, PathBuf};

use rusqlite::{params, Connection, OptionalExtension};

use crate::error::CatalogError;

const JOURNAL_TABLE: &str = "OPERATION_JOURNAL";
const RECOVERY_DIR: &str = "recovery";

// === State ================================================================

/// Lifecycle state of a managed dataset object.
///
/// Validates: Requirement 25.3
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ObjectState {
    /// Physical content staged in `datasets/staging/`; catalogue entry not yet reserved.
    Staging,
    /// Catalogue entry reserved (row exists, state = Reserved); physical content in staging.
    Reserved,
    /// Physical content moved to `datasets/objects/`; catalogue entry not yet marked Active.
    Published,
    /// Fully committed -- physical content live, catalogue entry active.
    Active,
    /// Marked for deletion; physical content not yet moved.
    PendingDelete,
    /// Physical content moved to `recovery/`; catalogue entry not yet removed.
    Tombstoned,
}

impl ObjectState {
    fn as_str(self) -> &'static str {
        match self {
            Self::Staging => "Staging",
            Self::Reserved => "Reserved",
            Self::Published => "Published",
            Self::Active => "Active",
            Self::PendingDelete => "PendingDelete",
            Self::Tombstoned => "Tombstoned",
        }
    }

    fn from_str(s: &str) -> Option<Self> {
        match s {
            "Staging" => Some(Self::Staging),
            "Reserved" => Some(Self::Reserved),
            "Published" => Some(Self::Published),
            "Active" => Some(Self::Active),
            "PendingDelete" => Some(Self::PendingDelete),
            "Tombstoned" => Some(Self::Tombstoned),
            _ => None,
        }
    }

    /// Whether this state represents an incomplete (recoverable) operation.
    pub fn is_transitional(self) -> bool {
        !matches!(self, Self::Active | Self::Tombstoned)
    }
}

// === Journal entry ========================================================

/// A single entry in the operation journal.
///
/// Validates: Requirement 25.3, 25.5
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct JournalEntry {
    /// Logical dataset name (DSN).
    pub dsn: String,
    /// Provider-specific physical locator.
    pub locator: String,
    /// Current lifecycle state.
    pub state: ObjectState,
    /// Optimistic-locking version counter.  Incremented on every state transition.
    pub version: u64,
}

// === Recovery action ======================================================

/// Proposed corrective action for an incomplete operation.
///
/// Validates: Requirement 25.4
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum RecoveryAction {
    /// Complete the interrupted create by advancing to Active.
    CompleteCreate {
        dsn: String,
        locator: String,
        staging_path: PathBuf,
        objects_path: PathBuf,
    },
    /// Roll back the interrupted create by removing staging content.
    RollbackCreate { dsn: String, locator: String },
    /// Complete the interrupted delete by removing the tombstoned content.
    CompleteDelete { dsn: String, locator: String },
    /// Roll back the interrupted delete by restoring from recovery directory.
    RollbackDelete {
        dsn: String,
        locator: String,
        recovery_path: PathBuf,
        objects_path: PathBuf,
    },
}

// === OperationJournal =====================================================

/// Persistent journal recording in-progress dataset operations.
///
/// Backed by a SQLite table in the workspace catalogue database (or a
/// dedicated journal file).  All state transitions are written before the
/// corresponding physical work so that a crash leaves a recoverable record.
///
/// Validates: Requirement 25.1, 25.2, 25.3, 25.4, 25.5, 25.6
pub struct OperationJournal {
    connection: Connection,
    workspace_root: PathBuf,
}

impl std::fmt::Debug for OperationJournal {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("OperationJournal")
            .field("workspace_root", &self.workspace_root)
            .finish_non_exhaustive()
    }
}

impl OperationJournal {
    /// Open or create the journal database at `<workspace_root>/journal.sqlite`.
    pub fn open(workspace_root: impl AsRef<Path>) -> Result<Self, CatalogError> {
        let root = workspace_root.as_ref().to_path_buf();
        std::fs::create_dir_all(&root).map_err(|source| CatalogError::IoError {
            operation: "open_journal".to_string(),
            source,
        })?;
        let db_path = root.join("journal.sqlite");
        let connection =
            Connection::open(&db_path).map_err(|source| CatalogError::SqliteError {
                operation: "open_journal".to_string(),
                source,
            })?;
        connection
            .pragma_update(None, "journal_mode", "WAL")
            .map_err(|source| CatalogError::SqliteError {
                operation: "open_journal".to_string(),
                source,
            })?;
        connection
            .execute_batch(&format!(
                "CREATE TABLE IF NOT EXISTS {JOURNAL_TABLE} (
                     DSN      TEXT PRIMARY KEY NOT NULL,
                     LOCATOR  TEXT NOT NULL,
                     STATE    TEXT NOT NULL,
                     VERSION  INTEGER NOT NULL DEFAULT 0
                 );"
            ))
            .map_err(|source| CatalogError::SqliteError {
                operation: "open_journal".to_string(),
                source,
            })?;
        Ok(Self {
            connection,
            workspace_root: root,
        })
    }

    // === Create protocol (Req 25.1) =======================================

    /// Begin a staged create: write a Staging entry to the journal.
    ///
    /// Must be called before any physical work.
    /// Validates: Requirement 25.1, 25.6
    pub fn begin_create(&self, dsn: &str, locator: &str) -> Result<(), CatalogError> {
        self.connection
            .execute(
                &format!(
                    "INSERT INTO {JOURNAL_TABLE} (DSN, LOCATOR, STATE, VERSION)
                     VALUES (?1, ?2, 'Staging', 0)"
                ),
                params![dsn, locator],
            )
            .map_err(|source| CatalogError::SqliteError {
                operation: "begin_create".to_string(),
                source,
            })?;
        Ok(())
    }

    /// Advance a create from Staging to Reserved.
    ///
    /// Validates: Requirement 25.1, 25.5
    pub fn reserve(&self, dsn: &str, expected_version: u64) -> Result<(), CatalogError> {
        self.transition(dsn, ObjectState::Staging, ObjectState::Reserved, expected_version)
    }

    /// Advance a create from Reserved to Published.
    ///
    /// Validates: Requirement 25.1, 25.5
    pub fn publish(&self, dsn: &str, expected_version: u64) -> Result<(), CatalogError> {
        self.transition(dsn, ObjectState::Reserved, ObjectState::Published, expected_version)
    }

    /// Advance a create from Published to Active.
    ///
    /// Validates: Requirement 25.1, 25.5, 25.6
    pub fn activate(&self, dsn: &str, expected_version: u64) -> Result<(), CatalogError> {
        self.transition(dsn, ObjectState::Published, ObjectState::Active, expected_version)
    }

    // === Delete protocol (Req 25.2) =======================================

    /// Begin a staged delete: transition Active -> PendingDelete.
    ///
    /// Validates: Requirement 25.2, 25.6
    pub fn begin_delete(&self, dsn: &str, expected_version: u64) -> Result<(), CatalogError> {
        self.transition(dsn, ObjectState::Active, ObjectState::PendingDelete, expected_version)
    }

    /// Advance a delete from PendingDelete to Tombstoned.
    ///
    /// Validates: Requirement 25.2, 25.5
    pub fn tombstone(&self, dsn: &str, expected_version: u64) -> Result<(), CatalogError> {
        self.transition(
            dsn,
            ObjectState::PendingDelete,
            ObjectState::Tombstoned,
            expected_version,
        )
    }

    /// Remove the journal entry once a delete is fully finalised.
    ///
    /// Validates: Requirement 25.2
    pub fn finalise_delete(&self, dsn: &str) -> Result<(), CatalogError> {
        self.connection
            .execute(
                &format!("DELETE FROM {JOURNAL_TABLE} WHERE DSN = ?1"),
                params![dsn],
            )
            .map_err(|source| CatalogError::SqliteError {
                operation: "finalise_delete".to_string(),
                source,
            })?;
        Ok(())
    }

    // === Rollback (Req 25.1, 25.2) ========================================

    /// Remove a journal entry to roll back an incomplete create.
    ///
    /// Validates: Requirement 25.1
    pub fn rollback_create(&self, dsn: &str) -> Result<(), CatalogError> {
        self.connection
            .execute(
                &format!("DELETE FROM {JOURNAL_TABLE} WHERE DSN = ?1"),
                params![dsn],
            )
            .map_err(|source| CatalogError::SqliteError {
                operation: "rollback_create".to_string(),
                source,
            })?;
        Ok(())
    }

    // === Query ============================================================

    /// Read the current journal entry for a DSN.
    pub fn get(&self, dsn: &str) -> Result<Option<JournalEntry>, CatalogError> {
        self.connection
            .query_row(
                &format!(
                    "SELECT DSN, LOCATOR, STATE, VERSION FROM {JOURNAL_TABLE} WHERE DSN = ?1"
                ),
                params![dsn],
                row_to_entry,
            )
            .optional()
            .map_err(|source| CatalogError::SqliteError {
                operation: "get_journal_entry".to_string(),
                source,
            })?
            .map(|r| r)
            .transpose()
    }

    /// Return all transitional (incomplete) entries for startup recovery.
    ///
    /// Validates: Requirement 25.3, 25.4
    pub fn incomplete_operations(&self) -> Result<Vec<JournalEntry>, CatalogError> {
        let mut stmt = self
            .connection
            .prepare(&format!(
                "SELECT DSN, LOCATOR, STATE, VERSION FROM {JOURNAL_TABLE}
                 WHERE STATE NOT IN ('Active', 'Tombstoned')"
            ))
            .map_err(|source| CatalogError::SqliteError {
                operation: "incomplete_operations".to_string(),
                source,
            })?;
        let rows = stmt
            .query_map([], row_to_entry)
            .map_err(|source| CatalogError::SqliteError {
                operation: "incomplete_operations".to_string(),
                source,
            })?;
        rows.map(|r| {
            r.map_err(|source| CatalogError::SqliteError {
                operation: "incomplete_operations".to_string(),
                source,
            })?
        })
        .collect()
    }

    /// Build recovery actions for all incomplete operations.
    ///
    /// Returns one `RecoveryAction` per incomplete journal entry describing
    /// what must be done to restore consistency.  Does not apply any changes.
    ///
    /// Validates: Requirement 25.4
    pub fn recovery_plan(&self) -> Result<Vec<RecoveryAction>, CatalogError> {
        let entries = self.incomplete_operations()?;
        let mut actions = Vec::new();
        for entry in entries {
            let action = match entry.state {
                ObjectState::Staging | ObjectState::Reserved => {
                    // Create was interrupted before physical content was published.
                    // Roll back: remove staging content.
                    RecoveryAction::RollbackCreate {
                        dsn: entry.dsn,
                        locator: entry.locator,
                    }
                }
                ObjectState::Published => {
                    // Create was interrupted after physical move but before Active.
                    // Complete: advance to Active.
                    let staging = self.workspace_root.join("datasets").join("staging");
                    let objects = self
                        .workspace_root
                        .join("datasets")
                        .join("objects")
                        .join(&entry.locator);
                    RecoveryAction::CompleteCreate {
                        dsn: entry.dsn,
                        locator: entry.locator,
                        staging_path: staging,
                        objects_path: objects,
                    }
                }
                ObjectState::PendingDelete => {
                    // Delete was interrupted before tombstone.
                    // Roll back: restore from recovery directory.
                    let recovery = self
                        .workspace_root
                        .join(RECOVERY_DIR)
                        .join(&entry.locator);
                    let objects = self
                        .workspace_root
                        .join("datasets")
                        .join("objects")
                        .join(&entry.locator);
                    RecoveryAction::RollbackDelete {
                        dsn: entry.dsn,
                        locator: entry.locator,
                        recovery_path: recovery,
                        objects_path: objects,
                    }
                }
                ObjectState::Tombstoned => {
                    // Delete was interrupted after tombstone but before finalise.
                    // Complete: remove tombstoned content.
                    RecoveryAction::CompleteDelete {
                        dsn: entry.dsn,
                        locator: entry.locator,
                    }
                }
                ObjectState::Active => unreachable!("Active filtered by query"),
            };
            actions.push(action);
        }
        Ok(actions)
    }

    // === Internal helpers =================================================

    fn transition(
        &self,
        dsn: &str,
        from: ObjectState,
        to: ObjectState,
        expected_version: u64,
    ) -> Result<(), CatalogError> {
        let changed = self
            .connection
            .execute(
                &format!(
                    "UPDATE {JOURNAL_TABLE}
                     SET STATE = ?1, VERSION = VERSION + 1
                     WHERE DSN = ?2 AND STATE = ?3 AND VERSION = ?4"
                ),
                params![to.as_str(), dsn, from.as_str(), expected_version as i64],
            )
            .map_err(|source| CatalogError::SqliteError {
                operation: "transition".to_string(),
                source,
            })?;
        if changed == 0 {
            // Either DSN not found, wrong state, or version conflict.
            let current = self.get(dsn)?;
            match current {
                None => Err(CatalogError::DatasetNotFound {
                    dsn: dsn.to_string(),
                    operation: format!("transition to {}", to.as_str()),
                }),
                Some(entry) if entry.version != expected_version => {
                    Err(CatalogError::InvalidAllocationParams {
                        reason: format!(
                            "version conflict for '{}': expected {}, found {}",
                            dsn, expected_version, entry.version
                        ),
                        operation: format!("transition to {}", to.as_str()),
                    })
                }
                Some(entry) => Err(CatalogError::InvalidAllocationParams {
                    reason: format!(
                        "invalid state transition for '{}': expected {:?}, found {:?}",
                        dsn,
                        from.as_str(),
                        entry.state.as_str()
                    ),
                    operation: format!("transition to {}", to.as_str()),
                }),
            }
        } else {
            Ok(())
        }
    }
}

fn row_to_entry(
    row: &rusqlite::Row<'_>,
) -> rusqlite::Result<Result<JournalEntry, CatalogError>> {
    let dsn: String = row.get(0)?;
    let locator: String = row.get(1)?;
    let state_str: String = row.get(2)?;
    let version: i64 = row.get(3)?;
    let state = ObjectState::from_str(&state_str).ok_or_else(|| {
        rusqlite::Error::FromSqlConversionFailure(
            2,
            rusqlite::types::Type::Text,
            Box::new(std::fmt::Error),
        )
    })?;
    Ok(Ok(JournalEntry {
        dsn,
        locator,
        state,
        version: version as u64,
    }))
}

#[cfg(test)]
mod tests {
    use super::*;
    use tempfile::TempDir;

    fn journal() -> (TempDir, OperationJournal) {
        let dir = tempfile::tempdir().expect("tempdir");
        let j = OperationJournal::open(dir.path()).expect("open journal");
        (dir, j)
    }

    // === Req 25.3 -- journal records transitional states ==================

    #[test]
    fn begin_create_writes_staging_entry() {
        // Validates: Requirement 25.3
        let (_dir, j) = journal();
        j.begin_create("PAY.INPUT", "indexed/abc.sqlite").unwrap();
        let entry = j.get("PAY.INPUT").unwrap().unwrap();
        assert_eq!(entry.state, ObjectState::Staging);
        assert_eq!(entry.version, 0);
        assert_eq!(entry.locator, "indexed/abc.sqlite");
    }

    #[test]
    fn incomplete_operations_returns_only_transitional_entries() {
        // Validates: Requirement 25.3
        let (_dir, j) = journal();
        j.begin_create("PAY.A", "loc/a").unwrap();
        j.begin_create("PAY.B", "loc/b").unwrap();
        // Advance PAY.B all the way to Active
        j.reserve("PAY.B", 0).unwrap();
        j.publish("PAY.B", 1).unwrap();
        j.activate("PAY.B", 2).unwrap();
        let incomplete = j.incomplete_operations().unwrap();
        assert_eq!(incomplete.len(), 1);
        assert_eq!(incomplete[0].dsn, "PAY.A");
    }

    // === Req 25.1 -- staged create protocol ===============================

    #[test]
    fn full_create_protocol_advances_through_all_states() {
        // Validates: Requirement 25.1
        let (_dir, j) = journal();
        j.begin_create("PAY.NEW", "loc/new").unwrap();
        assert_eq!(j.get("PAY.NEW").unwrap().unwrap().state, ObjectState::Staging);

        j.reserve("PAY.NEW", 0).unwrap();
        let e = j.get("PAY.NEW").unwrap().unwrap();
        assert_eq!(e.state, ObjectState::Reserved);
        assert_eq!(e.version, 1);

        j.publish("PAY.NEW", 1).unwrap();
        let e = j.get("PAY.NEW").unwrap().unwrap();
        assert_eq!(e.state, ObjectState::Published);
        assert_eq!(e.version, 2);

        j.activate("PAY.NEW", 2).unwrap();
        let e = j.get("PAY.NEW").unwrap().unwrap();
        assert_eq!(e.state, ObjectState::Active);
        assert_eq!(e.version, 3);

        // Active is not transitional
        assert!(j.incomplete_operations().unwrap().is_empty());
    }

    #[test]
    fn rollback_create_removes_journal_entry() {
        // Validates: Requirement 25.1
        let (_dir, j) = journal();
        j.begin_create("PAY.ROLLBACK", "loc/rb").unwrap();
        j.rollback_create("PAY.ROLLBACK").unwrap();
        assert!(j.get("PAY.ROLLBACK").unwrap().is_none());
    }

    // === Req 25.2 -- staged delete protocol ===============================

    #[test]
    fn full_delete_protocol_advances_through_all_states() {
        // Validates: Requirement 25.2
        let (_dir, j) = journal();
        // Set up an Active entry
        j.begin_create("PAY.DEL", "loc/del").unwrap();
        j.reserve("PAY.DEL", 0).unwrap();
        j.publish("PAY.DEL", 1).unwrap();
        j.activate("PAY.DEL", 2).unwrap();

        j.begin_delete("PAY.DEL", 3).unwrap();
        assert_eq!(
            j.get("PAY.DEL").unwrap().unwrap().state,
            ObjectState::PendingDelete
        );

        j.tombstone("PAY.DEL", 4).unwrap();
        assert_eq!(
            j.get("PAY.DEL").unwrap().unwrap().state,
            ObjectState::Tombstoned
        );

        j.finalise_delete("PAY.DEL").unwrap();
        assert!(j.get("PAY.DEL").unwrap().is_none());
    }

    // === Req 25.5 -- version tokens / optimistic locking ==================

    #[test]
    fn stale_version_is_rejected() {
        // Validates: Requirement 25.5
        let (_dir, j) = journal();
        j.begin_create("PAY.VER", "loc/ver").unwrap();
        // Correct version is 0; supply wrong version
        let err = j.reserve("PAY.VER", 99).unwrap_err();
        assert!(matches!(err, CatalogError::InvalidAllocationParams { .. }));
        // State must be unchanged
        assert_eq!(
            j.get("PAY.VER").unwrap().unwrap().state,
            ObjectState::Staging
        );
    }

    #[test]
    fn wrong_state_transition_is_rejected() {
        // Validates: Requirement 25.5
        let (_dir, j) = journal();
        j.begin_create("PAY.STATE", "loc/st").unwrap();
        // Try to publish without reserving first
        let err = j.publish("PAY.STATE", 0).unwrap_err();
        assert!(matches!(err, CatalogError::InvalidAllocationParams { .. }));
        assert_eq!(
            j.get("PAY.STATE").unwrap().unwrap().state,
            ObjectState::Staging
        );
    }

    // === Req 25.4 -- startup recovery plan ================================

    #[test]
    fn recovery_plan_for_staging_entry_is_rollback_create() {
        // Validates: Requirement 25.4
        let (_dir, j) = journal();
        j.begin_create("PAY.STAG", "loc/stag").unwrap();
        let plan = j.recovery_plan().unwrap();
        assert_eq!(plan.len(), 1);
        assert!(matches!(plan[0], RecoveryAction::RollbackCreate { .. }));
    }

    #[test]
    fn recovery_plan_for_published_entry_is_complete_create() {
        // Validates: Requirement 25.4
        let (_dir, j) = journal();
        j.begin_create("PAY.PUB", "loc/pub").unwrap();
        j.reserve("PAY.PUB", 0).unwrap();
        j.publish("PAY.PUB", 1).unwrap();
        let plan = j.recovery_plan().unwrap();
        assert_eq!(plan.len(), 1);
        assert!(matches!(plan[0], RecoveryAction::CompleteCreate { .. }));
    }

    #[test]
    fn recovery_plan_for_pending_delete_is_rollback_delete() {
        // Validates: Requirement 25.4
        let (_dir, j) = journal();
        j.begin_create("PAY.PDEL", "loc/pdel").unwrap();
        j.reserve("PAY.PDEL", 0).unwrap();
        j.publish("PAY.PDEL", 1).unwrap();
        j.activate("PAY.PDEL", 2).unwrap();
        j.begin_delete("PAY.PDEL", 3).unwrap();
        let plan = j.recovery_plan().unwrap();
        assert_eq!(plan.len(), 1);
        assert!(matches!(plan[0], RecoveryAction::RollbackDelete { .. }));
    }

    #[test]
    fn recovery_plan_for_tombstoned_entry_is_complete_delete() {
        // Validates: Requirement 25.4
        let (_dir, j) = journal();
        j.begin_create("PAY.TOMB", "loc/tomb").unwrap();
        j.reserve("PAY.TOMB", 0).unwrap();
        j.publish("PAY.TOMB", 1).unwrap();
        j.activate("PAY.TOMB", 2).unwrap();
        j.begin_delete("PAY.TOMB", 3).unwrap();
        j.tombstone("PAY.TOMB", 4).unwrap();
        let plan = j.recovery_plan().unwrap();
        assert_eq!(plan.len(), 1);
        assert!(matches!(plan[0], RecoveryAction::CompleteDelete { .. }));
    }

    #[test]
    fn active_entries_produce_no_recovery_actions() {
        // Validates: Requirement 25.4
        let (_dir, j) = journal();
        j.begin_create("PAY.OK", "loc/ok").unwrap();
        j.reserve("PAY.OK", 0).unwrap();
        j.publish("PAY.OK", 1).unwrap();
        j.activate("PAY.OK", 2).unwrap();
        assert!(j.recovery_plan().unwrap().is_empty());
    }

    // === Req 25.6 -- operation not reported successful until postconditions met

    #[test]
    fn activate_fails_if_not_in_published_state() {
        // Validates: Requirement 25.6
        let (_dir, j) = journal();
        j.begin_create("PAY.FAIL", "loc/fail").unwrap();
        // Skip reserve and publish -- try to activate directly
        let err = j.activate("PAY.FAIL", 0).unwrap_err();
        assert!(matches!(err, CatalogError::InvalidAllocationParams { .. }));
        // Entry must still be Staging, not Active
        assert_eq!(
            j.get("PAY.FAIL").unwrap().unwrap().state,
            ObjectState::Staging
        );
    }

    // === Journal survives reopen ==========================================

    #[test]
    fn journal_entries_survive_reopen() {
        // Validates: Requirement 25.3
        let dir = tempfile::tempdir().unwrap();
        {
            let j = OperationJournal::open(dir.path()).unwrap();
            j.begin_create("PAY.PERSIST", "loc/persist").unwrap();
            j.reserve("PAY.PERSIST", 0).unwrap();
        }
        let j2 = OperationJournal::open(dir.path()).unwrap();
        let entry = j2.get("PAY.PERSIST").unwrap().unwrap();
        assert_eq!(entry.state, ObjectState::Reserved);
        assert_eq!(entry.version, 1);
    }
}
