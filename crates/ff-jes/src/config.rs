//! Configuration for the JES plugin.
//!
//! Read from `[plugins.ffw-jes]` in the workbench configuration system.

/// Configuration for the JES plugin.
///
/// Validates: Cross-Cutting Configuration
#[derive(Debug, Clone)]
pub struct JesConfig {
    /// Number of initiators in the pool (default: 3).
    pub initiator_count: usize,
    /// Retention days for completed job output (default: 7).
    pub retention_days: u32,
    /// Maximum retained jobs (default: 1000).
    pub retention_max_jobs: usize,
    /// Job Monitor refresh interval in milliseconds (default: 2000).
    pub monitor_refresh_ms: u64,
    /// Scheduler poll interval in milliseconds (default: 500).
    pub scheduler_poll_ms: u64,
    /// Job cancellation timeout in milliseconds (default: 30000).
    pub job_cancel_timeout_ms: u64,
    /// Spool storage root path.
    pub spool_root: String,
    /// Queue database path.
    pub queue_db_path: String,
}

impl Default for JesConfig {
    fn default() -> Self {
        Self {
            initiator_count: 3,
            retention_days: 7,
            retention_max_jobs: 1000,
            monitor_refresh_ms: 2000,
            scheduler_poll_ms: 500,
            job_cancel_timeout_ms: 30000,
            spool_root: ".ffwb/spool".to_string(),
            queue_db_path: ".ffwb/jes-queue.db".to_string(),
        }
    }
}

impl JesConfig {
    /// Validates configuration values.
    ///
    /// Validates: Requirement 1 AC 4; Requirement 4 AC 1
    pub fn validate(&self) -> Result<(), crate::error::JesError> {
        if self.initiator_count == 0 {
            return Err(crate::error::JesError::ConfigError(
                "initiator_count must be > 0".to_string(),
            ));
        }
        if self.scheduler_poll_ms == 0 {
            return Err(crate::error::JesError::ConfigError(
                "scheduler_poll_ms must be > 0".to_string(),
            ));
        }
        if self.retention_days == 0 {
            return Err(crate::error::JesError::ConfigError(
                "retention_days must be > 0".to_string(),
            ));
        }
        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn default_config_is_valid() {
        // Validates: Requirement 1 AC 4; Requirement 4 AC 1
        let config = JesConfig::default();
        assert!(config.validate().is_ok());
        assert_eq!(config.initiator_count, 3);
        assert_eq!(config.retention_days, 7);
        assert_eq!(config.retention_max_jobs, 1000);
        assert_eq!(config.monitor_refresh_ms, 2000);
    }

    #[test]
    fn zero_initiator_count_is_invalid() {
        let mut config = JesConfig::default();
        config.initiator_count = 0;
        assert!(config.validate().is_err());
    }

    #[test]
    fn zero_scheduler_poll_is_invalid() {
        let mut config = JesConfig::default();
        config.scheduler_poll_ms = 0;
        assert!(config.validate().is_err());
    }
}
