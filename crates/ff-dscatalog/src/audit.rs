//! Catalogue audit trail.
//!
//! Every create, rename, move, delete, restore, import, export, and allocate
//! action is recorded in the `audit_log` table so that a complete history of
//! catalogue changes is available for review and compliance.
//!
//! Validates: Requirement 27.4, 28.6

use rusqlite::{params, Connection};

use crate::error::CatalogError;

// === AuditAction ==========================================================

/// The type of catalogue change being recorded.
///
/// Validates: Requirement 27.4
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum AuditAction {
    /// A new dataset was created or allocated.
    Create,
    /// A dataset was renamed.
    Rename,
    /// A dataset was moved to a different location.
    Move,
    /// A dataset was deleted.
    Delete,
    /// A dataset or workspace was restored from backup.
    Restore,
    /// A catalogue was imported from an archive.
    Import,
    /// A catalogue was exported to an archive.
    Export,
    /// A dataset was allocated (synonym for Create in JCL context).
    Allocate,
}

impl AuditAction {
    fn as_str(self) -> &'static str {
        match self {
            Self::Create => "create",
            Self::Rename => "rename",
            Self::Move => "move",
            Self::Delete => "delete",
            Self::Restore => "restore",
            Self::Import => "import",
            Self::Export => "export",
            Self::Allocate => "allocate",
        }
    }
}

// === AuditLog =============================================================

/// Records audit events into the `audit_log` table of a catalogue database.
///
/// Validates: Requirement 27.4, 28.6
pub struct AuditLog<'a> {
    conn: &'a Connection,
}

impl<'a> AuditLog<'a> {
    /// Create an `AuditLog` bound to the given connection.
    pub fn new(conn: &'a Connection) -> Self {
        Self { conn }
    }

    /// Record a successful catalogue action.
    ///
    /// - `action` -- the type of change
    /// - `object_dsn` -- the DSN of the affected dataset (or `None` for catalogue-level actions)
    /// - `principal` -- the initiating user or process (or `None` if unavailable)
    ///
    /// Validates: Requirement 27.4, 28.6
    pub fn record(
        &self,
        action: AuditAction,
        object_dsn: Option<&str>,
        principal: Option<&str>,
    ) -> Result<(), CatalogError> {
        self.insert(action, object_dsn, "ok", principal)
    }

    /// Record a failed catalogue action.
    pub fn record_err(
        &self,
        action: AuditAction,
        object_dsn: Option<&str>,
        principal: Option<&str>,
    ) -> Result<(), CatalogError> {
        self.insert(action, object_dsn, "err", principal)
    }

    fn insert(
        &self,
        action: AuditAction,
        object_dsn: Option<&str>,
        outcome: &str,
        principal: Option<&str>,
    ) -> Result<(), CatalogError> {
        let now = chrono::Utc::now().to_rfc3339();
        self.conn
            .execute(
                "INSERT INTO audit_log (action, object_dsn, outcome, timestamp, principal) \
                 VALUES (?1, ?2, ?3, ?4, ?5)",
                params![action.as_str(), object_dsn, outcome, now, principal],
            )
            .map_err(|source| CatalogError::SqliteError {
                operation: "audit_log_insert".to_string(),
                source,
            })?;
        Ok(())
    }
}

// === AuditEntry ===========================================================

/// A single row read back from the `audit_log` table.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct AuditEntry {
    /// Row id.
    pub id: i64,
    /// Action string (e.g. "create", "delete").
    pub action: String,
    /// DSN of the affected object, if applicable.
    pub object_dsn: Option<String>,
    /// Outcome: "ok" or "err".
    pub outcome: String,
    /// ISO 8601 timestamp.
    pub timestamp: String,
    /// Initiating principal, if known.
    pub principal: Option<String>,
}

