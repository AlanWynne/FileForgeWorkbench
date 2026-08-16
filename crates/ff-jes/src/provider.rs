//! Provider abstraction for job management operations.
//!
//! Defines the `JobProvider` trait and the `DesktopJesProvider` implementation.

use std::sync::{Arc, Mutex};

use crate::error::JesError;
use crate::ffjcl::{parse_ffjcl, validate_definition};
use crate::initiator::InitiatorPool;
use crate::model::{Job, JobEvent, JobFilter, JobId, JobLog, JobStatus, JobStatusUpdate};
use crate::queue::JobQueue;
use crate::scheduler::{Scheduler, SchedulingStrategy};

// ─── JobProvider Trait ───────────────────────────────────────────────────────

/// Provider abstraction for job management operations.
///
/// Enables future extensibility to remote JES environments.
///
/// Validates: Requirement 14
pub trait JobProvider: Send + Sync {
    /// Returns a unique identifier for this provider.
    fn provider_id(&self) -> &str;

    /// Returns a human-readable display name.
    fn display_name(&self) -> &str;

    /// Lists jobs matching the given filter.
    fn list_jobs(&self, filter: &JobFilter) -> Result<Vec<Job>, JesError>;

    /// Submits a job definition (FFJCL text).
    fn submit_job(&self, jcl: &str, owner: &str) -> Result<JobId, JesError>;

    /// Holds a queued job.
    fn hold_job(&self, id: JobId) -> Result<(), JesError>;

    /// Releases a held job.
    fn release_job(&self, id: JobId) -> Result<(), JesError>;

    /// Cancels a job (queued or active).
    fn cancel_job(&self, id: JobId, requester: &str) -> Result<(), JesError>;

    /// Gets the complete job log.
    fn get_job_log(&self, id: JobId) -> Result<JobLog, JesError>;

    /// Subscribes to job state change events from this provider.
    fn subscribe_to_events(&self) -> std::sync::mpsc::Receiver<JobEvent>;

    /// Returns supported actions for a job in its current state.
    fn supported_actions(&self, job: &Job) -> Vec<JobAction>;

    /// Checks provider health/connectivity.
    fn health_check(&self) -> ProviderHealth;
}

/// Actions that can be performed on a job.
///
/// Validates: Requirement 14 AC 5
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum JobAction {
    ViewLog,
    Hold,
    Release,
    Cancel,
    Purge,
    Properties,
}

/// Provider health status.
///
/// Validates: Requirement 14 AC 6
#[derive(Debug, Clone)]
pub enum ProviderHealth {
    /// Provider is healthy and responsive.
    Healthy,
    /// Provider is degraded (partial functionality).
    Degraded { reason: String },
    /// Provider is unavailable.
    Unavailable { reason: String },
}

// ─── DesktopJesProvider ──────────────────────────────────────────────────────

/// Local desktop implementation of the JobProvider trait.
///
/// Validates: Requirement 14 AC 2
pub struct DesktopJesProvider {
    queue: Arc<JobQueue>,
    pool: Arc<Mutex<InitiatorPool>>,
    scheduler: Arc<Scheduler>,
}

impl DesktopJesProvider {
    /// Creates a new desktop provider.
    pub fn new(
        queue: Arc<JobQueue>,
        pool: Arc<Mutex<InitiatorPool>>,
        strategy: SchedulingStrategy,
    ) -> Self {
        let scheduler = Arc::new(Scheduler::new(queue.clone(), pool.clone(), strategy));
        Self {
            queue,
            pool,
            scheduler,
        }
    }

    /// Runs one scheduler dispatch cycle.
    pub fn dispatch_cycle(&self) -> Result<usize, JesError> {
        self.scheduler.dispatch_cycle()
    }
}

impl JobProvider for DesktopJesProvider {
    fn provider_id(&self) -> &str {
        "desktop"
    }

    fn display_name(&self) -> &str {
        "Local Desktop JES"
    }

    fn list_jobs(&self, filter: &JobFilter) -> Result<Vec<Job>, JesError> {
        Ok(self.queue.query(filter))
    }

    fn submit_job(&self, jcl: &str, owner: &str) -> Result<JobId, JesError> {
        let definition = parse_ffjcl(jcl).map_err(|e| JesError::SubmissionFailed(e.to_string()))?;
        validate_definition(&definition).map_err(|e| JesError::SubmissionFailed(e.to_string()))?;
        self.queue.submit(definition, owner)
    }

    fn hold_job(&self, id: JobId) -> Result<(), JesError> {
        self.queue.update_status(id, JobStatusUpdate::Held)
    }

