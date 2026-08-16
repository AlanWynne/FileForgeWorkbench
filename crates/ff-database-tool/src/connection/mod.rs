//! Connection management — descriptors, state, and types.

use crate::types::{ConnectionId, IsolationLevel, SslMode};

/// Connection state.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ConnectionState {
    /// Not connected.
    Disconnected,
    /// Connection in progress.
    Connecting,
    /// Connected and ready.
    Connected,
    /// Connection error.
    Error,
}

impl std::fmt::Display for ConnectionState {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::Disconnected => write!(f, "Disconnected"),
            Self::Connecting => write!(f, "Connecting"),
            Self::Connected => write!(f, "Connected"),
            Self::Error => write!(f, "Error"),
        }
    }
}

/// Connection type classification with visual indicator.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ConnectionType {
    /// Type name (e.g., "Development", "Test", "Production").
    pub name: String,
    /// Colour indicator (hex RGB, e.g., "#FF0000" for red).
    pub colour: String,
    /// Whether to prompt for confirmation before executing DML/DDL.
    pub confirm_on_execute: bool,
}

impl ConnectionType {
    /// Development connection type (neutral).
    pub fn development() -> Self {
        Self {
            name: "Development".to_string(),
            colour: "#808080".to_string(),
            confirm_on_execute: false,
        }
    }

    /// Test connection type (green indicator).
    pub fn test() -> Self {
        Self {
            name: "Test".to_string(),
            colour: "#00AA00".to_string(),
            confirm_on_execute: false,
        }
    }

    /// Production connection type (red indicator, confirmation required).
    pub fn production() -> Self {
        Self {
            name: "Production".to_string(),
            colour: "#FF0000".to_string(),
            confirm_on_execute: true,
        }
    }
}

/// Pool configuration for a connection.
#[derive(Debug, Clone)]
pub struct PoolConfig {
    /// Minimum number of connections to maintain.
    pub min_connections: u32,
    /// Maximum number of connections allowed.
    pub max_connections: u32,
    /// Timeout for acquiring a connection (milliseconds).
    pub acquire_timeout_ms: u64,
    /// Idle connection timeout (milliseconds).
    pub idle_timeout_ms: u64,
    /// Validation query (e.g., "SELECT 1").
    pub validation_query: String,
    /// Whether auto-commit is enabled.
    pub auto_commit: bool,
    /// Transaction isolation level.
    pub isolation_level: IsolationLevel,
    /// Whether each SQL editor tab uses its own connection.
    pub separate_connections: bool,
}

impl Default for PoolConfig {
    fn default() -> Self {
        Self {
            min_connections: 1,
            max_connections: 10,
            acquire_timeout_ms: 30_000,
            idle_timeout_ms: 600_000,
            validation_query: "SELECT 1".to_string(),
            auto_commit: true,
            isolation_level: IsolationLevel::ReadCommitted,
            separate_connections: false,
        }
    }
}

/// SSH configuration for tunnelled connections.
#[derive(Debug, Clone)]
pub struct SshConfig {
    /// SSH server hostname.
    pub host: String,
    /// SSH server port.
    pub port: u16,
    /// SSH username.
    pub username: String,
    /// Authentication method.
    pub auth_method: SshAuthMethod,
    /// Optional jump hosts for multi-hop connections.
    pub jump_hosts: Vec<String>,
}

/// SSH authentication method.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum SshAuthMethod {
    /// Password authentication.
    Password,
    /// Private key file authentication.
    PrivateKey { key_path: String },
    /// SSH agent authentication.
    Agent,
}

/// A database connection descriptor — all configuration for one connection.
#[derive(Debug, Clone)]
pub struct ConnectionDescriptor {
    /// Unique connection identifier.
    pub id: ConnectionId,
    /// Human-readable connection name.
    pub name: String,
    /// Driver name (references DriverRegistry).
    pub driver_name: String,
    /// Database host.
    pub host: String,
    /// Database port.
    pub port: u16,
    /// Database name.
    pub database: String,
    /// Username.
    pub username: String,
    /// Reference to stored credential (not the credential itself).
    pub credential_ref: Option<String>,
    /// Connection type classification.
    pub connection_type: ConnectionType,
    /// SSL mode.
    pub ssl_mode: SslMode,
    /// Optional SSH tunnel configuration.
    pub ssh_config: Option<SshConfig>,
    /// Connection pool configuration.
    pub pool_config: PoolConfig,
    /// SQL statements to execute after connection establishment.
    pub bootstrap_queries: Vec<String>,
    /// Idle timeout in seconds (0 = no timeout).
    pub idle_timeout_secs: u64,
    /// Keep-alive interval in seconds (0 = no keep-alive).
    pub keepalive_interval_secs: u64,
}

