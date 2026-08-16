//! Job queue with persistence.
//!
//! Provides an in-memory job queue backed by a JSON file for persistence
//! across application restarts. Supports query, filter, and priority ordering.

use std::collections::HashMap;
use std::path::Path;
use std::sync::atomic::{AtomicU64, Ordering};
use std::sync::{Arc, RwLock};

use crate::error::JesError;
use crate::ffjcl::FfjclDefinition;
use crate::model::{Job, JobEvent, JobFilter, JobId, JobSortField, JobStatus, JobStatusUpdate};

/// Persistent job queue.
///
/// Validates: Requirements 2, 3
pub struct JobQueue {
    /// In-memory job store.
    jobs: RwLock<HashMap<u64, Job>>,
    /// Monotonically increasing job ID counter.
    next_id: AtomicU64,
    /// Event sender for job state changes.
    event_tx: std::sync::Mutex<Vec<std::sync::mpsc::Sender<JobEvent>>>,
    /// Optional persistence path.
    db_path: Option<std::path::PathBuf>,
}

impl JobQueue {
    /// Creates a new in-memory job queue.
    pub fn new() -> Self {
        Self {
            jobs: RwLock::new(HashMap::new()),
            next_id: AtomicU64::new(1),
            event_tx: std::sync::Mutex::new(Vec::new()),
            db_path: None,
        }
    }

    /// Creates a job queue backed by a JSON file at the given path.
    ///
    /// Validates: Requirement 2 AC 6
    pub fn open(db_path: &Path) -> Result<Self, JesError> {
        let mut queue = Self::new();
        queue.db_path = Some(db_path.to_path_buf());

        // Load existing jobs if file exists
        if db_path.exists() {
            let data = std::fs::read_to_string(db_path).map_err(JesError::IoError)?;
            if !data.trim().is_empty() {
                let jobs: Vec<Job> = serde_json::from_str(&data).map_err(|e| {
                    JesError::QueuePersistenceError(format!("failed to deserialize queue: {e}"))
                })?;
                let mut max_id = 0u64;
                let mut store = queue.jobs.write().unwrap();
                for job in jobs {
                    max_id = max_id.max(job.id.value());
                    store.insert(job.id.value(), job);
                }
                queue.next_id.store(max_id + 1, Ordering::SeqCst);
            }
        }

        Ok(queue)
    }

    /// Submits a new job to the queue.
    ///
    /// Validates: Requirement 2 AC 1–5
    pub fn submit(&self, definition: FfjclDefinition, owner: &str) -> Result<JobId, JesError> {
        let id = JobId::new(self.next_id.fetch_add(1, Ordering::SeqCst));
        let job = Job::new(id, definition, owner);

        {
            let mut jobs = self.jobs.write().unwrap();
            jobs.insert(id.value(), job.clone());
        }

        self.emit_event(JobEvent {
            job_id: id,
            new_status: JobStatus::Queued,
            previous_status: None,
            timestamp: chrono::Utc::now(),
            provider_id: "desktop".to_string(),
        });

        self.persist_if_needed()?;
        Ok(id)
    }

    /// Gets a job by ID.
    pub fn get(&self, id: JobId) -> Option<Job> {
        self.jobs.read().unwrap().get(&id.value()).cloned()
    }

    /// Updates job status with associated metadata.
    ///
    /// Validates: Requirements 3, 6, 10
    pub fn update_status(&self, id: JobId, update: JobStatusUpdate) -> Result<(), JesError> {
        let previous_status;
        let new_status;

        {
            let mut jobs = self.jobs.write().unwrap();
            let job = jobs.get_mut(&id.value()).ok_or(JesError::JobNotFound(id))?;

            previous_status = Some(job.status);

            match update {
                JobStatusUpdate::Dispatched {
                    initiator_id,
                    start_time,
                } => {
                    job.status = JobStatus::Active;
                    job.initiator_id = Some(initiator_id);
                    job.start_time = Some(start_time);
                }
                JobStatusUpdate::StepProgress { step_name } => {
                    job.current_step = Some(step_name);
                    // Status stays Active
                }
                JobStatusUpdate::Completed {
                    end_time,
                    return_code,
                } => {
                    job.status = JobStatus::Completed;
                    job.end_time = Some(end_time);
                    job.return_code = Some(return_code);
                }
                JobStatusUpdate::Failed {
                    end_time,
                    reason,
                    failing_step,
                } => {
                    job.status = JobStatus::Failed;
                    job.end_time = Some(end_time);
                    job.failure_reason = Some(reason);
                    job.failing_step = failing_step;
                }
                JobStatusUpdate::Cancelled {
                    cancel_time,
                    cancelled_by,
                } => {
                    job.status = JobStatus::Cancelled;
                    job.cancel_time = Some(cancel_time);
                    job.cancelled_by = Some(cancelled_by);
                    job.end_time = Some(cancel_time);
                }
                JobStatusUpdate::Held => {
                    if job.status != JobStatus::Queued {
                        return Err(JesError::InvalidJobState {
                            job_id: id,
                            action: "hold".to_string(),
                            current_status: job.status,
                        });
                    }
                    job.status = JobStatus::Held;
                }
                JobStatusUpdate::Released => {
                    if job.status != JobStatus::Held {
                        return Err(JesError::InvalidJobState {
                            job_id: id,
                            action: "release".to_string(),
                            current_status: job.status,
                        });
                    }
                    job.status = JobStatus::Queued;
                }
            }

            new_status = job.status;
        }

        self.emit_event(JobEvent {
            job_id: id,
            new_status,
            previous_status,
            timestamp: chrono::Utc::now(),
            provider_id: "desktop".to_string(),
        });

        self.persist_if_needed()?;
        Ok(())
    }

