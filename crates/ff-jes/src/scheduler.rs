//! Job scheduler — dispatches eligible jobs to available initiators.

use std::sync::{Arc, Mutex};

use crate::error::JesError;
use crate::initiator::InitiatorPool;
use crate::model::JobStatusUpdate;
use crate::queue::JobQueue;

/// Scheduling strategy for job dispatch.
///
/// Validates: Requirement 3 AC 1, 2
#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
pub enum SchedulingStrategy {
    /// First-in-first-out by submission time (default).
    Fifo,
    /// Higher-priority jobs dispatched first, then FIFO within same priority.
    #[default]
    Priority,
}

/// The job scheduler.
///
/// Selects eligible jobs from the queue and dispatches them to available initiators.
///
/// Validates: Requirement 3
pub struct Scheduler {
    queue: Arc<JobQueue>,
    pool: Arc<Mutex<InitiatorPool>>,
    strategy: SchedulingStrategy,
}

impl Scheduler {
    /// Creates a new scheduler.
    pub fn new(
        queue: Arc<JobQueue>,
        pool: Arc<Mutex<InitiatorPool>>,
        strategy: SchedulingStrategy,
    ) -> Self {
        Self {
            queue,
            pool,
            strategy,
        }
    }

    /// Runs one dispatch cycle: matches eligible jobs to available initiators.
    ///
    /// Returns the number of jobs dispatched.
    ///
    /// Validates: Requirement 3 AC 3–7
    pub fn dispatch_cycle(&self) -> Result<usize, JesError> {
        let eligible = self.queue.eligible_jobs();
        if eligible.is_empty() {
            return Ok(0);
        }

        let mut pool = self.pool.lock().unwrap();
        let mut dispatched = 0;

        for job in eligible {
            let Some(initiator_id) = pool.get_available() else {
                break; // No more available initiators
            };

            // Transition job to Active
            self.queue.update_status(
                job.id,
                JobStatusUpdate::Dispatched {
                    initiator_id,
                    start_time: chrono::Utc::now(),
                },
            )?;

            // Assign job to initiator
            pool.assign(initiator_id, job.id)?;
            dispatched += 1;
        }

        Ok(dispatched)
    }

    /// Returns the scheduling strategy.
    pub fn strategy(&self) -> SchedulingStrategy {
        self.strategy
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::ffjcl::{FfjclDefinition, FfjclStep};
    use crate::model::JobStatus;

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
    fn dispatch_cycle_moves_job_to_active() {
        // Validates: Requirement 3 AC 6
        let queue = Arc::new(JobQueue::new());
        let pool = Arc::new(Mutex::new(InitiatorPool::new(2)));
        let scheduler = Scheduler::new(queue.clone(), pool, SchedulingStrategy::Priority);

        let id = queue.submit(make_def("JOB1"), "user").unwrap();
        let dispatched = scheduler.dispatch_cycle().unwrap();
        assert_eq!(dispatched, 1);

        let job = queue.get(id).unwrap();
        assert_eq!(job.status, JobStatus::Active);
        assert!(job.initiator_id.is_some());
    }

    #[test]
    fn dispatch_cycle_skips_held_jobs() {
        // Validates: Requirement 3 AC 4
        let queue = Arc::new(JobQueue::new());
        let pool = Arc::new(Mutex::new(InitiatorPool::new(2)));
        let scheduler = Scheduler::new(queue.clone(), pool, SchedulingStrategy::Priority);

        let id = queue.submit(make_def("JOB1"), "user").unwrap();
        queue
            .update_status(id, crate::model::JobStatusUpdate::Held)
            .unwrap();

        let dispatched = scheduler.dispatch_cycle().unwrap();
        assert_eq!(dispatched, 0);
        assert_eq!(queue.get(id).unwrap().status, JobStatus::Held);
    }

    #[test]
    fn dispatch_cycle_respects_pool_capacity() {
        // Validates: Requirement 3 AC 7
        let queue = Arc::new(JobQueue::new());
        let pool = Arc::new(Mutex::new(InitiatorPool::new(2))); // capacity 2
        let scheduler = Scheduler::new(queue.clone(), pool, SchedulingStrategy::Priority);

        // Submit 5 jobs
        for i in 0..5 {
            queue.submit(make_def(&format!("JOB{i}")), "user").unwrap();
        }

        let dispatched = scheduler.dispatch_cycle().unwrap();
        assert_eq!(dispatched, 2); // Only 2 initiators available

        let active_count = queue
            .query(&crate::model::JobFilter {
                statuses: Some(vec![JobStatus::Active]),
                ..Default::default()
            })
            .len();
        assert_eq!(active_count, 2);
    }

    #[test]
    fn dispatch_cycle_dispatches_highest_priority_first() {
        // Validates: Requirement 3 AC 2, 3
        let queue = Arc::new(JobQueue::new());
        let pool = Arc::new(Mutex::new(InitiatorPool::new(1))); // only 1 initiator
        let scheduler = Scheduler::new(queue.clone(), pool, SchedulingStrategy::Priority);

        let id_low = queue
            .submit(make_def_with_priority("LOW", 1), "user")
            .unwrap();
        let id_high = queue
            .submit(make_def_with_priority("HIGH", 10), "user")
            .unwrap();

        scheduler.dispatch_cycle().unwrap();

        // HIGH priority should be dispatched
        assert_eq!(queue.get(id_high).unwrap().status, JobStatus::Active);
        assert_eq!(queue.get(id_low).unwrap().status, JobStatus::Queued);
    }

    #[test]
    fn dispatch_cycle_with_no_jobs_returns_zero() {
        let queue = Arc::new(JobQueue::new());
        let pool = Arc::new(Mutex::new(InitiatorPool::new(3)));
        let scheduler = Scheduler::new(queue, pool, SchedulingStrategy::Fifo);
        assert_eq!(scheduler.dispatch_cycle().unwrap(), 0);
    }
}
