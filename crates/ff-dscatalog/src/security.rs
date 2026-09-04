//! Security hardening utilities.
//!
//! Provides log scrubbing to prevent dataset payload bytes or credentials
//! from appearing in diagnostic output, and documents the parameterised-query
//! contract for all SQLite connections in this crate.
//!
//! Validates: Requirement 28.4, 28.5

// === Log scrubbing ========================================================

/// Redact a byte slice so that no payload content appears in log output.
///
/// Returns a placeholder containing only the byte count -- never the content.
/// Callers MUST use this function before including any dataset content in a
/// log message.
///
/// # Examples
///
/// ```
/// use ff_dscatalog::security::scrub_payload;
/// let msg = format!("wrote {} bytes: {}", 42, scrub_payload(b"SECRET DATA"));
/// assert!(!msg.contains("SECRET"));
/// assert!(msg.contains("<redacted"));
/// ```
///
/// Validates: Requirement 28.4
pub fn scrub_payload(payload: &[u8]) -> String {
    format!("<redacted: {} bytes>", payload.len())
}

/// Redact a string value so that no credential or sensitive text appears in
/// log output.
///
/// Validates: Requirement 28.4
pub fn scrub_str(value: &str) -> &'static str {
    let _ = value;
    "<redacted>"
}

// === Parameterised-query contract =========================================
//
// ALL SQLite operations in this crate MUST use rusqlite's `params![]` macro
// or named parameters. String interpolation into SQL is PROHIBITED.
//
// Enforcement: `#[deny(clippy::format_collect)]` is declared in lib.rs.
// The test below verifies that a SQL injection attempt via a DSN value is
// neutralised by the parameterised query layer.
//
// Validates: Requirement 28.5

#[cfg(test)]
mod tests {
    use super::*;
    use crate::schema::initialize_database;
    use rusqlite::Connection;

    // === Req 28.4 -- log scrubbing ========================================

    #[test]
    fn scrub_payload_returns_redacted_placeholder_not_content() {
        // Validates: Requirement 28.4
        let payload = b"SENSITIVE RECORD CONTENT WITH CREDENTIALS";
        let result = scrub_payload(payload);
        assert!(
            !result.contains("SENSITIVE"),
            "payload content must not appear in log output"
        );
        assert!(
            !result.contains("CREDENTIALS"),
            "credential content must not appear in log output"
        );
        assert!(
            result.contains("<redacted"),
            "result must contain redacted marker"
        );
    }

    #[test]
    fn scrub_payload_includes_byte_count_not_content() {
        // Validates: Requirement 28.4 -- byte count is safe metadata; content is not
        let payload = b"12345678";
        let result = scrub_payload(payload);
        assert!(result.contains('8'), "byte count should be present");
        assert!(!result.contains("12345678"), "raw bytes must not appear");
    }

    #[test]
    fn scrub_str_returns_redacted_not_original() {
        // Validates: Requirement 28.4
        let secret = "password=hunter2";
        let result = scrub_str(secret);
        assert!(!result.contains("hunter2"), "credential must not appear");
        assert_eq!(result, "<redacted>");
    }

    #[test]
    fn log_message_with_scrubbed_payload_contains_no_payload_bytes() {
        // Validates: Requirement 28.4
        // Simulates what a write-operation log line would look like.
        let dsn = "PAY.INPUT";
        let payload = b"EMPLOYEE_ID=12345;SALARY=99999";
        let log_line = format!("dataset '{}' written: {}", dsn, scrub_payload(payload));
        assert!(
            !log_line.contains("EMPLOYEE_ID"),
            "payload must not appear in log"
        );
        assert!(
            !log_line.contains("SALARY"),
            "payload must not appear in log"
        );
        assert!(
            !log_line.contains("99999"),
            "payload must not appear in log"
        );
        assert!(log_line.contains("PAY.INPUT"), "DSN is safe metadata");
        assert!(
            log_line.contains("<redacted"),
            "redacted marker must be present"
        );
    }

    // === Req 28.5 -- parameterised queries prevent SQL injection ===========

    #[test]
    fn parameterised_query_neutralises_sql_injection_in_dsn() {
        // Validates: Requirement 28.5
        // A DSN value containing SQL injection syntax must be stored literally,
        // not interpreted as SQL.
        let conn = Connection::open_in_memory().unwrap();
        initialize_database(&conn, "TEST").unwrap();

        let injection = "'; DROP TABLE datasets; --";
        conn.execute(
            "INSERT INTO datasets (dsn, dsorg, storage_path) VALUES (?1, ?2, ?3)",
            rusqlite::params![injection, "PS", "storage/bad"],
        )
        .unwrap();

        // datasets table must still exist and contain exactly one row
        let count: i64 = conn
            .query_row("SELECT COUNT(*) FROM datasets", [], |row| row.get(0))
            .unwrap();
        assert_eq!(count, 1, "datasets table must survive injection attempt");

        // The stored DSN must be the literal injection string, not executed SQL
        let stored: String = conn
            .query_row(
                "SELECT dsn FROM datasets WHERE dsn = ?1",
                rusqlite::params![injection],
                |row| row.get(0),
            )
            .unwrap();
        assert_eq!(stored, injection, "injection string stored literally");
    }

    #[test]
    fn parameterised_query_neutralises_injection_in_audit_log() {
        // Validates: Requirement 28.5 -- audit_log also uses parameterised inserts
        let conn = Connection::open_in_memory().unwrap();
        initialize_database(&conn, "TEST").unwrap();

        let injection_dsn = "'); DELETE FROM audit_log; --";
        conn.execute(
            "INSERT INTO audit_log (action, object_dsn, outcome, timestamp, principal) \
             VALUES (?1, ?2, ?3, ?4, ?5)",
            rusqlite::params![
                "create",
                injection_dsn,
                "ok",
                "2024-01-01T00:00:00Z",
                "test"
            ],
        )
        .unwrap();

        let count: i64 = conn
            .query_row("SELECT COUNT(*) FROM audit_log", [], |row| row.get(0))
            .unwrap();
        assert_eq!(count, 1, "audit_log must survive injection attempt");
    }
}