/// Read all audit log entries from the given connection, newest first.
pub fn read_audit_log(conn: &Connection) -> Result<Vec<AuditEntry>, CatalogError> {
    let mut stmt = conn
        .prepare(
            "SELECT id, action, object_dsn, outcome, timestamp, principal \
             FROM audit_log ORDER BY id DESC",
        )
        .map_err(|source| CatalogError::SqliteError {
            operation: "read_audit_log".to_string(),
            source,
        })?;

    let rows = stmt
        .query_map([], |row| {
            Ok(AuditEntry {
                id: row.get(0)?,
                action: row.get(1)?,
                object_dsn: row.get(2)?,
                outcome: row.get(3)?,
                timestamp: row.get(4)?,
                principal: row.get(5)?,
            })
        })
        .map_err(|source| CatalogError::SqliteError {
            operation: "read_audit_log".to_string(),
            source,
        })?;

    rows.map(|r| {
        r.map_err(|source| CatalogError::SqliteError {
            operation: "read_audit_log".to_string(),
            source,
        })
    })
    .collect()
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::schema::initialize_database;
    use rusqlite::Connection;

    fn db() -> Connection {
        let conn = Connection::open_in_memory().expect("in-memory db");
        initialize_database(&conn, "TEST").expect("init");
        conn
    }

    // === Req 27.4 -- audit log records all action types ===================

    #[test]
    fn audit_log_records_create_action() {
        // Validates: Requirement 27.4
        let conn = db();
        let log = AuditLog::new(&conn);
        log.record(AuditAction::Create, Some("PAY.INPUT"), Some("user1"))
            .unwrap();

        let entries = read_audit_log(&conn).unwrap();
        assert_eq!(entries.len(), 1);
        assert_eq!(entries[0].action, "create");
        assert_eq!(entries[0].object_dsn.as_deref(), Some("PAY.INPUT"));
        assert_eq!(entries[0].outcome, "ok");
        assert_eq!(entries[0].principal.as_deref(), Some("user1"));
    }

    #[test]
    fn audit_log_records_delete_action() {
        // Validates: Requirement 27.4
        let conn = db();
        let log = AuditLog::new(&conn);
        log.record(AuditAction::Delete, Some("PAY.OLD"), None)
            .unwrap();

        let entries = read_audit_log(&conn).unwrap();
        assert_eq!(entries[0].action, "delete");
        assert_eq!(entries[0].outcome, "ok");
        assert!(entries[0].principal.is_none());
    }

    #[test]
    fn audit_log_records_all_action_variants() {
        // Validates: Requirement 27.4
        let conn = db();
        let log = AuditLog::new(&conn);
        let actions = [
            AuditAction::Create,
            AuditAction::Rename,
            AuditAction::Move,
            AuditAction::Delete,
            AuditAction::Restore,
            AuditAction::Import,
            AuditAction::Export,
            AuditAction::Allocate,
        ];
        for action in actions {
            log.record(action, Some("TEST.DSN"), None).unwrap();
        }
        let entries = read_audit_log(&conn).unwrap();
        assert_eq!(entries.len(), 8);
        // Newest first -- last inserted is first in result
        assert_eq!(entries[0].action, "allocate");
        assert_eq!(entries[7].action, "create");
    }

    #[test]
    fn audit_log_records_err_outcome() {
        // Validates: Requirement 27.4, 28.6
        let conn = db();
        let log = AuditLog::new(&conn);
        log.record_err(AuditAction::Delete, Some("PAY.MISSING"), Some("admin"))
            .unwrap();

        let entries = read_audit_log(&conn).unwrap();
        assert_eq!(entries[0].outcome, "err");
        assert_eq!(entries[0].action, "delete");
    }

    #[test]
    fn audit_log_catalogue_level_action_has_no_dsn() {
        // Validates: Requirement 27.4 -- catalogue-level actions (import/export) may have no DSN
        let conn = db();
        let log = AuditLog::new(&conn);
        log.record(AuditAction::Export, None, Some("admin"))
            .unwrap();

        let entries = read_audit_log(&conn).unwrap();
        assert!(entries[0].object_dsn.is_none());
    }

    #[test]
    fn audit_log_timestamp_is_nonempty() {
        // Validates: Requirement 28.6 -- timestamp field populated
        let conn = db();
        let log = AuditLog::new(&conn);
        log.record(AuditAction::Allocate, Some("PAY.NEW"), None)
            .unwrap();

        let entries = read_audit_log(&conn).unwrap();
        assert!(!entries[0].timestamp.is_empty());
    }

    #[test]
    fn audit_log_entries_ordered_newest_first() {
        // Validates: Requirement 27.4
        let conn = db();
        let log = AuditLog::new(&conn);
        log.record(AuditAction::Create, Some("A.FIRST"), None)
            .unwrap();
        log.record(AuditAction::Delete, Some("B.SECOND"), None)
            .unwrap();

        let entries = read_audit_log(&conn).unwrap();
        assert_eq!(entries[0].object_dsn.as_deref(), Some("B.SECOND"));
        assert_eq!(entries[1].object_dsn.as_deref(), Some("A.FIRST"));
    }
}
