//! Core types for the database tool.

use std::fmt;

/// Unique identifier for a database connection.
#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub struct ConnectionId(pub String);

impl ConnectionId {
    /// Create a new ConnectionId.
    pub fn new(id: impl Into<String>) -> Self {
        Self(id.into())
    }
}

impl fmt::Display for ConnectionId {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{}", self.0)
    }
}

/// Unique identifier for a query execution.
#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub struct ExecutionId(pub u64);

/// SQL dialect for a database connection.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum SqlDialect {
    /// PostgreSQL dialect.
    PostgreSql,
    /// MySQL/MariaDB dialect.
    MySql,
    /// SQLite dialect.
    Sqlite,
    /// Microsoft T-SQL dialect.
    TSql,
    /// Oracle PL/SQL dialect.
    PlSql,
    /// Generic SQL (unknown dialect).
    Generic,
}

impl fmt::Display for SqlDialect {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::PostgreSql => write!(f, "PostgreSQL"),
            Self::MySql => write!(f, "MySQL"),
            Self::Sqlite => write!(f, "SQLite"),
            Self::TSql => write!(f, "T-SQL"),
            Self::PlSql => write!(f, "PL/SQL"),
            Self::Generic => write!(f, "Generic SQL"),
        }
    }
}

/// SQL data type.
#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub enum SqlType {
    Integer,
    BigInt,
    SmallInt,
    Float,
    Double,
    Decimal { precision: u8, scale: u8 },
    Varchar { max_length: Option<u32> },
    Text,
    Boolean,
    Date,
    Time,
    Timestamp,
    Blob,
    Clob,
    Json,
    Uuid,
    Array(Box<SqlType>),
    Custom(String),
}

impl fmt::Display for SqlType {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::Integer => write!(f, "INTEGER"),
            Self::BigInt => write!(f, "BIGINT"),
            Self::SmallInt => write!(f, "SMALLINT"),
            Self::Float => write!(f, "FLOAT"),
            Self::Double => write!(f, "DOUBLE"),
            Self::Decimal { precision, scale } => write!(f, "DECIMAL({precision},{scale})"),
            Self::Varchar {
                max_length: Some(n),
            } => write!(f, "VARCHAR({n})"),
            Self::Varchar { max_length: None } => write!(f, "VARCHAR"),
            Self::Text => write!(f, "TEXT"),
            Self::Boolean => write!(f, "BOOLEAN"),
            Self::Date => write!(f, "DATE"),
            Self::Time => write!(f, "TIME"),
            Self::Timestamp => write!(f, "TIMESTAMP"),
            Self::Blob => write!(f, "BLOB"),
            Self::Clob => write!(f, "CLOB"),
            Self::Json => write!(f, "JSON"),
            Self::Uuid => write!(f, "UUID"),
            Self::Array(inner) => write!(f, "{inner}[]"),
            Self::Custom(name) => write!(f, "{name}"),
        }
    }
}

/// Transaction isolation level.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum IsolationLevel {
    ReadUncommitted,
    ReadCommitted,
    RepeatableRead,
    Serializable,
}

impl fmt::Display for IsolationLevel {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::ReadUncommitted => write!(f, "READ UNCOMMITTED"),
            Self::ReadCommitted => write!(f, "READ COMMITTED"),
            Self::RepeatableRead => write!(f, "REPEATABLE READ"),
            Self::Serializable => write!(f, "SERIALIZABLE"),
        }
    }
}

/// SSL mode for database connections.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum SslMode {
    Disable,
    Allow,
    Prefer,
    Require,
    VerifyCa,
    VerifyFull,
}

impl fmt::Display for SslMode {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::Disable => write!(f, "disable"),
            Self::Allow => write!(f, "allow"),
            Self::Prefer => write!(f, "prefer"),
            Self::Require => write!(f, "require"),
            Self::VerifyCa => write!(f, "verify-ca"),
            Self::VerifyFull => write!(f, "verify-full"),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn connection_id_display() {
        let id = ConnectionId::new("my-db");
        assert_eq!(id.to_string(), "my-db");
    }

    #[test]
    fn sql_dialect_display() {
        // Validates: Requirement 2 AC 1
        assert_eq!(SqlDialect::PostgreSql.to_string(), "PostgreSQL");
        assert_eq!(SqlDialect::MySql.to_string(), "MySQL");
        assert_eq!(SqlDialect::Sqlite.to_string(), "SQLite");
        assert_eq!(SqlDialect::TSql.to_string(), "T-SQL");
    }

    #[test]
    fn sql_type_display() {
        // Validates: Requirement 2 AC 1
        assert_eq!(SqlType::Integer.to_string(), "INTEGER");
        assert_eq!(
            SqlType::Varchar {
                max_length: Some(255)
            }
            .to_string(),
            "VARCHAR(255)"
        );
        assert_eq!(
            SqlType::Decimal {
                precision: 10,
                scale: 2
            }
            .to_string(),
            "DECIMAL(10,2)"
        );
        assert_eq!(
            SqlType::Array(Box::new(SqlType::Integer)).to_string(),
            "INTEGER[]"
        );
    }

    #[test]
    fn isolation_level_display() {
        assert_eq!(IsolationLevel::ReadCommitted.to_string(), "READ COMMITTED");
        assert_eq!(IsolationLevel::Serializable.to_string(), "SERIALIZABLE");
    }

    #[test]
    fn ssl_mode_display() {
        assert_eq!(SslMode::Require.to_string(), "require");
        assert_eq!(SslMode::VerifyFull.to_string(), "verify-full");
    }
}