impl ConnectionDescriptor {
    /// Create a new connection descriptor with minimal required fields.
    pub fn new(
        id: ConnectionId,
        name: impl Into<String>,
        driver_name: impl Into<String>,
        host: impl Into<String>,
        port: u16,
        database: impl Into<String>,
        username: impl Into<String>,
    ) -> Self {
        Self {
            id,
            name: name.into(),
            driver_name: driver_name.into(),
            host: host.into(),
            port,
            database: database.into(),
            username: username.into(),
            credential_ref: None,
            connection_type: ConnectionType::development(),
            ssl_mode: SslMode::Prefer,
            ssh_config: None,
            pool_config: PoolConfig::default(),
            bootstrap_queries: Vec::new(),
            idle_timeout_secs: 0,
            keepalive_interval_secs: 0,
        }
    }

    /// Returns true if this connection requires confirmation before DML/DDL.
    pub fn requires_confirmation(&self) -> bool {
        self.connection_type.confirm_on_execute
    }

    /// Returns true if SSH tunnelling is configured.
    pub fn has_ssh_tunnel(&self) -> bool {
        self.ssh_config.is_some()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn make_descriptor() -> ConnectionDescriptor {
        ConnectionDescriptor::new(
            ConnectionId::new("test-1"),
            "Test DB",
            "postgresql",
            "localhost",
            5432,
            "mydb",
            "admin",
        )
    }

    #[test]
    fn connection_state_display() {
        // Validates: Requirement 3 AC 7
        assert_eq!(ConnectionState::Connected.to_string(), "Connected");
        assert_eq!(ConnectionState::Disconnected.to_string(), "Disconnected");
        assert_eq!(ConnectionState::Error.to_string(), "Error");
    }

    #[test]
    fn development_type_no_confirmation() {
        // Validates: Requirement 3 AC 5
        let ct = ConnectionType::development();
        assert!(!ct.confirm_on_execute);
    }

    #[test]
    fn production_type_requires_confirmation() {
        // Validates: Requirement 3 AC 5, AC 6
        let ct = ConnectionType::production();
        assert!(ct.confirm_on_execute);
        assert_eq!(ct.colour, "#FF0000");
    }

    #[test]
    fn test_type_green_indicator() {
        // Validates: Requirement 3 AC 5
        let ct = ConnectionType::test();
        assert!(ct.colour.contains("AA"));
    }

    #[test]
    fn descriptor_requires_confirmation_for_production() {
        // Validates: Requirement 3 AC 6
        let mut desc = make_descriptor();
        desc.connection_type = ConnectionType::production();
        assert!(desc.requires_confirmation());
    }

    #[test]
    fn descriptor_no_confirmation_for_development() {
        let desc = make_descriptor();
        assert!(!desc.requires_confirmation());
    }

    #[test]
    fn descriptor_no_ssh_by_default() {
        let desc = make_descriptor();
        assert!(!desc.has_ssh_tunnel());
    }

    #[test]
    fn descriptor_with_ssh_tunnel() {
        // Validates: Requirement 3 AC 12
        let mut desc = make_descriptor();
        desc.ssh_config = Some(SshConfig {
            host: "bastion.example.com".to_string(),
            port: 22,
            username: "sshuser".to_string(),
            auth_method: SshAuthMethod::Agent,
            jump_hosts: vec![],
        });
        assert!(desc.has_ssh_tunnel());
    }

    #[test]
    fn pool_config_defaults() {
        // Validates: Requirement 4 AC 1
        let pool = PoolConfig::default();
        assert_eq!(pool.min_connections, 1);
        assert_eq!(pool.max_connections, 10);
        assert!(pool.auto_commit);
        assert_eq!(pool.isolation_level, IsolationLevel::ReadCommitted);
    }
}
