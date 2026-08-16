//! Subsystem registration with platform-core.
//!
//! Defines [`BackgroundIoSubsystem`] which implements the `ff_core::Subsystem`
//! trait, enabling the background I/O service to participate in the platform's
//! lifecycle management (ordered startup, graceful shutdown).

use async_trait::async_trait;

use ff_core::{
    CoreError, ServiceRegistry, StartupOrder, Subsystem, SubsystemCriticality, SubsystemDescriptor,
};

use crate::config::IoConfig;
use crate::service::BackgroundIoService;

/// Background I/O subsystem implementing the platform-core `Subsystem` trait.
///
/// Manages the lifecycle of the `BackgroundIoService` — creating it during
/// initialization and ensuring graceful shutdown (cancel loads, await saves).
pub struct BackgroundIoSubsystem {
    /// The background I/O service instance (created during initialize).
    service: Option<BackgroundIoService>,
    /// Configuration for the service.
    config: IoConfig,
}

impl BackgroundIoSubsystem {
    /// Create a new subsystem instance with the given configuration.
    pub fn new(config: IoConfig) -> Self {
        Self {
            service: None,
            config,
        }
    }

    /// Create a new subsystem with default configuration.
    pub fn with_defaults() -> Self {
        Self::new(IoConfig::default())
    }

    /// Get a reference to the BackgroundIoService (available after initialization).
    pub fn service(&self) -> Option<&BackgroundIoService> {
        self.service.as_ref()
    }
}

#[async_trait]
impl Subsystem for BackgroundIoSubsystem {
    fn descriptor(&self) -> SubsystemDescriptor {
        SubsystemDescriptor {
            name: "background-io",
            criticality: SubsystemCriticality::NonCritical,
            order: StartupOrder::Plugins, // Starts after VFS and config
        }
    }

    async fn initialize(&mut self, _registry: &ServiceRegistry) -> Result<(), CoreError> {
        let service = BackgroundIoService::new(self.config.clone());
        ff_logging::log(
            ff_logging::LogLevel::Info,
            "background-io",
            &format!(
                "initialized: max_concurrent={}, chunk_size={}KB, threshold={}MB",
                self.config.max_concurrent_tasks,
                self.config.chunk_size.as_bytes() / 1024,
                self.config.large_file_threshold.as_bytes() / (1024 * 1024),
            ),
        );
        self.service = Some(service);
        Ok(())
    }

    async fn shutdown(&mut self) -> Result<(), CoreError> {
        if let Some(service) = &self.service {
            service.shutdown().await;
            ff_logging::log(
                ff_logging::LogLevel::Info,
                "background-io",
                "shutdown complete",
            );
        }
        self.service = None;
        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn subsystem_descriptor_has_correct_metadata() {
        // Validates: Requirement 7 AC 8
        let subsystem = BackgroundIoSubsystem::with_defaults();
        let descriptor = subsystem.descriptor();
        assert_eq!(descriptor.name, "background-io");
        assert_eq!(descriptor.criticality, SubsystemCriticality::NonCritical);
    }

    #[tokio::test]
    async fn subsystem_initializes_service() {
        // Validates: Requirement 7 AC 8
        let mut subsystem = BackgroundIoSubsystem::with_defaults();
        let registry = ServiceRegistry::new();

        assert!(subsystem.service().is_none());
        let result = subsystem.initialize(&registry).await;
        assert!(result.is_ok());
        assert!(subsystem.service().is_some());
    }

    #[tokio::test]
    async fn subsystem_shutdown_cleans_up() {
        // Validates: Requirement 7 AC 8
        let mut subsystem = BackgroundIoSubsystem::with_defaults();
        let registry = ServiceRegistry::new();

        subsystem.initialize(&registry).await.unwrap();
        assert!(subsystem.service().is_some());

        let result = subsystem.shutdown().await;
        assert!(result.is_ok());
        assert!(subsystem.service().is_none());
    }

    #[tokio::test]
    async fn subsystem_full_lifecycle() {
        // Validates: Requirement 7 AC 8
        let mut subsystem = BackgroundIoSubsystem::new(IoConfig::new(32, 50, 2, 3, 500, 5));
        let registry = ServiceRegistry::new();

        // Initialize
        subsystem.initialize(&registry).await.unwrap();
        let service = subsystem.service().unwrap();
        assert_eq!(service.config().max_concurrent_tasks, 2);

        // Shutdown
        subsystem.shutdown().await.unwrap();
        assert!(subsystem.service().is_none());
    }
}