    fn release_job(&self, id: JobId) -> Result<(), JesError> {
        self.queue.update_status(id, JobStatusUpdate::Released)
    }

    fn cancel_job(&self, id: JobId, requester: &str) -> Result<(), JesError> {
        let job = self.queue.get(id).ok_or(JesError::JobNotFound(id))?;

        match job.status {
            JobStatus::Queued | JobStatus::Held => self.queue.update_status(
                id,
                JobStatusUpdate::Cancelled {
                    cancel_time: chrono::Utc::now(),
                    cancelled_by: requester.to_string(),
                },
            ),
            JobStatus::Active => {
                // Signal cancellation — in a full implementation this would
                // send a signal to the executing process
                self.queue.update_status(
                    id,
                    JobStatusUpdate::Cancelled {
                        cancel_time: chrono::Utc::now(),
                        cancelled_by: requester.to_string(),
                    },
                )?;
                // Release the initiator
                if let Some(init_id) = job.initiator_id {
                    let _ = self.pool.lock().unwrap().release(init_id);
                }
                Ok(())
            }
            _ => Err(JesError::InvalidJobState {
                job_id: id,
                action: "cancel".to_string(),
                current_status: job.status,
            }),
        }
    }

    fn get_job_log(&self, id: JobId) -> Result<JobLog, JesError> {
        let job = self.queue.get(id).ok_or(JesError::JobNotFound(id))?;
        Ok(JobLog::for_job(&job))
    }

    fn subscribe_to_events(&self) -> std::sync::mpsc::Receiver<JobEvent> {
        self.queue.subscribe()
    }

    fn supported_actions(&self, job: &Job) -> Vec<JobAction> {
        let mut actions = vec![JobAction::ViewLog, JobAction::Properties];
        match job.status {
            JobStatus::Queued => {
                actions.push(JobAction::Hold);
                actions.push(JobAction::Cancel);
            }
            JobStatus::Held => {
                actions.push(JobAction::Release);
                actions.push(JobAction::Cancel);
            }
            JobStatus::Active => {
                actions.push(JobAction::Cancel);
            }
            _ if job.is_terminal() => {
                actions.push(JobAction::Purge);
            }
            _ => {}
        }
        actions
    }

    fn health_check(&self) -> ProviderHealth {
        ProviderHealth::Healthy
    }
}

// ─── ProviderRegistry ────────────────────────────────────────────────────────

/// Manages multiple registered providers.
///
/// Validates: Requirement 14 AC 3
pub struct ProviderRegistry {
    providers: Vec<Box<dyn JobProvider>>,
}

impl ProviderRegistry {
    /// Creates an empty registry.
    pub fn new() -> Self {
        Self {
            providers: Vec::new(),
        }
    }

    /// Registers a provider.
    pub fn register(&mut self, provider: Box<dyn JobProvider>) {
        self.providers.push(provider);
    }

    /// Lists all jobs from all providers matching the filter.
    ///
    /// Provider errors are isolated — one failing provider does not affect others.
    ///
    /// Validates: Requirement 14 AC 6
    pub fn list_all_jobs(&self, filter: &JobFilter) -> Vec<Job> {
        let mut all_jobs = Vec::new();
        for provider in &self.providers {
            match provider.list_jobs(filter) {
                Ok(jobs) => all_jobs.extend(jobs),
                Err(_) => {
                    // Provider error is isolated — continue with other providers
                }
            }
        }
        all_jobs
    }

    /// Returns the number of registered providers.
    pub fn provider_count(&self) -> usize {
        self.providers.len()
    }
}

impl Default for ProviderRegistry {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn make_provider() -> DesktopJesProvider {
        let queue = Arc::new(JobQueue::new());
        let pool = Arc::new(Mutex::new(InitiatorPool::new(3)));
        DesktopJesProvider::new(queue, pool, SchedulingStrategy::Priority)
    }

    const VALID_JCL: &str = "//MYJOB   JOB\n//STEP1   EXEC PGM=IEFBR14\n";

    #[test]
    fn provider_id_is_desktop() {
        // Validates: Requirement 14 AC 2
        let provider = make_provider();
        assert_eq!(provider.provider_id(), "desktop");
    }

    #[test]
    fn submit_valid_jcl_returns_job_id() {
        // Validates: Requirement 2 AC 1
        let provider = make_provider();
        let id = provider.submit_job(VALID_JCL, "user").unwrap();
        assert_eq!(id.value(), 1);
    }

    #[test]
    fn submit_invalid_jcl_returns_error() {
        // Validates: Requirement 2 AC 7
        let provider = make_provider();
        let result = provider.submit_job("not valid jcl", "user");
        assert!(result.is_err());
    }

