//! Retention policy and purge engine.
//!
//! Enforces configurable rules for how long completed job output is retained.

use std::sync::Arc;

use crate::error::JesError;
use crate::model::{JobFilter, JobId, JobStatus};
use crate::queue::JobQueue;

/// Configurable rules for job output retention and purge.
///
/// Validates: Requirement 8
#[derive(Debug, Clone)]
pub struct RetentionPolicy {
    /// Maximum days to retain completed job output (default: 7).
    pub retention_days: u32,
    /// Maximum number of retained jobs (default: 1000).
    pub max_jobs: usize,
}

impl Default for RetentionPolicy {
    fn default() -> Self {
        Self {
            retention_days: 7,
            max_jobs: 1000,
        }
    }
}

/// Manages retention policy enforcement and purge operations.
///
/// Validates: Requirement 8
pub struct RetentionEngine {
    policy: RetentionPolicy,
    queue: Arc<JobQueue>,
}

impl RetentionEngine {
    /// Creates a new retention engine.
    pub fn new(policy: RetentionPolicy, queue: Arc<JobQueue>) -> Self {
        Self { policy, queue }
    }

    /// Purges a single job from the queue.
    ///
    /// Does NOT remove catalogued datasets.
    ///
    /// Validates: Requirement 8 AC 2, 4, 5
    pub fn purge_job(&self, job_id: JobId) -> Result<(), JesError> {
        self.queue.purge(job_id)
    }

    /// Runs auto-purge: removes jobs exceeding the retention policy.
    ///
    /// Purges oldest jobs first when count exceeds max_jobs.
    /// Purges jobs older than retention_days.
    ///
    /// Validates: Requirement 8 AC 3
    pub fn auto_purge(&self) -> Result<usize, JesError> {
        let terminal_statuses = vec![
            JobStatus::Completed,
            JobStatus::Failed,
            JobStatus::Cancelled,
        ];

        let filter = JobFilter {
            statuses: Some(terminal_statuses),
            ..Default::default()
        };
        let mut terminal_jobs = self.queue.query(&filter);

        // Sort by end_time ascending (oldest first)
        terminal_jobs.sort_by_key(|j| j.end_time);

        let now = chrono::Utc::now();
        let max_age = chrono::Duration::days(self.policy.retention_days as i64);
        let mut purged = 0;

        // Purge by age
        for job in &terminal_jobs {
            if job
                .end_time
                .is_some_and(|end_time| now - end_time > max_age)
                && self.queue.purge(job.id).is_ok()
            {
                purged += 1;
            }
        }

        // Re-query after age purge
        let filter2 = JobFilter {
            statuses: Some(vec![
                JobStatus::Completed,
                JobStatus::Failed,
                JobStatus::Cancelled,
            ]),
            ..Default::default()
        };
        let mut remaining = self.queue.query(&filter2);
        remaining.sort_by_key(|j| j.end_time);

        // Purge by count (oldest first)
        if remaining.len() > self.policy.max_jobs {
            let excess = remaining.len() - self.policy.max_jobs;
            for job in remaining.iter().take(excess) {
                if self.queue.purge(job.id).is_ok() {
                    purged += 1;
                }
            }
        }

        Ok(purged)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::ffjcl::{FfjclDefinition, FfjclStep};
    use crate::model::JobStatusUpdate;

    fn make_def(name: &str) -> FfjclDefinition {
        FfjclDefinition {
            job_name: name.to_string(),
            owner: None,
            priority: None,
            class: None,
            steps: vec![FfjclStep {
                name: "STEP1".to_string(),
                program: "PROG1".to_string(),
                args: vec![],
                dds: vec![],
                condition: None,
            }],
            source: String::new(),
        }
    }

    fn complete_job(queue: &JobQueue, id: JobId) {
        queue
            .update_status(
                id,
                JobStatusUpdate::Dispatched {
                    initiator_id: crate::model::InitiatorId::new(1),
                    start_time: chrono::Utc::now(),
                },
            )
            .unwrap();
        queue
            .update_status(
                id,
                JobStatusUpdate::Completed {
                    end_time: chrono::Utc::now(),
                    return_code: 0,
                },
            )
            .unwrap();
    }

    #[test]
    fn purge_job_removes_from_queue() {
        // Validates: Requirement 8 AC 2
        let queue = Arc::new(JobQueue::new());
        let engine = RetentionEngine::new(RetentionPolicy::default(), queue.clone());

        let id = queue.submit(make_def("JOB1"), "user").unwrap();
        complete_job(&queue, id);

        engine.purge_job(id).unwrap();
        assert!(queue.get(id).is_none());
    }

    #[test]
    fn auto_purge_removes_excess_jobs() {
        // Validates: Requirement 8 AC 3
        let queue = Arc::new(JobQueue::new());
        let policy = RetentionPolicy {
            retention_days: 365, // Don't purge by age
            max_jobs: 2,
        };
        let engine = RetentionEngine::new(policy, queue.clone());

        // Submit and complete 4 jobs
        for i in 0..4 {
            let id = queue.submit(make_def(&format!("JOB{i}")), "user").unwrap();
            complete_job(&queue, id);
        }

        let purged = engine.auto_purge().unwrap();
        assert_eq!(purged, 2); // 4 - 2 = 2 purged

        let remaining = queue.query(&JobFilter::default());
        assert_eq!(remaining.len(), 2);
    }

    #[test]
    fn auto_purge_preserves_active_jobs() {
        // Validates: Requirement 8 AC 5 (active jobs not purged)
        let queue = Arc::new(JobQueue::new());
        let policy = RetentionPolicy {
            retention_days: 365,
            max_jobs: 0, // Would purge everything terminal
        };
        let engine = RetentionEngine::new(policy, queue.clone());

        let id = queue.submit(make_def("JOB1"), "user").unwrap();
        // Job stays Queued (not terminal)

        engine.auto_purge().unwrap();
        assert!(queue.get(id).is_some()); // Still in queue
    }

    #[test]
    fn default_retention_policy_values() {
        // Validates: Requirement 8 AC 1
        let policy = RetentionPolicy::default();
        assert_eq!(policy.retention_days, 7);
        assert_eq!(policy.max_jobs, 1000);
    }
}