    /// Queries jobs matching the given filter.
    ///
    /// Validates: Requirement 9 AC 4, 5
    pub fn query(&self, filter: &JobFilter) -> Vec<Job> {
        let jobs = self.jobs.read().unwrap();
        let mut result: Vec<Job> = jobs
            .values()
            .filter(|j| matches_filter(j, filter))
            .cloned()
            .collect();

        // Sort
        let ascending = filter.ascending;
        match filter.sort {
            Some(JobSortField::Id) | None => {
                result.sort_by_key(|j| j.id);
                if !ascending {
                    result.reverse();
                }
            }
            Some(JobSortField::Name) => {
                result.sort_by(|a, b| {
                    if ascending {
                        a.name.cmp(&b.name)
                    } else {
                        b.name.cmp(&a.name)
                    }
                });
            }
            Some(JobSortField::Priority) => {
                result.sort_by(|a, b| {
                    if ascending {
                        a.priority.cmp(&b.priority)
                    } else {
                        b.priority.cmp(&a.priority)
                    }
                });
            }
            Some(JobSortField::SubmitTime) => {
                result.sort_by(|a, b| {
                    if ascending {
                        a.submit_time.cmp(&b.submit_time)
                    } else {
                        b.submit_time.cmp(&a.submit_time)
                    }
                });
            }
            Some(JobSortField::Status) => {
                result.sort_by(|a, b| {
                    let a_s = a.status.to_string();
                    let b_s = b.status.to_string();
                    if ascending {
                        a_s.cmp(&b_s)
                    } else {
                        b_s.cmp(&a_s)
                    }
                });
            }
            Some(JobSortField::Owner) => {
                result.sort_by(|a, b| {
                    if ascending {
                        a.owner.cmp(&b.owner)
                    } else {
                        b.owner.cmp(&a.owner)
                    }
                });
            }
            Some(JobSortField::ReturnCode) => {
                result.sort_by(|a, b| {
                    if ascending {
                        a.return_code.cmp(&b.return_code)
                    } else {
                        b.return_code.cmp(&a.return_code)
                    }
                });
            }
        }

        result
    }

    /// Returns eligible jobs for scheduling (QUEUED, ordered by priority DESC then submit_time ASC).
    ///
    /// Validates: Requirement 3 AC 3–5
    pub fn eligible_jobs(&self) -> Vec<Job> {
        let filter = JobFilter {
            statuses: Some(vec![JobStatus::Queued]),
            ..Default::default()
        };
        let mut jobs = self.query(&filter);
        // Sort: highest priority first, then oldest first within same priority
        jobs.sort_by(|a, b| {
            b.priority
                .cmp(&a.priority)
                .then_with(|| a.submit_time.cmp(&b.submit_time))
        });
        jobs
    }

    /// Returns job counts grouped by status.
    ///
    /// Validates: Requirement 9 AC 2
    pub fn counts_by_status(&self) -> HashMap<JobStatus, usize> {
        let jobs = self.jobs.read().unwrap();
        let mut counts = HashMap::new();
        for job in jobs.values() {
            *counts.entry(job.status).or_insert(0) += 1;
        }
        counts
    }

    /// Purges a job from the queue.
    ///
    /// Validates: Requirement 8 AC 2
    pub fn purge(&self, id: JobId) -> Result<(), JesError> {
        let mut jobs = self.jobs.write().unwrap();
        if jobs.remove(&id.value()).is_none() {
            return Err(JesError::JobNotFound(id));
        }
        drop(jobs);
        self.persist_if_needed()?;
        Ok(())
    }

    /// Subscribes to job state change events.
    ///
    /// Validates: Requirement 12 AC 4
    pub fn subscribe(&self) -> std::sync::mpsc::Receiver<JobEvent> {
        let (tx, rx) = std::sync::mpsc::channel();
        self.event_tx.lock().unwrap().push(tx);
        rx
    }