    #[test]
    fn hold_and_release_job() {
        // Validates: Requirement 10 AC 1, 2
        let provider = make_provider();
        let id = provider.submit_job(VALID_JCL, "user").unwrap();
        provider.hold_job(id).unwrap();
        let jobs = provider.list_jobs(&JobFilter::default()).unwrap();
        assert_eq!(jobs[0].status, JobStatus::Held);

        provider.release_job(id).unwrap();
        let jobs = provider.list_jobs(&JobFilter::default()).unwrap();
        assert_eq!(jobs[0].status, JobStatus::Queued);
    }

    #[test]
    fn cancel_queued_job() {
        // Validates: Requirement 6 AC 3
        let provider = make_provider();
        let id = provider.submit_job(VALID_JCL, "user").unwrap();
        provider.cancel_job(id, "operator").unwrap();
        let jobs = provider.list_jobs(&JobFilter::default()).unwrap();
        assert_eq!(jobs[0].status, JobStatus::Cancelled);
        assert_eq!(jobs[0].cancelled_by.as_deref(), Some("operator"));
    }

    #[test]
    fn supported_actions_for_queued_job() {
        // Validates: Requirement 14 AC 5
        let provider = make_provider();
        let id = provider.submit_job(VALID_JCL, "user").unwrap();
        let job = provider.list_jobs(&JobFilter::default()).unwrap().remove(0);
        let actions = provider.supported_actions(&job);
        assert!(actions.contains(&JobAction::Hold));
        assert!(actions.contains(&JobAction::Cancel));
        assert!(!actions.contains(&JobAction::Release));
    }

    #[test]
    fn supported_actions_for_held_job() {
        let provider = make_provider();
        let id = provider.submit_job(VALID_JCL, "user").unwrap();
        provider.hold_job(id).unwrap();
        let job = provider.list_jobs(&JobFilter::default()).unwrap().remove(0);
        let actions = provider.supported_actions(&job);
        assert!(actions.contains(&JobAction::Release));
        assert!(!actions.contains(&JobAction::Hold));
    }

    #[test]
    fn health_check_returns_healthy() {
        // Validates: Requirement 14 AC 6
        let provider = make_provider();
        assert!(matches!(provider.health_check(), ProviderHealth::Healthy));
    }

    #[test]
    fn provider_registry_isolates_errors() {
        // Validates: Requirement 14 AC 6
        struct FailingProvider;
        impl JobProvider for FailingProvider {
            fn provider_id(&self) -> &str {
                "failing"
            }
            fn display_name(&self) -> &str {
                "Failing Provider"
            }
            fn list_jobs(&self, _: &JobFilter) -> Result<Vec<Job>, JesError> {
                Err(JesError::ProviderUnavailable {
                    provider: "failing".to_string(),
                    reason: "connection refused".to_string(),
                })
            }
            fn submit_job(&self, _: &str, _: &str) -> Result<JobId, JesError> {
                Err(JesError::ProviderUnavailable {
                    provider: "failing".to_string(),
                    reason: "".to_string(),
                })
            }
            fn hold_job(&self, _: JobId) -> Result<(), JesError> {
                Ok(())
            }
            fn release_job(&self, _: JobId) -> Result<(), JesError> {
                Ok(())
            }
            fn cancel_job(&self, _: JobId, _: &str) -> Result<(), JesError> {
                Ok(())
            }
            fn get_job_log(&self, id: JobId) -> Result<JobLog, JesError> {
                Err(JesError::JobNotFound(id))
            }
            fn subscribe_to_events(&self) -> std::sync::mpsc::Receiver<JobEvent> {
                std::sync::mpsc::channel().1
            }
            fn supported_actions(&self, _: &Job) -> Vec<JobAction> {
                vec![]
            }
            fn health_check(&self) -> ProviderHealth {
                ProviderHealth::Unavailable {
                    reason: "down".to_string(),
                }
            }
        }

        let mut registry = ProviderRegistry::new();
        registry.register(Box::new(FailingProvider));

        // Should not panic — error is isolated
        let jobs = registry.list_all_jobs(&JobFilter::default());
        assert!(jobs.is_empty());
    }

    #[test]
    fn dispatch_cycle_activates_queued_jobs() {
        // Validates: Requirement 3 AC 3, 6
        let provider = make_provider();
        provider.submit_job(VALID_JCL, "user").unwrap();
        let dispatched = provider.dispatch_cycle().unwrap();
        assert_eq!(dispatched, 1);
        let jobs = provider.list_jobs(&JobFilter::default()).unwrap();
        assert_eq!(jobs[0].status, JobStatus::Active);
    }
}