    /// Persists the queue to disk if a path is configured.
    ///
    /// Validates: Requirement 2 AC 6
    pub fn persist(&self) -> Result<(), JesError> {
        let Some(ref path) = self.db_path else {
            return Ok(());
        };

        if let Some(parent) = path.parent() {
            std::fs::create_dir_all(parent).map_err(JesError::IoError)?;
        }

        let jobs: Vec<Job> = self.jobs.read().unwrap().values().cloned().collect();
        let data = serde_json::to_string_pretty(&jobs)
            .map_err(|e| JesError::QueuePersistenceError(format!("serialization failed: {e}")))?;
        std::fs::write(path, data).map_err(JesError::IoError)?;
        Ok(())
    }

    fn persist_if_needed(&self) -> Result<(), JesError> {
        if self.db_path.is_some() {
            self.persist()
        } else {
            Ok(())
        }
    }

    fn emit_event(&self, event: JobEvent) {
        let mut senders = self.event_tx.lock().unwrap();
        // Remove dead senders
        senders.retain(|tx| tx.send(event.clone()).is_ok());
    }
}

impl Default for JobQueue {
    fn default() -> Self {
        Self::new()
    }
}

fn matches_filter(job: &Job, filter: &JobFilter) -> bool {
    if let Some(ref owner) = filter.owner {
        if !job.owner.eq_ignore_ascii_case(owner) {
            return false;
        }
    }
    if let Some(ref name) = filter.name {
        if !job.name.to_uppercase().starts_with(&name.to_uppercase()) {
            return false;
        }
    }
    if let Some(id) = filter.id {
        if job.id != id {
            return false;
        }
    }
    if let Some(ref statuses) = filter.statuses {
        if !statuses.contains(&job.status) {
            return false;
        }
    }
    if let Some(ref provider) = filter.provider_id {
        if job.provider_id != *provider {
            return false;
        }
    }
    true
}

/// Wraps a JobQueue in an Arc for shared ownership.
pub type SharedQueue = Arc<JobQueue>;

#[cfg(test)]
mod tests {
    use super::*;
    use crate::ffjcl::{FfjclDefinition, FfjclStep};

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

    fn make_def_with_priority(name: &str, priority: u32) -> FfjclDefinition {
        FfjclDefinition {
            job_name: name.to_string(),
            owner: None,
            priority: Some(priority),
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

    #[test]
    fn submit_assigns_monotonic_ids() {
        // Validates: Requirement 2 AC 2
        let queue = JobQueue::new();
        let id1 = queue.submit(make_def("JOB1"), "user").unwrap();
        let id2 = queue.submit(make_def("JOB2"), "user").unwrap();
        let id3 = queue.submit(make_def("JOB3"), "user").unwrap();
        assert!(id1 < id2);
        assert!(id2 < id3);
    }

    #[test]
    fn submit_sets_queued_status() {
        // Validates: Requirement 2 AC 4
        let queue = JobQueue::new();
        let id = queue.submit(make_def("JOB1"), "user").unwrap();
        let job = queue.get(id).unwrap();
        assert_eq!(job.status, JobStatus::Queued);
    }

    #[test]
    fn submit_records_owner() {
        // Validates: Requirement 2 AC 3
        let queue = JobQueue::new();
        let id = queue.submit(make_def("JOB1"), "alice").unwrap();
        let job = queue.get(id).unwrap();
        assert_eq!(job.owner, "alice");
    }

    #[test]
    fn hold_changes_status_to_held() {
        // Validates: Requirement 10 AC 1
        let queue = JobQueue::new();
        let id = queue.submit(make_def("JOB1"), "user").unwrap();
        queue.update_status(id, JobStatusUpdate::Held).unwrap();
        let job = queue.get(id).unwrap();
        assert_eq!(job.status, JobStatus::Held);
    }

    #[test]
    fn hold_active_job_returns_error() {
        // Validates: Requirement 10 AC 4
        let queue = JobQueue::new();
        let id = queue.submit(make_def("JOB1"), "user").unwrap();
        queue
            .update_status(
                id,
                JobStatusUpdate::Dispatched {
                    initiator_id: crate::model::InitiatorId::new(1),
                    start_time: chrono::Utc::now(),
                },
            )
            .unwrap();
        let result = queue.update_status(id, JobStatusUpdate::Held);
        assert!(result.is_err());
        match result.unwrap_err() {
            JesError::InvalidJobState { action, .. } => assert_eq!(action, "hold"),
            e => panic!("unexpected: {e}"),
        }
    }

    #[test]
    fn release_held_job_returns_to_queued() {
        // Validates: Requirement 10 AC 2
        let queue = JobQueue::new();
        let id = queue.submit(make_def("JOB1"), "user").unwrap();
        queue.update_status(id, JobStatusUpdate::Held).unwrap();
        queue.update_status(id, JobStatusUpdate::Released).unwrap();
        let job = queue.get(id).unwrap();
        assert_eq!(job.status, JobStatus::Queued);
    }

    #[test]
    fn release_non_held_job_returns_error() {
        let queue = JobQueue::new();
        let id = queue.submit(make_def("JOB1"), "user").unwrap();
        let result = queue.update_status(id, JobStatusUpdate::Released);
        assert!(result.is_err());
    }

    #[test]
    fn eligible_jobs_excludes_held_and_cancelled() {
        // Validates: Requirement 3 AC 4
        let queue = JobQueue::new();
        let id1 = queue.submit(make_def("JOB1"), "user").unwrap();
        let id2 = queue.submit(make_def("JOB2"), "user").unwrap();
        let _id3 = queue.submit(make_def("JOB3"), "user").unwrap();

        queue.update_status(id1, JobStatusUpdate::Held).unwrap();
        queue
            .update_status(
                id2,
                JobStatusUpdate::Cancelled {
                    cancel_time: chrono::Utc::now(),
                    cancelled_by: "user".to_string(),
                },
            )
            .unwrap();

        let eligible = queue.eligible_jobs();
        assert_eq!(eligible.len(), 1);
        assert_eq!(eligible[0].name, "JOB3");
    }

    #[test]
    fn eligible_jobs_ordered_by_priority_then_fifo() {
        // Validates: Requirement 3 AC 1, 2, 3
        let queue = JobQueue::new();
        queue
            .submit(make_def_with_priority("LOW", 1), "user")
            .unwrap();
        queue
            .submit(make_def_with_priority("HIGH", 10), "user")
            .unwrap();
        queue
            .submit(make_def_with_priority("MED", 5), "user")
            .unwrap();

        let eligible = queue.eligible_jobs();
        assert_eq!(eligible[0].name, "HIGH");
        assert_eq!(eligible[1].name, "MED");
        assert_eq!(eligible[2].name, "LOW");
    }

    #[test]
    fn counts_by_status_correct() {
        // Validates: Requirement 9 AC 2
        let queue = JobQueue::new();
        queue.submit(make_def("JOB1"), "user").unwrap();
        queue.submit(make_def("JOB2"), "user").unwrap();
        let id3 = queue.submit(make_def("JOB3"), "user").unwrap();
        queue.update_status(id3, JobStatusUpdate::Held).unwrap();

        let counts = queue.counts_by_status();
        assert_eq!(counts.get(&JobStatus::Queued), Some(&2));
        assert_eq!(counts.get(&JobStatus::Held), Some(&1));
    }

    #[test]
    fn filter_does_not_mutate_state() {
        // Validates: Requirement 9 AC 5
        let queue = JobQueue::new();
        queue.submit(make_def("JOB1"), "alice").unwrap();
        queue.submit(make_def("JOB2"), "bob").unwrap();

        let before_count = queue.jobs.read().unwrap().len();

        let mut filter = JobFilter::default();
        filter.owner = Some("alice".to_string());
        let results = queue.query(&filter);
        assert_eq!(results.len(), 1);

        let after_count = queue.jobs.read().unwrap().len();
        assert_eq!(before_count, after_count);
    }

    #[test]
    fn queue_persistence_round_trip() {
        // Validates: Requirement 2 AC 6
        let dir = tempfile::TempDir::new().unwrap();
        let db_path = dir.path().join("queue.json");

        {
            let queue = JobQueue::open(&db_path).unwrap();
            queue.submit(make_def("JOB1"), "user").unwrap();
            queue.submit(make_def("JOB2"), "user").unwrap();
        }

        let queue2 = JobQueue::open(&db_path).unwrap();
        let jobs = queue2.query(&JobFilter::default());
        assert_eq!(jobs.len(), 2);
    }

    #[test]
    fn purge_removes_job() {
        // Validates: Requirement 8 AC 2
        let queue = JobQueue::new();
        let id = queue.submit(make_def("JOB1"), "user").unwrap();
        queue.purge(id).unwrap();
        assert!(queue.get(id).is_none());
    }

    #[test]
    fn subscribe_receives_events() {
        // Validates: Requirement 12 AC 4
        let queue = JobQueue::new();
        let rx = queue.subscribe();
        let id = queue.submit(make_def("JOB1"), "user").unwrap();
        let event = rx
            .recv_timeout(std::time::Duration::from_millis(100))
            .unwrap();
        assert_eq!(event.job_id, id);
        assert_eq!(event.new_status, JobStatus::Queued);
    }
}
